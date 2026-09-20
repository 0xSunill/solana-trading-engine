# Chapter 1: Ownership

> **Connection to the bigger picture**: Ownership is the single most important concept in Rust. Every concurrency feature — threads, `Send`/`Sync`, `Arc`, `Mutex`, channels, async tasks — builds directly on ownership. If you don't deeply understand ownership, concurrency in Rust will feel like fighting the compiler. If you do understand ownership, the compiler becomes your ally that prevents entire classes of bugs that crash production systems.

---

## What Is Ownership?

Ownership is Rust's system for managing memory without a garbage collector.

In most programming languages, memory is managed in one of two ways:

1. **Garbage collection** (Java, Go, Python): The runtime periodically scans memory and frees things that are no longer used. This is convenient but adds unpredictable pauses and uses more memory.

2. **Manual memory management** (C, C++): The programmer explicitly calls `malloc`/`free` or `new`/`delete`. This is fast but leads to use-after-free bugs, double frees, memory leaks, and security vulnerabilities.

Rust chose a third path:

3. **Ownership**: Every piece of data has exactly one owner. When the owner goes out of scope, the data is automatically freed. The compiler checks ownership rules at compile time. There is zero runtime cost.

### The Three Rules of Ownership

```
Rule 1: Each value in Rust has exactly one owner.
Rule 2: There can only be one owner at a time.
Rule 3: When the owner goes out of scope, the value is dropped (freed).
```

These three rules sound simple, but they have profound consequences that affect everything from basic variable assignment to concurrent async task spawning.

---

## Why Does Rust Have Ownership?

The core problem ownership solves is: **who is responsible for freeing memory?**

Consider this C code:

```c
char* name = malloc(6);
strcpy(name, "Sunil");
char* alias = name;   // Both name and alias point to the same memory
free(name);            // Free through name
printf("%s", alias);   // BUG: use-after-free! alias points to freed memory
```

This is a **use-after-free** bug. The program might crash, print garbage, or appear to work but corrupt memory silently. These bugs cause security vulnerabilities in real-world systems.

Rust prevents this at compile time:

```rust
let name = String::from("Sunil");
let alias = name;        // Ownership MOVES to alias
// println!("{}", name);  // COMPILE ERROR: name no longer owns the data
println!("{}", alias);    // Works fine - alias is the owner now
```

The compiler sees that `name` was moved to `alias` and prevents you from using `name` afterward. No runtime check needed — it's all done at compile time.

> **Why this matters for concurrency**: When you spawn a thread or an async task, the same question arises: who owns the data? Can two threads both access the same String? If one thread frees it, what happens to the other? Ownership answers these questions at compile time.

---

## Stack vs Heap

To understand ownership deeply, you need to understand where data lives in memory.

### The Stack

```
┌─────────────────────┐  ← High address
│    Stack frame       │
│    for main()        │
│  ┌─────────────────┐ │
│  │ x: i32 = 42     │ │  ← Fixed size, stored directly
│  │ y: bool = true  │ │  ← Fixed size, stored directly
│  │ z: f64 = 3.14   │ │  ← Fixed size, stored directly
│  └─────────────────┘ │
│                       │
│    Stack frame        │
│    for foo()          │
│  ┌─────────────────┐ │
│  │ a: i32 = 10     │ │
│  └─────────────────┘ │
└─────────────────────┘  ← Low address (stack grows down)
```

**Properties of the stack:**
- Very fast allocation/deallocation (just moving a pointer)
- Fixed-size data only (compiler must know the size at compile time)
- Automatically cleaned up when a function returns
- LIFO (last in, first out) order

**Types that live on the stack:**
- `i32`, `i64`, `u8`, `u64`, etc. (integers)
- `f32`, `f64` (floats)
- `bool`
- `char`
- Tuples of stack types: `(i32, bool)`
- Arrays of fixed size: `[i32; 5]`
- References: `&T`, `&mut T` (a reference is just a pointer, which has a fixed size)

### The Heap

