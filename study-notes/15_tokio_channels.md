# Chapter 15: Tokio Channels

> **Connection to the bigger picture**: Tokio provides async-aware channel types that integrate with the async runtime. Unlike `std::sync::mpsc`, Tokio channels are designed for async code — `send()` and `recv()` are async operations that yield instead of blocking. In Solana backends, channels connect components: WebSocket listeners → event processors → strategy engines → transaction submitters.

---

## Overview

| Channel Type | Senders | Receivers | Buffered | Use Case |
|-------------|---------|-----------|----------|----------|
| `mpsc` | Many | One | Yes (bounded) | Most common — producer/consumer |
| `oneshot` | One | One | One value | Request/response, completion signal |
| `broadcast` | Many | Many | Yes (bounded) | Pub/sub, event broadcasting |
| `watch` | One | Many | Latest value | Config updates, shared state notification |

---

## `tokio::sync::mpsc` — Multi-Producer, Single Consumer

The most commonly used channel in async Rust.

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Create a bounded channel with capacity 100
    let (tx, mut rx) = mpsc::channel::<String>(100);
    //                                         ^^^
    //                              Buffer size — max messages
    //                              before send() blocks
    
    // Producer task
    let tx_clone = tx.clone();  // Clone for multiple producers
    tokio::spawn(async move {
        for i in 0..5 {
            tx_clone.send(format!("message {}", i)).await.unwrap();
        }
    });
    
    // Drop the original sender (only clones remain)
    drop(tx);
    
    // Consumer — receives until all senders are dropped
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
    println!("Channel closed");
}
```

### What the buffer size means

```rust
let (tx, rx) = mpsc::channel(100);
```

The `100` is the **buffer capacity**:
- Up to 100 messages can be buffered in the channel.
- If the buffer is full, `tx.send().await` **yields** until space is available (backpressure).
- If the buffer is empty, `rx.recv().await` **yields** until a message arrives.

**Why bounded channels are important:**

If the producer is faster than the consumer, an unbounded channel would accumulate messages indefinitely, consuming more and more memory. Bounded channels provide **backpressure** — the producer slows down when the consumer can't keep up.

> **Real-world use**: In a Solana event processor, market data might arrive faster than your strategy can process it. A bounded channel prevents unbounded memory growth by slowing down the event receiver when the strategy is behind.

### `mpsc::unbounded_channel`

For cases where backpressure isn't needed:

```rust
let (tx, mut rx) = mpsc::unbounded_channel::<String>();

// send() is NOT async — it never blocks
tx.send("hello".into()).unwrap();

// recv() IS async — yields until a message arrives
let msg = rx.recv().await;
```

> **Warning**: Unbounded channels can cause memory issues if the producer is much faster than the consumer. Prefer bounded channels in production.

---

## `tokio::sync::oneshot` — One-Shot Channel

Sends exactly one value from one sender to one receiver.

```rust
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    
    tokio::spawn(async move {
        // Do some work...
        let result = expensive_computation().await;
        tx.send(result).unwrap();  // Send one value
    });
    
    let value = rx.await.unwrap();  // Receive the one value
    println!("Got: {}", value);
}

async fn expensive_computation() -> i32 { 42 }
```

### Request/response pattern

```rust
use tokio::sync::{mpsc, oneshot};

struct Request {
    query: String,
    respond_to: oneshot::Sender<String>,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Request>(32);
    
    // Worker that processes requests
    tokio::spawn(async move {
        while let Some(req) = rx.recv().await {
            let response = format!("Result for: {}", req.query);
            let _ = req.respond_to.send(response);
        }
    });
    
    // Send a request and wait for the response
    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(Request {
        query: "SELECT * FROM orders".into(),
        respond_to: resp_tx,
    }).await.unwrap();
    
    let response = resp_rx.await.unwrap();
    println!("Response: {}", response);
}
```

---

## `tokio::sync::broadcast` — Broadcast Channel

Every receiver gets a copy of every message.

```rust
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel::<String>(100);
    
    // Create multiple receivers
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    
    // Send a message — ALL receivers get it
    tx.send("market update".into()).unwrap();
    
    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();
    
    assert_eq!(msg1, msg2);  // Both got the same message
}
```

> **Real-world use**: Broadcasting price updates to multiple strategy components.

---

## `tokio::sync::watch` — Watch Channel

Only keeps the latest value. Receivers are notified when it changes.

```rust
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = watch::channel(0u64);  // Initial value: 0
    
    // Producer updates the value periodically
    tokio::spawn(async move {
        for i in 1..=5 {
            tx.send(i).unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    
    // Consumer reads the latest value when notified
    while rx.changed().await.is_ok() {
        let value = *rx.borrow();
        println!("Latest value: {}", value);
    }
}
```

> **Real-world use**: Configuration that might change at runtime. Background tasks watch for config changes. Also perfect for the latest price of a token.

---

## Choosing the Right Channel

```
Do you need to send one value or many?
├── One value → oneshot
└── Many values
    ├── How many receivers?
    │   ├── One receiver → mpsc
    │   └── Multiple receivers
    │       ├── All receivers need all messages → broadcast
    │       └── Receivers only need latest value → watch
    └── Do you need backpressure?
        ├── Yes → mpsc::channel(N) (bounded)
        └── No → mpsc::unbounded_channel()
```

---

## Things to Remember

1. **`mpsc::channel(N)`** — most common. N is buffer size for backpressure.
2. **`oneshot`** — single value, perfect for request/response.
3. **`broadcast`** — every subscriber gets every message.
4. **`watch`** — only latest value, receivers notified on change.
5. **Bounded channels prevent memory exhaustion** in production systems.
6. **Channels close when all senders (or the receiver) are dropped.**

---

> **Next**: [Chapter 16: Async Mutex and Shared State](./16_async_mutex_shared_state.md)
