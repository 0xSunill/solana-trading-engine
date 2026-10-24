// use sqlx::postgres::PgPoolOptions;

// #[tokio::main]
// async fn main() -> Result<(), sqlx::Error> {
//     let database_url = std::env::var("DATABASE_URL")
//         .expect("DATABASE_URL must be set");

//     let pool = PgPoolOptions::new()
//         .max_connections(5)
//         .connect(&database_url)
//         .await?;

//     let value: i32 = sqlx::query_scalar("SELECT 1")
//         .fetch_one(&pool)
//         .await?;

//     println!("Database connected! Result: {value}");

//     Ok(())
// }

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Deserialize)]
struct CreateOrder {
    symbol: String,
    side: String,
    quantity: f64,
}

#[derive(Serialize)]
struct Order {
    id: i64,
    symbol: String,
    side: String,
    quantity: f64,
}

type ApiError = (StatusCode, String);

fn database_error(error: sqlx::Error) -> ApiError {
    eprintln!("Database error: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database operation failed".to_string(),
    )
}

// POST /orders — save an order
async fn create_order(
    State(state): State<AppState>,
    Json(input): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), ApiError> {
    if input.symbol.trim().is_empty()
        || !matches!(input.side.as_str(), "buy" | "sell")
        || !input.quantity.is_finite()
        || input.quantity <= 0.0
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid order details".to_string()));
    }

    let (id, symbol, side, quantity) = sqlx::query_as::<_, (i64, String, String, f64)>(
        "INSERT INTO orders (symbol, side, quantity)
             VALUES ($1, $2, $3)
             RETURNING id, symbol, side, quantity::double precision",
    )
    .bind(input.symbol)
    .bind(input.side)
    .bind(input.quantity)
    .fetch_one(&state.pool)
    .await
    .map_err(database_error)?;

    Ok((
        StatusCode::CREATED,
        Json(Order {
            id,
            symbol,
            side,
            quantity,
        }),
    ))
}

// GET /orders/{id} — retrieve a saved order
async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Order>, ApiError> {
    let result = sqlx::query_as::<_, (i64, String, String, f64)>(
        "SELECT id, symbol, side, quantity::double precision
             FROM orders
             WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(database_error)?;

    match result {
        Some((id, symbol, side, quantity)) => Ok(Json(Order {
            id,
            symbol,
            side,
            quantity,
        })),
        None => Err((StatusCode::NOT_FOUND, format!("Order {id} not found"))),
    }
}

async fn list_orders(State(state): State<AppState>) -> Result<Json<Vec<Order>>, ApiError> {
    let rows = sqlx::query_as::<_, (i64, String, String, f64)>(
        "SELECT id, symbol, side, quantity::double precision
             FROM orders
             ORDER BY id ASC
             LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(database_error)?;

    let orders: Vec<Order> = rows
        .into_iter()
        .map(|(id, symbol, side, quantity)| Order {
            id,
            symbol,
            side,
            quantity,
        })
        .collect();

    Ok(Json(orders))
}

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Connected to PostgreSQL!");

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/orders", get(list_orders).post(create_order))
        .route("/orders/{id}", get(get_order))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind port");

    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("Server failed");
}
