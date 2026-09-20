# Chapter 14: `async move`, Ownership, and Lifetimes in Async Rust

> **Connection to the bigger picture**: This chapter is where ownership (Chapter 1), lifetimes (Chapter 3), `Send` (Chapter 8), and async (Chapters 11-13) all converge. The most common struggle in async Rust is understanding why the compiler rejects certain data captures in async blocks. This chapter gives you the mental model to resolve these issues.

---

## `async {}` vs `async move {}`

### `async {}` — Captures by reference

```rust
#[tokio::main]
async fn main() {
    let name = String::from("Sunil");
    
    let future = async {
        println!("{}", name);  // Borrows name from outer scope
    };
    
    future.await;
    println!("Still have: {}", name);  // ✅ name wasn't moved
}
```

The async block borrows `name`. Since we `.await` the future in the same scope, the borrow is valid.

### `async move {}` — Captures by value (moves)

```rust
#[tokio::main]
async fn main() {
    let name = String::from("Sunil");
    
    let future = async move {
        println!("{}", name);  // Owns name — moved into the block
    };
    
    // println!("{}", name);  // ❌ name was moved
    future.await;
}
```

The async block takes ownership of `name`. After creating the block, `name` is no longer available.

---

## When You Need `async move`

### Rule: `tokio::spawn` ALWAYS needs owned data

```rust
// ❌ Won't compile — async block borrows name
let name = String::from("Sunil");
tokio::spawn(async {
    println!("{}", name);
});

// ✅ Works — async move takes ownership
let name = String::from("Sunil");
tokio::spawn(async move {
    println!("{}", name);
});
```

### When you need the value in both the outer scope AND the spawned task

```rust
// Solution 1: Clone before moving
let name = String::from("Sunil");
let name_clone = name.clone();
tokio::spawn(async move {
    println!("Task: {}", name_clone);
});
println!("Main: {}", name);  // ✅ Original still available

// Solution 2: Arc for shared access
use std::sync::Arc;
let name = Arc::new(String::from("Sunil"));
let name_task = Arc::clone(&name);
tokio::spawn(async move {
    println!("Task: {}", name_task);
});
println!("Main: {}", name);  // ✅ Arc clone still available
```

### Capturing multiple variables

```rust
let name = String::from("Sunil");
let age = 25u32;
let scores = vec![95, 87, 92];

// `move` captures ALL referenced variables
tokio::spawn(async move {
    // name, age, and scores are all moved in
    println!("{} (age {}): {:?}", name, age, scores);
});

// name, age, scores are all gone from this scope
// (age was copied because u32: Copy)
```

---

## The `'static` Requirement Explained

`tokio::spawn` requires `'static`. What does this really mean for async blocks?

```
'static for an async block means:
  "The async block does not borrow anything with a limited lifetime"
  
This is satisfied when:
  1. The block owns all its data (async move with owned types)
  2. The block only borrows 'static data (string literals, leaked data)
  3. The block borrows nothing
```

### Examples

```rust
// ✅ 'static: owns a String
tokio::spawn(async move {
    let owned = String::from("hello");
    println!("{}", owned);
});

// ✅ 'static: borrows a string literal (&'static str)
tokio::spawn(async {
    let s: &'static str = "hello";
    println!("{}", s);
});

// ❌ NOT 'static: borrows from a local variable
let local = String::from("hello");
tokio::spawn(async {
    println!("{}", local);  // borrows local, which is not 'static
});
```

---

## Holding Data Across `.await` Points

The data a future holds across `.await` points becomes part of its state machine. This has implications for `Send`.

```rust
async fn example() {
    let data = String::from("hello");  // data is part of the state
    
    some_async_fn().await;  // ← .await point
    // data must survive across this point
    // It's stored in the state machine
    
    println!("{}", data);  // data used after .await
}
```

If `data` is NOT `Send`, and you try to `tokio::spawn` this future, you'll get a compiler error:

```rust
use std::rc::Rc;

async fn problematic() {
    let data = Rc::new(42);  // Rc is NOT Send
    
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // data is held across .await → part of the state machine
    // state machine is NOT Send because it contains Rc
    
    println!("{}", data);
}

// ❌ tokio::spawn(problematic());
// Error: future cannot be sent between threads safely
```

**Fix**: Use `Arc` instead of `Rc`, or ensure non-Send data doesn't span `.await` points:

```rust
async fn fixed() {
    {
        let data = Rc::new(42);
        println!("{}", data);
    }  // data is dropped before .await
    
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // No Rc in the state machine → it's Send
}
```

---

## Common Patterns

### Pattern: Clone Arc before moving into task

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

struct AppState {
    counter: Mutex<u64>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        counter: Mutex::new(0),
    });
    
    for _ in 0..10 {
        let state = Arc::clone(&state);  // Clone Arc, not AppState
        tokio::spawn(async move {
            let mut counter = state.counter.lock().await;
            *counter += 1;
        });
    }
}
```

### Pattern: Owned vs borrowed in helper functions

```rust
// This function can be called without spawning — borrows are fine
async fn process(data: &str) -> usize {
    data.len()
}

// This function is meant to be spawned — needs owned data
async fn spawn_processor(data: String) {
    println!("Processing: {}", data);
}

#[tokio::main]
async fn main() {
    let data = String::from("hello");
    
    // Direct call — borrowing works
    let len = process(&data).await;
    
    // Spawning — needs owned data
    tokio::spawn(spawn_processor(data));
}
```

---

## Things to Remember

1. **`async {}` borrows**, `async move {}` takes ownership.
2. **`tokio::spawn` needs `'static`** → use `async move` or `Arc`.
3. **Clone before moving** when you need data in multiple places.
4. **Data across `.await` must be `Send`** for spawned tasks.
5. **Drop non-Send data before `.await`** if you can't use Send alternatives.
6. **`Arc::clone` is cheap** — just an atomic increment, not a deep copy.

---

> **Next**: [Chapter 15: Tokio Channels](./15_tokio_channels.md) — Async-aware message passing
