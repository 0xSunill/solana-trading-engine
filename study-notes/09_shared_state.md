# Chapter 9: Shared State — Mutex, RwLock, and Arc

> **Connection to the bigger picture**: When multiple threads or async tasks need to access the same data, you have two main approaches: shared state (this chapter) and message passing (next chapter). Shared state using `Arc<Mutex<T>>` is the most common pattern in Rust backends. In Solana trading systems, shared state holds order books, position trackers, and configuration that multiple handlers and background tasks access concurrently.

---

## Mutex in Detail

### Why Mutex exists

Multiple threads accessing the same data simultaneously can corrupt it:

```
Thread A reads counter: 5
Thread B reads counter: 5
Thread A writes counter: 6   (5 + 1)
Thread B writes counter: 6   (5 + 1)
Expected: 7, Got: 6          ← LOST UPDATE
```

A Mutex (mutual exclusion) ensures only one thread can access the data at a time.

### How Mutex works

```rust
use std::sync::Mutex;

fn main() {
    let m = Mutex::new(5);
    
    {
        let mut num = m.lock().unwrap();  // Acquire lock, get MutexGuard
        *num = 6;                         // Modify through the guard
        println!("Value: {}", num);
        // MutexGuard dropped here → lock automatically released (RAII)
    }
    
    println!("Final: {:?}", m);  // Mutex { data: 6 }
}
```

**Step by step:**

1. `m.lock()` — attempts to acquire the lock.
   - If no one else holds it: succeeds immediately, returns `Ok(MutexGuard)`.
   - If another thread holds it: **blocks** (the current thread sleeps until the lock is available).
   - If the mutex is poisoned (previous holder panicked): returns `Err(PoisonError)`.

2. `MutexGuard<T>` — a smart pointer that:
   - Implements `Deref<Target = T>` — gives you `&T`.
   - Implements `DerefMut` — gives you `&mut T`.
   - Automatically releases the lock when dropped (RAII pattern).

3. When the guard goes out of scope, the lock is released and other threads can acquire it.

### RAII Guards — Automatic Unlock

```rust
use std::sync::Mutex;

fn main() {
    let m = Mutex::new(String::from("hello"));
    
    // Guard is created
    let mut guard = m.lock().unwrap();
    guard.push_str(" world");
    // What if we forget to unlock? No problem!
    // When `guard` is dropped, the lock is released automatically.
    
    drop(guard);  // Explicitly dropping — lock released here
    
    // Or just let it go out of scope:
    {
        let guard = m.lock().unwrap();
        println!("{}", guard);
    }  // guard dropped → lock released
}
```

> **Key idea**: In C/C++, forgetting to unlock a mutex is a common bug that causes deadlocks. Rust's RAII pattern makes this impossible — the lock is always released when the guard is dropped.

### Poisoning

When a thread panics while holding a mutex lock, the mutex becomes "poisoned":

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);
    
    // This thread panics while holding the lock
    let handle = thread::spawn(move || {
        let mut guard = data_clone.lock().unwrap();
        guard.push(4);
        panic!("oops!");
        // guard is dropped during stack unwinding → lock released
        // BUT the mutex is now "poisoned"
    });
    
    let _ = handle.join();  // Thread panicked
    
    // Trying to lock a poisoned mutex:
    match data.lock() {
        Ok(guard) => println!("Data: {:?}", guard),
        Err(poisoned) => {
            println!("Mutex was poisoned!");
            // You can still access the data:
            let guard = poisoned.into_inner();
            println!("Data (possibly inconsistent): {:?}", guard);
        }
    }
}
```

**Why poisoning exists**: The thread that panicked might have left the data in an inconsistent state (e.g., partially updated). Poisoning warns other threads about this.

### Deadlocks

A deadlock occurs when two or more threads are each waiting for a lock held by the other:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let lock_a = Arc::new(Mutex::new(0));
    let lock_b = Arc::new(Mutex::new(0));
    
    let a1 = Arc::clone(&lock_a);
    let b1 = Arc::clone(&lock_b);
    let a2 = Arc::clone(&lock_a);
    let b2 = Arc::clone(&lock_b);
    
    // Thread 1: locks A then tries to lock B
    let t1 = thread::spawn(move || {
        let _a = a1.lock().unwrap();
        thread::sleep(std::time::Duration::from_millis(100));
        let _b = b1.lock().unwrap();  // DEADLOCK: waiting for B
    });
    
    // Thread 2: locks B then tries to lock A
    let t2 = thread::spawn(move || {
        let _b = b2.lock().unwrap();
        thread::sleep(std::time::Duration::from_millis(100));
        let _a = a2.lock().unwrap();  // DEADLOCK: waiting for A
    });
    
    t1.join().unwrap();
    t2.join().unwrap();
    // This program hangs forever!
}
```

