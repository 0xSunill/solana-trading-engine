# Chapter 20: Joining Tasks — `join!`, `try_join!`, and `tokio::spawn`

> **Connection to the bigger picture**: While `select!` races futures (first wins), `join!` runs futures concurrently and waits for ALL of them. This is essential when you need results from multiple independent operations — like fetching prices from multiple exchanges simultaneously.

---

## `tokio::join!` — Wait for All

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(exchange: &str) -> f64 {
    sleep(Duration::from_millis(100)).await;
    match exchange {
        "binance" => 150.5,
        "coinbase" => 150.3,
        _ => 150.0,
    }
}

#[tokio::main]
async fn main() {
    // Run CONCURRENTLY, wait for ALL to complete
    let (price1, price2) = tokio::join!(
        fetch_price("binance"),
        fetch_price("coinbase"),
    );
    
    println!("Binance: {}, Coinbase: {}", price1, price2);
    // Both finish in ~100ms, not ~200ms (concurrent, not sequential)
}
```

### Sequential vs `join!`

```rust
// Sequential: 200ms total
let p1 = fetch_price("binance").await;
let p2 = fetch_price("coinbase").await;

// Concurrent with join!: 100ms total (both run at the same time)
let (p1, p2) = tokio::join!(
    fetch_price("binance"),
    fetch_price("coinbase"),
);
```

---

## `tokio::try_join!` — Wait for All, Short-Circuit on Error

```rust
async fn fetch_price(exchange: &str) -> Result<f64, String> {
    if exchange == "broken" {
        return Err("Exchange unavailable".into());
    }
    Ok(150.0)
}

#[tokio::main]
async fn main() {
    let result = tokio::try_join!(
        fetch_price("binance"),
        fetch_price("coinbase"),
    );
    
    match result {
        Ok((p1, p2)) => println!("Prices: {}, {}", p1, p2),
        Err(e) => println!("Error: {}", e),
    }
}
```

If ANY future returns `Err`, `try_join!` immediately returns that error and cancels the remaining futures.

---

## `join!` vs `tokio::spawn` vs sequential

| Method | Concurrency | New task? | Error handling |
|--------|-------------|-----------|----------------|
| Sequential (`.await` one by one) | None | No | Simple `?` |
| `tokio::join!` | Yes (same task) | No | Returns tuple |
| `tokio::try_join!` | Yes (same task) | No | Short-circuits on first Err |
| `tokio::spawn` | Yes (new tasks) | Yes | `JoinHandle.await` returns `Result` |

### When to use each

- **`join!`**: Few futures, same error handling, no need for independent cancellation.
- **`spawn`**: Many futures, need independent lifecycle, or futures are `'static`.
- **Sequential**: Operations depend on each other (output of one is input of next).

---

## Things to Remember

1. **`join!` runs futures concurrently** within the same task.
2. **`try_join!` short-circuits** on the first error.
3. **`join!` doesn't require `Send + 'static`** — unlike `tokio::spawn`.
4. **Use `join!` for a known set of futures** — use `FuturesUnordered` for dynamic sets.

---

> **Next**: [Chapter 21: Async Streams](./21_streams.md)
