# Chapter 39: Revision Cheat Sheets

---

## Rust Ownership Cheat Sheet

```
let a = String::from("hi");
let b = a;          // MOVE (String is not Copy)
// a is invalid

let x = 42;
let y = x;          // COPY (i32 is Copy)
// x is still valid

let c = b.clone();  // CLONE (explicit deep copy)
// b and c are independent

fn take(s: String) {} // Moves s into the function
fn borrow(s: &str) {} // Borrows, caller keeps ownership
```

---

## Borrowing Cheat Sheet

```
&T    = immutable reference (many allowed simultaneously)
&mut T = mutable reference (only ONE allowed, no &T at same time)

Rules:
  Many &T  ← OK (multiple readers)
  One &mut T ← OK (single writer)
  &T + &mut T ← COMPILE ERROR (reader + writer conflict)
  
Borrows end at last use (NLL), not end of scope.
```

---

## Smart Pointer Cheat Sheet

```
Box<T>         → Heap allocation, single owner
Rc<T>          → Shared ownership, single-threaded, NOT Send
Arc<T>         → Shared ownership, thread-safe, Send
RefCell<T>     → Interior mutability, runtime borrow check, NOT Sync
Mutex<T>       → Interior mutability, thread-safe, blocks on lock
RwLock<T>      → Interior mutability, many readers OR one writer

Common patterns:
  Rc<RefCell<T>>     → shared mutable, single-threaded
  Arc<Mutex<T>>      → shared mutable, multi-threaded
  Arc<RwLock<T>>     → shared mutable, read-heavy, multi-threaded
  Arc<T>             → shared read-only, multi-threaded (no lock needed)
```

---

## Thread Cheat Sheet

```rust
// Spawn
let handle = thread::spawn(move || { /* owned data only */ });
handle.join().unwrap();

// Scoped (can borrow from parent)
thread::scope(|s| {
    s.spawn(|| { /* can borrow from outer scope */ });
});

// Share data
Arc::clone(&data)           // Share ownership
Arc::new(Mutex::new(value)) // Share + mutate
```

---

## Send/Sync Cheat Sheet

```
Send: can MOVE to another thread
  ✅ String, Vec, Box, Arc, Mutex
  ❌ Rc, raw pointers

Sync: can SHARE &T across threads
  ✅ primitives, String, Vec, Mutex, RwLock
  ❌ Cell, RefCell, Rc

T: Sync ⟺ &T: Send

tokio::spawn requires: Future + Send + 'static
```

---

## Mutex/Arc Cheat Sheet

```rust
let state = Arc::new(Mutex::new(initial_value));

// In each thread/task:
let state = Arc::clone(&state);
tokio::spawn(async move {
    let mut guard = state.lock().await;  // or .lock().unwrap() for std
    *guard = new_value;
    // guard dropped → lock released
});

// ⚠️ Never hold std::sync::Mutex across .await
// ✅ Use tokio::sync::Mutex for locks across .await
```

---

## Channel Cheat Sheet

```rust
// Std (blocking)
let (tx, rx) = std::sync::mpsc::channel();
tx.send(value).unwrap();
let v = rx.recv().unwrap();

// Tokio mpsc (async, bounded)
let (tx, mut rx) = tokio::sync::mpsc::channel(100);
tx.send(value).await.unwrap();
let v = rx.recv().await;

// Tokio oneshot (single value)
let (tx, rx) = tokio::sync::oneshot::channel();
tx.send(value).unwrap();
let v = rx.await.unwrap();

// Tokio broadcast (all receivers get all messages)
let (tx, _) = tokio::sync::broadcast::channel(100);
let mut rx = tx.subscribe();

// Tokio watch (latest value only)
let (tx, rx) = tokio::sync::watch::channel(initial);
```

---

## Future Cheat Sheet

```
Future trait:
  poll() → Poll::Ready(value) or Poll::Pending

Futures are LAZY — nothing happens until polled.
.await polls the future (does NOT block the thread).

async fn → returns impl Future
async {} → creates a Future (captures by reference)
async move {} → creates a Future (captures by value/ownership)
```

---

## Tokio Cheat Sheet

```rust
#[tokio::main]              // Multi-threaded runtime
async fn main() {}

tokio::spawn(async move {}) // New task (Send + 'static)
tokio::join!(a, b, c)       // Run all, wait for all
tokio::select! { ... }      // Run all, wait for first
tokio::time::sleep(dur).await // Async sleep (don't use thread::sleep!)
tokio::task::spawn_blocking(|| {}) // For blocking/CPU work

// Channels
tokio::sync::mpsc
tokio::sync::oneshot
tokio::sync::broadcast
tokio::sync::watch
tokio::sync::Mutex          // Async-aware mutex
```

---

## Async/Await Cheat Sheet

```
✅ DO:
  tokio::time::sleep().await    // Async sleep
  tokio::fs::read().await       // Async file I/O
  reqwest::get().await           // Async HTTP
  sqlx::query().fetch().await    // Async database

❌ DON'T:
  std::thread::sleep()          // Blocks thread!
  std::fs::read()               // Blocks thread!
  Synchronous HTTP/DB calls      // Block thread!

If you must block: tokio::task::spawn_blocking(|| { ... })
```

---

## Axum Cheat Sheet

```rust
Router::new()
    .route("/path", get(handler))
    .route("/path/:id", get(handler_with_id))
    .layer(middleware)
    .with_state(Arc::new(state))

// Handler signature IS the request spec
async fn handler(
    State(s): State<Arc<AppState>>,  // Shared state
    Path(id): Path<u64>,              // URL path parameter
    Query(q): Query<Params>,          // URL query params
    Json(body): Json<Request>,        // JSON body
) -> Result<Json<Response>, AppError> { }
```

---

## Concurrency Cheat Sheet

```
Pattern           | Mechanism
Producer/Consumer | Channel (mpsc)
Worker Pool       | Shared channel receiver
Shared State      | Arc<Mutex<T>> or Arc<RwLock<T>>
Actor Model       | Channel per actor, message types
Pipeline          | Chain of channels
Fan-out/Fan-in    | broadcast + mpsc
```

---

## Solana Async Architecture Cheat Sheet

```
Solana Backend Components:
  WS Listener      → tokio::spawn + async stream
  RPC Client        → reqwest or solana-client (async)
  Event Pipeline    → mpsc channels
  Strategy Engine   → async task + spawn_blocking for CPU
  Tx Submitter      → async HTTP to Solana RPC
  State Manager     → Arc<RwLock<State>>
  HTTP API          → Axum + with_state
  Shutdown          → CancellationToken + select!
  Logging           → tracing + tracing-subscriber
```

---

> **Next**: [Chapter 40: Final Mental Model](./40_final_mental_model.md)
