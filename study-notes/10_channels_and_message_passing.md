# Chapter 10: Channels and Message Passing

> **Connection to the bigger picture**: Channels are the second major approach to concurrency (alongside shared state). The idea comes from CSP (Communicating Sequential Processes): "Do not communicate by sharing memory; instead, share memory by communicating." In Solana trading systems, channels connect components — a WebSocket listener sends market data through a channel to a strategy engine, which sends orders through another channel to a transaction submitter.

---

## Why Channels?

Shared state (`Arc<Mutex<T>>`) works, but has downsides:
- Lock contention reduces parallelism
- Deadlocks are possible
- Complex lock ordering requirements

Channels provide a different model: **one-way pipes** that transfer ownership of data.

```
Thread A                    Thread B
   │                           │
   │  send(value) ────────▶   recv()
   │                           │
   │  Ownership of value       │  Now owns value
   │  is GONE from Thread A    │
```

---

## `std::sync::mpsc` — Standard Library Channels

`mpsc` stands for **Multiple Producer, Single Consumer**.

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // Create a channel
    let (tx, rx) = mpsc::channel();
    //  ^^  ^^
    //  │   └── Receiver (consumer end)
    //  └────── Sender (producer end)
    
    // Spawn a thread that sends data
    thread::spawn(move || {
        let message = String::from("hello from thread");
        tx.send(message).unwrap();
        // message is MOVED into the channel — can't use it anymore
        // println!("{}", message);  // ❌ COMPILE ERROR: moved
    });
    
    // Receive on the main thread
    let received = rx.recv().unwrap();
    println!("Got: {}", received);
}
```

### `send()` — Transfer ownership through the channel

```rust
tx.send(value).unwrap();
```

- **Moves** `value` into the channel. The sender no longer owns it.
- Returns `Result<(), SendError<T>>`. Fails if the receiver has been dropped.
- **Blocks** if the channel is full (for bounded channels — covered later with Tokio).

### `recv()` — Receive ownership from the channel

```rust
let value = rx.recv().unwrap();
```

- **Blocks** the current thread until a value is available.
- Returns `Result<T, RecvError>`. Fails if all senders have been dropped (channel closed).
- The receiver now **owns** the value.

### `try_recv()` — Non-blocking receive

```rust
match rx.try_recv() {
    Ok(value) => println!("Got: {}", value),
    Err(mpsc::TryRecvError::Empty) => println!("Nothing yet"),
    Err(mpsc::TryRecvError::Disconnected) => println!("Channel closed"),
}
```

Does not block — returns immediately with either a value or an error.

---

## Multiple Producers

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    // Clone the sender for each producer thread
    for i in 0..5 {
        let tx = tx.clone();  // Each thread gets its own Sender
        thread::spawn(move || {
            tx.send(format!("Message from thread {}", i)).unwrap();
        });
    }
    
    drop(tx);  // Drop the original sender
    // Now only the cloned senders exist in the threads
    
    // Receive all messages
    for received in rx {
        // rx implements Iterator — yields values until all senders are dropped
        println!("{}", received);
    }
    
    println!("All senders dropped, channel closed");
}
```

**Key point**: You can `clone()` the `Sender` to create multiple producers. But there's only one `Receiver` (that's the "SC" in "MPSC").

---

## Ownership Transfer Through Channels

This is a critical concept. Channels enforce Rust's ownership rules across thread boundaries.

```
Thread A                    Channel                    Thread B
   │                           │                          │
   │  name: String             │                          │
   │  "Sunil"                  │                          │
   │                           │                          │
   │  tx.send(name) ──────▶   │  name: String            │
   │                           │  "Sunil"                 │
   │  name is MOVED            │                          │
   │  (can't use anymore)      │  ──────────────────▶   rx.recv()
   │                           │                          │
   │                           │                          │  received: String
   │                           │                          │  "Sunil"
   │                           │                          │  (now owned by B)
```