```
Stack                          Heap
┌──────────────────┐          ┌──────────────────────────┐
│                  │          │                          │
│  name: String    │          │                          │
│  ┌────────────┐  │          │  ┌──────────────────┐   │
│  │ ptr ───────┼──┼──────────┼─▶│ S │ u │ n │ i │ l │   │
│  │ len: 5     │  │          │  └──────────────────┘   │
│  │ cap: 5     │  │          │                          │
│  └────────────┘  │          │                          │
│                  │          │                          │
└──────────────────┘          └──────────────────────────┘
```

**Properties of the heap:**
- Slower allocation (needs to find free space, bookkeeping)
- Can store dynamically-sized data
- Data lives until explicitly freed
- Can be shared between different parts of the program (via pointers)

**Types that use the heap:**
- `String` (the string data is on the heap; the pointer/len/capacity are on the stack)
- `Vec<T>` (same structure as String)
- `Box<T>` (heap-allocated single value)
- `HashMap<K, V>`
- Any dynamically-sized data

### Why This Matters

When a stack value goes out of scope, it's instantly gone (the stack frame is popped). When a heap value goes out of scope, Rust needs to free the heap memory. Ownership determines **when** that heap memory gets freed.

```rust
fn main() {
    let name = String::from("Sunil");  // Heap allocation happens here
    // ... use name ...
}  // name goes out of scope here → Rust calls `drop()` → heap memory is freed
```

> **Key idea**: Ownership is fundamentally about knowing when to free heap memory. Stack memory is trivial — it's freed automatically when the function returns. Heap memory is the hard part.

---

## Move Semantics

Move semantics is what makes ownership work for heap-allocated data.

### Example 1: String (Heap Data)

```rust
let name = String::from("Sunil");
let other = name;
```

Let's trace what happens step by step:

**Step 1: `let name = String::from("Sunil");`**

```
Stack                    Heap
┌──────────────┐        ┌───────────────┐
│ name:        │        │               │
│  ptr ────────┼───────▶│ S u n i l     │
│  len: 5      │        │               │
│  cap: 5      │        └───────────────┘
└──────────────┘
```

`name` is a `String`. On the stack, it stores a pointer, length, and capacity. The actual character data "Sunil" is on the heap.

**Step 2: `let other = name;`**

Rust does NOT copy the heap data. Instead, it copies the stack data (pointer, length, capacity) to `other` and **invalidates** `name`.

```
Stack                    Heap
┌──────────────┐
│ name:        │         (INVALID - cannot be used)
│  (moved)     │
└──────────────┘
                         ┌───────────────┐
┌──────────────┐         │               │
│ other:       │         │               │
│  ptr ────────┼────────▶│ S u n i l     │
│  len: 5      │         │               │
│  cap: 5      │         └───────────────┘
└──────────────┘
```

**Why doesn't Rust just copy the heap data?**

If Rust copied the heap data, you'd have two independent `String`s, each owning their own copy. That's what `clone()` does. But copying heap data by default would be:

1. **Slow**: Copying a 1MB string every time you assign it would be terrible for performance.
2. **Surprising**: Most of the time, you don't actually need two copies.

**Why doesn't Rust just share the heap data (like C)?**

If both `name` and `other` pointed to the same heap data, who would free it? If `name` goes out of scope and frees it, `other` has a dangling pointer. If both try to free it, that's a double-free.

**Rust's solution: move semantics.** Transfer ownership. After the move, only `other` is valid. When `other` goes out of scope, it frees the heap data. Clean and safe.

### What happens if you try to use the moved value?

```rust
let name = String::from("Sunil");
let other = name;
println!("{}", name);  // ← COMPILE ERROR
```

**Compiler error:**

```
error[E0382]: borrow of moved value: `name`
 --> src/main.rs:4:20
  |
2 |     let name = String::from("Sunil");
  |         ---- move occurs because `name` has type `String`,
  |              which does not implement the `Copy` trait
3 |     let other = name;
  |                 ---- value moved here
4 |     println!("{}", name);
  |                    ^^^^ value borrowed here after move
```

