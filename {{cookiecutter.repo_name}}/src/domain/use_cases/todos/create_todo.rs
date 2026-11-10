//! Create a todo.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::data::base::BaseRepository;
use crate::data::todos::repository::TodoRepository;
use crate::data::todos::TodoDTO;
use crate::domain::use_cases::base::BaseUseCase;
use crate::infrastructure::core::errors::exceptions::OtherError;

/// Data of a new todo.
#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: String,
}

/// The created todo.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTodoResponse {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// Store a new todo.
pub struct CreateTodoUseCase {
    todo_repository: TodoRepository,
}

impl CreateTodoUseCase {
    /// Inject the todos repository.
    pub fn new(todo_repository: TodoRepository) -> Self {
        CreateTodoUseCase { todo_repository }
    }
}

#[async_trait]
impl BaseUseCase<CreateTodoRequest, CreateTodoResponse> for CreateTodoUseCase {
    /// Store the todo and return it with the generated id.
    async fn execute(
        &self,
        request_object: CreateTodoRequest,
    ) -> Result<CreateTodoResponse, OtherError> {
        let todo: TodoDTO = self.todo_repository.create(&request_object).await?;
        Ok(todo.into())
    }
}

impl From<TodoDTO> for CreateTodoResponse {
    fn from(todo: TodoDTO) -> Self {
        CreateTodoResponse {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            completed: todo.completed,
        }
    }
}
