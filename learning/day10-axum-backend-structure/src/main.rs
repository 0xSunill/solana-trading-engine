mod handlers;
mod models;
mod order_service;
mod routes;

use std::sync::Arc;
use order_service::OrderService;

#[derive(Clone)]
pub struct AppState {
    pub order_service: Arc<OrderService>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        order_service: Arc::new(OrderService::default()),
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Trading backend running on port 3000");

    axum::serve(listener, app).await.unwrap();
}