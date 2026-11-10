//! Service context ORM model: the Diesel rows of the `service_contexts` table.

use diesel::prelude::*;

use crate::infrastructure::clients::postgres::schema::service_contexts;

/// Row of the `service_contexts` table.
#[derive(Queryable, Insertable, AsChangeset)]
#[diesel(table_name = service_contexts)]
pub struct ServiceContextDiesel {
    pub id: i32,
    pub maintenance: bool,
}