**What the compiler is telling you:**

1. `name` is a `String`, which does NOT implement the `Copy` trait (because it has heap data).
2. When you wrote `let other = name`, the value was **moved** from `name` to `other`.
3. After the move, `name` is no longer valid.
4. You're trying to use `name` after it was moved — that's not allowed.

**The fix depends on what you want:**

```rust
// Fix 1: Just use `other` instead
let name = String::from("Sunil");
let other = name;
println!("{}", other);  // ✅ Use the new owner

// Fix 2: Clone if you need two independent copies
let name = String::from("Sunil");
let other = name.clone();  // Deep copy - both are independent
println!("{}", name);   // ✅ name is still valid
println!("{}", other);  // ✅ other has its own copy

// Fix 3: Borrow instead of moving
let name = String::from("Sunil");
let other = &name;      // Borrow - name is still the owner
println!("{}", name);   // ✅ name is still valid
println!("{}", other);  // ✅ other can read the data
```

---

### Example 2: Integer (Stack Data)

```rust
let x = 10;
let y = x;
println!("{}", x);  // ✅ This works!
println!("{}", y);  // ✅ This also works!
```

**Why does this behave differently from `String`?**

`i32` is a **Copy type**. It implements the `Copy` trait. When you write `let y = x`, Rust copies the value `10` from `x` to `y`. Both `x` and `y` are independent and valid.

```
Stack (before)       Stack (after let y = x)
┌──────────┐         ┌──────────┐
│ x: 10    │         │ x: 10    │  ← Still valid!
└──────────┘         │ y: 10    │  ← Independent copy
                     └──────────┘
```

This is safe because:
1. `i32` lives entirely on the stack (no heap data).
2. Copying 4 bytes is extremely cheap.
3. There's no shared resource that could cause problems.

> **The rule**: Types that live entirely on the stack and are cheap to copy implement `Copy`. Assignment copies the value. Types that have heap data do NOT implement `Copy`. Assignment moves the value.

---

## Copy Types vs Move Types

### Copy Types (assignment copies the value)

| Type | Size | Why it's Copy |
|------|------|--------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | 1-16 bytes | Small, no heap data |
| `u8`, `u16`, `u32`, `u64`, `u128` | 1-16 bytes | Small, no heap data |
| `f32`, `f64` | 4-8 bytes | Small, no heap data |
| `bool` | 1 byte | Trivial |
| `char` | 4 bytes | Small, no heap data |
| `(i32, bool)` | Tuple of Copy types | All fields are Copy |
| `[i32; 5]` | Array of Copy type | All elements are Copy |
| `&T` | pointer size | Immutable reference, just a pointer |

### Move Types (assignment moves ownership)

| Type | Why it moves |
|------|-------------|
| `String` | Owns heap-allocated character data |
| `Vec<T>` | Owns heap-allocated array |
| `Box<T>` | Owns heap-allocated value |
| `HashMap<K, V>` | Owns heap-allocated hash table |
| `File` | Owns an OS file handle (resource) |
| `TcpStream` | Owns an OS socket (resource) |
| `Sender<T>` | Owns a channel endpoint |
| `MutexGuard<T>` | Owns a lock |

> **Key idea**: Copy types are values. Move types are resources. You can trivially duplicate a value, but you can't duplicate a file handle or a heap allocation — that would cause resource conflicts.

---

## Clone

`Clone` is the explicit deep copy. Unlike `Copy` (which happens automatically and cheaply), `Clone` requires you to explicitly call `.clone()` and may be expensive.

```rust
let original = String::from("Sunil");
let copy = original.clone();

// Both are independent, valid Strings
println!("Original: {}", original);  // ✅
println!("Copy: {}", copy);          // ✅
```

What `.clone()` does for `String`:

1. Allocates new heap memory for the copy
2. Copies all the character bytes from the original's heap to the new heap
3. Returns a new `String` that owns the new heap memory

