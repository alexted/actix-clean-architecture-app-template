//! Todos CRUD endpoints.

use actix_web::{web, HttpResponse, Scope};

use crate::data::todos::repository::TodoRepository;
use crate::domain::use_cases::base::BaseUseCase;
use crate::domain::use_cases::todos::{
    CreateTodoRequest, CreateTodoResponse, CreateTodoUseCase, DeleteTodoRequest, DeleteTodoUseCase,
    GetTodoRequest, GetTodoResponse, GetTodoUseCase, ListTodosRequest, ListTodosResponse,
    ListTodosUseCase,
};
use crate::infrastructure::core::errors::exceptions::OtherError;

/// Routes of the todos storage.
pub fn routes() -> Scope {
    web::scope("/todos")
        .route("", web::post().to(create_todo))
        .route("", web::get().to(list_todos))
        .route("/{id}", web::get().to(get_todo))
        .route("/{id}", web::delete().to(delete_todo))
}

/// Create a todo.
async fn create_todo(
    todo_repository: web::Data<TodoRepository>,
    post_data: web::Json<CreateTodoRequest>,
) -> Result<web::Json<CreateTodoResponse>, OtherError> {
    let use_case = CreateTodoUseCase::new(todo_repository.get_ref().clone());
    let response = use_case.execute(post_data.into_inner()).await?;
    Ok(web::Json(response))
}

/// List the todos.
async fn list_todos(
    todo_repository: web::Data<TodoRepository>,
    params: web::Query<ListTodosRequest>,
) -> Result<web::Json<ListTodosResponse>, OtherError> {
    let use_case = ListTodosUseCase::new(todo_repository.get_ref().clone());
    let response = use_case.execute(params.into_inner()).await?;
    Ok(web::Json(response))
}

/// Get a todo by id.
async fn get_todo(
    todo_repository: web::Data<TodoRepository>,
    path: web::Path<i32>,
) -> Result<web::Json<GetTodoResponse>, OtherError> {
    let use_case = GetTodoUseCase::new(todo_repository.get_ref().clone());
    let response = use_case
        .execute(GetTodoRequest {
            id: path.into_inner(),
        })
        .await?;
    Ok(web::Json(response))
}

/// Delete a todo; deleting a missing todo also succeeds.
async fn delete_todo(
    todo_repository: web::Data<TodoRepository>,
    path: web::Path<i32>,
) -> Result<HttpResponse, OtherError> {
    let use_case = DeleteTodoUseCase::new(todo_repository.get_ref().clone());
    use_case
        .execute(DeleteTodoRequest {
            id: path.into_inner(),
        })
        .await?;
    Ok(HttpResponse::NoContent().finish())
}
