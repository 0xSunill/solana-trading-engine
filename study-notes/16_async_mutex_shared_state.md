# Chapter 16: Async Mutex and Shared State

> **Connection to the bigger picture**: In async code, you still need shared mutable state. But using `std::sync::Mutex` across `.await` points can block entire Tokio worker threads. `tokio::sync::Mutex` is async-aware — it yields the task instead of blocking the thread. Knowing when to use which is critical for Solana backend performance.

---

## `std::sync::Mutex` vs `tokio::sync::Mutex`

### `std::sync::Mutex` — Blocks the thread

```rust
use std::sync::{Arc, Mutex};

let data = Arc::new(Mutex::new(0));
let d = data.clone();

tokio::spawn(async move {
    let mut guard = d.lock().unwrap();  // Blocks the OS thread until lock available
    *guard += 1;
    // guard dropped → lock released
});
```

### `tokio::sync::Mutex` — Yields the task

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(0));
let d = data.clone();

tokio::spawn(async move {
    let mut guard = d.lock().await;  // Yields the task, thread runs other tasks
    *guard += 1;
    // guard dropped → lock released
});
```

### When to use which

| | `std::sync::Mutex` | `tokio::sync::Mutex` |
|---|---|---|
| **Lock/unlock** | Synchronous, blocks thread | Async, yields task |
| **Holding across `.await`** | ❌ DANGEROUS | ✅ Safe |
| **Performance** | Faster for quick locks | Slightly slower |
| **Use when** | Lock is held briefly, no `.await` inside | Lock is held across `.await` points |

### The danger of `std::sync::Mutex` across `.await`

```rust
// ❌ DANGEROUS — DO NOT DO THIS
let data = Arc::new(std::sync::Mutex::new(0));
let d = data.clone();

tokio::spawn(async move {
    let mut guard = d.lock().unwrap();
    // If this .await suspends the task, the lock is held
    // and the entire thread is blocked waiting for the lock
    some_async_operation().await;  // ← Task suspends, lock still held!
    *guard += 1;
});
```

If the task suspends while holding a `std::sync::Mutex`, other tasks on the same thread can't proceed, AND other tasks trying to acquire the same lock will block their threads too. This can effectively deadlock the runtime.

### The correct approach

```rust
// ✅ Option 1: Use tokio::sync::Mutex for locks across .await
let data = Arc::new(tokio::sync::Mutex::new(0));
let d = data.clone();

tokio::spawn(async move {
    let mut guard = d.lock().await;
    some_async_operation().await;  // ✅ Safe — yields task, not thread
    *guard += 1;
});

// ✅ Option 2: Use std::sync::Mutex but don't hold across .await
let data = Arc::new(std::sync::Mutex::new(0));
let d = data.clone();

tokio::spawn(async move {
    let value = {
        let guard = d.lock().unwrap();
        *guard  // Copy the value
    };  // Lock released BEFORE .await
    
    some_async_operation().await;
    
    {
        let mut guard = d.lock().unwrap();
        *guard = value + 1;
    }  // Lock released immediately
});
```

---

## Tokio's Recommendation

From the [Tokio documentation](https://docs.rs/tokio):

> Use `std::sync::Mutex` when the lock is held for a very short time and never across `.await` points. Use `tokio::sync::Mutex` when the lock must be held across `.await` points or when contention is expected.

`std::sync::Mutex` is actually faster for short critical sections because it doesn't involve the async runtime's overhead. The key rule: **don't hold it across `.await`**.

---

## Things to Remember

1. **`std::sync::Mutex`** — faster but blocks threads. Never hold across `.await`.
2. **`tokio::sync::Mutex`** — async-aware, yields tasks. Safe across `.await`.
3. **Minimize lock scope** — regardless of which Mutex you use.
4. **Consider channels instead** — they often simplify shared state management.

---

> **Next**: [Chapter 17: Blocking vs Non-Blocking Operations](./17_blocking_vs_nonblocking.md)
