# Chapter 19: `tokio::select!` and Concurrent Tasks

> **Connection to the bigger picture**: `select!` lets you race multiple futures against each other and act on whichever completes first. This is essential for Solana backends: you might need to wait for a market event OR a timeout OR a shutdown signal — whichever comes first.

---

## What `tokio::select!` Does

`select!` waits for multiple async operations simultaneously and executes the branch of whichever completes first.

```rust
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(10);
    
    // Simulate sending a message after 500ms
    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        tx.send("data arrived".into()).await.unwrap();
    });
    
    tokio::select! {
        msg = rx.recv() => {
            println!("Got message: {:?}", msg);
        }
        _ = sleep(Duration::from_secs(1)) => {
            println!("Timeout! No message received in 1 second");
        }
    }
}
```

Output: `Got message: Some("data arrived")` (because message arrives in 500ms, before the 1s timeout).

### What happens to the other branches?

When one branch completes, **all other branches are cancelled** (their futures are dropped). This is important — it means resources in other branches are cleaned up.

---

## Common Patterns

### Pattern 1: Timeout

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout() -> Result<String, &'static str> {
    match timeout(Duration::from_secs(5), fetch_data()).await {
        Ok(result) => result.map_err(|_| "fetch failed"),
        Err(_) => Err("timed out"),
    }
}
```

### Pattern 2: Shutdown signal

```rust
use tokio::signal;

async fn run_server(mut rx: tokio::sync::mpsc::Receiver<String>) {
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                println!("Processing: {}", msg);
            }
            _ = signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
        }
    }
}
```

### Pattern 3: Multiple event sources

```rust
async fn event_loop(
    mut market_events: tokio::sync::mpsc::Receiver<MarketEvent>,
    mut timer: tokio::time::Interval,
    shutdown: tokio_util::sync::CancellationToken,
) {
    loop {
        tokio::select! {
            Some(event) = market_events.recv() => {
                handle_market_event(event).await;
            }
            _ = timer.tick() => {
                run_periodic_check().await;
            }
            _ = shutdown.cancelled() => {
                println!("Shutdown signal received");
                break;
            }
        }
    }
}
```

---

## Cancellation Safety

When a branch is cancelled in `select!`, the future is dropped mid-execution. This is generally safe, but be aware:

```rust
// ⚠️ The recv() branch might "lose" a message if it reads
// the message but then gets cancelled before processing it
tokio::select! {
    msg = rx.recv() => {
        // If this branch wins, great
    }
    _ = something_else() => {
        // If THIS branch wins, rx.recv() is cancelled
        // But it's OK — the message stays in the channel
    }
}
```

`tokio::sync::mpsc::Receiver::recv()` is cancellation-safe — if cancelled, no message is lost. But not all operations are. Check the docs for cancellation safety.

---

## Things to Remember

1. **`select!` races futures** — first to complete wins, others are cancelled.
2. **Cancellation safety matters** — dropped futures should not lose data.
3. **Use `select!` for** timeouts, shutdown signals, and multiplexing event sources.
4. **All branches are polled concurrently** — not sequentially.
5. **`biased;`** option makes `select!` check branches in order (for priority).

---

> **Next**: [Chapter 20: Joining Tasks](./20_joining_tasks.md)