```
Stack                    Heap
┌──────────────┐        ┌───────────────┐
│ original:    │        │               │
│  ptr ────────┼───────▶│ S u n i l     │  ← original's heap data
│  len: 5      │        └───────────────┘
│  cap: 5      │
└──────────────┘
                        ┌───────────────┐
┌──────────────┐        │               │
│ copy:        │        │               │
│  ptr ────────┼───────▶│ S u n i l     │  ← copy's own heap data
│  len: 5      │        └───────────────┘
│  cap: 5      │
└──────────────┘
```

### Clone vs Copy

| | Copy | Clone |
|---|---|---|
| **When it happens** | Automatically on assignment | Only when you call `.clone()` |
| **Cost** | Always cheap (bitwise copy) | Can be expensive (deep copy) |
| **Heap data** | No heap data involved | Duplicates heap data |
| **Requires** | `impl Copy` | `impl Clone` |
| **Relationship** | All `Copy` types are also `Clone` | Not all `Clone` types are `Copy` |

> **Common mistake**: Calling `.clone()` everywhere to "make the compiler happy." This works but is wasteful. Before cloning, ask: do I actually need two independent copies, or can I borrow?

---

## Ownership Transfer in Function Calls

When you pass a value to a function, ownership follows the same rules as assignment.

### Moving into a function

```rust
fn greet(name: String) {
    println!("Hello, {}!", name);
}  // name is dropped here - heap memory is freed

fn main() {
    let my_name = String::from("Sunil");
    greet(my_name);  // ownership of my_name is moved into the function
    
    // println!("{}", my_name);  // COMPILE ERROR: my_name was moved
}
```

**What happens internally:**

1. `my_name` is created on the stack (pointing to heap data "Sunil")
2. `greet(my_name)` — ownership moves to the `name` parameter inside `greet`
3. Inside `greet`, `name` is valid and can be used
4. When `greet` returns, `name` goes out of scope, and `drop()` is called, freeing the heap memory
5. Back in `main`, `my_name` is no longer valid

**The fix — borrow instead of move:**

```rust
fn greet(name: &String) {  // Takes a reference (borrow)
    println!("Hello, {}!", name);
}  // name (the reference) goes out of scope, but the data is NOT freed

fn main() {
    let my_name = String::from("Sunil");
    greet(&my_name);  // Lend a reference - ownership stays with my_name
    
    println!("{}", my_name);  // ✅ Works! my_name was never moved
}
```

Even better, use `&str` instead of `&String`:

```rust
fn greet(name: &str) {  // More idiomatic - accepts both &String and &str
    println!("Hello, {}!", name);
}
```

### Copying into a function

```rust
fn double(x: i32) -> i32 {
    x * 2
}

fn main() {
    let num = 42;
    let result = double(num);
    
    println!("num: {}", num);       // ✅ num is still valid (Copy type)
    println!("result: {}", result);  // ✅
}
```

Since `i32` is `Copy`, passing it to a function copies the value. The original remains valid.

---

## Ownership and Return Values

Functions can transfer ownership back to the caller by returning values.

```rust
fn create_greeting(name: &str) -> String {
    let greeting = format!("Hello, {}!", name);
    greeting  // Ownership of the String is moved to the caller
}

fn main() {
    let msg = create_greeting("Sunil");  // msg now owns the String
    println!("{}", msg);
}  // msg goes out of scope here → String is dropped
```

### The "give and take" pattern (anti-pattern)

```rust
// Don't do this - it's cumbersome
fn add_exclamation(s: String) -> String {
    let mut result = s;
    result.push('!');
    result  // Give ownership back
}

fn main() {
    let name = String::from("Sunil");
    let name = add_exclamation(name);  // Move in, get it back
    println!("{}", name);
}
```

The better approach is to borrow:

```rust
fn add_exclamation(s: &mut String) {
    s.push('!');
}

fn main() {
    let mut name = String::from("Sunil");
    add_exclamation(&mut name);
    println!("{}", name);  // "Sunil!"
}
```

