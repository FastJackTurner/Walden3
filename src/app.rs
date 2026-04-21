use serde::Deserialize;
use sqlx::postgres::PgPool;
use std::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Deserialize)]
pub struct Todos {
    pub todos: Mutex<Vec<String>>,
}
