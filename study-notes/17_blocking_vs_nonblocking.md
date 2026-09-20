# Chapter 17: Blocking vs Non-Blocking Operations

> **Connection to the bigger picture**: The single most important rule of async Rust is "never block the runtime." Understanding what constitutes a blocking operation and how to handle it correctly is the difference between a backend that handles 10,000 connections and one that mysteriously hangs under load.

---

## What Is a Blocking Operation?

A blocking operation is anything that makes the current OS thread wait (sleep) instead of allowing other async tasks to run.

### Common blocking operations

| Operation | Why it blocks | Async alternative |
|-----------|--------------|-------------------|
| `std::thread::sleep()` | OS thread sleeps | `tokio::time::sleep().await` |
| `std::fs::read()` | Waits for disk I/O | `tokio::fs::read().await` |
| `std::net::TcpStream` | Waits for network | `tokio::net::TcpStream` |
| CPU-heavy computation | Uses CPU continuously | `spawn_blocking` |
| Synchronous HTTP client | Waits for response | `reqwest` (async) |
| Synchronous DB query | Waits for response | `sqlx`, `tokio-postgres` |
| `Mutex::lock()` (contended) | Waits for lock | `tokio::sync::Mutex` |

---

## `tokio::task::spawn_blocking`

For operations that MUST block (CPU-heavy work, sync libraries), use `spawn_blocking`:

```rust
#[tokio::main]
async fn main() {
    // This runs on a separate blocking thread pool
    let result = tokio::task::spawn_blocking(|| {
        // CPU-intensive work — OK to block here
        let mut sum = 0u64;
        for i in 0..1_000_000 {
            sum += i;
        }
        sum
    }).await.unwrap();
    
    println!("Sum: {}", result);
}
```

**How `spawn_blocking` works:**

```
Tokio Runtime
├── Worker Thread Pool (async tasks — don't block these!)
│   ├── Thread 1: [task A][task B][task C]...
│   ├── Thread 2: [task D][task E]...
│   └── Thread 3: [task F][task G]...
│
└── Blocking Thread Pool (for spawn_blocking — OK to block)
    ├── Blocking Thread 1: [heavy computation........]
    ├── Blocking Thread 2: [sync file read...........]
    └── (grows as needed, up to 512 threads by default)
```

`spawn_blocking` moves the blocking work to a separate thread pool, keeping the async worker threads free for async tasks.

### When to use `spawn_blocking`

```rust
// Crypto signature verification (CPU-bound)
let sig_valid = tokio::task::spawn_blocking(move || {
    verify_signature(&transaction)
}).await.unwrap();

// Synchronous file operations
let contents = tokio::task::spawn_blocking(|| {
    std::fs::read_to_string("config.toml")
}).await.unwrap()?;

// Third-party sync library
let result = tokio::task::spawn_blocking(move || {
    sync_library::expensive_operation(data)
}).await.unwrap();
```

---

## The Golden Rule

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│  NEVER block a Tokio worker thread.                    │
│                                                        │
│  • Use async versions of I/O operations                │
│  • Use tokio::task::spawn_blocking for CPU work        │
│  • Use tokio::time::sleep instead of thread::sleep     │
│  • Use async database drivers (sqlx, not diesel)       │
│  • Use async HTTP clients (reqwest, not ureq)          │
│                                                        │
└────────────────────────────────────────────────────────┘
```

---

## Things to Remember

1. **Blocking = OS thread waits.** Other async tasks on that thread are starved.
2. **`spawn_blocking`** moves blocking work to a separate thread pool.
3. **Async I/O** (tokio::fs, tokio::net) yields the task, not the thread.
4. **CPU-bound work > ~10-100μs** should use `spawn_blocking`.
5. **Quick synchronous operations** (a few microseconds) are fine on async threads.

---

> **Next**: [Chapter 18: Async Error Handling](./18_async_error_handling.md)
