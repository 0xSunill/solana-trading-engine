# Chapter 13: Tokio Runtime and `tokio::spawn`

> **Connection to the bigger picture**: Tokio is the async runtime that drives futures to completion. Without a runtime, async code does nothing — futures are lazy and need an executor to poll them. Tokio provides the executor, I/O driver, timer system, and task scheduler. For Solana backends built with Axum, Tokio is the foundation everything runs on.

---

## What Tokio Is

Rust's standard library defines the `Future` trait but does NOT include a runtime to execute futures. You need a third-party runtime. Tokio is the de facto standard.

**What Tokio provides:**

```
Tokio Runtime
├── Executor (schedules and polls tasks)
├── I/O Driver (monitors sockets, files for readiness)
├── Timer Driver (manages sleep, timeout, interval)
├── Task Scheduler (work-stealing, distributes tasks across threads)
└── Sync Primitives (Mutex, RwLock, channels, Semaphore, etc.)
```

### `#[tokio::main]`

```rust
#[tokio::main]
async fn main() {
    println!("Hello from async Rust!");
}
```

This macro expands to approximately:

```rust
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        println!("Hello from async Rust!");
    });
}
```

**What `block_on` does:**
1. Creates the Tokio runtime (thread pool, I/O driver, timer driver).
2. Takes the provided future and starts polling it on the current thread.
3. Blocks the current thread until the future completes.
4. Returns the future's output.

### Runtime flavors

```rust
// Multi-threaded (default) — for production
#[tokio::main]
async fn main() { }
// Creates a thread pool (default: one thread per CPU core)

// Single-threaded — for testing or simple apps
#[tokio::main(flavor = "current_thread")]
async fn main() { }
// Runs everything on the current thread

// Custom configuration
#[tokio::main(worker_threads = 4)]
async fn main() { }
// 4 worker threads
```

---

## `tokio::spawn` — Spawning Async Tasks

`tokio::spawn` creates a new async task that runs concurrently with other tasks.

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        "task completed"
    });
    
    println!("Task is running in the background...");
    
    let result = handle.await.unwrap();
    println!("Result: {}", result);
}
```

### The signature of `tokio::spawn`

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
```

**Breaking down each constraint:**

| Constraint | Why |
|-----------|-----|
| `F: Future` | Obviously — we're spawning a future |
| `F: Send` | Tokio may move the task to a different worker thread |
| `F: 'static` | The task runs independently — it might outlive the spawning scope |
| `F::Output: Send` | The result is sent back to whoever `.await`s the JoinHandle |
| `F::Output: 'static` | Same reason — the output must not borrow from limited scopes |

### `async move` — moving ownership into tasks

```rust
#[tokio::main]
async fn main() {
    let name = String::from("Sunil");
    
    // ❌ Without move: borrows name
    // tokio::spawn(async {
    //     println!("{}", name);  // Borrows name from outer scope
    // });
    // Error: borrowed value does not live long enough / not 'static
    
    // ✅ With move: takes ownership
    tokio::spawn(async move {
        println!("{}", name);  // Owns name — 'static satisfied
    });
    
    // println!("{}", name);  // ❌ name was moved
}
```

**Why `move` is needed:**

Without `move`, the async block would borrow `name` from the outer scope. But `tokio::spawn` requires `'static` — the task must own all its data because it might run after the spawning function returns. `move` transfers ownership into the async block, making it `'static`.

This is exactly the same concept as `thread::spawn(move || { ... })` from Chapter 7.

### `JoinHandle`

```rust
#[tokio::main]
async fn main() {
    let handle: tokio::task::JoinHandle<i32> = tokio::spawn(async {
        42
    });
    
    // .await the handle to get the result
    match handle.await {
        Ok(value) => println!("Got: {}", value),
        Err(join_error) => {
            if join_error.is_panic() {
                println!("Task panicked!");
            } else if join_error.is_cancelled() {
                println!("Task was cancelled!");
            }
        }
    }
}
```

`JoinHandle.await` returns `Result<T, JoinError>`:
- `Ok(value)` — task completed successfully
- `Err(JoinError)` — task panicked or was cancelled

### Dropping JoinHandle doesn't cancel the task

```rust
#[tokio::main]
async fn main() {
    // This task runs to completion even though we don't await the handle
    let _handle = tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Task completed!");
    });
    
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    // "Task completed!" is printed after 1 second
}
```

