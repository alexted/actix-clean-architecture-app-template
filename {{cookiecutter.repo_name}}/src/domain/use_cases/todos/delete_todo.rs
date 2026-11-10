//! Delete a todo.

use async_trait::async_trait;

use crate::data::todos::repository::TodoRepository;
use crate::domain::use_cases::base::BaseUseCase;
use crate::infrastructure::core::errors::exceptions::OtherError;

/// Id of the todo to delete.
#[derive(Debug)]
pub struct DeleteTodoRequest {
    pub id: i32,
}

/// Delete a todo by id.
pub struct DeleteTodoUseCase {
    todo_repository: TodoRepository,
}

impl DeleteTodoUseCase {
    /// Inject the todos repository.
    pub fn new(todo_repository: TodoRepository) -> Self {
        DeleteTodoUseCase { todo_repository }
    }
}

#[async_trait]
impl BaseUseCase<DeleteTodoRequest, ()> for DeleteTodoUseCase {
    /// Delete the todo; deleting a missing todo is not an error.
    async fn execute(&self, request_object: DeleteTodoRequest) -> Result<(), OtherError> {
        self.todo_repository.delete(request_object.id).await?;
        Ok(())
    }
}