**Preventing deadlocks:**
1. Always acquire locks in the same order across all threads.
2. Minimize the time locks are held.
3. Use `try_lock()` with a timeout.
4. Prefer channels over shared state when possible.

> **Important**: Rust prevents **data races** at compile time, but NOT deadlocks. Deadlocks are a logic error, not a memory safety issue.

---

## RwLock in Detail

`RwLock` (Read-Write Lock) allows either multiple readers or a single writer, but not both.

```rust
use std::sync::RwLock;

fn main() {
    let lock = RwLock::new(vec![1, 2, 3]);
    
    // Multiple readers simultaneously
    {
        let r1 = lock.read().unwrap();   // Read lock
        let r2 = lock.read().unwrap();   // Another read lock — OK!
        println!("r1: {:?}, r2: {:?}", r1, r2);
    }  // Both read locks released
    
    // Single writer (exclusive)
    {
        let mut w = lock.write().unwrap();  // Write lock
        w.push(4);
        // No readers or other writers allowed while w exists
    }  // Write lock released
}
```

### When RwLock helps

**Good for**: Data that is read far more often than written.

```
Example: Configuration data
- Read by 100 handlers every second
- Updated once per minute

With Mutex: Every read blocks other readers → bottleneck
With RwLock: 100 readers can read simultaneously → much better
```

### When RwLock can be worse

**Bad for**: Write-heavy workloads.

```
Example: Counter incremented by every request
- Every access is a write
- RwLock overhead > Mutex overhead (tracks readers + writers)
- Writer must wait for ALL readers to finish

With RwLock: Higher overhead, writers starved by readers
With Mutex: Simpler, similar performance
```

### Mutex vs RwLock comparison

| | `Mutex<T>` | `RwLock<T>` |
|---|---|---|
| Read access | Exclusive (one at a time) | Shared (many simultaneous) |
| Write access | Exclusive | Exclusive |
| Overhead | Lower | Higher |
| Best for | Write-heavy or mixed | Read-heavy |
| Deadlock risk | Lower | Higher (writer starvation) |
| API | `.lock()` | `.read()` / `.write()` |

---

## Arc Deep Dive

### Reference counting in detail

```rust
use std::sync::Arc;

fn main() {
    let a = Arc::new(String::from("hello"));
    // State: ref_count = 1, data = "hello"
    
    let b = Arc::clone(&a);
    // State: ref_count = 2 (atomic increment)
    // Both a and b point to the SAME heap allocation
    
    let c = Arc::clone(&a);
    // State: ref_count = 3
    
    drop(c);
    // State: ref_count = 2 (atomic decrement)
    
    drop(b);
    // State: ref_count = 1
    
    drop(a);
    // State: ref_count = 0 → data is freed!
}
```

### Why `Rc` is not thread-safe (in detail)

```
Rc internal reference count: a normal integer (e.g., i32)

Thread A: read count (5) → increment → write count (6)
Thread B: read count (5) → increment → write count (6)

Both threads read 5 at the "same time"
Both write 6
Actual count should be 7, but it's 6

Later: one Rc is dropped, count goes to 5
But there are still 6 Rcs alive
When count reaches 0, data is freed
But 1 Rc still exists → USE-AFTER-FREE!
```

Arc fixes this by using **atomic operations** — CPU instructions that are guaranteed to complete without interruption:

```
Arc internal reference count: AtomicUsize

Thread A: atomic_increment(count)  // CPU guarantees this is indivisible
Thread B: atomic_increment(count)  // CPU guarantees this is indivisible

Result is always correct: 7
```