Dropping the `JoinHandle` does NOT cancel the task. The task continues running. To cancel, call `handle.abort()`.

---

## How Tokio Schedules Tasks

Tokio uses a **work-stealing** scheduler:

```
┌─────────────────────────────────────────────────────────────┐
│                    Tokio Runtime                            │
│                                                             │
│  Worker Thread 1        Worker Thread 2                     │
│  ┌───────────────┐     ┌───────────────┐                   │
│  │ Local Queue:  │     │ Local Queue:  │                   │
│  │ [Task A]      │     │ [Task D]      │                   │
│  │ [Task B]      │     │ [Task E]      │                   │
│  │ [Task C]      │     │               │ ← Queue empty!    │
│  └───────────────┘     └───────────────┘                   │
│                               │                             │
│                               │ "Steal" work               │
│                               ▼                             │
│                         Steals Task C from Thread 1         │
│                                                             │
│  Global Queue (for newly spawned tasks):                    │
│  [Task F] [Task G] [Task H]                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

When a worker thread's local queue is empty, it "steals" tasks from other threads' queues. This ensures all threads stay busy and work is distributed evenly.

**This is why `Send` is required**: If a task can be stolen by another thread, it must be safe to move between threads (`Send`).

---

## Common Patterns with `tokio::spawn`

### Pattern 1: Fire and forget

```rust
tokio::spawn(async {
    if let Err(e) = send_metrics().await {
        eprintln!("Failed to send metrics: {}", e);
    }
});
// Don't await — we don't need the result
```

### Pattern 2: Spawn with shared state

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            let mut count = counter.lock().await;
            *count += 1;
            println!("Task {} incremented counter to {}", i, *count);
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    println!("Final: {}", *counter.lock().await);
}
```

### Pattern 3: Background task with channel

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Background processing task
    let processor = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Processing: {}", msg);
        }
    });
    
    // Send some messages
    for i in 0..5 {
        tx.send(format!("message {}", i)).await.unwrap();
    }
    
    drop(tx);  // Close the channel
    processor.await.unwrap();  // Wait for processing to finish
}
```

---

## Why References Often Don't Work

```rust
#[tokio::main]
async fn main() {
    let data = vec![1, 2, 3];
    
    // ❌ Borrowing from outer scope
    tokio::spawn(async {
        println!("{:?}", data);  // borrows data
    });
    // Error: `data` does not live long enough
    // The spawned task is 'static but data is not
}
```

**Solutions:**

```rust
// Solution 1: Move ownership
tokio::spawn(async move {
    println!("{:?}", data);  // owns data
});

// Solution 2: Clone
let data_clone = data.clone();
tokio::spawn(async move {
    println!("{:?}", data_clone);
});
// data is still available here

// Solution 3: Arc for shared access
let data = Arc::new(data);
let data_clone = Arc::clone(&data);
tokio::spawn(async move {
    println!("{:?}", data_clone);
});
// Other tasks can also Arc::clone(&data)
```

---

## Things to Remember

1. **Tokio is the runtime** — it provides the executor, I/O driver, and scheduler.
2. **`#[tokio::main]`** creates the runtime and blocks on your async main.
3. **`tokio::spawn` requires `Send + 'static`** — tasks may run on any thread, must own their data.
4. **`async move`** moves captured variables into the async block (same as thread `move`).
5. **Tasks are not cancelled when JoinHandle is dropped** — use `.abort()` to cancel.
6. **Work-stealing scheduler** distributes tasks across threads — this is why `Send` is needed.
7. **Don't block Tokio's threads** — use `spawn_blocking` for blocking operations.

---

## Interview Questions

**Q: What happens if you call `tokio::spawn` without `async move`?**

> The async block would borrow from the surrounding scope by default. Since `tokio::spawn` requires `'static`, any borrows from non-static scopes would cause a compile error. `async move` is needed to transfer ownership of captured variables into the task.

**Q: Does dropping a JoinHandle cancel the task?**

> No. The task continues running in the background. To cancel a task, you must call `handle.abort()` or use a cancellation mechanism like `CancellationToken`.

**Q: Why does `tokio::spawn` require `Send`?**

> Tokio's multi-threaded runtime uses a work-stealing scheduler that may move tasks between threads. If a task could only run on one specific thread, work-stealing would be impossible. The `Send` bound ensures the task's state machine can be safely moved between threads.

---

> **Next**: [Chapter 14: async move, Ownership, and Lifetimes](./14_async_move_ownership_lifetimes.md)
