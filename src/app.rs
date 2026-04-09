use serde::Deserialize;
use sqlx::postgres::PgPool;
use std::sync::Mutex;

pub struct AppState {
    pub pool: PgPool,
    pub todos: Todos,
}

#[derive(Deserialize)]
pub struct Todos {
    pub todos: Mutex<Vec<String>>,
}
