
use axum::{
    routing::get,
    Json,
    Router,
};
use serde::Serialize;

#[derive(Serialize)]
struct PriceResponse {
    symbol: String,
    price: f64,
}

async fn health() -> &'static str {
    "Trading backend is running!"
}

async fn price() -> Json<PriceResponse> {
    Json(PriceResponse {
        symbol: "SOL".to_string(),
        price: 200.0,
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/price", get(price));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");

    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
