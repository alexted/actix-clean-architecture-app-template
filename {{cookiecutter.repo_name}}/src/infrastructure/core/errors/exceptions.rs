//! Application errors that the error handlers turn into HTTP error responses.
//!
//! [`OtherError`] mirrors the `OtherError` class hierarchy of the reference application:
//! every variant carries the error code and the default HTTP status of one error class.
//! The error handler returns the variant's `message` as the `details` field of the error
//! response body.

use super::constants::ErrorType;

/// Base of the application errors.
///
/// Variants map to the error classes of the reference application (`InternalError`,
/// `ExternalServiceError`, `BadRequestError`, `NotFoundError`, `AccessDeniedError`); each
/// carries a human-readable description returned to the client.
#[derive(Debug)]
pub enum OtherError {
    /// Unexpected server-side failure: 500.
    Internal(Option<String>),
    /// A call to an external service failed: 500.
    ExternalService(Option<String>),
    /// The request cannot be processed: 400.
    BadRequest(Option<String>),
    /// The requested object does not exist: 404.
    NotFound(Option<String>),
    /// The client is not allowed to perform the operation: 403.
    AccessDenied(Option<String>),
}

impl OtherError {
    /// The request cannot be processed: 400.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(Some(message.into()))
    }

    /// The requested object does not exist: 404.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(Some(message.into()))
    }

    /// Unexpected server-side failure: 500.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(Some(message.into()))
    }

    /// A call to an external service failed: 500.
    pub fn external_service(message: impl Into<String>) -> Self {
        Self::ExternalService(Some(message.into()))
    }

    /// The client is not allowed to perform the operation: 403.
    pub fn access_denied(message: impl Into<String>) -> Self {
        Self::AccessDenied(Some(message.into()))
    }

    /// Machine-readable error code.
    pub fn error_type(&self) -> ErrorType {
        match self {
            Self::Internal(_) => ErrorType::InternalError,
            Self::ExternalService(_) => ErrorType::ExternalServiceError,
            Self::BadRequest(_) => ErrorType::BadRequest,
            Self::NotFound(_) => ErrorType::NotFound,
            Self::AccessDenied(_) => ErrorType::AccessDenied,
        }
    }

    /// HTTP status code of the error class.
    pub fn code(&self) -> u16 {
        match self {
            Self::Internal(_) | Self::ExternalService(_) => 500,
            Self::BadRequest(_) => 400,
            Self::NotFound(_) => 404,
            Self::AccessDenied(_) => 403,
        }
    }

    /// Human-readable description returned to the client.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Internal(message)
            | Self::ExternalService(message)
            | Self::BadRequest(message)
            | Self::NotFound(message)
            | Self::AccessDenied(message) => message.as_deref(),
        }
    }
}

impl std::fmt::Display for OtherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "message: {:?}, code: {}",
            self.message().unwrap_or(""),
            self.code()
        )
    }
}
