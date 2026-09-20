# Chapter 8: Send and Sync

> **Connection to the bigger picture**: `Send` and `Sync` are the two traits that make Rust's thread safety guarantees possible at compile time. They're the bridge between the ownership system (Chapters 1-3) and concurrent execution (Chapters 6-7). When you see the error "future cannot be sent between threads safely," it's `Send` being enforced. When you see "cannot be shared between threads safely," it's `Sync`. Understanding these traits deeply is essential because `tokio::spawn` requires `Send`, and almost every async Rust compiler error about thread safety comes back to these two traits.

---

## The Problem These Traits Solve

In C/C++, you can share any data between threads. The compiler doesn't stop you. If you share data incorrectly, you get data races — undefined behavior that can cause crashes, corrupt data, or produce wrong results silently.

Rust makes thread safety a **compile-time guarantee**. The mechanism: two marker traits that the compiler checks automatically.

```
Can this value be moved to another thread?  → Send
Can references to this value be shared across threads?  → Sync
```

---

## `Send` — Transfer Between Threads

### What `Send` means

`Send` means: **a value of this type can be safely transferred from one thread to another.**

When you move a value to another thread (via `thread::spawn`, `tokio::spawn`, or a channel), Rust checks that the value implements `Send`.

```rust
use std::thread;

fn main() {
    let data = String::from("hello");  // String: Send ✅
    
    thread::spawn(move || {
        // data's ownership has been transferred to this thread
        println!("{}", data);
    }).join().unwrap();
}
```

### What "safely transferred" actually means

When a value is moved to another thread:

1. **The original thread gives up access** — ownership moves.
2. **The new thread becomes the sole owner** — no other thread can access it.
3. **The type's internal state is safe to access from any thread.**

For most types, this is trivially true. The value is just bytes in memory, and since ownership ensures only one thread can access it, there's no conflict.

But some types have internal state that is **thread-specific**:

```rust
use std::rc::Rc;

// Rc uses a NON-ATOMIC reference count.
// If you move an Rc to another thread, and both threads try to
// clone/drop Rc handles, the non-atomic counter gets corrupted.
let data = Rc::new(42);

// ❌ COMPILE ERROR: Rc<i32> is not Send
// thread::spawn(move || {
//     println!("{}", data);
// });
```

### Types that are `Send`

Most types are `Send`:

| Type | Send? | Why |
|------|-------|-----|
| `i32`, `f64`, `bool`, `char` | ✅ | Pure values, no shared state |
| `String` | ✅ | Owns its heap data exclusively |
| `Vec<T>` (if T: Send) | ✅ | Owns its elements exclusively |
| `Box<T>` (if T: Send) | ✅ | Owns its data exclusively |
| `Arc<T>` (if T: Send + Sync) | ✅ | Atomic ref count, safe to share |
| `Mutex<T>` (if T: Send) | ✅ | Lock ensures exclusive access |
| `Sender<T>`, `Receiver<T>` | ✅ | Channel endpoints designed for threads |
| `Option<T>` (if T: Send) | ✅ | Wraps T |

### Types that are NOT `Send`

| Type | Send? | Why not |
|------|-------|---------|
| `Rc<T>` | ❌ | Non-atomic ref count — would corrupt |
| `*const T`, `*mut T` | ❌ | Raw pointers — compiler can't verify safety |
| `MutexGuard<T>` | ❌ (on some platforms) | Must be unlocked from the same thread |
| Types containing non-Send fields | ❌ | Auto trait propagation |

### How the compiler uses `Send`

`Send` is an **auto trait**. The compiler automatically implements it for your types if all fields are `Send`:

```rust
struct MyData {
    name: String,        // Send ✅
    count: i32,          // Send ✅
    items: Vec<String>,  // Send ✅
}
// MyData is automatically Send ✅ (all fields are Send)

struct NotSendable {
    name: String,                // Send ✅
    cached: std::rc::Rc<String>, // Send ❌
}
// NotSendable is automatically NOT Send ❌ (one field is not Send)
```

You don't write `impl Send for MyData`. The compiler does it automatically. This is why it's called an "auto trait."

---

## `Sync` — Shared References Between Threads

### What `Sync` means

`Sync` means: **it is safe for multiple threads to hold `&T` references simultaneously.**

More precisely:

```
T is Sync  ⟺  &T is Send
```

This means: "If I can safely send a reference to T to another thread, then T is Sync."

### Why `Sync` matters

