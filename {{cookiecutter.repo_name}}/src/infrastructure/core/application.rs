//! Application factory.

use std::sync::Arc;

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{web, App, Error};

use crate::api;
use crate::container::Container;
use crate::infrastructure::core::logging_config::init_logging;
use crate::infrastructure::core::middlewares::error_handling::{
    json_config, path_config, query_config,
};
use crate::infrastructure::core::middlewares::log_requests::LogRequests;
use crate::infrastructure::core::middlewares::maintenance::ServiceContextMaintenanceCheck;
use crate::infrastructure::core::middlewares::trace_id::TraceId;
use crate::infrastructure::core::settings::get_config;

/// Build the application: logging, database, error handlers, middlewares and routes.
///
/// The container is the composition root built once per process; its dependencies are
/// published here as the shared state of the application, and the endpoints build their
/// use cases from them for every request.
pub fn create_app(
    container: Arc<Container>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    let config = get_config();
    init_logging(config);

    // The shared repositories of the application, like the original factory took the
    // services out of the container.
    let todo_repository = web::Data::from(container.todo_repository.clone());
    let service_context_repository = web::Data::from(container.service_context_repository.clone());

    App::new()
        .app_data(todo_repository)
        .app_data(service_context_repository)
        // Extractor failures are returned in the common error response body.
        .app_data(json_config())
        .app_data(query_config())
        .app_data(path_config())
        // The middleware registered last runs first: the trace id is created before the
        // request log and the maintenance check, so both see it; the maintenance check runs
        // before the request log, like before the refactoring.
        .wrap(LogRequests)
        .wrap(ServiceContextMaintenanceCheck)
        .wrap(TraceId)
        .configure(api::routes)
}
