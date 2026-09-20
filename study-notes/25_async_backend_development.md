# Chapter 25: Async Rust for Backend Development

> **Connection to the bigger picture**: This chapter bridges async Rust theory to practical backend development — HTTP servers, databases, WebSockets, and external APIs. These are the building blocks of a Solana trading backend.

---

## The Async Backend Ecosystem

| Need | Crate | Role |
|------|-------|------|
| HTTP framework | `axum` | Routes, handlers, middleware |
| Async runtime | `tokio` | Task scheduling, I/O |
| HTTP client | `reqwest` | Making HTTP/RPC calls |
| Database | `sqlx` | Async PostgreSQL/MySQL/SQLite |
| Serialization | `serde` + `serde_json` | JSON parsing/generation |
| WebSocket | `tokio-tungstenite` | WebSocket connections |
| Logging | `tracing` | Structured async-aware logging |
| Error handling | `anyhow` / `thiserror` | Error types and propagation |
| Config | `config` or `dotenvy` | Environment/file configuration |
| Connection pool | `deadpool` or built into `sqlx` | Database connection reuse |

---

## How Async Rust Powers Backend Components

```
Client Request
      │
      ▼
┌─────────────┐
│   Axum      │ ← Accepts HTTP/WS connections (async)
│   Router    │
└─────┬───────┘
      │
      ▼
┌─────────────┐
│  Handler    │ ← Processes request (async fn)
│  (async fn) │
└─────┬───────┘
      │
      ├──▶ Database Query (sqlx — async .await)
      ├──▶ External API (reqwest — async .await)
      ├──▶ Cache Lookup (redis — async .await)
      └──▶ Background Task (tokio::spawn)
      │
      ▼
┌─────────────┐
│  Response   │ ← Return JSON/HTML to client
└─────────────┘
```

Each step is async — the handler function yields at each `.await`, allowing the thread to serve other requests while waiting for database results or API responses.

### Database connections

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect("postgres://user:pass@localhost/mydb")
    .await?;

// Query is async
let row = sqlx::query!("SELECT price FROM tokens WHERE symbol = $1", "SOL")
    .fetch_one(&pool)
    .await?;

println!("SOL price: {}", row.price);
```

### HTTP client (RPC calls)

```rust
let client = reqwest::Client::new();

let response = client
    .post("https://api.mainnet-beta.solana.com")
    .json(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": ["YOUR_WALLET_ADDRESS"]
    }))
    .send()
    .await?;

let body: serde_json::Value = response.json().await?;
println!("Balance: {}", body["result"]["value"]);
```

### WebSocket connections

```rust
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;

let (ws_stream, _) = connect_async("wss://api.mainnet-beta.solana.com")
    .await?;

let (write, mut read) = ws_stream.split();

// Read messages as a stream
while let Some(msg) = read.next().await {
    let msg = msg?;
    println!("Received: {}", msg);
}
```

### Background workers

```rust
// Spawn a background task for periodic work
tokio::spawn(async move {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        cleanup_expired_orders(&pool).await;
    }
});
```

---

## Things to Remember

1. **Everything is async** — HTTP, database, file I/O, WebSocket, timers.
2. **Connection pools** share database connections across handlers.
3. **`reqwest`** for HTTP client calls (API, RPC).
4. **`tokio-tungstenite`** for WebSocket connections.
5. **Background tasks** (`tokio::spawn`) for periodic work.
6. **The async ecosystem is mature** — production-ready for Solana backends.

---

> **Next**: [Chapter 26: Axum Framework](./26_axum.md)
