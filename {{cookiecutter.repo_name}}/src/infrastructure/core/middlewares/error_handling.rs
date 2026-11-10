//! Application error handling: turns the application errors into the common error response body.
//!
//! Every error produced by the application is returned in one format:
//!
//! ```json
//! {"error": "BAD_REQUEST", "details": "Todo 42 not found", "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736", "timestamp": "2026-01-01T00:00:00Z"}
//! ```
//!
//! `details` is a text, or `null` when the error has no message. The trace id is also returned
//! in the `X-Trace-Id` response header (see the `trace_id` middleware).

use chrono::{DateTime, Utc};
use serde::Serialize;

use actix_web::error::{JsonPayloadError, PathError, QueryPayloadError};
use actix_web::http::StatusCode;
use actix_web::web::{JsonConfig, PathConfig, QueryConfig};
use actix_web::{Error, HttpRequest, HttpResponse};

use super::trace_id::get_current_trace_id;
use crate::infrastructure::core::errors::constants::ErrorType;
use crate::infrastructure::core::errors::exceptions::OtherError;

/// Body of every error response.
#[derive(Debug, Serialize)]
pub struct ErrorSchema {
    pub error: &'static str,
    pub details: Option<String>,
    pub trace_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Build the error response body of an error code and message.
fn error_body(error: ErrorType, details: Option<String>) -> ErrorSchema {
    ErrorSchema {
        error: error.as_str(),
        details,
        trace_id: get_current_trace_id(),
        timestamp: Utc::now(),
    }
}

/// Turn an application error into its HTTP response.
///
/// The status comes from the error class; the body carries the error code, the message as
/// `details`, the trace id of the current request and the timestamp.
pub fn error_response(error: &OtherError) -> HttpResponse {
    let status = StatusCode::from_u16(error.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    HttpResponse::build(status).json(error_body(
        error.error_type(),
        error.message().map(String::from),
    ))
}

/// Application errors are turned into the common error response body by the error handler.
impl actix_web::ResponseError for OtherError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        error_response(self)
    }
}

/// Request body deserialization failure: 400 (the framework's contract for this status is kept).
#[derive(Debug)]
struct ValidationError(String);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl actix_web::ResponseError for ValidationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(error_body(ErrorType::ValidationError, Some(self.0.clone())))
    }
}

/// Map an extractor failure onto the common error response body.
fn extractor_error(message: String) -> Error {
    ValidationError(message).into()
}

/// JSON body extractor configuration with unified error responses.
pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(|error: JsonPayloadError, _req: &HttpRequest| {
        extractor_error(error.to_string())
    })
}

/// Query string extractor configuration with unified error responses.
pub fn query_config() -> QueryConfig {
    QueryConfig::default().error_handler(|error: QueryPayloadError, _req: &HttpRequest| {
        extractor_error(error.to_string())
    })
}

/// Path parameter extractor configuration with unified error responses.
pub fn path_config() -> PathConfig {
    PathConfig::default()
        .error_handler(|error: PathError, _req: &HttpRequest| extractor_error(error.to_string()))
}
