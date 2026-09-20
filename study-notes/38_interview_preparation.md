# Chapter 38: Interview Preparation

> Questions organized from beginner to advanced, covering Rust, concurrency, async, backend, and trading systems.

---

## Rust Fundamentals

### Q: What are the three rules of ownership?

> **Short**: Each value has one owner, only one at a time, value is dropped when owner goes out of scope.
> 
> **Detailed**: (1) Every value has exactly one variable that owns it. (2) When the owner is reassigned (`let b = a`), ownership transfers and the original variable is invalidated (for non-Copy types). (3) When the owning variable goes out of scope, `drop()` is called and the value's resources are freed. This system eliminates garbage collection while preventing use-after-free, double-free, and memory leaks at compile time.
> 
> **Common wrong answer**: "Ownership is like reference counting" — No, ownership is compile-time, with exactly one owner, not multiple.

### Q: What is the difference between `&T` and `&mut T`?

> **Short**: `&T` is a shared, immutable reference. `&mut T` is an exclusive, mutable reference.
> 
> **Detailed**: You can have many `&T` references simultaneously (safe because nobody can modify). You can have exactly one `&mut T` (safe because nobody else can read or write). You cannot have both simultaneously. This prevents data races at compile time — the same principle behind `RwLock` (many readers or one writer).

### Q: Why can't a struct with a `String` field implement `Copy`?

> **Short**: `Copy` requires bitwise copy to be safe. `String` has heap data — bitwise copy would create two owners of the same heap memory, causing double-free.
> 
> **Detailed**: `Copy` means implicit, bitwise duplication on assignment. `String` stores a pointer to heap-allocated data. A bitwise copy would create two `String`s pointing to the same buffer. When both are dropped, the buffer would be freed twice (undefined behavior). `Clone` is the explicit, safe alternative that allocates new heap memory for the copy.

### Q: What is `'static` and when do you see it?

> **Short**: `'static` means the value can live for the entire program — it doesn't borrow from anything with a limited lifetime.
> 
> **Detailed**: Common in `tokio::spawn` (`F: 'static`), thread::spawn, and lazy_static. Owned types (`String`, `Vec`, `i32`) satisfy `'static` because they don't borrow from scoped data. It does NOT mean the value actually lives forever — just that it could. A `String` is `'static` but gets dropped when its owner goes out of scope.

---

## Concurrency

### Q: What is the difference between concurrency and parallelism?

> **Short**: Concurrency = managing multiple tasks (structure). Parallelism = executing simultaneously (hardware).
> 
> **Detailed**: Concurrency is about designing a program to handle multiple overlapping tasks — they may interleave on a single core. Parallelism is about actually running tasks at the same instant on multiple CPU cores. Async Rust provides concurrency (many tasks, few threads). OS threads provide parallelism (true simultaneous execution on multiple cores).

### Q: What are `Send` and `Sync`?

> **Short**: `Send` = can transfer ownership between threads. `Sync` = can share `&T` between threads.
> 
> **Detailed**: `Send` means a value can be safely moved to another thread — important for `thread::spawn`, `tokio::spawn`, channels. `Sync` means `&T` can be safely shared across threads — formally, `T: Sync ⟺ &T: Send`. Both are auto traits (compiler implements them automatically based on fields) and marker traits (no methods, zero runtime cost). `Rc` is not `Send` (non-atomic ref count). `RefCell` is not `Sync` (unsynchronized interior mutability).

### Q: How does `Arc<Mutex<T>>` work?

> **Short**: `Arc` provides shared ownership across threads. `Mutex` provides exclusive mutable access. Together: safe shared mutable state.
> 
> **Detailed**: `Arc` uses atomic reference counting so multiple threads can hold a handle to the same data. `Mutex` ensures only one thread can access the inner `T` at a time — `lock()` blocks until available, returns a `MutexGuard` that releases the lock when dropped (RAII). `Arc` alone only gives `&T` (immutable). `Mutex` wrapping provides `&mut T` through the lock guard. You need both because: without `Arc`, you can't share the Mutex; without `Mutex`, you can't mutate through `Arc`.

### Q: Does Rust prevent deadlocks?

> **Short**: No. Rust prevents data races, not deadlocks.
> 
> **Detailed**: Data races (unsynchronized concurrent access to memory) are prevented at compile time through ownership and Send/Sync. Deadlocks are logic errors — two threads each holding a lock the other needs — they don't cause undefined behavior, just programs that hang. Prevention: consistent lock ordering, try_lock with timeouts, prefer channels, minimize lock scope.

---

## Async Rust

### Q: What is a Future and how does polling work?

> **Short**: A Future is a value that will produce a result eventually. The executor calls `poll()`, which returns `Ready(value)` or `Pending` with a waker for notification.
> 
> **Detailed**: `Future` has one method: `poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Output>`. The executor calls `poll()`. If the result is available, it returns `Ready(value)`. If not, the future registers a `Waker` from the context and returns `Pending`. The waker is a callback that tells the executor "poll me again" — it's triggered by the I/O driver when data arrives. This avoids busy-polling. The compiler transforms async functions into state machines where each `.await` is a state transition.

### Q: Why does `tokio::spawn` require `Send + 'static`?

