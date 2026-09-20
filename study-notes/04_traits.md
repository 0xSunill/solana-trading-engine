# Chapter 4: Traits

> **Connection to the bigger picture**: Traits are Rust's primary abstraction mechanism — they define shared behavior across types. For concurrency and async Rust, traits are fundamental: `Send` determines whether data can be transferred between threads, `Sync` determines whether data can be shared between threads, and `Future` is the trait that makes all of async Rust possible. If you don't understand traits, you can't understand why `tokio::spawn` has the constraints it does.

---

## What Are Traits?

A trait defines a set of methods that a type can implement. Think of it as a contract: "If a type implements this trait, it promises to provide these methods."

Traits are similar to interfaces in Java/Go or abstract classes in C++, but with Rust-specific capabilities.

```rust
// Define a trait
trait Greet {
    fn hello(&self) -> String;
    
    // Default implementation (types can override this)
    fn goodbye(&self) -> String {
        String::from("Goodbye!")
    }
}

// Implement the trait for a type
struct User {
    name: String,
}

impl Greet for User {
    fn hello(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }
    // goodbye() uses the default implementation
}

fn main() {
    let user = User { name: String::from("Sunil") };
    println!("{}", user.hello());    // "Hello, I'm Sunil!"
    println!("{}", user.goodbye());  // "Goodbye!"
}
```

---

## Trait Bounds and Generics

Traits become powerful when combined with generics. A **trait bound** says "this generic type must implement this trait."

```rust
// Without trait bound — can't do anything useful with T
fn print_something<T>(value: T) {
    // println!("{}", value);  // ❌ T might not be printable
}

// With trait bound — T must implement Display
fn print_something<T: std::fmt::Display>(value: T) {
    println!("{}", value);  // ✅ T is guaranteed to be printable
}

fn main() {
    print_something(42);          // ✅ i32 implements Display
    print_something("hello");     // ✅ &str implements Display
    // print_something(vec![1]);  // ❌ Vec doesn't implement Display
}
```

### Multiple trait bounds

```rust
use std::fmt::{Display, Debug};

// T must implement BOTH Display AND Debug
fn inspect<T: Display + Debug>(value: T) {
    println!("Display: {}", value);
    println!("Debug: {:?}", value);
}

// Using `where` clause for readability
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Debug + Send,
{
    format!("{} - {:?}", t, u)
}
```

> **Why this matters**: `tokio::spawn` uses trait bounds extensively:
> ```rust
> pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
> where
>     F: Future + Send + 'static,      // The future must be Send + 'static
>     F::Output: Send + 'static,       // Its output must also be Send + 'static
> ```
> Every constraint here is a trait bound. Understanding trait bounds is understanding why `tokio::spawn` accepts or rejects your code.

---

## `impl Trait` — Return Position and Argument Position

### Argument position (syntactic sugar for trait bounds)

```rust
// These are equivalent:
fn greet(person: &impl Greet) {
    println!("{}", person.hello());
}

fn greet<T: Greet>(person: &T) {
    println!("{}", person.hello());
}
```

### Return position (existential type)

```rust
// "I return something that implements Display, but I won't tell you what"
fn make_greeting() -> impl std::fmt::Display {
    String::from("Hello, World!")
    // The caller knows it implements Display
    // but can't know it's specifically a String
}
```

This is crucial for async Rust because `async fn` uses `impl Future`:

```rust
// This:
async fn fetch_data() -> String {
    "data".to_string()
}

// Is desugared to approximately:
fn fetch_data() -> impl Future<Output = String> {
    async {
        "data".to_string()
    }
}
```

---

## Static Dispatch vs Dynamic Dispatch

This distinction affects performance and flexibility, and it matters for async programming.

### Static dispatch (generics / `impl Trait`)

```rust
fn print_it<T: std::fmt::Display>(value: T) {
    println!("{}", value);
}

fn main() {
    print_it(42);       // Compiler generates: print_it_i32(42)
    print_it("hello");  // Compiler generates: print_it_str("hello")
}
```

**What happens at compile time**: The compiler generates a separate copy of `print_it` for each type it's called with. This is called **monomorphization**.

```
Source code:              Compiled code:
print_it(42)       →     print_it_i32(42)       ← specialized for i32
print_it("hello")  →     print_it_str("hello")  ← specialized for &str
```

**Pros**: Zero runtime overhead — direct function calls, enables inlining.
**Cons**: Code bloat (multiple copies), the concrete type must be known at compile time.

### Dynamic dispatch (`dyn Trait`)

```rust
fn print_it(value: &dyn std::fmt::Display) {
    println!("{}", value);
}

fn main() {
    print_it(&42);       // Same function, different data
    print_it(&"hello");  // Same function, different data
}
```

**What happens at runtime**: One copy of the function exists. It uses a **vtable** (virtual method table) to look up the correct method implementation at runtime.

```
value: &dyn Display
┌──────────────────┐
│ data_ptr ────────┼──▶ actual value (42 or "hello")
│ vtable_ptr ──────┼──▶ ┌──────────────────┐
└──────────────────┘    │ fmt() method ptr  │ ← points to the correct
                        │ drop() method ptr │    implementation
                        └──────────────────┘
```

