
use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    quantity: f64,
}

#[derive(Deserialize)]
struct CreateOrder {
    symbol: String,
    side: String,
    quantity: f64,
}

#[derive(Deserialize)]
struct OrderFilters {
    symbol: Option<String>,
}

async fn get_order(Path(id): Path<u64>) -> Json<Order> {
    Json(Order {
        id,
        symbol: "SOL".to_string(),
        side: "buy".to_string(),
        quantity: 1.5,
    })
}

async fn list_orders(
    Query(filters): Query<OrderFilters>,
) -> String {
    match filters.symbol {
        Some(symbol) => format!("Filtering orders for {symbol}"),
        None => "Returning all orders".to_string(),
    }
}

async fn create_order(
    Json(payload): Json<CreateOrder>,
) -> Json<Order> {
    Json(Order {
        id: 1001,
        symbol: payload.symbol,
        side: payload.side,
        quantity: payload.quantity,
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/orders", get(list_orders).post(create_order))
        .route("/orders/{id}", get(get_order));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on port 3000");

    axum::serve(listener, app).await.unwrap();
}