---

## Ownership Inside Structs

When a struct owns data, the struct is responsible for freeing that data.

```rust
struct User {
    name: String,     // User struct OWNS this String
    email: String,    // User struct OWNS this String
    age: u32,         // Copy type - just a value
}

fn main() {
    let user = User {
        name: String::from("Sunil"),
        email: String::from("sunil@example.com"),
        age: 25,
    };
    
    // Moving a field out of the struct
    let name = user.name;  // name field is moved OUT of user
    
    // println!("{}", user.name);   // COMPILE ERROR: field was moved
    println!("{}", user.email);     // ✅ This field wasn't moved
    println!("{}", user.age);       // ✅ Copy type, always valid
    // println!("{:?}", user);      // COMPILE ERROR: can't use partial struct
}
```

**What the compiler checks:**

When you move a field out of a struct, Rust tracks which fields are still valid. You can still access the un-moved fields individually, but you can't use the struct as a whole (because it's partially moved).

### Deriving Clone and Copy for structs

```rust
#[derive(Clone, Debug)]
struct User {
    name: String,
    email: String,
    age: u32,
}

// This will NOT compile:
// #[derive(Copy, Clone)]
// struct User {
//     name: String,  // String is not Copy, so User can't be Copy
//     age: u32,
// }
```

A struct can only be `Copy` if ALL its fields are `Copy`. Since `String` is not `Copy`, any struct containing `String` cannot be `Copy`.

```rust
// This CAN be Copy because all fields are Copy
#[derive(Copy, Clone, Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;  // Copied, not moved
    println!("{:?}", p1);  // ✅ Still valid
    println!("{:?}", p2);  // ✅
}
```

---

## Ownership Inside Collections

Collections like `Vec`, `HashMap`, etc. own their elements.

```rust
fn main() {
    let mut names: Vec<String> = Vec::new();
    
    let name = String::from("Sunil");
    names.push(name);  // Ownership of name is moved INTO the Vec
    
    // println!("{}", name);  // COMPILE ERROR: name was moved into the Vec
    
    // The Vec now owns the String
    println!("{}", names[0]);  // ✅ Access through the Vec
}
```

### Getting values out of a Vec

```rust
fn main() {
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
    ];
    
    // This borrows - doesn't move
    let first: &String = &names[0];
    println!("{}", first);  // ✅
    println!("{:?}", names);  // ✅ names still owns everything
    
    // This moves - takes ownership of the element
    // let owned: String = names[0];  // COMPILE ERROR
    // You can't move out of an indexed collection because it would
    // leave a "hole" in the Vec
    
    // To get ownership, use methods like:
    let mut names = names;
    let last = names.pop().unwrap();  // Removes and returns the last element
    println!("Removed: {}", last);    // last owns "Charlie"
    println!("Remaining: {:?}", names);  // ["Alice", "Bob"]
}
```

### Iterating and ownership

```rust
fn main() {
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
    ];
    
    // Borrowing iteration (most common)
    for name in &names {  // name: &String
        println!("{}", name);
    }
    println!("{:?}", names);  // ✅ names still valid
    
    // Consuming iteration (moves each element out)
    for name in names {  // name: String (owned)
        println!("{}", name);
    }
    // println!("{:?}", names);  // COMPILE ERROR: names was consumed
}
```

This is critical for concurrency: when you send a `Vec<String>` to another thread, the entire Vec and all its elements move to that thread. The original thread can no longer access any of it.

---

## Drop — Automatic Cleanup

When a value goes out of scope, Rust calls its `Drop` implementation (if it has one).

```rust
struct DatabaseConnection {
    url: String,
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        println!("Closing connection to: {}", self.url);
        // In real code: close the actual connection
    }
}

fn main() {
    let conn = DatabaseConnection {
        url: String::from("postgres://localhost/mydb"),
    };
    println!("Using connection...");
}  // conn goes out of scope → drop() is called → "Closing connection to: ..."
```

