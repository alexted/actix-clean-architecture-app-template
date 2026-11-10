//! Todo ORM model: the Diesel rows of the `todos` table.

use diesel::prelude::*;

use crate::infrastructure::clients::postgres::schema::todos;

/// Row of the `todos` table.
#[derive(Queryable)]
pub struct TodoDiesel {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// Data of a new row of the `todos` table.
#[derive(Insertable)]
#[diesel(table_name = todos)]
pub struct CreateTodoDiesel {
    pub title: String,
    pub description: String,
}
