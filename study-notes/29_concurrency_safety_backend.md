# Chapter 29: Concurrency Safety in Backend Systems

> **Connection to the bigger picture**: Rust prevents data races at compile time, but doesn't prevent ALL concurrency bugs. Race conditions, deadlocks, lost updates, and backpressure problems can still occur. Understanding what Rust protects you from — and what it doesn't — is critical for building correct trading systems.

---

## What Rust Prevents vs What It Doesn't

| Bug class | Rust prevents? | How |
|-----------|---------------|-----|
| **Data races** | ✅ Yes | Ownership + Send/Sync at compile time |
| **Dangling pointers** | ✅ Yes | Lifetimes at compile time |
| **Double frees** | ✅ Yes | Ownership (single owner) |
| **Buffer overflows** | ✅ Yes (mostly) | Bounds checking |
| **Race conditions** | ❌ No | Logic error, not memory safety |
| **Deadlocks** | ❌ No | Logic error, not memory safety |
| **Lost updates** | ❌ No | Logic error |
| **Priority inversion** | ❌ No | Design error |
| **Backpressure issues** | ❌ No | Design error |

---

## Race Conditions

A race condition is a logic error where the result depends on the order of operations.

```rust
// Both tasks check balance, both see $100, both withdraw $80
// Result: -$60 balance (should have rejected second withdrawal)

async fn withdraw(state: Arc<Mutex<f64>>, amount: f64) -> Result<(), String> {
    let mut balance = state.lock().await;
    if *balance >= amount {
        // ⚠️ Window between check and update
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        *balance -= amount;
        Ok(())
    } else {
        Err("Insufficient funds".into())
    }
}
```

**Fix**: Hold the lock through the entire check-and-update operation, or use database transactions with proper isolation levels.

---

## Deadlocks

```rust
// Task 1: lock A, then lock B
// Task 2: lock B, then lock A
// Both wait forever
```

**Prevention:**
1. Always acquire locks in the same order
2. Use `try_lock` with timeouts
3. Prefer channels over multiple locks
4. Use the actor model (single lock per actor)

---

## Backpressure

When producers create work faster than consumers process it:

```
Producer: 1000 events/sec ──▶ Channel ──▶ Consumer: 100 events/sec
                                  │
                          Buffer grows indefinitely!
                          Memory exhaustion!
```

**Fix**: Use bounded channels.

```rust
let (tx, rx) = mpsc::channel(100);  // Max 100 buffered messages
// tx.send().await WAITS when buffer is full — producer slows down
```

---

## Task Cancellation Safety

When a task is cancelled (via `select!` or `abort()`), any partially-completed work is lost:

```rust
// ⚠️ If cancelled between database write and cache update,
// data is inconsistent
async fn update_data(db: &PgPool, cache: &Mutex<Cache>) {
    db.execute("UPDATE ...").await?;  // ← might be cancelled HERE
    cache.lock().await.invalidate();  // ← never reached!
}
```

**Mitigation**: Use `tokio::spawn` for operations that must complete, or use transactions.

---

## Things to Remember

1. **Rust prevents data races**, not race conditions.
2. **Deadlocks are logic errors** — design your lock ordering carefully.
3. **Use bounded channels** for backpressure.
4. **Task cancellation** can leave state inconsistent — design for it.
5. **Database transactions** protect against concurrent modification.
6. **Test concurrent code** under load — bugs often appear only under contention.

---

> **Next**: [Chapter 30: Graceful Shutdown](./30_graceful_shutdown.md)
