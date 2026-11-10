//! Service context persistence.

use actix_threadpool::run;
use diesel::prelude::*;
use log::info;

use crate::data::base::{RepositoryError, RepositoryResult};
use crate::data::service_context::dto::ServiceContextDTO;
use crate::infrastructure::clients::postgres::engine::PostgresPool;
use crate::infrastructure::clients::postgres::models::service_context::ServiceContextDiesel;
use crate::infrastructure::clients::postgres::schema::service_contexts::dsl::{
    id, service_contexts,
};

/// Service context storage working on the PostgreSQL connection pool.
#[derive(Clone)]
pub struct ServiceContextRepository {
    pool: PostgresPool,
}

impl ServiceContextRepository {
    /// Bind the repository to the connection pool of the application.
    pub fn new(pool: PostgresPool) -> Self {
        ServiceContextRepository { pool }
    }

    /// Return the service context, creating the default one when it does not exist.
    pub async fn get_or_create(&self) -> RepositoryResult<ServiceContextDTO> {
        let pool = self.pool.clone();
        let existing = run(move || -> RepositoryResult<Option<ServiceContextDiesel>> {
            let mut conn = pool.get()?;
            Ok(service_contexts
                .filter(id.eq(1))
                .first::<ServiceContextDiesel>(&mut conn)
                .optional()?)
        })
        .await
        .map_err(RepositoryError::from)?;

        match existing {
            Some(service_context) => Ok(service_context.into()),
            None => {
                info!("Service context does not exist, creating a service context...");
                self.create().await
            }
        }
    }

    /// Create the default service context and return it.
    async fn create(&self) -> RepositoryResult<ServiceContextDTO> {
        let pool = self.pool.clone();
        let service_context = run(move || -> RepositoryResult<ServiceContextDiesel> {
            let mut conn = pool.get()?;
            Ok(diesel::insert_into(service_contexts)
                .values(ServiceContextDiesel {
                    id: 1,
                    maintenance: false,
                })
                .get_result::<ServiceContextDiesel>(&mut conn)?)
        })
        .await
        .map_err(RepositoryError::from)?;

        Ok(service_context.into())
    }

    /// Replace the state of the service context and return it.
    pub async fn update(
        &self,
        service_context: ServiceContextDTO,
    ) -> RepositoryResult<ServiceContextDTO> {
        let new_row = ServiceContextDiesel::from(service_context);
        let pool = self.pool.clone();
        let updated = run(move || -> RepositoryResult<ServiceContextDiesel> {
            let mut conn = pool.get()?;
            Ok(diesel::update(service_contexts)
                .filter(id.eq(1))
                .set(new_row)
                .get_result::<ServiceContextDiesel>(&mut conn)?)
        })
        .await
        .map_err(RepositoryError::from)?;

        Ok(updated.into())
    }
}
