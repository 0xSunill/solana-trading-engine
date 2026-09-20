# Chapter 11: Why Async Rust Exists

> **Connection to the bigger picture**: This is where we transition from synchronous concurrency (threads) to asynchronous concurrency (async/await). Everything you've learned so far — ownership, borrowing, lifetimes, Send, Sync, Arc, Mutex, channels — remains relevant. Async Rust doesn't replace these concepts; it builds on top of them. Understanding WHY async exists is crucial because blindly using async where threads would be better (or vice versa) leads to poor architecture.

---

## The Problem with Threads for I/O

Consider a backend that handles 10,000 concurrent connections (like a Solana WebSocket event listener):

### Approach 1: One thread per connection

```rust
use std::net::TcpListener;
use std::thread;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            handle_connection(stream);  // Each connection gets its own thread
        });
    }
}
```

**Problems with 10,000 threads:**

| Resource | Per Thread | × 10,000 |
|----------|-----------|----------|
| Stack memory | 2-8 MB | 20-80 GB |
| OS scheduling overhead | Each thread is scheduled | 10,000 context switches |
| Thread creation time | ~50-100 μs | 0.5-1 second total |
| Context switch cost | ~1-10 μs each | Constant overhead |

Most of these threads spend 99% of their time **waiting**:
- Waiting for network data to arrive
- Waiting for a database query to complete
- Waiting for an RPC response from Solana

While a thread waits, it still uses 2-8 MB of stack memory. The OS still schedules it. The CPU still context-switches to check it. This is enormously wasteful.

### Approach 2: Async — many tasks, few threads

```rust
#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    loop {
        let (stream, _addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}
```

**With async:**

| Resource | Per Task | × 10,000 |
|----------|---------|----------|
| Memory | ~few hundred bytes | ~few MB |
| OS threads | ~4-8 (runtime thread pool) | Same 4-8 |
| Context switches | Cooperative (userspace) | Nanoseconds |
| Task creation time | ~few μs | ~few ms total |

10,000 async tasks on 8 threads: each task is a small state machine that gets polled when its I/O is ready. No wasted stack memory. No expensive context switches.

---

## How Async Works at a High Level

```
┌──────────────────────────────────────────────────────────────┐
│                     Tokio Runtime                            │
│                                                              │
│  OS Thread 1       OS Thread 2       OS Thread 3             │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐             │
│  │ Task A   │     │ Task D   │     │ Task G   │             │
│  │ Task B   │     │ Task E   │     │ Task H   │             │
│  │ Task C   │     │ Task F   │     │ Task I   │             │
│  │ ...      │     │ ...      │     │ ...      │             │
│  └──────────┘     └──────────┘     └──────────┘             │
│                                                              │
│  Each thread runs many tasks cooperatively:                  │
│  - Run Task A until it hits .await                           │
│  - Task A yields → switch to Task B (no OS involvement!)     │
│  - Run Task B until it hits .await                           │
│  - Task B yields → switch to Task C                          │
│  - ...                                                       │
│  - Task A's I/O is ready → resume Task A                     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Key insight**: Async tasks are **cooperatively** scheduled. They voluntarily yield at `.await` points. This is much cheaper than OS preemptive scheduling because:
1. No kernel involvement (userspace scheduling)
2. No full context switch (just swap a few pointers)
3. No wasted stack memory (tasks are state machines, not full stacks)

---

## Blocking vs Non-Blocking Operations

This distinction is fundamental.

### Blocking operation (OS thread waits)

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // This blocks the OS thread for 2 seconds
    thread::sleep(Duration::from_secs(2));
    // The thread is doing NOTHING during this time
    // But it still uses 2-8 MB of stack memory
    
    println!("Done sleeping");
}
```

### Non-blocking operation (task yields, thread does other work)

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // This yields the current task, NOT the thread
    sleep(Duration::from_secs(2)).await;
    // The OS thread runs other tasks during these 2 seconds
    // This task uses only a few hundred bytes while suspended
    
    println!("Done sleeping");
}
```

### Why this matters

```
thread::sleep(2s) on 8 threads:
  Thread 1: [sleeping..........................]
  Thread 2: [sleeping..........................]
  ... all 8 threads are blocked ...
  → System can handle 0 requests during this time!

tokio::time::sleep(2s) on 8 threads, 10,000 tasks:
  Thread 1: [task1 yields][runs task9][runs task17][runs task25]...
  Thread 2: [task2 yields][runs task10][runs task18][runs task26]...
  → System handles thousands of other tasks while these sleep!
```

> **Critical rule**: Never call blocking operations inside async code. Use `tokio::time::sleep` instead of `thread::sleep`. Use `tokio::fs` instead of `std::fs`. Use `tokio::task::spawn_blocking` for unavoidable blocking operations.

---

## When to Use Threads vs Async

| Scenario | Use | Why |
|----------|-----|-----|
| 10,000 HTTP connections | Async | Mostly waiting for I/O, low memory per task |
| WebSocket event listener | Async | Waiting for messages, many connections |
| Image processing | Threads | CPU-bound, needs all cores |
| Matrix multiplication | Threads | CPU-bound, no I/O waiting |
| Database queries | Async | Waiting for network response |
| Solana RPC calls | Async | Waiting for network response |
| Cryptographic hashing | Threads (spawn_blocking) | CPU-bound |
| File reading | Async or spawn_blocking | I/O-bound but may block |
| Mixed workload | Async + spawn_blocking | Async for I/O, blocking for CPU |

---

## The Connection to Your Goal

As a Solana backend engineer, your system is overwhelmingly **I/O-bound**:

```
Your Solana Backend
├── WebSocket connections to Solana (waiting for data)        → Async
├── HTTP RPC calls to Solana nodes (waiting for response)     → Async
├── Database queries (waiting for PostgreSQL)                  → Async
├── HTTP API serving clients (waiting for requests)            → Async
├── Timer events (waiting for specific times)                  → Async
├── Channel communication between components                   → Async
├── Transaction signature verification (CPU work)              → spawn_blocking
└── Complex strategy calculations (CPU work)                   → spawn_blocking
```

Async Rust (Tokio) is the foundation of your entire backend architecture.

---

## Things to Remember

1. **Threads are expensive** — 2-8 MB stack, kernel scheduling, slow context switches.
2. **Async tasks are cheap** — few hundred bytes, userspace scheduling, fast switching.
3. **Async is for I/O-bound work** — many tasks waiting for network/disk.
4. **Threads are for CPU-bound work** — parallel computation across cores.
5. **Never block inside async** — use async versions or `spawn_blocking`.
6. **Tokio manages a thread pool** — async tasks run on a small number of OS threads.
7. **Cooperative scheduling** — tasks yield at `.await`, no OS involvement.

---

> **Next**: [Chapter 12: Futures](./12_futures.md) — The core abstraction behind async/await
