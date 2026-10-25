
// use axum::{
//     extract::{Path, Query},
//     routing::get,
//     Json, Router,
// };
// use serde::{Deserialize, Serialize};

// #[derive(Serialize)]
// struct Order {
//     id: u64,
//     symbol: String,
//     side: String,
//     quantity: f64,
// }

// #[derive(Deserialize)]
// struct CreateOrder {
//     symbol: String,
//     side: String,
//     quantity: f64,
// }

// #[derive(Deserialize)]
// struct OrderFilters {
//     symbol: Option<String>,
// }

// async fn get_order(Path(id): Path<u64>) -> Json<Order> {
//     Json(Order {
//         id,
//         symbol: "SOL".to_string(),
//         side: "buy".to_string(),
//         quantity: 1.5,
//     })
// }

// async fn list_orders(
//     Query(filters): Query<OrderFilters>,
// ) -> String {
//     match filters.symbol {
//         Some(symbol) => format!("Filtering orders for {symbol}"),
//         None => "Returning all orders".to_string(),
//     }
// }

// async fn create_order(
//     Json(payload): Json<CreateOrder>,
// ) -> Json<Order> {
//     Json(Order {
//         id: 1001,
//         symbol: payload.symbol,
//         side: payload.side,
//         quantity: payload.quantity,
//     })
// }

// #[tokio::main]
// async fn main() {
//     let app = Router::new()
//         .route("/orders", get(list_orders).post(create_order))
//         .route("/orders/{id}", get(get_order));

//     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//         .await
//         .unwrap();

//     println!("Server running on port 3000");

//     axum::serve(listener, app).await.unwrap();
// }








// use axum::{
//     extract::Path,
//     http::StatusCode,
//     routing::get,
//     Json,
//     Router,
// };
// use serde::{Deserialize, Serialize};

// #[derive(Serialize)]
// struct Order {
//     id: u64,
//     symbol: String,
//     side: String,
//     quantity: f64,
// }

// #[derive(Deserialize)]
// struct CreateOrder {
//     symbol: String,
//     side: String,
//     quantity: f64,
// }

// async fn get_order(
//     Path(id): Path<u64>,
// ) -> Result<Json<Order>, (StatusCode, String)> {
//     if id == 0 {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             "Order ID must be greater than zero".to_string(),
//         ));
//     }

//     if id != 123 {
//         return Err((
//             StatusCode::NOT_FOUND,
//             format!("Order {id} was not found"),
//         ));
//     }

//     Ok(Json(Order {
//         id: 123,
//         symbol: "SOL".to_string(),
//         side: "buy".to_string(),
//         quantity: 1.5,
//     }))
// }

// async fn create_order(
//     Json(payload): Json<CreateOrder>,
// ) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
//     if payload.quantity <= 0.0 {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             "Quantity must be greater than zero".to_string(),
//         ));
//     }

//     if payload.side != "buy" && payload.side != "sell" {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             "Side must be buy or sell".to_string(),
//         ));
//     }

//     let order = Order {
//         id: 1001,
//         symbol: payload.symbol,
//         side: payload.side,
//         quantity: payload.quantity,
//     };

//     Ok((StatusCode::CREATED, Json(order)))
// }

// #[tokio::main]
// async fn main() {
//     let app = Router::new()
//         .route("/orders/{id}", get(get_order))
//         .route("/orders", axum::routing::post(create_order));

//     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//         .await
//         .unwrap();

//     axum::serve(listener, app).await.unwrap();
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
    status: String,
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

    let (id, symbol, side, quantity, status) =
        sqlx::query_as::<_, (i64, String, String, f64, String)>(
            "INSERT INTO orders (symbol, side, quantity)
         VALUES ($1, $2, $3)
         RETURNING id, symbol, side,
                   quantity::double precision, status",
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
            status,
        }),
    ))
}

// GET /orders/{id} — retrieve a saved order
async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Order>, ApiError> {
    let result = sqlx::query_as::<_, (i64, String, String, f64, String)>(
        "SELECT id, symbol, side, quantity::double precision, status
             FROM orders
             WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(database_error)?;

    match result {
        Some((id, symbol, side, quantity,status)) => Ok(Json(Order {
            id,
            symbol,
            side,
            quantity,
            status,
        })),
        None => Err((StatusCode::NOT_FOUND, format!("Order {id} not found"))),
    }
}

async fn list_orders(State(state): State<AppState>) -> Result<Json<Vec<Order>>, ApiError> {
    let rows = sqlx::query_as::<_, (i64, String, String, f64,String)>(
        "SELECT id, symbol, side, quantity::double precision, status
             FROM orders
             ORDER BY id ASC
             LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(database_error)?;

    let orders: Vec<Order> = rows
        .into_iter()
        .map(|(id, symbol, side, quantity  ,status)| Order {
            id,
            symbol,
            side,
            quantity,
            status,
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
