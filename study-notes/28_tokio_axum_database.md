# Chapter 28: Tokio + Axum + Database — Realistic Backend Example

> **Connection to the bigger picture**: This chapter puts it all together — a complete, working backend using Axum, Tokio, and PostgreSQL. This is the architecture pattern your Solana trading backend will follow.

---

## Complete Example: Order Management API

### `Cargo.toml`

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenvy = "0.15"
tower-http = { version = "0.6", features = ["cors", "trace"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### `main.rs`

```rust
use axum::{
    routing::{get, post},
    Router, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// ── Models ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Order {
    id: uuid::Uuid,
    symbol: String,
    side: String,     // "buy" or "sell"
    price: f64,
    quantity: f64,
    status: String,   // "pending", "filled", "cancelled"
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
struct CreateOrderRequest {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

// ── State ──

struct AppState {
    db: sqlx::PgPool,
}

// ── Error Handling ──

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// ── Handlers ──

async fn health_check() -> &'static str {
    "OK"
}

async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<Order>), AppError> {
    if req.price <= 0.0 || req.quantity <= 0.0 {
        return Err(AppError::BadRequest("Price and quantity must be positive".into()));
    }
    
    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (id, symbol, side, price, quantity, status, created_at)
        VALUES ($1, $2, $3, $4, $5, 'pending', NOW())
        RETURNING *
        "#
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&req.symbol)
    .bind(&req.side)
    .bind(req.price)
    .bind(req.quantity)
    .fetch_one(&state.db)
    .await?;
    
    tracing::info!(order_id = %order.id, symbol = %order.symbol, "Order created");
    Ok((StatusCode::CREATED, Json(order)))
}

async fn get_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Order>, AppError> {
    let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", id)))?;
    
    Ok(Json(order))
}

async fn list_orders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Order>>, AppError> {
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(orders))
}

// ── Main ──

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load config
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/trading".into());
    
    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;
    
    tracing::info!("Connected to database");
    
    // Create shared state
    let state = Arc::new(AppState { db: pool });
    
    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/orders", get(list_orders).post(create_order))
        .route("/orders/{id}", get(get_order))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

---

## What This Example Demonstrates

1. **Async database queries** — `sqlx` with `.await`
2. **Shared state** — `Arc<AppState>` across all handlers
3. **Connection pooling** — `PgPool` with max 20 connections
4. **Error handling** — Custom `AppError` implementing `IntoResponse`
5. **Extractors** — `State`, `Path`, `Json`
6. **Middleware** — CORS and tracing layers
7. **Structured logging** — `tracing` crate

This is the foundation pattern for your Solana trading backend.

---

> **Next**: [Chapter 29: Concurrency Safety in Backend Systems](./29_concurrency_safety_backend.md)
