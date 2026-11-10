//! Request/response logging middleware.
//!
//! Logs the method, path, query string, headers, client ip, trace id, status and duration of
//! every request. Credential headers are masked. Request and response bodies are not logged.

use std::future::{ready, Ready};
use std::time::Instant;

use actix_web::body::MessageBody;
use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;

use super::trace_id::get_current_trace_id;

/// The logger of the request log; application modules log under their own module names.
const REQUEST_LOG_TARGET: &str = env!("CARGO_PKG_NAME");

/// Credentials must not reach the logs.
const REDACTED_HEADERS: [&str; 5] = [
    "authorization",
    "proxy-authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
];

/// The request/response logging middleware: see the module documentation.
pub struct LogRequests;

impl<S, B> Transform<S, ServiceRequest> for LogRequests
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LogRequestsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LogRequestsMiddleware { service }))
    }
}

/// Middleware service produced by [`LogRequests`].
pub struct LogRequestsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LogRequestsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();

        let method = request.method().to_string();
        let resource = request.path().to_string();
        let query_params = request.query_string().to_string();
        let headers = request
            .headers()
            .iter()
            .map(|(name, value)| {
                let header = name.as_str().to_ascii_lowercase();
                if REDACTED_HEADERS.contains(&header.as_str()) {
                    format!("{name}: ***")
                } else {
                    format!("{name}: {}", value.to_str().unwrap_or("<binary>"))
                }
            })
            .collect::<Vec<String>>()
            .join(", ");
        let client_ip = request
            .connection_info()
            .realip_remote_addr()
            .map(String::from);
        let trace_id = get_current_trace_id();

        let result = self.service.call(request);
        Box::pin(async move {
            let response = result.await?;
            log::info!(
                target: REQUEST_LOG_TARGET,
                "method={} resource={} query_params=\"{}\" headers=\"{}\" trace_id={} client_ip={} response_status={} execution_duration={:.3}s",
                method,
                resource,
                query_params,
                headers,
                trace_id.as_deref().unwrap_or("null"),
                client_ip.as_deref().unwrap_or("null"),
                response.status().as_u16(),
                start_time.elapsed().as_secs_f64(),
            );
            Ok(response)
        })
    }
}
