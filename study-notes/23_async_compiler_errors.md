# Chapter 23: Common Async Rust Compiler Errors

> **Connection to the bigger picture**: The Rust compiler's error messages are your best teacher. Instead of runtime crashes in production, you get clear explanations at compile time. This chapter catalogs the most common errors you'll encounter in async Rust, explains WHY they happen, and shows the correct fix.

---

## Error 1: "future cannot be sent between threads safely"

### Code

```rust
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let data = Rc::new(42);
    
    tokio::spawn(async move {
        println!("{}", data);
    });
}
```

### Compiler error

```
error: future cannot be sent between threads safely
   |
   = help: within `impl Future<Output = ()>`, the trait `Send` is not
     implemented for `Rc<i32>`
note: future is not `Send` as this value is used across an await
```

### Why it happens

`tokio::spawn` requires `Send` because Tokio's work-stealing scheduler might move the task between threads. `Rc<i32>` is NOT `Send` because its reference count is non-atomic.

### Fix

```rust
use std::sync::Arc;  // Use Arc instead of Rc

let data = Arc::new(42);
tokio::spawn(async move {
    println!("{}", data);  // ✅ Arc is Send
});
```

### General lesson

When you see "future cannot be sent between threads safely," look for non-`Send` types held across `.await` points. Replace `Rc` with `Arc`, `Cell`/`RefCell` with `Mutex`.

---

## Error 2: "borrowed value does not live long enough"

### Code

```rust
#[tokio::main]
async fn main() {
    let data = String::from("hello");
    
    tokio::spawn(async {
        println!("{}", data);  // Borrows data
    });
}
```

### Compiler error

```
error[E0597]: `data` does not live long enough
  |
  = note: argument requires that `data` is borrowed for `'static`
```

### Why it happens

`tokio::spawn` requires `'static`. The async block borrows `data` from the outer scope, but the spawned task might outlive the scope.

### Fix

```rust
tokio::spawn(async move {  // Use `move` to take ownership
    println!("{}", data);
});
```

---

## Error 3: "value moved here" (use after move)

### Code

```rust
let name = String::from("Sunil");

tokio::spawn(async move {
    println!("{}", name);
});

println!("{}", name);  // ❌ name was moved
```

### Fix

```rust
let name = String::from("Sunil");
let name_clone = name.clone();  // Clone before moving

tokio::spawn(async move {
    println!("{}", name_clone);
});

println!("{}", name);  // ✅ Original still available
```

---

## Error 4: "cannot borrow as mutable because it is also borrowed as immutable"

### Code

```rust
let mut data = vec![1, 2, 3];
let first = &data[0];
data.push(4);  // ❌ Mutable borrow while immutable borrow exists
println!("{}", first);
```

### Fix

```rust
let mut data = vec![1, 2, 3];
let first = data[0];  // Copy the value (i32 is Copy)
data.push(4);  // ✅ No outstanding borrows
println!("{}", first);
```

---

## Error 5: "holding `MutexGuard` across `.await`"

### Code

```rust
let data = std::sync::Mutex::new(vec![1, 2, 3]);

async {
    let mut guard = data.lock().unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    guard.push(4);  // Guard held across .await
};
```

### Compiler warning/error

The future is NOT `Send` because `MutexGuard` is held across an `.await` point.

### Fix

```rust
// Option 1: Use tokio::sync::Mutex
let data = tokio::sync::Mutex::new(vec![1, 2, 3]);
async {
    let mut guard = data.lock().await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    guard.push(4);  // ✅ tokio Mutex guard is Send
};

// Option 2: Drop guard before .await
let data = std::sync::Mutex::new(vec![1, 2, 3]);
async {
    {
        let mut guard = data.lock().unwrap();
        guard.push(4);
    }  // Guard dropped here
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;  // ✅ No guard held
};
```

---

## Error 6: "`'static` lifetime required"

### Code

```rust
async fn process(data: &str) {
    tokio::spawn(async {
        println!("{}", data);  // data is &str with limited lifetime
    });
}
```

### Fix

```rust
async fn process(data: String) {  // Take owned String
    tokio::spawn(async move {
        println!("{}", data);  // ✅ Owned data is 'static
    });
}
```

---

## Error 7: "the trait bound `X: Send` is not satisfied"

This often appears when a type contains a non-Send field and is used across `.await`:

### General fix checklist

1. **Replace `Rc` with `Arc`**
2. **Replace `RefCell` with `Mutex`**
3. **Ensure non-Send values don't span `.await` points**
4. **Use `Arc::clone` to share data between tasks**
5. **Use `async move` instead of `async`**

---

## Quick Reference: Error → Fix

| Error message | Likely cause | Fix |
|---------------|-------------|-----|
| "future cannot be sent between threads safely" | Non-Send type across `.await` | Use Send types (Arc, Mutex) |
| "borrowed value does not live long enough" | Borrowing in spawned task | Use `async move` |
| "value moved here" | Using value after move | Clone before move |
| "cannot borrow as mutable" | Aliasing + mutation | Restructure borrows |
| "`'static` lifetime required" | Reference in spawned task | Use owned data |
| "trait `Send` not implemented" | Non-Send field in future | Replace with Send alternative |

---

> **Next**: [Chapter 24: Concurrency Design Patterns](./24_concurrency_design_patterns.md)
