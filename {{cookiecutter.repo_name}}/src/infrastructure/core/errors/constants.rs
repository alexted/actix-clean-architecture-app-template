//! Error codes of the error response body.

/// Machine-readable error codes returned in the `error` field of error responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    ValidationError,
    InternalError,
    ExternalServiceError,
    BadRequest,
    NotFound,
    ExpiredToken,
    BadToken,
    AccessDenied,
}

impl ErrorType {
    /// Machine-readable code of the error type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationError => "VALIDATION_ERROR",
            Self::InternalError => "INTERNAL_ERROR",
            Self::ExternalServiceError => "EXTERNAL_SERVICE_ERROR",
            Self::BadRequest => "BAD_REQUEST",
            Self::NotFound => "NOT_FOUND",
            Self::ExpiredToken => "EXPIRED_TOKEN",
            Self::BadToken => "BAD_TOKEN",
            Self::AccessDenied => "ACCESS_DENIED",
        }
    }
}