### Arc + Mutex pattern explained

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Create shared state
    let state = Arc::new(Mutex::new(Vec::new()));
    // Arc wraps Mutex wraps Vec
    // Arc: shared ownership (multiple threads can hold a reference)
    // Mutex: exclusive access (only one thread accesses Vec at a time)
    // Vec: the actual data
    
    let mut handles = vec![];
    
    for i in 0..5 {
        let state = Arc::clone(&state);  // Clone Arc (not Mutex or Vec!)
        // Each clone increments the atomic ref count
        
        let handle = thread::spawn(move || {
            // move: ownership of this Arc clone moves into the thread
            
            let mut data = state.lock().unwrap();
            // lock: blocks until we have exclusive access
            // data: MutexGuard that gives us &mut Vec
            
            data.push(i);
            // Push to the Vec through the MutexGuard
            
            // MutexGuard dropped here → lock released
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let data = state.lock().unwrap();
    println!("Final data: {:?}", data);
    // Order may vary: [0, 3, 1, 4, 2] (depends on thread scheduling)
}
```

**Data flow diagram:**

```
Thread 0          Thread 1          Thread 2
   │                 │                 │
   ▼                 ▼                 ▼
Arc::clone()     Arc::clone()     Arc::clone()
   │                 │                 │
   └────────────────┼────────────────┘
                     │
                     ▼
              ┌─────────────┐
              │   Arc       │  ref_count: 4 (original + 3 clones)
              │  ┌────────┐ │
              │  │ Mutex  │ │
              │  │ ┌────┐ │ │
              │  │ │ Vec │ │ │  ← Only ONE thread accesses at a time
              │  │ └────┘ │ │
              │  └────────┘ │
              └─────────────┘
```

---

## Real-World Pattern: Application State

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

// Shared application state for a Solana trading backend
struct AppState {
    // Read-heavy: use RwLock
    prices: RwLock<HashMap<String, f64>>,
    
    // Write-heavy: use Mutex
    pending_orders: std::sync::Mutex<Vec<Order>>,
    
    // Read-only: no lock needed
    config: Config,
}

struct Order {
    id: u64,
    symbol: String,
    price: f64,
}

struct Config {
    rpc_url: String,
    max_orders: usize,
}

fn main() {
    let state = Arc::new(AppState {
        prices: RwLock::new(HashMap::new()),
        pending_orders: std::sync::Mutex::new(Vec::new()),
        config: Config {
            rpc_url: "https://api.mainnet-beta.solana.com".into(),
            max_orders: 100,
        },
    });
    
    // Reading prices (many concurrent readers)
    let state_clone = Arc::clone(&state);
    let reader = std::thread::spawn(move || {
        let prices = state_clone.prices.read().unwrap();
        if let Some(price) = prices.get("SOL/USD") {
            println!("SOL price: {}", price);
        }
    });
    
    // Updating prices (exclusive writer)
    let state_clone = Arc::clone(&state);
    let writer = std::thread::spawn(move || {
        let mut prices = state_clone.prices.write().unwrap();
        prices.insert("SOL/USD".into(), 150.0);
    });
    
    // Accessing read-only config (no lock needed)
    println!("RPC URL: {}", state.config.rpc_url);
    
    reader.join().unwrap();
    writer.join().unwrap();
}
```

---

## Things to Remember

1. **Mutex provides exclusive access** — one thread at a time. RAII guard releases automatically.
2. **RwLock allows multiple readers** — but exclusive writers. Better for read-heavy workloads.
3. **Arc provides shared ownership** — atomic reference counting for thread safety.
4. **`Arc<Mutex<T>>`** — the standard pattern for shared mutable state.
5. **Minimize lock scope** — hold locks for the shortest time possible.
6. **Deadlocks are possible** — Rust prevents data races, NOT deadlocks.
7. **Poisoned mutexes** indicate a previous holder panicked — data might be inconsistent.
8. **Lock-free when possible** — read-only shared data with `Arc<T>` needs no lock.

---

## Interview Questions

**Q: What is a deadlock and how do you prevent it?**

> A deadlock occurs when two or more threads each hold a lock and wait for a lock held by the other. Prevention strategies: always acquire locks in the same order, minimize lock duration, use `try_lock()` with timeouts, or use message passing instead of shared state.

**Q: When would you choose `RwLock` over `Mutex`?**

> When reads are much more frequent than writes. `RwLock` allows multiple simultaneous readers, improving throughput for read-heavy workloads. For write-heavy or balanced workloads, `Mutex` is simpler and can be faster due to lower overhead.

**Q: Does Rust prevent deadlocks?**

> No. Rust prevents data races (undefined behavior from unsynchronized concurrent access) at compile time through the ownership system and `Send`/`Sync` traits. Deadlocks are logic errors — they don't cause undefined behavior, just programs that hang. Preventing deadlocks requires careful design.

---

> **Next**: [Chapter 10: Channels and Message Passing](./10_channels_and_message_passing.md) — An alternative to shared state
