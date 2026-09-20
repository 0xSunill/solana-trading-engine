# Chapter 21: Async Streams

> **Connection to the bigger picture**: Streams are the async equivalent of iterators. Instead of producing values synchronously, they produce values asynchronously — perfect for processing sequences of events from WebSockets, database cursors, or Solana account subscriptions.

---

## Stream vs Iterator

| | `Iterator` | `Stream` |
|---|---|---|
| Method | `fn next(&mut self) -> Option<Item>` | `fn poll_next(...) -> Poll<Option<Item>>` |
| Usage | `for item in iter { ... }` | `while let Some(item) = stream.next().await { ... }` |
| Blocking | Synchronous (blocks thread) | Async (yields task) |
| Use case | In-memory data | Network data, events, time-based sequences |

```rust
use tokio_stream::{self as stream, StreamExt};

#[tokio::main]
async fn main() {
    // Create a stream from an iterator
    let mut stream = stream::iter(vec![1, 2, 3, 4, 5]);
    
    // Consume asynchronously
    while let Some(value) = stream.next().await {
        println!("Got: {}", value);
    }
}
```

---

## Stream Combinators

Streams support familiar combinators like map, filter, and take:

```rust
use tokio_stream::{self as stream, StreamExt};

#[tokio::main]
async fn main() {
    let stream = stream::iter(1..=10)
        .filter(|x| *x % 2 == 0)     // Keep evens
        .map(|x| x * x)               // Square them
        .take(3);                      // First 3 results
    
    tokio::pin!(stream);
    while let Some(value) = stream.next().await {
        println!("{}", value);  // 4, 16, 36
    }
}
```

---

## Real-World Stream: Channel as Stream

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(100);
    
    // Convert receiver to a stream
    let mut stream = ReceiverStream::new(rx);
    
    // Producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    
    // Process as a stream
    while let Some(value) = stream.next().await {
        println!("Received: {}", value);
    }
}
```

> **Real-world use**: Solana WebSocket subscriptions naturally produce streams of account updates, transaction signatures, or slot notifications.

---

## Things to Remember

1. **Streams = async iterators** — produce values over time.
2. **`.next().await`** to get the next value.
3. **Stream combinators** (map, filter, take) work like iterator combinators.
4. **`tokio_stream`** crate provides stream utilities and wrappers.
5. **Channels, WebSockets, and timers** naturally map to streams.

---

> **Next**: [Chapter 22: Async Traits](./22_async_traits.md)
