# Chapter 18: Async Error Handling

> **Connection to the bigger picture**: In production Solana backends, errors are inevitable — network timeouts, RPC failures, invalid transactions, database errors. Rust's `Result` type combined with the `?` operator provides clean error handling in async code. The `anyhow` and `thiserror` crates are standard for application-level and library-level error handling respectively.

---

## `Result` + `?` in Async Functions

The `?` operator works naturally in async functions:

```rust
use std::io;

async fn read_config() -> Result<String, io::Error> {
    let contents = tokio::fs::read_to_string("config.toml").await?;
    //                                                           ^
    //                                    If Err, return early from the async fn
    Ok(contents)
}

#[tokio::main]
async fn main() {
    match read_config().await {
        Ok(config) => println!("Config: {}", config),
        Err(e) => eprintln!("Failed to read config: {}", e),
    }
}
```

### Error propagation across `.await`

```rust
async fn fetch_price(symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!("https://api.example.com/price/{}", symbol);
    let response = reqwest::get(&url).await?;       // Network error?
    let body = response.text().await?;               // Read error?
    let price: f64 = body.trim().parse()?;           // Parse error?
    Ok(price)
}
```

Each `?` can produce a different error type. `Box<dyn Error>` accepts any error type. In production, use `anyhow` for applications or custom errors for libraries.

---

## `anyhow` — Application Error Handling

```rust
use anyhow::{Context, Result};

async fn fetch_and_process() -> Result<()> {
    let config = tokio::fs::read_to_string("config.toml")
        .await
        .context("Failed to read config file")?;
    
    let data = make_rpc_call(&config)
        .await
        .context("RPC call failed")?;
    
    save_to_database(&data)
        .await
        .context("Database save failed")?;
    
    Ok(())
}
```

`anyhow::Result<T>` is `Result<T, anyhow::Error>`. It accepts any error type and provides:
- `.context("message")` — adds context to the error chain
- Automatic conversion from any `Error` type via `?`
- Error chain (backtrace of what went wrong)

### When to use `anyhow`

Use `anyhow` in **applications** (binaries) where you want to propagate and display errors but don't need callers to match on specific error variants.

---

## `thiserror` — Library Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Invalid price: {price} for {symbol}")]
    InvalidPrice { symbol: String, price: f64 },
    
    #[error("Insufficient balance: need {needed}, have {available}")]
    InsufficientBalance { needed: f64, available: f64 },
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

async fn place_order(symbol: &str, amount: f64) -> Result<(), TradingError> {
    let price = get_price(symbol).await?;  // Network errors auto-convert
    
    if price <= 0.0 {
        return Err(TradingError::InvalidPrice {
            symbol: symbol.to_string(),
            price,
        });
    }
    
    let balance = get_balance().await?;
    if balance < amount * price {
        return Err(TradingError::InsufficientBalance {
            needed: amount * price,
            available: balance,
        });
    }
    
    Ok(())
}
```

### When to use `thiserror`

Use `thiserror` in **libraries** where callers need to match on specific error variants and handle them differently.

---

## Error Handling in Spawned Tasks

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        might_fail().await
    });
    
    match handle.await {
        Ok(Ok(value)) => println!("Success: {}", value),
        Ok(Err(e)) => println!("Task returned error: {}", e),
        Err(join_error) => println!("Task panicked: {}", join_error),
    }
    //  ^^                                     ^^
    //  JoinError (panic/cancel)               Task's Result error
}
```

Note the double `Result`: `JoinHandle.await` returns `Result<TaskOutput, JoinError>`, and `TaskOutput` itself might be a `Result`.

---

## Things to Remember

1. **`?` works in async functions** — propagates errors naturally across `.await`.
2. **`anyhow`** for applications — easy, any error type, context messages.
3. **`thiserror`** for libraries — custom error enums, callers can match variants.
4. **Spawned tasks have double Results** — JoinError (panic) + task error.
5. **Add context to errors** — `context("what was happening")` for debugging.
6. **Don't `.unwrap()` in production** — handle errors or use `expect("reason")`.

---

> **Next**: [Chapter 19: `tokio::select!` and Concurrent Tasks](./19_select_and_concurrent_tasks.md)
