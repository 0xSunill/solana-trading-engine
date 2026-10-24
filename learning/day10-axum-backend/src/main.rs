
use axum::{
    extract::State,
    routing::get,
    Router,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone)]
struct AppState {
    app_name: String,
    health_checks: Arc<AtomicUsize>,
}

async fn health(State(state): State<AppState>) -> String {
    let count = state.health_checks.fetch_add(1, Ordering::Relaxed) + 1;

    format!("{} is running. Health checks: {}", state.app_name, count)
}

async fn stats(State(state): State<AppState>) -> String {
    let count = state.health_checks.load(Ordering::Relaxed);

    format!("Total health checks: {count}")
}

#[tokio::main]
async fn main() {
    let state = AppState {
        app_name: "Trading Backend".to_string(),
        health_checks: Arc::new(AtomicUsize::new(0)),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/stats", get(stats))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on port 3000");

    axum::serve(listener, app).await.unwrap();
}
