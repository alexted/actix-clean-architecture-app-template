//! Trace id of the current request.
//!
//! Every request gets a 32-character hex trace id: the incoming W3C `traceparent` header is
//! honored when present, otherwise a new random id is generated. The id is available to the
//! whole request processing chain through the task-local context, is included in the error
//! response body and is returned in the `X-Trace-Id` response header.

use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::body::MessageBody;
use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use tokio::task_local;
use uuid::Uuid;

/// Name of the response header carrying the trace id.
pub const TRACE_ID_HEADER: &str = "x-trace-id";

task_local! {
    static CURRENT_TRACE_ID: String;
}

/// Return the trace id of the current request, or `None` outside a traced request.
pub fn get_current_trace_id() -> Option<String> {
    CURRENT_TRACE_ID.try_with(|trace_id| trace_id.clone()).ok()
}

/// The trace id middleware: see the module documentation.
pub struct TraceId;

impl<S, B> Transform<S, ServiceRequest> for TraceId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TraceIdMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // The service is shared with the middleware future so the trace id scope can own it;
        // `Rc` is safe because middlewares run on one thread.
        ready(Ok(TraceIdMiddleware {
            service: Rc::new(service),
        }))
    }
}

/// Middleware service produced by [`TraceId`].
pub struct TraceIdMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for TraceIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let trace_id = trace_id_of(&request);
        let service = Rc::clone(&self.service);
        // The inner service is called inside the scope so the trace id is available to the
        // whole request processing chain (the other middlewares and the endpoints).
        Box::pin(CURRENT_TRACE_ID.scope(trace_id, async move {
            let trace_id = get_current_trace_id().expect("trace id is set");
            let mut response = service.call(request).await?;
            if let Ok(value) = HeaderValue::from_str(&trace_id) {
                response
                    .headers_mut()
                    .insert(HeaderName::from_static(TRACE_ID_HEADER), value);
            }
            Ok(response)
        }))
    }
}

/// Take the trace id from the incoming `traceparent` header, or generate a new one.
fn trace_id_of(request: &ServiceRequest) -> String {
    request
        .headers()
        .get("traceparent")
        .and_then(|value| value.to_str().ok())
        .and_then(trace_id_from_traceparent)
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string())
}

/// Extract the trace id part of a W3C `traceparent` header (`version-traceid-parentid-flags`).
fn trace_id_from_traceparent(header: &str) -> Option<String> {
    let mut parts = header.split('-');
    let version = parts.next()?;
    let trace_id = parts.next()?;
    if version.len() == 2
        && trace_id.len() == 32
        && trace_id.chars().all(|c| c.is_ascii_hexdigit())
        && trace_id != "00000000000000000000000000000000"
    {
        Some(trace_id.to_string())
    } else {
        None
    }
}
