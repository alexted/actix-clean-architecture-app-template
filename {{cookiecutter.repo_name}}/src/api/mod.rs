//! HTTP layer: routers that build the use case request from the HTTP request and return the
//! use case response.

use actix_web::web;

pub mod todos;

/// Register the application routes.
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(todos::routes());
}
