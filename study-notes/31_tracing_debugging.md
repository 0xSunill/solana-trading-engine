# Chapter 31: Tracing and Debugging

> **Connection to the bigger picture**: `println!()` doesn't cut it for production async applications. When 10,000 tasks are running concurrently, you need structured logging with context — which task, which request, which user, what timestamp. The `tracing` crate is the standard for async Rust.

---

## Why `println!` Is Insufficient

In async code, output from multiple tasks interleaves unpredictably:

```
Processing order 123        ← Which task?
Error: connection refused   ← For which order?
Processing order 456        ← On which thread?
Result: success             ← For order 123 or 456?
```

`tracing` provides **structured, contextual logging**:

```
2024-01-15T10:30:45Z INFO handle_order{order_id=123 symbol="SOL"}: processing order
2024-01-15T10:30:45Z INFO handle_order{order_id=456 symbol="ETH"}: processing order
2024-01-15T10:30:46Z ERROR handle_order{order_id=123}: connection refused
2024-01-15T10:30:46Z INFO handle_order{order_id=456}: order filled
```

---

## Setting Up Tracing

```rust
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{self, EnvFilter};

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(true)        // Show module path
        .with_thread_ids(true)    // Show thread IDs
        .with_file(true)          // Show source file
        .with_line_number(true)   // Show line number
        .json()                   // JSON format for log aggregation
        .init();
}
```

---

## Using Tracing

### Log levels

```rust
tracing::trace!("Very detailed debug info");
tracing::debug!("Debug information");
tracing::info!("General information");
tracing::warn!("Warning — something unexpected");
tracing::error!("Error — something failed");
```

### Structured fields

```rust
tracing::info!(
    order_id = %order.id,
    symbol = %order.symbol,
    price = order.price,
    quantity = order.quantity,
    "Order placed successfully"
);
```

### Spans — contextual grouping

```rust
use tracing::instrument;

#[instrument(skip(db), fields(order_id = %id))]
async fn process_order(db: &PgPool, id: uuid::Uuid) -> Result<(), AppError> {
    tracing::info!("Starting order processing");
    
    let order = fetch_order(db, id).await?;
    tracing::debug!(status = %order.status, "Order fetched");
    
    execute_order(&order).await?;
    tracing::info!("Order processing complete");
    
    Ok(())
}
```

The `#[instrument]` attribute creates a span that groups all log messages within this function with the `order_id` field. Even if multiple orders are processed concurrently, their logs are distinguishable.

---

## Tracing with Axum

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/orders", post(create_order))
    .layer(TraceLayer::new_for_http());  // Automatic request/response logging
```

This automatically logs:
- Request method and path
- Response status code
- Request duration
- Unique request ID

---

## Things to Remember

1. **Use `tracing`** instead of `println!` in async applications.
2. **Structured logging** — key-value fields, not string concatenation.
3. **Spans** group related log messages across async boundaries.
4. **`#[instrument]`** automatically creates spans for functions.
5. **Use `EnvFilter`** to control log levels at runtime (`RUST_LOG=debug`).
6. **JSON output** for production log aggregation systems.

---

> **Next**: [Chapter 32: Solana Connection](./32_solana_connection.md)