### Why you can't send references through channels

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    let name = String::from("Sunil");
    
    // ❌ This is problematic
    // tx.send(&name);  
    // The reference &name borrows from main's stack.
    // The thread might receive it AFTER main drops name.
    // That would be a dangling reference.
    
    // ✅ Send the owned value
    tx.send(name).unwrap();  // Moves name into the channel
    
    // ✅ Or clone
    // let name_clone = name.clone();
    // tx.send(name_clone).unwrap();
}
```

**The compiler prevents this because `Sender<&String>` would require the reference to be `'static`, which a reference to a local variable is not.**

---

## Channel Patterns

### Pattern 1: Request-Response

```rust
use std::sync::mpsc;
use std::thread;

struct Request {
    query: String,
    response_tx: mpsc::Sender<String>,
}

fn main() {
    let (tx, rx) = mpsc::channel();
    
    // Worker thread
    thread::spawn(move || {
        for request in rx {
            let request: Request = request;
            let result = format!("Result for: {}", request.query);
            request.response_tx.send(result).unwrap();
        }
    });
    
    // Send a request
    let (resp_tx, resp_rx) = mpsc::channel();
    tx.send(Request {
        query: "SELECT * FROM orders".into(),
        response_tx: resp_tx,
    }).unwrap();
    
    let response = resp_rx.recv().unwrap();
    println!("Response: {}", response);
}
```

### Pattern 2: Pipeline

```
Producer ──▶ Channel ──▶ Transformer ──▶ Channel ──▶ Consumer
```

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (raw_tx, raw_rx) = mpsc::channel();
    let (processed_tx, processed_rx) = mpsc::channel();
    
    // Producer
    thread::spawn(move || {
        for i in 0..10 {
            raw_tx.send(i).unwrap();
        }
    });
    
    // Transformer
    thread::spawn(move || {
        for value in raw_rx {
            processed_tx.send(value * 2).unwrap();
        }
    });
    
    // Consumer
    for result in processed_rx {
        println!("Result: {}", result);
    }
}
```

---

## `sync_channel` — Bounded Channels

`mpsc::channel()` is unbounded — the sender never blocks. This can be dangerous if the producer is faster than the consumer (memory grows without limit).

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // Bounded channel with capacity 3
    let (tx, rx) = mpsc::sync_channel(3);
    
    thread::spawn(move || {
        for i in 0..10 {
            println!("Sending: {}", i);
            tx.send(i).unwrap();  // BLOCKS when buffer is full
            println!("Sent: {}", i);
        }
    });
    
    for received in rx {
        thread::sleep(std::time::Duration::from_millis(500));
        println!("Received: {}", received);
    }
}
```

With `sync_channel(3)`, the sender blocks after sending 3 values until the receiver consumes some. This provides **backpressure** — the producer can't overwhelm the consumer.

> **Real-world use**: In a trading system, backpressure prevents memory exhaustion when market data arrives faster than your strategy can process it.

---

## Things to Remember

1. **Channels transfer ownership** — the sender gives up the value, the receiver gets it.
2. **`mpsc::channel()`** — unbounded, multiple producers, single consumer.
3. **`mpsc::sync_channel(n)`** — bounded, provides backpressure.
4. **Clone `Sender` for multiple producers** — each producer needs its own sender.
5. **Channel closes when all senders are dropped** — receiver's `recv()` returns `Err`.
6. **Prefer channels when data flows in one direction** — simpler than shared state.
7. **References can't (easily) be sent through channels** — send owned values.

---

## Interview Questions

**Q: What is the difference between shared state and message passing?**

> Shared state (Arc<Mutex<T>>) gives all threads access to the same data through locks. Message passing (channels) transfers data ownership between threads. Shared state is simpler for bidirectional access but has deadlock risks. Channels are simpler for pipeline architectures but require data to be moved or cloned.

**Q: When would you use a bounded channel?**

> When the producer might be faster than the consumer. Bounded channels provide backpressure — the producer blocks when the channel is full, preventing memory exhaustion. This is critical in data processing pipelines and event-driven systems.

**Q: How does Rust's ownership system make channels safer than in other languages?**

> In Rust, `send()` moves ownership to the channel. The sender can no longer access the value, preventing both threads from modifying the same data. In languages like Go, channels carry values but the sender can still access its copy of the data if it keeps a reference, potentially causing races.

---

> **Next**: [Chapter 11: Why Async Rust Exists](./11_async_fundamentals.md) — The transition from threads to async
