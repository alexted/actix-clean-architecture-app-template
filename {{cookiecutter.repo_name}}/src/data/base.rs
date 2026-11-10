//! Base contract of the data-layer repositories.
//!
//! ORM rows stay inside the repositories, DTOs go out. Repositories execute their statements
//! in the connections they take from the pool and never commit: every statement is
//! auto-committed by PostgreSQL, the pool owns the connections.

use actix_threadpool::BlockingError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::infrastructure::core::errors::exceptions::OtherError;

/// Result of the data-layer operations.
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// A storage operation failed.
#[derive(Debug)]
pub struct RepositoryError {
    pub message: String,
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<diesel::result::Error> for RepositoryError {
    fn from(error: diesel::result::Error) -> Self {
        RepositoryError {
            message: error.to_string(),
        }
    }
}

impl From<::r2d2::Error> for RepositoryError {
    fn from(error: ::r2d2::Error) -> Self {
        RepositoryError {
            message: error.to_string(),
        }
    }
}

impl<T: std::fmt::Debug> From<BlockingError<T>> for RepositoryError {
    fn from(error: BlockingError<T>) -> Self {
        RepositoryError {
            message: error.to_string(),
        }
    }
}

/// Storage failures are reported as request errors, like in the service layer before the
/// refactoring.
impl From<RepositoryError> for OtherError {
    fn from(error: RepositoryError) -> Self {
        OtherError::bad_request(error.message)
    }
}

/// Page of rows returned by the repositories.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResultPaging<T> {
    pub total: i64,
    pub items: Vec<T>,
}

/// Read filters with paging: the number of rows to return and to skip.
///
/// Repositories take the values through this contract, so every filters model plays along.
pub trait QueryParams {
    /// Number of rows returned.
    fn limit(&self) -> i64;
    /// Number of rows skipped.
    fn offset(&self) -> i64;
}

/// Common contract of the repositories.
///
/// Type parameters: `Model` is the Diesel row, `Dto` the DTO returned to the domain layer,
/// `Create` the model with the data of a new row and `Filters` the model with read filters.
#[async_trait]
pub trait BaseRepository {
    /// The Diesel row type of the storage.
    type Model;
    /// The DTO returned to the domain layer.
    type Dto;
    /// The model with the data of a new row.
    type Create;
    /// The model with the read filters.
    type Filters;

    /// Insert a row and return it as a DTO.
    async fn create(&self, obj_data: &Self::Create) -> RepositoryResult<Self::Dto>;

    /// Return the rows matching the filters as a page of DTOs.
    async fn get(&self, filters: Self::Filters) -> RepositoryResult<ResultPaging<Self::Dto>>;
}
