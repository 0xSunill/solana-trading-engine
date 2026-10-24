use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    models::{CreateOrderRequest, Order},
    AppState,
};

pub async fn create_order(
    State(state): State<AppState>,
    Json(input): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
    let order = state
        .order_service
        .create_order(input)
        .map_err(|error| (StatusCode::BAD_REQUEST, error))?;

    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Order>, (StatusCode, String)> {
    let order = state
        .order_service
        .get_order(id)
        .ok_or((
            StatusCode::NOT_FOUND,
            "Order not found".to_string(),
        ))?;

    Ok(Json(order))
}