//! Base contract of the use cases.

use async_trait::async_trait;

use crate::infrastructure::core::errors::exceptions::OtherError;

/// One business operation: takes a request model and returns the operation result.
///
/// Concrete use cases are built by the endpoints with the repositories they need, like the
/// reference application builds them by dependency injection.
#[async_trait]
pub trait BaseUseCase<Request, Response> {
    /// Run the operation.
    async fn execute(&self, request_object: Request) -> Result<Response, OtherError>;
}
