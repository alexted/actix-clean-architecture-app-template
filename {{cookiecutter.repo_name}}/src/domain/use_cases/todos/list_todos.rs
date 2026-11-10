//! List todos.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::data::base::BaseRepository;
use crate::data::todos::dto::TodoFilters;
use crate::data::todos::repository::TodoRepository;
use crate::data::todos::TodoDTO;
use crate::domain::use_cases::base::BaseUseCase;
use crate::infrastructure::core::errors::exceptions::OtherError;

/// Listing parameters of the todos.
#[derive(Debug, Deserialize)]
pub struct ListTodosRequest {
    /// Number of todos returned; defaults to the storage page size.
    pub limit: Option<i64>,
    /// Number of todos skipped.
    pub offset: Option<i64>,
    /// Title filter; accepted by the listing contract but not applied by the storage yet.
    pub title: Option<String>,
}

/// Page of todos.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListTodosResponse {
    pub total: i64,
    pub items: Vec<ListTodosItem>,
}

/// One todo of the listing.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListTodosItem {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// Read a page of todos.
pub struct ListTodosUseCase {
    todo_repository: TodoRepository,
}

impl ListTodosUseCase {
    /// Inject the todos repository.
    pub fn new(todo_repository: TodoRepository) -> Self {
        ListTodosUseCase { todo_repository }
    }
}

#[async_trait]
impl BaseUseCase<ListTodosRequest, ListTodosResponse> for ListTodosUseCase {
    /// Return the matching page of todos.
    async fn execute(
        &self,
        request_object: ListTodosRequest,
    ) -> Result<ListTodosResponse, OtherError> {
        let filters = TodoFilters {
            id: None,
            limit: request_object.limit,
            offset: request_object.offset,
        };
        let page = self.todo_repository.get(filters).await?;
        Ok(ListTodosResponse {
            total: page.total,
            items: page.items.into_iter().map(ListTodosItem::from).collect(),
        })
    }
}

impl From<TodoDTO> for ListTodosItem {
    fn from(todo: TodoDTO) -> Self {
        ListTodosItem {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            completed: todo.completed,
        }
    }
}
