use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, AppState};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/orders", post(handlers::create_order))
        .route("/orders/{id}", get(handlers::get_order))
        .with_state(state)
}