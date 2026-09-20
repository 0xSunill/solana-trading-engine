# Chapter 5: Smart Pointers

> **Connection to the bigger picture**: Smart pointers solve problems that ownership alone can't handle — shared ownership, interior mutability, heap allocation, and thread-safe shared state. In concurrent Rust, `Arc<Mutex<T>>` is the most common pattern for sharing mutable state between threads/tasks. In async Rust, you'll use `Arc` constantly. Understanding these smart pointers and when to use each one is essential for building concurrent Solana backends.

---

## What Are Smart Pointers?

Smart pointers are data structures that act like pointers but also have additional metadata and capabilities. Unlike regular references (`&T`), smart pointers often **own** the data they point to.

| Smart Pointer | Purpose | Thread-safe? |
|--------------|---------|-------------|
| `Box<T>` | Heap allocation, single owner | Yes (if T is Send) |
| `Rc<T>` | Shared ownership (reference counting) | **No** |
| `Arc<T>` | Shared ownership (atomic reference counting) | **Yes** |
| `RefCell<T>` | Interior mutability (runtime borrow checking) | **No** |
| `Mutex<T>` | Mutual exclusion (thread-safe interior mutability) | **Yes** |
| `RwLock<T>` | Read-write lock (multiple readers OR one writer) | **Yes** |

---

## `Box<T>` — Heap Allocation

`Box<T>` puts a value on the heap and gives you a pointer to it.

```rust
fn main() {
    let x = Box::new(42);  // 42 is allocated on the heap
    println!("x = {}", x); // Auto-deref: treats Box<i32> like i32
}
```

**Memory layout:**

```
Stack                    Heap
┌──────────────┐        ┌──────┐
│ x: Box<i32>  │        │      │
│  ptr ────────┼───────▶│  42  │
└──────────────┘        └──────┘
```

### When to use `Box<T>`

**1. Recursive types (required)**

```rust
// This won't compile — infinite size
// enum List {
//     Cons(i32, List),  // How big is List? It contains another List...
//     Nil,
// }

// Box breaks the recursion — Box has a known size (pointer)
enum List {
    Cons(i32, Box<List>),
    Nil,
}

fn main() {
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
}
```

**2. Trait objects**

```rust
fn create_handler(debug: bool) -> Box<dyn Fn(i32) -> String> {
    if debug {
        Box::new(|x| format!("DEBUG: {}", x))
    } else {
        Box::new(|x| format!("{}", x))
    }
}
```

**3. Large data that you want to avoid copying**

```rust
struct LargeData {
    buffer: [u8; 1_000_000],  // 1MB on the stack? Bad idea.
}

let data = Box::new(LargeData { buffer: [0; 1_000_000] });
// Now only a pointer (8 bytes) is on the stack
```

### `Box<T>` ownership

Box has single ownership — when the Box is dropped, the heap data is freed:

```rust
fn main() {
    let data = Box::new(String::from("hello"));
    let moved = data;  // Ownership moves
    // println!("{}", data);  // ❌ data is moved
    println!("{}", moved);    // ✅
}  // moved is dropped → heap memory is freed
```

---

## `Rc<T>` — Reference Counting (Single-Threaded)

`Rc<T>` enables **multiple owners** of the same data. It keeps a count of how many `Rc`s point to the data. When the count reaches zero, the data is freed.

```rust
use std::rc::Rc;

fn main() {
    let a = Rc::new(String::from("hello"));  // ref count = 1
    println!("Count after a: {}", Rc::strong_count(&a));  // 1
    
    let b = Rc::clone(&a);  // ref count = 2 (doesn't clone the String!)
    println!("Count after b: {}", Rc::strong_count(&a));  // 2
    
    {
        let c = Rc::clone(&a);  // ref count = 3
        println!("Count after c: {}", Rc::strong_count(&a));  // 3
    }  // c is dropped → ref count = 2
    
    println!("Count after c dropped: {}", Rc::strong_count(&a));  // 2
}  // b dropped (count=1), a dropped (count=0) → String is freed
```

**Memory layout:**