> **Short**: `Send` because the task may run on any thread (work-stealing). `'static` because the task's lifetime is independent of the spawner.
> 
> **Detailed**: Tokio's multi-threaded scheduler uses work-stealing — a task started on one thread may be resumed on another. `Send` ensures this is safe. `'static` means the task cannot borrow from the spawning scope because the spawned task may outlive the function that created it. This drives the use of `async move`, owned data, and `Arc` for sharing.

### Q: What is the difference between `tokio::join!` and `tokio::select!`?

> **Short**: `join!` waits for ALL futures to complete. `select!` waits for the FIRST future to complete.
> 
> **Detailed**: `join!(a, b, c)` runs a, b, c concurrently and returns all results as a tuple when all complete. `select!` runs multiple futures and executes the branch of whichever completes first, cancelling the rest. Use `join!` when you need all results (e.g., fetching prices from multiple exchanges). Use `select!` for racing (e.g., event OR timeout OR shutdown signal).

### Q: Why should you never call blocking operations in async code?

> **Short**: Blocking operations block the OS thread, preventing all other async tasks on that thread from running.
> 
> **Detailed**: Tokio has a small pool of worker threads (typically = CPU cores). Each thread runs many async tasks cooperatively. If one task calls `std::thread::sleep()` or a synchronous I/O operation, that thread is blocked — all other tasks scheduled on it are starved. With 4 threads, blocking one reduces capacity by 25%. Use `tokio::time::sleep().await`, async I/O libraries, or `spawn_blocking` for unavoidable blocking work.

---

## Backend

### Q: How does Axum handle shared state?

> **Short**: State is wrapped in `Arc<AppState>` and passed to the router via `Router::with_state()`. Handlers extract it with `State(state): State<Arc<AppState>>`.
> 
> **Detailed**: Axum clones the state for each request (hence `Arc` — clone is just an atomic increment). For mutable state, wrap fields in `Mutex` or `RwLock`. For read-only state (like config or database pool), no lock is needed. The type system ensures handlers extract the correct state type at compile time.

### Q: What is graceful shutdown?

> **Short**: Stopping the server cleanly — finish in-flight requests, close connections, save state, then exit.
> 
> **Detailed**: Listen for SIGTERM/Ctrl+C. Signal all tasks via `CancellationToken`. Stop accepting new requests. Wait for in-flight requests to complete (with timeout). Close database connections and WebSocket streams. Flush logs. Axum supports this via `.with_graceful_shutdown()`. Critical for trading systems — you don't want to lose pending transactions.

---

## Trading Systems

### Q: What is backpressure and why does it matter?

> **Short**: Backpressure slows producers when consumers can't keep up, preventing memory exhaustion.
> 
> **Detailed**: If market events arrive at 10,000/s but your strategy processes 1,000/s, an unbounded queue would grow without limit until the system runs out of memory. Bounded channels (`mpsc::channel(1000)`) provide backpressure — when the channel is full, `send().await` yields until space is available, automatically throttling the producer. Essential for stable, long-running trading systems.

### Q: Why is Rust good for trading systems?

> **Short**: No GC pauses, predictable latency, compile-time safety, zero-cost abstractions, efficient async for I/O.
> 
> **Detailed**: Garbage-collected languages (Java, Go) have unpredictable pauses that can cost trading opportunities. Rust gives: (1) No GC — deterministic memory management. (2) Compile-time thread safety — no data races in production. (3) Zero-cost async — thousands of connections with minimal overhead. (4) Predictable performance — no runtime surprises. (5) Strong type system — catches logic errors early. The tradeoff is development speed, but for latency-sensitive systems, the investment pays off.

---

> **Next**: [Chapter 39: Cheat Sheets](./39_cheat_sheets.md)
