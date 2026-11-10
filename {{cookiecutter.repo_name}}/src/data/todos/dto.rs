//! Data transfer objects of the todos storage.

use crate::data::base::QueryParams;
use crate::infrastructure::clients::postgres::models::todo::TodoDiesel;

/// Page size applied when the request does not give one.
pub const DEFAULT_LIMIT: i64 = 25;

/// Page offset applied when the request does not give one.
pub const DEFAULT_OFFSET: i64 = 0;

/// Todo as stored in the database.
#[derive(Debug, Clone)]
pub struct TodoDTO {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// Read filters of the todos storage.
#[derive(Debug, Clone, Default)]
pub struct TodoFilters {
    /// Return only the todos with these ids.
    pub id: Option<Vec<i32>>,
    /// Number of rows returned.
    pub limit: Option<i64>,
    /// Number of rows skipped.
    pub offset: Option<i64>,
}

/// The paging contract of the common repository base: defaults when the request gives none.
impl QueryParams for TodoFilters {
    fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT)
    }

    fn offset(&self) -> i64 {
        self.offset.unwrap_or(DEFAULT_OFFSET)
    }
}

/// Convert an ORM row into its DTO.
impl From<TodoDiesel> for TodoDTO {
    fn from(todo: TodoDiesel) -> Self {
        TodoDTO {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            completed: todo.completed,
        }
    }
}