**Output:**
```
Using connection...
Closing connection to: postgres://localhost/mydb
```

> **Real-world use**: This is called RAII (Resource Acquisition Is Initialization). In Rust, if you have a value, you have the resource. When the value is dropped, the resource is released. This pattern is used for files, network connections, locks (Mutex guards), and many other resources.

---

## Things to Remember

1. **Every value has exactly one owner.** No exceptions.
2. **Assignment of heap data = move.** The old variable becomes invalid.
3. **Assignment of stack-only data = copy.** Both variables remain valid.
4. **Functions consume ownership** unless you pass a reference.
5. **Structs own their fields.** When the struct is dropped, its fields are dropped.
6. **Collections own their elements.** When the collection is dropped, its elements are dropped.
7. **`.clone()` creates an independent deep copy.** Use it when you genuinely need two copies.
8. **Don't clone everything just to make the compiler happy.** Prefer borrowing first.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "Move means the data is physically moved in memory" | Move just means the compiler transfers the concept of ownership. The stack data might be copied, but the heap data stays where it is. |
| "Copy is the same as Clone" | Copy is automatic and cheap (bitwise). Clone is explicit and can be expensive. |
| "I should use `.clone()` whenever the compiler complains about moves" | Usually, borrowing (`&`) is the better solution. Clone only when you need independent copies. |
| "Ownership is just about memory" | It's also about resources: file handles, sockets, locks, channels. |
| "Ownership makes Rust harder" | It makes Rust different. Once internalized, it prevents bugs that would crash your production trading system. |

---

## Exercises

### Exercise 1: Predict the Output

What happens when you compile and run this code?

```rust
fn main() {
    let a = String::from("hello");
    let b = a;
    let c = b;
    println!("{}", c);
}
```

<details>
<summary>Answer</summary>

It compiles and prints "hello". Ownership moves from `a` → `b` → `c`. Only `c` is valid at the `println!`.

</details>

### Exercise 2: Fix the Compiler Error

```rust
fn take_string(s: String) {
    println!("{}", s);
}

fn main() {
    let name = String::from("Sunil");
    take_string(name);
    println!("Name is: {}", name);
}
```

<details>
<summary>Answer</summary>

Change the function to borrow:
```rust
fn take_string(s: &str) {
    println!("{}", s);
}

fn main() {
    let name = String::from("Sunil");
    take_string(&name);
    println!("Name is: {}", name);
}
```

</details>

### Exercise 3: Why does this work?

```rust
fn main() {
    let x = 42;
    let y = x;
    let z = x;
    println!("{} {} {}", x, y, z);
}
```

<details>
<summary>Answer</summary>

`i32` implements `Copy`. Each assignment copies the value (just 4 bytes on the stack). All three variables are independent and valid.

</details>

---

## Interview Questions

**Q: What is ownership in Rust and why does it exist?**

> Ownership is Rust's compile-time memory management system. Every value has exactly one owner, and the value is freed when the owner goes out of scope. It exists to provide memory safety without garbage collection, giving both safety and performance.

**Q: What is the difference between move and copy semantics?**

> Move semantics transfers ownership — the source becomes invalid. Copy semantics duplicates the value — both source and destination are valid. Types that implement `Copy` (small, stack-only types like integers) use copy semantics. Types with heap data (like `String`) use move semantics.

**Q: Why can't `String` implement `Copy`?**

> `Copy` requires that the type can be duplicated with a simple bitwise copy. `String` owns heap-allocated data. A bitwise copy would create two `String`s pointing to the same heap memory, leading to a double-free when both are dropped. To get an independent copy, you must use `Clone`, which explicitly allocates new heap memory.

**Q: How does ownership relate to concurrency?**

> Ownership ensures that when you transfer data to another thread (via `move` or channels), the sending thread can no longer access it. This prevents data races at compile time. The `Send` trait marks types whose ownership can be safely transferred between threads.

---

> **Next**: [Chapter 2: Borrowing](./02_borrowing.md) — How to use data without taking ownership
