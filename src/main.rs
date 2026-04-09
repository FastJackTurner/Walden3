use axum::{routing::get, routing::post, Router};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use std::sync::Mutex;
use std::sync::Arc;

use W3::db::{get_pool};
use W3::app::{AppState, Todos};
use W3::routes;


#[tokio::main]
async fn main() {    

    let app_state = Arc::new(AppState {
        todos: Todos {todos: Mutex::new(vec![])},
        pool: get_pool().await
    });
    
    let serve_dir = ServeDir::new("Assets");
    let api_router = Router::new()
        .route("/hello", get(routes::hello_from_the_server))
        .route("/sql_test", get(routes::sql_test))
        .route("/todos", post(routes::add_todo));
    
    let app = Router::new()
        .route("/", get(routes::index))
        .route("/schedule", get(routes::schedule))
        .route("/sessionnote", get(routes::session_note))
        .route("/create_tp", get(routes::create_tp))
        .route("/view_tp", get(routes::view_tp))
        .route("/view_goals", post(routes::view_goals))
        .route("/post_tp", post(routes::post_tp))
        .route("/create_goal", post(routes::create_goal))
        .route("/post_goal", post(routes::post_goal))
        .route("/review_notes", get(routes::review_notes))
        .route("/make_schedule", get(routes::make_schedule))
        //.route("/post_schedule", post(routes::post_schedule))
        .nest_service("/Assets", serve_dir)
        .nest("/api", api_router)
        .with_state(app_state);
        

        
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}