When multiple threads need to read the same data:

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3]);
    
    // Multiple threads get &Vec<i32> through the Arc
    let mut handles = vec![];
    for i in 0..3 {
        let data = Arc::clone(&data);  // Clone Arc (not Vec)
        handles.push(thread::spawn(move || {
            // data.as_ref() gives &Vec<i32>
            // This is safe because Vec<i32> is Sync
            println!("Thread {}: {:?}", i, *data);
        }));
    }
    
    for h in handles { h.join().unwrap(); }
}
```

`Arc<T>` gives shared access. For `Arc<T>` to be `Send` (so you can clone and move it to threads), `T` must be `Send + Sync`:

- `Send` because Arc might drop on a different thread (the last Arc drops the data)
- `Sync` because multiple threads can access `&T` through their Arcs simultaneously

### Types that are `Sync`

| Type | Sync? | Why |
|------|-------|-----|
| `i32`, `f64`, `bool` | ✅ | Immutable shared access is always safe |
| `String`, `Vec<T>` | ✅ | If T is Sync; immutable refs are safe |
| `&T` (if T: Sync) | ✅ | Sharing a shared reference is fine |
| `Mutex<T>` | ✅ | Lock ensures exclusive access |
| `RwLock<T>` | ✅ | Lock ensures safe concurrent reads |
| `Arc<T>` (if T: Send + Sync) | ✅ | Atomic ref counting |
| `AtomicI32`, `AtomicBool` | ✅ | Designed for concurrent access |

### Types that are NOT `Sync`

| Type | Sync? | Why not |
|------|-------|---------|
| `Cell<T>` | ❌ | Interior mutability without synchronization |
| `RefCell<T>` | ❌ | Runtime borrow checking not thread-safe |
| `Rc<T>` | ❌ | Non-atomic ref count |
| `*const T`, `*mut T` | ❌ | Raw pointers |

### Why `Cell` and `RefCell` are not `Sync`

```rust
use std::cell::Cell;

// Cell allows mutation through &Cell<T>
let value = Cell::new(0);

// If two threads had &Cell<i32> simultaneously:
// Thread A: value.set(1)
// Thread B: value.set(2)  // ← DATA RACE on the inner value
```

`Cell` and `RefCell` provide interior mutability WITHOUT synchronization. If two threads accessed them simultaneously, the internal data could be corrupted. They're perfectly safe in single-threaded code but not across threads.

---

## The Relationship: `Send` vs `Sync`

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│  Send:  Can I MOVE this value to another thread?            │
│         (ownership transfer)                                │
│                                                             │
│  Sync:  Can multiple threads SHARE &T references?           │
│         (shared access)                                     │
│                                                             │
│  Formal relationship:                                       │
│    T: Sync  ⟺  &T: Send                                   │
│                                                             │
│  Informal:                                                  │
│    Send = "I can give you this"                             │
│    Sync = "We can both look at this"                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Detailed comparison table

| Concept | Send | Sync |
|---------|------|------|
| **Question** | Can this value be moved to another thread? | Can `&T` be shared between threads? |
| **Transferring** | Ownership | References |
| **Key operation** | `thread::spawn(move \|\| ...)` | `Arc<T>` shared access |
| **Example: Pass** | `String` (owns data, transfer is safe) | `Vec<i32>` (immutable refs are safe) |
| **Example: Fail** | `Rc<T>` (non-atomic ref count) | `RefCell<T>` (unsynchronized mutation) |
| **Used by** | `thread::spawn`, `tokio::spawn`, channels | `Arc<T>`, shared references |
| **Formal** | "T can be moved to another thread" | "&T can be sent to another thread" |

### Common misconceptions

| Misconception | Reality |
|---------------|---------|
| "Send means sending messages" | No — Send means the value's ownership can be safely transferred between threads. It has nothing to do with message passing. |
| "Sync means synchronous" | No — Sync means the type can be safely accessed by multiple threads simultaneously via shared references. It has nothing to do with sync vs async. |
| "Send and Sync are about performance" | No — they're about safety. They have zero runtime cost. They're compile-time markers. |
| "I need to implement Send manually" | Almost never. Send is an auto trait — the compiler implements it automatically based on your type's fields. |
| "If T is Send, it's also Sync" | Not necessarily. Some types are Send but not Sync (like `Cell<T>` — you can move it between threads, but multiple threads shouldn't share references). |

---

## `Send` + `Sync` in Practice

### Pattern 1: Moving data to a thread

```rust
use std::thread;

