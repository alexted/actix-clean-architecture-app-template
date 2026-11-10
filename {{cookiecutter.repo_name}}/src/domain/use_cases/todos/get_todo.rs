//! Get a todo by id.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::data::base::BaseRepository;
use crate::data::todos::dto::TodoFilters;
use crate::data::todos::repository::TodoRepository;
use crate::data::todos::TodoDTO;
use crate::domain::use_cases::base::BaseUseCase;
use crate::infrastructure::core::errors::exceptions::OtherError;

/// Id of the requested todo.
#[derive(Debug)]
pub struct GetTodoRequest {
    pub id: i32,
}

/// The requested todo.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTodoResponse {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// Read one todo.
pub struct GetTodoUseCase {
    todo_repository: TodoRepository,
}

impl GetTodoUseCase {
    /// Inject the todos repository.
    pub fn new(todo_repository: TodoRepository) -> Self {
        GetTodoUseCase { todo_repository }
    }
}

#[async_trait]
impl BaseUseCase<GetTodoRequest, GetTodoResponse> for GetTodoUseCase {
    /// Return the todo.
    async fn execute(&self, request_object: GetTodoRequest) -> Result<GetTodoResponse, OtherError> {
        let filters = TodoFilters {
            id: Some(vec![request_object.id]),
            ..Default::default()
        };
        let page = self.todo_repository.get(filters).await?;
        let todo: TodoDTO = page.items.into_iter().next().ok_or_else(|| {
            OtherError::bad_request(format!("Todo {} not found", request_object.id))
        })?;
        Ok(todo.into())
    }
}

impl From<TodoDTO> for GetTodoResponse {
    fn from(todo: TodoDTO) -> Self {
        GetTodoResponse {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            completed: todo.completed,
        }
    }
}