**Pros**: One copy of the function, flexible (can store different types in the same collection).
**Cons**: Small runtime overhead (vtable lookup), prevents inlining.

### Comparison

| | Static Dispatch (`impl Trait` / generics) | Dynamic Dispatch (`dyn Trait`) |
|---|---|---|
| **Resolved at** | Compile time | Runtime |
| **Performance** | Faster (inlining possible) | Slight overhead (vtable lookup) |
| **Code size** | Larger (monomorphization) | Smaller (one copy) |
| **Flexibility** | Type must be known at compile time | Can mix different types at runtime |
| **Use case** | Performance-critical code, generic libraries | Heterogeneous collections, plugin systems |

---

## Trait Objects

A trait object (`dyn Trait`) lets you store different types that implement the same trait:

```rust
trait Animal {
    fn speak(&self) -> &str;
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn speak(&self) -> &str { "Woof!" }
}

impl Animal for Cat {
    fn speak(&self) -> &str { "Meow!" }
}

fn main() {
    // Heterogeneous collection using trait objects
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog),
        Box::new(Cat),
        Box::new(Dog),
    ];
    
    for animal in &animals {
        println!("{}", animal.speak());
    }
}
```

### Object safety

Not all traits can be used as trait objects. A trait is **object-safe** if:

1. It doesn't have methods that return `Self` (the concrete type is erased behind `dyn`)
2. It doesn't have generic method parameters
3. It doesn't require `Sized`

```rust
// Object-safe ✅
trait Drawable {
    fn draw(&self);
}

// NOT object-safe ❌
trait Cloneable {
    fn clone_self(&self) -> Self;  // Returns Self — can't work with dyn
}
```

---

## Critical Traits for Concurrency

### The `Send` Trait

```rust
// From the standard library (simplified)
pub unsafe auto trait Send { }
```

`Send` means: "This type can be safely transferred to another thread."

- It's an **auto trait** — the compiler automatically implements it for types where all fields are `Send`.
- It's a **marker trait** — it has no methods. It just marks a type as thread-transfer-safe.
- It's `unsafe` to implement manually — because the compiler trusts you that it's safe.

**Types that are `Send`:**
- `i32`, `f64`, `bool`, `String`, `Vec<T>` (if T is Send)
- `Box<T>` (if T is Send)
- `Arc<T>` (if T is Send + Sync)
- `Mutex<T>` (if T is Send)