// Works because String is Send
let name = String::from("Sunil");
thread::spawn(move || {
    println!("{}", name);
});
```

### Pattern 2: Sharing data between threads

```rust
use std::sync::Arc;
use std::thread;

// Works because Vec<i32> is Send + Sync
let data = Arc::new(vec![1, 2, 3]);
let d = Arc::clone(&data);
thread::spawn(move || {
    println!("{:?}", d);  // d gives &Vec<i32>, which is safe because Vec is Sync
});
```

### Pattern 3: Mutable shared state

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// Mutex<T> is Sync (even if T isn't Sync!)
// Because Mutex provides synchronization
let counter = Arc::new(Mutex::new(0));
let c = Arc::clone(&counter);
thread::spawn(move || {
    *c.lock().unwrap() += 1;
});
```

### Pattern 4: The `tokio::spawn` signature

```rust
// tokio::spawn requires:
// F: Future + Send + 'static
// F::Output: Send + 'static

// This means the async block and everything it captures must be Send.
// Any data held across .await points must be Send.

#[tokio::main]
async fn main() {
    let data = String::from("hello");  // Send ✅
    
    tokio::spawn(async move {
        println!("{}", data);
    });
}
```

### Pattern 5: Non-Send type causing errors

```rust
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let data = Rc::new(42);  // NOT Send
    
    // ❌ COMPILE ERROR
    tokio::spawn(async move {
        println!("{}", data);
    });
}
```

Error:
```
error: future cannot be sent between threads safely
   |
   = help: within `impl Future<Output = ()>`, the trait `Send` is not 
     implemented for `Rc<i32>`
```

Fix: Use `Arc` instead of `Rc`.

---

## Why Some `Send`/`Sync` Combinations Exist

| Send? | Sync? | Example | Explanation |
|-------|-------|---------|-------------|
| ✅ | ✅ | `i32`, `String`, `Vec<T>`, `Arc<T>` | Can be moved to threads, can be shared via references |
| ✅ | ❌ | `Cell<T>`, `mpsc::Sender<T>` | Can be moved to another thread, but shouldn't be shared (interior mutability without sync) |
| ❌ | ❌ | `Rc<T>` | Neither move nor share — single-threaded only |
| ❌ | ✅ | `MutexGuard<T>` (on some platforms) | Can be shared (it provides &T), but must be unlocked on the same thread it was locked |

---

## Things to Remember

1. **`Send` = safe to transfer ownership between threads.** The compiler checks this automatically.
2. **`Sync` = safe to share `&T` references between threads.** T: Sync ⟺ &T: Send.
3. **Both are auto traits.** You don't implement them manually — the compiler does it based on your fields.
4. **Both have zero runtime cost.** They're compile-time markers.
5. **`Rc` is not Send** because non-atomic ref count. Use `Arc` for threads.
6. **`RefCell` is not Sync** because unsynchronized interior mutability. Use `Mutex` for threads.
7. **`tokio::spawn` requires `Send + 'static`** — this is the most common place you encounter these traits in async Rust.
8. **`Arc<T>` requires `T: Send + Sync`** for full thread safety.

---

## Interview Questions

**Q: What is `Send` in Rust?**

> `Send` is a marker trait indicating that a type's ownership can be safely transferred to another thread. It's automatically implemented for types whose fields are all `Send`. Types like `Rc<T>` are not `Send` because their non-atomic reference count would be corrupted if accessed from multiple threads.

**Q: What is `Sync` in Rust?**

> `Sync` means it's safe for multiple threads to hold immutable references (`&T`) to the value simultaneously. Formally, `T: Sync` if and only if `&T: Send`. Types like `RefCell<T>` are not `Sync` because they allow mutation through `&T` without thread synchronization.

**Q: Why does `tokio::spawn` require `Send + 'static`?**

> `Send` because Tokio's multi-threaded runtime might execute the future on any worker thread, so the future must be safe to move between threads. `'static` because the spawned task runs independently with no guarantee about when it finishes, so it can't borrow data from the spawning scope.

**Q: If `Mutex<T>` requires `T: Send`, why doesn't it require `T: Sync`?**

> Because `Mutex` itself provides synchronization. Only one thread can access `T` at a time (through the lock), so `T` doesn't need to be safe for concurrent access. `Mutex` makes `T` safely accessible across threads by guaranteeing exclusive access.

---

> **Next**: [Chapter 9: Shared State](./09_shared_state.md) — Deep dive into Mutex, RwLock, and Arc patterns