```
Stack                               Heap
┌──────────────┐                   ┌──────────────────────┐
│ a: Rc<String> │                   │ ref_count: 2         │
│  ptr ─────────┼──────────────────▶│ data: String         │
└──────────────┘                   │  ptr ──▶ "hello"     │
                                   └──────────────────────┘
┌──────────────┐                        ▲
│ b: Rc<String> │                        │
│  ptr ─────────┼────────────────────────┘
└──────────────┘
```

### `Rc::clone` is cheap

`Rc::clone(&a)` does NOT clone the underlying data. It just increments the reference count (a single integer increment). This is very different from `a.clone()` on a `String`, which would copy all the heap data.

### Why `Rc` is NOT thread-safe

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let data = Rc::new(42);
    
    // ❌ COMPILE ERROR
    thread::spawn(move || {
        println!("{}", data);
    });
}
```

**Compiler error:**

```
error[E0277]: `Rc<i32>` cannot be sent between threads safely
   |
   = help: the trait `Send` is not implemented for `Rc<i32>`
```

**Why**: `Rc`'s reference count is a regular integer. If two threads increment/decrement it simultaneously, they might both read the same count, both increment it, and the count ends up wrong. This is a **data race on the reference count itself**.

```
Thread A reads count: 2          Thread B reads count: 2
Thread A increments: 3           Thread B increments: 3
                                 Actual should be: 4  ← BUG!
```

This could lead to the data being freed while references still exist (use-after-free) or never being freed (memory leak).

> **Rule**: `Rc` is for single-threaded scenarios only. For multi-threaded, use `Arc`.

---

## `Arc<T>` — Atomic Reference Counting (Thread-Safe)

`Arc<T>` is the thread-safe version of `Rc<T>`. It uses **atomic operations** to update the reference count, which are safe for concurrent access.

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(String::from("shared data"));
    
    let mut handles = vec![];
    
    for i in 0..5 {
        let data_clone = Arc::clone(&data);  // Increment atomic ref count
        
        let handle = thread::spawn(move || {
            println!("Thread {}: {}", i, data_clone);
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

**What's happening:**

1. `Arc::new(...)` creates the data with an atomic reference count of 1.
2. `Arc::clone(&data)` increments the count atomically (thread-safe increment).
3. Each thread gets its own `Arc` that points to the same data.
4. When a thread's `Arc` is dropped, the count is atomically decremented.
5. When the count reaches 0, the data is freed.

### `Arc` gives you read-only access

```rust
use std::sync::Arc;

fn main() {
    let data = Arc::new(String::from("hello"));
    // data.push_str(" world");  // ❌ Can't mutate through Arc
    // Arc only gives you &T (immutable reference)
    
    println!("{}", data);  // ✅ Reading is fine
}
```

To mutate data behind `Arc`, you need interior mutability — that's where `Mutex` and `RwLock` come in.

### `Rc<T>` vs `Arc<T>`

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| **Thread-safe** | No | Yes |
| **Performance** | Slightly faster (non-atomic ops) | Slightly slower (atomic ops) |
| **Use in threads** | ❌ Not Send | ✅ Send (if T: Send + Sync) |
| **Use in async tasks** | Only with `LocalSet` | ✅ Standard choice |
| **Reference counting** | Regular integer | Atomic integer |

> **Real-world use**: In Solana backends, `Arc` is everywhere. Your application state (database pool, configuration, WebSocket connections) is wrapped in `Arc` so it can be shared across all Axum handlers and background tasks.

---

## `RefCell<T>` — Interior Mutability (Single-Threaded)

`RefCell<T>` lets you mutate data even when you only have an immutable reference. It moves borrow checking from compile time to runtime.

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(vec![1, 2, 3]);
    
    // Borrow immutably
    let borrowed = data.borrow();
    println!("{:?}", borrowed);  // [1, 2, 3]
    drop(borrowed);  // Must drop before mutable borrow
    
    // Borrow mutably
    let mut borrowed_mut = data.borrow_mut();
    borrowed_mut.push(4);
    println!("{:?}", borrowed_mut);  // [1, 2, 3, 4]
}
```

