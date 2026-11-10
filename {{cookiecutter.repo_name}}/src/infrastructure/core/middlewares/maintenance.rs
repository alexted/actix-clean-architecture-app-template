//! Maintenance-window middleware.
//!
//! When the service context reports an active maintenance window, every request is answered
//! with `503 Service Unavailable` instead of being forwarded to the endpoints.

use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::body::EitherBody;
use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web;
use actix_web::{Error, HttpResponse};
use futures_util::future::LocalBoxFuture;
use log::info;

use crate::data::service_context::repository::ServiceContextRepository;
use crate::infrastructure::core::errors::exceptions::OtherError;
use crate::infrastructure::core::middlewares::error_handling::error_response;

/// The maintenance-window middleware: see the module documentation.
pub struct ServiceContextMaintenanceCheck;

impl<S, B> Transform<S, ServiceRequest> for ServiceContextMaintenanceCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = ServiceContextMaintenanceCheckMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ServiceContextMaintenanceCheckMiddleware {
            // The service is shared with the middleware future, which decides only after an
            // await whether the request is forwarded; `Rc` is safe because middlewares run on
            // one thread.
            service: Rc::new(service),
        }))
    }
}

/// Middleware service produced by [`ServiceContextMaintenanceCheck`].
pub struct ServiceContextMaintenanceCheckMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ServiceContextMaintenanceCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let service_context_repository = request
            .app_data::<web::Data<ServiceContextRepository>>()
            .expect("The service context repository must be registered in the application")
            .clone();
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let service_context = match service_context_repository.get_or_create().await {
                Ok(service_context) => service_context,
                Err(error) => {
                    return Ok(ServiceResponse::new(
                        request.into_parts().0,
                        error_response(&OtherError::internal(error.message)).map_into_right_body(),
                    ));
                }
            };

            if service_context.maintenance {
                info!("Service is in maintenance mode");
                return Ok(ServiceResponse::new(
                    request.into_parts().0,
                    HttpResponse::ServiceUnavailable()
                        .finish()
                        .map_into_right_body(),
                ));
            }

            let response = service.call(request).await?;
            Ok(response.map_into_left_body())
        })
    }
}
