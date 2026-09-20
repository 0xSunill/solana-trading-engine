# Chapter 26: Axum Framework

> **Connection to the bigger picture**: Axum is the leading async web framework for Rust, built on top of Tokio and the `tower` service ecosystem. If you're building a Solana backend with HTTP APIs, WebSocket endpoints, or webhook handlers, Axum is the framework you'll use.

---

## What Axum Is

Axum is an ergonomic, modular web framework that uses Rust's type system to its fullest:

```
Client ──▶ Axum Router ──▶ Handler (async fn) ──▶ Response
              │
              ├── Extractors (parse request data)
              ├── State (shared application data)
              ├── Middleware (auth, logging, CORS)
              └── Error handling
```

### Basic Axum Application

```rust
use axum::{
    routing::{get, post},
    Router, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// Application state
struct AppState {
    prices: RwLock<std::collections::HashMap<String, f64>>,
}

#[tokio::main]
async fn main() {
    // Create shared state
    let state = Arc::new(AppState {
        prices: RwLock::new(std::collections::HashMap::new()),
    });
    
    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/price/{symbol}", get(get_price))
        .route("/price", post(update_price))
        .with_state(state);
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// Handler: no extractors needed
async fn health_check() -> &'static str {
    "OK"
}

// Handler: extract path parameter and shared state
async fn get_price(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<Json<PriceResponse>, StatusCode> {
    let prices = state.prices.read().await;
    match prices.get(&symbol) {
        Some(&price) => Ok(Json(PriceResponse { symbol, price })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
struct UpdatePriceRequest {
    symbol: String,
    price: f64,
}

#[derive(Serialize)]
struct PriceResponse {
    symbol: String,
    price: f64,
}

// Handler: extract JSON body and shared state
async fn update_price(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdatePriceRequest>,
) -> StatusCode {
    let mut prices = state.prices.write().await;
    prices.insert(payload.symbol, payload.price);
    StatusCode::OK
}
```

---

## Extractors

Extractors parse data from requests automatically:

| Extractor | What it extracts | Example |
|-----------|-----------------|---------|
| `Path<T>` | URL path parameters | `/users/{id}` → `Path(id): Path<u64>` |
| `Query<T>` | URL query parameters | `?page=1&limit=10` → `Query(params): Query<Params>` |
| `Json<T>` | JSON request body | `Json(body): Json<CreateOrder>` |
| `State<T>` | Shared application state | `State(state): State<Arc<AppState>>` |
| `HeaderMap` | Request headers | `headers: HeaderMap` |
| `Extension<T>` | Injected data | From middleware |

### How extractors work

Axum uses the type system to determine what to extract. The order of parameters doesn't matter (mostly). The handler function signature IS the request specification:

```rust
// Axum sees the types and automatically:
// 1. Reads Arc<AppState> from the router state
// 2. Parses {id} from the URL path as u64
// 3. Deserializes the JSON body into CreateOrder
async fn create_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(order): Json<CreateOrder>,
) -> impl IntoResponse {
    // Use state, id, and order...
    StatusCode::CREATED
}
```

---

## Middleware

```rust
use axum::middleware;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/api/orders", get(list_orders).post(create_order))
    .layer(CorsLayer::permissive())          // CORS
    .layer(TraceLayer::new_for_http())        // Request tracing
    .layer(middleware::from_fn(auth_middleware))  // Custom auth
    .with_state(state);
```

### Custom middleware

```rust
use axum::{
    http::Request, 
    middleware::Next, 
    response::Response,
    http::StatusCode,
};

async fn auth_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    
    match auth_header {
        Some(token) if token.starts_with("Bearer ") => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

---

## WebSocket Support

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::IntoResponse,
};

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                socket.send(Message::Text(
                    format!("Echo: {}", text).into()
                )).await.ok();
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

// Add to router
let app = Router::new()
    .route("/ws", get(ws_handler));
```

---

## Error Handling in Axum

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// Custom error type
enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", err),
            ),
        };
        
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// Handler using custom error
async fn get_order(Path(id): Path<u64>) -> Result<Json<serde_json::Value>, AppError> {
    if id == 0 {
        return Err(AppError::BadRequest("ID must be > 0".into()));
    }
    
    // Simulated database lookup
    Ok(Json(json!({ "id": id, "status": "filled" })))
}
```

---

## Application Architecture with Axum

```
src/
├── main.rs              ← Entry point, router setup
├── routes/
│   ├── mod.rs           ← Route definitions
│   ├── orders.rs        ← Order-related routes
│   └── prices.rs        ← Price-related routes
├── handlers/
│   ├── mod.rs
│   ├── orders.rs        ← Order handler functions
│   └── prices.rs        ← Price handler functions
├── services/
│   ├── mod.rs
│   └── trading.rs       ← Business logic
├── models/
│   ├── mod.rs
│   └── order.rs         ← Data structures
├── state.rs             ← AppState definition
└── errors.rs            ← Error types
```

---

## Things to Remember

1. **Axum handlers are async functions** — they integrate naturally with Tokio.
2. **Extractors** parse request data using the type system — no manual parsing.
3. **State is shared via `Arc`** — passed through `Router::with_state()`.
4. **Middleware** via `tower` layers — CORS, auth, logging.
5. **WebSocket** support built-in.
6. **Error handling** via `IntoResponse` trait.
7. **Everything is type-safe** — invalid handler signatures are caught at compile time.

---

> **Next**: [Chapter 27: Backend Architecture in Rust](./27_backend_architecture.md)
