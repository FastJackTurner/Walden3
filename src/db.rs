use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Consultant {
    pub co_id: i32,
    pub co_f_name: String,
    pub co_l_name: String,
}

pub async fn get_pool() -> PgPool {
    match PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://postgres:postgres@127.0.0.1:5432/W3")
        .await
    {
        Ok(pool) => {
            println!("connect succes");
            pool
        }
        Err(e) => {
            eprintln!("err connecting to db: {}", e);
            std::process::exit(1);
        }
    }
}