### Runtime borrow checking

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(42);
    
    let r1 = data.borrow();      // Immutable borrow
    let r2 = data.borrow_mut();  // 💥 PANIC at runtime!
    // "already borrowed: BorrowMutError"
}
```

The same rule applies (one mutable OR many immutable), but it's checked at runtime, not compile time. Violating it causes a panic instead of a compile error.

### When to use `RefCell`

- When you need to mutate data inside an immutable struct
- When the borrow checker can't understand that your borrows don't actually overlap
- Combined with `Rc` for shared mutable data: `Rc<RefCell<T>>`

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Shared mutable state (single-threaded only)
let shared = Rc::new(RefCell::new(0));

let a = Rc::clone(&shared);
let b = Rc::clone(&shared);

*a.borrow_mut() += 1;
*b.borrow_mut() += 1;

println!("Value: {}", shared.borrow());  // 2
```

> **Important**: `RefCell` is NOT thread-safe. For thread-safe interior mutability, use `Mutex` or `RwLock`.

---

## `Mutex<T>` — Mutual Exclusion (Thread-Safe)

`Mutex<T>` provides thread-safe interior mutability. Only one thread can access the data at a time.

```rust
use std::sync::Mutex;

fn main() {
    let counter = Mutex::new(0);
    
    // Lock the mutex to access the data
    let mut num = counter.lock().unwrap();
    *num += 1;
    println!("Counter: {}", num);
    // num (the MutexGuard) is dropped here → lock is released
}
```

### How `Mutex` works

```
Thread A: lock() → gets MutexGuard → reads/writes data → drop guard → unlock
Thread B: lock() → BLOCKS (waits for Thread A to release) → gets guard → ...
```

The `lock()` method:
1. Tries to acquire the lock.
2. If another thread holds the lock, **blocks** until it's available.
3. Returns a `MutexGuard<T>` — a smart pointer that:
   - Gives you `&mut T` access to the data
   - Automatically releases the lock when dropped (RAII)

### `Mutex` poisoning

If a thread panics while holding a `Mutex` lock, the mutex becomes "poisoned":

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(42));
    let data_clone = Arc::clone(&data);
    
    let handle = thread::spawn(move || {
        let mut guard = data_clone.lock().unwrap();
        *guard += 1;
        panic!("oh no!");  // Panics while holding the lock
    });
    
    let _ = handle.join();  // Thread panicked
    
    // The mutex is now poisoned
    match data.lock() {
        Ok(guard) => println!("Data: {}", guard),
        Err(poisoned) => {
            // You can still access the data if you choose to
            let guard = poisoned.into_inner();
            println!("Recovered data: {}", guard);
        }
    }
}
```

(We'll cover Mutex in much more detail in Chapter 9.)

---

## `RwLock<T>` — Read-Write Lock (Thread-Safe)

`RwLock<T>` is like `Mutex<T>` but allows multiple simultaneous readers.

```rust
use std::sync::RwLock;

fn main() {
    let data = RwLock::new(vec![1, 2, 3]);
    
    // Multiple readers allowed simultaneously
    {
        let read1 = data.read().unwrap();
        let read2 = data.read().unwrap();
        println!("{:?}, {:?}", read1, read2);  // Both reading at the same time
    }
    
    // Only one writer at a time (and no readers while writing)
    {
        let mut write = data.write().unwrap();
        write.push(4);
    }
}
```

### `Mutex` vs `RwLock`

| | `Mutex<T>` | `RwLock<T>` |
|---|---|---|
| **Read access** | One thread at a time | Multiple threads simultaneously |
| **Write access** | One thread at a time | One thread at a time |
| **Read while writing** | Not applicable | Not allowed |
| **Simpler** | Yes | No |
| **Better when** | Writes are frequent, or reads are rare | Reads are much more frequent than writes |
| **Overhead** | Lower | Higher (tracks readers and writers) |

> **When RwLock can be worse**: If writes are frequent, `RwLock` has higher overhead than `Mutex` because it needs to track reader/writer counts. The writer must wait for ALL readers to finish. In practice, benchmark before choosing.

(We'll cover RwLock in more detail in Chapter 9.)

---

## The Most Common Pattern: `Arc<Mutex<T>>`

This is the standard pattern for shared mutable state across threads:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final count: {}", *counter.lock().unwrap());  // 10
}
```

