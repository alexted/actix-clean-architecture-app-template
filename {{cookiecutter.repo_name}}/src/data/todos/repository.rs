//! Todo persistence.

use actix_threadpool::run;
use async_trait::async_trait;
use diesel::prelude::*;

use crate::data::base::{
    BaseRepository, QueryParams, RepositoryError, RepositoryResult, ResultPaging,
};
use crate::data::todos::dto::{TodoDTO, TodoFilters};
use crate::domain::use_cases::todos::create_todo::CreateTodoRequest;
use crate::infrastructure::clients::postgres::engine::PostgresPool;
use crate::infrastructure::clients::postgres::models::todo::{CreateTodoDiesel, TodoDiesel};
use crate::infrastructure::clients::postgres::schema::todos::dsl::{id, todos};

/// Todos storage working on the PostgreSQL connection pool.
#[derive(Clone)]
pub struct TodoRepository {
    pool: PostgresPool,
}

impl TodoRepository {
    /// Bind the repository to the connection pool of the application.
    pub fn new(pool: PostgresPool) -> Self {
        TodoRepository { pool }
    }
}

#[async_trait]
impl BaseRepository for TodoRepository {
    type Model = TodoDiesel;
    type Dto = TodoDTO;
    type Create = CreateTodoRequest;
    type Filters = TodoFilters;

    async fn create(&self, obj_data: &Self::Create) -> RepositoryResult<Self::Dto> {
        let new_row = CreateTodoDiesel {
            title: obj_data.title.clone(),
            description: obj_data.description.clone(),
        };
        let pool = self.pool.clone();
        let todo: TodoDiesel = run(move || -> RepositoryResult<TodoDiesel> {
            let mut conn = pool.get()?;
            Ok(diesel::insert_into(todos)
                .values(new_row)
                .get_result::<TodoDiesel>(&mut conn)?)
        })
        .await
        .map_err(RepositoryError::from)?;

        Ok(todo.into())
    }

    async fn get(&self, filters: Self::Filters) -> RepositoryResult<ResultPaging<Self::Dto>> {
        let pool = self.pool.clone();
        let rows: Vec<TodoDiesel> = run(move || -> RepositoryResult<Vec<TodoDiesel>> {
            let mut conn = pool.get()?;
            let mut query = todos.into_boxed();
            if let Some(ids) = &filters.id {
                query = query.filter(id.eq_any(ids));
            }
            Ok(query
                .limit(filters.limit())
                .offset(filters.offset())
                .load::<TodoDiesel>(&mut conn)?)
        })
        .await
        .map_err(RepositoryError::from)?;

        Ok(ResultPaging {
            // The total of all matching rows is not computed yet; the listing contract keeps
            // the field of the previous implementation.
            total: 0,
            items: rows.into_iter().map(TodoDTO::from).collect(),
        })
    }
}

impl TodoRepository {
    /// Delete a todo; deleting a missing todo is a no-op.
    pub async fn delete(&self, todo_id: i32) -> RepositoryResult<()> {
        let pool = self.pool.clone();
        run(move || -> RepositoryResult<usize> {
            let mut conn = pool.get()?;
            Ok(diesel::delete(todos)
                .filter(id.eq(todo_id))
                .execute(&mut conn)?)
        })
        .await
        .map_err(RepositoryError::from)?;

        Ok(())
    }
}
