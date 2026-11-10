//! Composition root of the application dependencies.
//!
//! The container constructs the data-layer dependencies once and shares them: the
//! application factory takes it and publishes its parts as the shared state of the
//! application, the endpoints build their use cases from them for every request.

use std::sync::Arc;

use crate::data::service_context::repository::ServiceContextRepository;
use crate::data::todos::repository::TodoRepository;
use crate::infrastructure::clients::postgres::engine::db_pool;

/// The application dependencies, constructed once and shared through the app state.
pub struct Container {
    /// The todos storage.
    pub todo_repository: Arc<TodoRepository>,
    /// The service context storage (the maintenance window).
    pub service_context_repository: Arc<ServiceContextRepository>,
}

impl Container {
    /// Build the dependencies from the process-wide database pool.
    pub fn new() -> Self {
        let pool = db_pool();
        let todo_repository = Arc::new(TodoRepository::new(pool.clone()));
        let service_context_repository = Arc::new(ServiceContextRepository::new(pool));
        Container {
            todo_repository,
            service_context_repository,
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