**Why each layer:**

```
Arc<Mutex<i32>>
 │     │    │
 │     │    └── The actual data (i32)
 │     │
 │     └── Mutex: provides exclusive access (interior mutability)
 │         Only one thread can access the i32 at a time
 │
 └── Arc: provides shared ownership across threads
     Multiple threads can hold an Arc pointing to the same Mutex
```

Without `Arc`: Can't share the `Mutex` between threads (each thread needs its own reference).
Without `Mutex`: Can't mutate the data (Arc only gives `&T`, not `&mut T`).

> **Real-world use**: In a Solana trading engine, you might have:
> ```rust
> struct AppState {
>     order_book: Arc<RwLock<OrderBook>>,
>     positions: Arc<Mutex<HashMap<String, Position>>>,
>     config: Arc<Config>,  // Read-only, no lock needed
> }
> ```

---

## Decision Guide: Which Smart Pointer?

```
Do you need heap allocation?
├── Yes, single owner → Box<T>
└── Yes, multiple owners
    ├── Single-threaded?
    │   ├── Read-only → Rc<T>
    │   └── Need mutation → Rc<RefCell<T>>
    └── Multi-threaded?
        ├── Read-only → Arc<T>
        ├── Need mutation (any access pattern) → Arc<Mutex<T>>
        └── Need mutation (many readers, few writers) → Arc<RwLock<T>>
```

---

## Things to Remember

1. **`Box<T>`** = heap allocation with single ownership. Use for recursive types and large data.
2. **`Rc<T>`** = shared ownership, single-threaded. NOT Send, NOT Sync.
3. **`Arc<T>`** = shared ownership, thread-safe. The go-to for async Rust.
4. **`RefCell<T>`** = interior mutability, single-threaded. Runtime borrow checking.
5. **`Mutex<T>`** = interior mutability, thread-safe. One accessor at a time.
6. **`RwLock<T>`** = interior mutability, thread-safe. Multiple readers OR one writer.
7. **`Arc<Mutex<T>>`** is the standard pattern for shared mutable state in concurrent Rust.
8. **Don't use `Rc` in async code** (unless using `LocalSet`) — use `Arc` instead.
9. **Prefer `Arc<T>` for read-only shared data** — no lock needed.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "`Rc::clone` is expensive" | It just increments a counter. It does NOT clone the underlying data. |
| "Always use `Arc` instead of `Rc`" | Use `Rc` when you know code is single-threaded — it's slightly faster. |
| "Mutex prevents all concurrency bugs" | Mutex prevents data races, but doesn't prevent deadlocks or logic errors. |
| "RwLock is always better than Mutex" | RwLock has higher overhead. For write-heavy workloads, Mutex can be faster. |

---

## Interview Questions

**Q: What is the difference between `Rc` and `Arc`?**

> Both provide shared ownership through reference counting. `Rc` uses non-atomic operations (faster, single-threaded only). `Arc` uses atomic operations (slightly slower, thread-safe). `Rc` does not implement `Send` or `Sync`, so the compiler prevents using it across threads.

**Q: Why do we need `Arc<Mutex<T>>` instead of just `Mutex<T>` or just `Arc<T>`?**

> `Arc` alone only gives shared immutable access (`&T`). `Mutex` alone can't be shared across threads (each thread needs its own handle). Together: `Arc` provides shared ownership across threads, and `Mutex` provides exclusive mutable access to the inner data.

**Q: What is interior mutability?**

> Interior mutability allows you to mutate data even when accessed through an immutable reference. `RefCell` does this with runtime borrow checking (single-threaded). `Mutex` and `RwLock` do this with locking (thread-safe). This pattern is essential when you need shared mutable state.

---

> **Next**: [Chapter 6: Concurrency vs Parallelism](./06_concurrency_vs_parallelism.md) — Understanding the fundamentals before diving into threads
