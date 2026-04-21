use axum::middleware::Next;
use axum::response::Response;
use axum::{Router, routing::get, routing::post};
use axum_extra::extract::cookie::SameSite;
use axum_login::{AuthManagerLayerBuilder, login_required};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::EnvFilter;

use W3::app::AppState;
use W3::db::get_pool;
use W3::login::Backend;
use W3::routes;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("W3=error,tower_http=warn"))
        .unwrap(),
    )
    .init();

  let app_state = AppState {
    pool: get_pool().await,
  };

  let backend = Backend {
    db: app_state.pool.clone(),
  };

  let session_store = PostgresStore::new(app_state.pool.clone());
  session_store.migrate().await.unwrap();

  let session_layer = SessionManagerLayer::new(session_store)
    .with_expiry(Expiry::OnInactivity(Duration::seconds(60 * 60)))
    .with_secure(false) //fix in prod
    .with_same_site(SameSite::Lax)
    .with_name("W3 cookie")
    .with_http_only(true);
  let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

  let serve_dir = ServeDir::new("Assets");

  let protected_routes = Router::new()
    .route("/schedule", get(routes::schedule))
    .route("/sessionnote", get(routes::session_note))
    //
    .route("/tp", get(routes::tp))
    .route("/view_tp", get(routes::view_tp))
    .route("/create_tp", get(routes::create_tp))
    .route("/view_goals", post(routes::view_goals))
    .route("/post_tp", post(routes::post_tp))
    //
    .route("/create_goal", post(routes::create_goal))
    .route("/post_goal", post(routes::post_goal))
    //
    .route("/review_notes", get(routes::review_notes))
    .route("/schedules", get(routes::schedules))
    .route("/view_schedules", get(routes::view_schedules))
    .route("/make_schedule", get(routes::make_schedule))
    .route("/post_schedule", post(routes::post_tech_schedule))
    //
    .route("/view_clients", get(routes::view_clients))
    //
    .route("/logout", get(routes::logout))
    .route("/get_my_id", get(routes::get_my_id))
    .route_layer(login_required!(Backend, login_url = "/login"));

  let unprotected_routes = Router::new()
    .route("/login", post(routes::login))
    .route("/login", get(routes::get_login))
    .route("/register", post(routes::register));

  let app = Router::new()
    .nest_service("/Assets", serve_dir)
    .merge(protected_routes)
    .merge(unprotected_routes)
    .layer(auth_layer)
    .layer(axum::middleware::from_fn(debug_middleware))
    .layer(TraceLayer::new_for_http())
    .with_state(app_state);

  let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

  axum::serve(listener, app).await.unwrap();
}

async fn debug_middleware(req: axum::extract::Request, next: Next) -> Response {
  let res = next.run(req).await;
  println!("Response status: {}", res.status());
  res
}