**Types that are NOT `Send`:**
- `Rc<T>` — reference count is not atomic, would corrupt if accessed from multiple threads
- `*const T`, `*mut T` — raw pointers (compiler can't verify safety)
- Types containing `Rc<T>` or raw pointers

(We'll cover `Send` in extreme detail in Chapter 8.)

### The `Sync` Trait

```rust
pub unsafe auto trait Sync { }
```

`Sync` means: "This type can be safely shared between threads via references."

More precisely: `T` is `Sync` if `&T` is `Send`. If multiple threads can safely hold `&T` (immutable reference) simultaneously, then `T` is `Sync`.

**Types that are `Sync`:**
- All primitive types
- `&T` if T is Sync
- `Mutex<T>` (even if T is not Sync — Mutex provides synchronization)
- `Arc<T>` if T is Send + Sync

**Types that are NOT `Sync`:**
- `Cell<T>`, `RefCell<T>` — interior mutability without synchronization
- `Rc<T>` — non-atomic reference counting

(We'll cover `Sync` in extreme detail in Chapter 8.)

### The `Future` Trait

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

This is the heart of async Rust. Every `async fn` and `async {}` block produces a type that implements `Future`.

**What the `Future` trait says:**
- It has an associated type `Output` — the value the future will eventually produce.
- It has one method: `poll()` — which the runtime calls to check if the future is done.
- `poll()` returns `Poll::Ready(value)` when done, or `Poll::Pending` when still waiting.

(We'll cover `Future` in extreme detail in Chapter 12.)

### The `Iterator` Trait

```rust
pub trait Iterator {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
}
```

Understanding `Iterator` helps understand `Stream` (async iterator):

```rust
// Sync iteration:
for item in vec.iter() {
    process(item);
}

// Async iteration (Stream):
while let Some(item) = stream.next().await {
    process(item);
}
```

---

## How Traits Connect to Async Rust

Here's the chain of dependencies:

```
Traits
  │
  ├── Send (marker trait)
  │     └── Required by tokio::spawn
  │         "The future can be moved to another thread"
  │
  ├── Sync (marker trait)
  │     └── Required for sharing data via Arc across tasks
  │         "References to T can be shared between threads"
  │
  ├── Future (core trait)
  │     └── Every async fn returns impl Future
  │         "This value will produce a result eventually"
  │
  ├── Trait bounds on spawn:
  │     F: Future + Send + 'static
  │     └── "The future must:
  │          - be a Future (obviously)
  │          - be Send (can run on any thread)
  │          - be 'static (owns all its data)"
  │
  └── Trait objects (dyn Trait)
        └── Used for dynamic handler types in Axum
            "Accept any handler that matches this signature"
```

### Real example: Why tokio::spawn needs Send

```rust
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let data = Rc::new(42);  // Rc is NOT Send
    
    // ❌ COMPILE ERROR
    tokio::spawn(async move {
        println!("{}", data);
    });
}
```

**Compiler error:**

```
error: future cannot be sent between threads safely
   |
   = help: within `impl Future<Output = ()>`, the trait `Send` is not
     implemented for `Rc<i32>`
```

**Fix: Use `Arc` instead of `Rc`:**

```rust
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let data = Arc::new(42);  // Arc IS Send
    
    tokio::spawn(async move {
        println!("{}", data);  // ✅
    });
}
```

---

## Deriving Traits

Many traits can be automatically derived:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
}

fn main() {
    let order = Order {
        id: 1,
        symbol: "SOL/USD".to_string(),
        quantity: 10.0,
        price: 150.0,
    };
    
    println!("{:?}", order);           // Debug
    let copy = order.clone();          // Clone
    assert_eq!(order.id, copy.id);     // PartialEq
}
```

### Common derivable traits

| Trait | What it provides |
|-------|-----------------|
| `Debug` | `{:?}` formatting |
| `Clone` | `.clone()` method (deep copy) |
| `Copy` | Implicit copy on assignment (requires Clone) |
| `PartialEq` | `==` and `!=` operators |
| `Eq` | Full equality (requires PartialEq) |
| `Hash` | Hashing (for HashMap keys) |
| `PartialOrd` | `<`, `>`, `<=`, `>=` operators |
| `Ord` | Full ordering (requires PartialOrd + Eq) |
| `Default` | `Default::default()` — zero/empty values |
| `Serialize` / `Deserialize` | JSON/binary serialization (from serde) |

---

## Supertraits

A trait can require another trait as a prerequisite:

```rust
trait Animal: std::fmt::Display {
    // Any type implementing Animal must also implement Display
    fn name(&self) -> &str;
}

struct Dog {
    name: String,
}

impl std::fmt::Display for Dog {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Dog({})", self.name)
    }
}

impl Animal for Dog {
    fn name(&self) -> &str {
        &self.name
    }
}
```

This pattern is used in frameworks like Axum where handler traits have supertraits.

---

## Associated Types vs Generic Parameters

```rust
// Associated type — one implementation per type
trait Iterator {
    type Item;  // Each iterator type has ONE item type
    fn next(&mut self) -> Option<Self::Item>;
}

// Generic parameter — multiple implementations per type
trait From<T> {
    fn from(value: T) -> Self;
}
// String can implement From<&str>, From<Vec<u8>>, etc.
```

`Future` uses an associated type:

```rust
trait Future {
    type Output;  // Each future produces ONE type of output
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

---

## Things to Remember

1. **Traits define shared behavior** — they're contracts types can implement.
2. **Trait bounds constrain generics** — `T: Display + Send` means T must be both displayable and thread-safe.
3. **Static dispatch** (generics) is zero-cost; **dynamic dispatch** (`dyn Trait`) has a small vtable overhead.
4. **`Send`** = safe to transfer between threads. **`Sync`** = safe to share references between threads.
5. **`Future`** = the core trait of async Rust. Every `async fn` returns `impl Future`.
6. **Auto traits** (`Send`, `Sync`) are implemented automatically when all fields satisfy them.
7. **Understanding trait bounds is essential** for reading `tokio::spawn`, Axum handler signatures, and error messages.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "Traits are the same as interfaces" | Similar, but traits support default methods, associated types, and auto-implementation (auto traits). |
| "`dyn Trait` is always slower" | The vtable overhead is tiny — often a single pointer indirection. In I/O-bound async code, it's negligible. |
| "I should always use generics for performance" | Use generics when the type must be known at compile time. Use `dyn Trait` for flexibility and smaller binaries. |
| "`Send` means sending messages" | No — `Send` is about transferring ownership between threads. It's a marker trait, not a messaging system. |

---

## Interview Questions

**Q: What is the difference between static and dynamic dispatch?**

> Static dispatch (generics, `impl Trait`) resolves the concrete type at compile time. The compiler generates specialized code for each type (monomorphization). Zero runtime overhead but larger binary. Dynamic dispatch (`dyn Trait`) uses a vtable to resolve methods at runtime. Single function copy, smaller binary, but small pointer indirection overhead.

**Q: What are `Send` and `Sync` traits?**

> `Send` means a type's ownership can be safely transferred between threads. `Sync` means multiple threads can safely hold immutable references to the type simultaneously. They're auto traits — implemented automatically when all fields satisfy them. They're essential for Rust's thread safety guarantees.

**Q: Why does `tokio::spawn` require `Send + 'static`?**

> `Send` because Tokio's multi-threaded runtime may execute the task on any thread. `'static` because the spawned task runs independently — the runtime doesn't know when it will finish, so the task must own all its data rather than borrowing from the spawning scope.

---

> **Next**: [Chapter 5: Smart Pointers](./05_smart_pointers.md) — Box, Rc, Arc, RefCell, Mutex, RwLock
