# Chapter 3: References and Lifetimes

> **Connection to the bigger picture**: Lifetimes are how Rust guarantees that references are always valid — they never point to freed memory. In async Rust, lifetimes become more complex because data needs to survive across `.await` points where the task might be suspended and resumed later, potentially on a different thread. Understanding lifetimes deeply is essential for understanding why `tokio::spawn` requires `'static` and why async closures often need `move`.

---

## What Is a Reference Really?

At the machine level, a reference is a pointer — a memory address pointing to data stored somewhere else. But unlike raw pointers in C/C++, Rust references carry compile-time guarantees:

1. **A reference is always valid.** It never points to freed memory (no dangling pointers).
2. **A reference always points to a valid value of the correct type.** No null pointers (use `Option<&T>` instead).
3. **References obey borrowing rules.** No data races through references.

```rust
fn main() {
    let x: i32 = 42;
    let r: &i32 = &x;  // r is a reference to x
    
    println!("x = {}", x);
    println!("r = {}", r);   // Prints 42 (auto-deref)
    println!("r points to address: {:p}", r);  // Prints the memory address
}
```

**Memory layout:**

```
Stack
┌─────────────────────────────┐
│ x: i32 = 42                │  ← address: 0x7ffd123456
│                             │
│ r: &i32                    │
│   = 0x7ffd123456           │  ← stores the ADDRESS of x
└─────────────────────────────┘
```

---

## The Problem Lifetimes Solve

Consider this broken C code:

```c
int* dangling_pointer() {
    int x = 42;
    return &x;  // Returns pointer to local variable
}  // x is freed when function returns

int main() {
    int* p = dangling_pointer();
    printf("%d\n", *p);  // UNDEFINED BEHAVIOR: p points to freed memory
}
```

This is a **dangling pointer** — a reference to memory that has been freed. In C, this compiles without warning and causes unpredictable behavior at runtime.

Rust prevents this at compile time:

```rust
fn dangling_reference() -> &i32 {  // COMPILE ERROR
    let x = 42;
    &x  // Can't return reference to local variable
}  // x is dropped here, so the reference would be dangling
```

**Compiler error:**

```
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:28
  |
1 | fn dangling_reference() -> &i32 {
  |                            ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value,
    but there is no value for it to be borrowed from
```

The compiler is saying: "You're trying to return a reference, but what does it point to? The local variable `x` is freed when the function returns, so the reference would be dangling. I can't allow this."

---

## What Are Lifetimes?

A lifetime is a span of code during which a reference is valid. Every reference in Rust has a lifetime, even if you don't write it explicitly.

```rust
fn main() {
    let r;                  // r's lifetime starts here
    {
        let x = 42;         // x's lifetime starts here
        r = &x;             // r borrows x
    }                       // x's lifetime ENDS here — x is dropped
    
    // println!("{}", r);   // COMPILE ERROR: r outlives x
}
```

**Compiler error:**

```
error[E0597]: `x` does not live long enough
 --> src/main.rs:5:13
  |
4 |         let x = 42;
  |             - binding `x` declared here
5 |         r = &x;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `x` dropped here while still borrowed
7 |
8 |     println!("{}", r);
  |                    - borrow later used here
```

**What the compiler is checking:**

```
main() {
    let r;                   ─┐
    {                         │ r's lifetime
        let x = 42;    ─┐    │
        r = &x;         │    │ ← r borrows x, but...
    }                   ─┘    │ ← x dies here
                              │
    println!("{}", r);        │ ← r is used here, but x is already dead!
}                            ─┘
```

The reference `r` lives longer than the data `x` it points to. This would be a dangling pointer. Rust prevents it.

**The fix:**

```rust
fn main() {
    let x = 42;         // x lives in the outer scope
    let r = &x;         // r borrows x — both live in the same scope
    println!("{}", r);   // ✅ x is still alive
}
```

---

## Lifetime Annotations

Most of the time, the Rust compiler infers lifetimes automatically. But when a function takes multiple references and returns a reference, the compiler needs your help to understand which input reference the output reference is derived from.

### When you DON'T need annotations

```rust
// Only one input reference → compiler knows the output must come from it
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[0..i];
        }
    }
    s
}
```

The compiler applies **lifetime elision rules** (covered below) and infers that the output `&str` lives as long as the input `&str`.

### When you DO need annotations

```rust
// Two input references → compiler doesn't know which one the output comes from
fn longest(x: &str, y: &str) -> &str {  // COMPILE ERROR
    if x.len() >= y.len() { x } else { y }
}
```

**Compiler error:**

```
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----      ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
    signature does not say whether it is borrowed from `x` or `y`
```

**Why the compiler needs help:** The returned reference could come from either `x` or `y`. The compiler needs to know: "How long does the returned reference live?" The answer depends on which reference is returned, which depends on runtime data (string lengths). So the compiler asks you to explicitly state the relationship.

### Adding lifetime annotations

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}
```

**What this means:**

- `<'a>` declares a lifetime parameter named `'a` (like a generic type parameter but for lifetimes).
- `x: &'a str` means "x is a reference that lives for at least `'a`."
- `y: &'a str` means "y is a reference that lives for at least `'a`."
- `-> &'a str` means "the return value is a reference that lives for `'a`."

**What `'a` actually represents:** The **intersection** (overlap) of the lifetimes of `x` and `y`. The returned reference is guaranteed to be valid for the shorter of the two input lifetimes.

```
x lives:    |─────────────────────|
y lives:    |───────────|
'a is:      |───────────|         (the overlap)
return:     |───────────|         (valid for 'a)
```

### Practical example

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    let result;
    let string1 = String::from("long string");
    
    {
        let string2 = String::from("xyz");
        result = longest(&string1, &string2);
        println!("Longest: {}", result);  // ✅ Both strings alive here
    }
    // string2 is dropped here
    
    // println!("Longest: {}", result);  // ❌ COMPILE ERROR
    // result might reference string2, which is now dropped
}
```

If we try to use `result` after `string2` is dropped, the compiler catches it because `'a` (the overlap) ends when `string2` ends.

---

## Lifetime Elision Rules

Writing lifetime annotations everywhere would be tedious. Rust has three **elision rules** that let the compiler infer lifetimes in common cases:

### Rule 1: Each input reference gets its own lifetime

```rust
fn foo(x: &str, y: &str)
// becomes:
fn foo<'a, 'b>(x: &'a str, y: &'b str)
```

### Rule 2: If there's exactly one input lifetime, it's assigned to all output lifetimes

```rust
fn first_word(s: &str) -> &str
// becomes:
fn first_word<'a>(s: &'a str) -> &'a str
```

### Rule 3: If one of the inputs is `&self` or `&mut self`, its lifetime is assigned to all output lifetimes

```rust
impl MyStruct {
    fn get_name(&self) -> &str
    // becomes:
    fn get_name<'a>(&'a self) -> &'a str
}
```

**If these three rules don't resolve all lifetimes, you must annotate manually.**

That's why `longest(x: &str, y: &str) -> &str` needs annotations — Rule 1 gives them different lifetimes, and neither Rule 2 nor Rule 3 applies (there are two inputs, and it's not a method).

---

## The `'static` Lifetime

`'static` is a special lifetime that means "lives for the entire duration of the program."

### What has `'static` lifetime?

```rust
// String literals are 'static - they're embedded in the binary
let s: &'static str = "hello world";

// Owned values can satisfy 'static bounds (they don't borrow anything)
let name: String = String::from("Sunil");  // Owned, can be 'static
```

### `'static` doesn't mean "lives forever"

This is a critical misconception. `'static` means **"CAN live for the entire program."** It doesn't mean the value actually will. An owned `String` satisfies the `'static` bound because it doesn't borrow from anything with a limited lifetime — but it will still be dropped when its owner goes out of scope.

```rust
fn takes_static<T: 'static>(value: T) {
    // T doesn't borrow anything with a limited lifetime
    // T could be String, i32, Vec<u8>, etc.
}

fn main() {
    let name = String::from("Sunil");
    takes_static(name);  // ✅ String is 'static (it's owned, no borrows)
    
    let x = 42;
    takes_static(x);     // ✅ i32 is 'static
    
    let borrowed = "hello";
    takes_static(borrowed);  // ✅ &'static str
}
```

But:

```rust
fn takes_static<T: 'static>(value: T) {}

fn main() {
    let name = String::from("Sunil");
    let r: &String = &name;
    // takes_static(r);  // ❌ &String has a limited lifetime (tied to name)
}
```

> **Why this matters for async Rust**: `tokio::spawn` requires the future to be `'static`. This means the future cannot borrow data from the calling scope — it must own all its data or use `'static` references. This is why you often need `async move` blocks.

### `'static` in practice

```rust
// This is what tokio::spawn requires:
// pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
// where
//     F: Future + Send + 'static,
//     F::Output: Send + 'static,

#[tokio::main]
async fn main() {
    let name = String::from("Sunil");
    
    // ❌ This borrows `name` — the future is NOT 'static
    // tokio::spawn(async {
    //     println!("{}", name);
    // });
    
    // ✅ This MOVES `name` — the future IS 'static (owns its data)
    tokio::spawn(async move {
        println!("{}", name);
    });
}
```

---

## References Inside Structs

When a struct holds a reference, you must specify the lifetime:

```rust
// This won't compile:
// struct Excerpt {
//     text: &str,  // How long does this reference live?
// }

// This will:
struct Excerpt<'a> {
    text: &'a str,
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = &novel[..16];
    
    let excerpt = Excerpt {
        text: first_sentence,
    };
    
    println!("Excerpt: {}", excerpt.text);
}
```

**What `<'a>` means on the struct:** "This struct contains a reference, and the struct cannot outlive the data that reference points to."

```
novel: String          ─────────────────────────────────┐
  │                                                     │ novel's lifetime
  ▼                                                     │
"Call me Ishmael. Some years ago..."                    │
  ▲                                                     │
  │                                                     │
excerpt: Excerpt<'a>   ──────────────────┐              │
  text: &'a str        points to novel   │ excerpt's    │
                       ─────────────────┘  lifetime ≤   │
                                          novel's       │
                                          lifetime     ─┘
```

### The problem with references in structs and async

```rust
struct Config<'a> {
    database_url: &'a str,
}

// This is problematic for async:
// tokio::spawn(async move {
//     let config = Config { database_url: &some_string };
//     // The future borrows some_string through config
//     // Not 'static — can't be spawned!
// });

// Better for async: own the data
struct Config {
    database_url: String,  // Owned — no lifetime parameter needed
}
```

> **Key insight**: In async Rust, structs that will be used across `.await` points or spawned into tasks should generally own their data rather than borrowing it. This avoids lifetime issues.

---

## Why Async Makes Lifetimes More Complicated

When you call `.await`, the current task is **suspended**. The task might be resumed later — possibly on a different thread. Any data the task needs must survive across the suspension.

```rust
async fn process(data: &str) {
    // Before .await, data is borrowed
    println!("Processing: {}", data);
    
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // ↑ TASK IS SUSPENDED HERE
    // The task might not resume for a while
    // `data` must still be valid when it resumes
    
    println!("Done: {}", data);
    // data is used AFTER the .await
}
```

If `data` is a reference to a local variable, the variable might be dropped while the task is suspended. That would be a dangling pointer.

**This is why:**
1. `tokio::spawn` requires `'static` — spawned tasks must own all their data.
2. `async move` is so common — it moves ownership into the future.
3. `Arc` is prevalent in async code — it provides shared ownership with an indefinite lifetime.

We'll explore this deeply in later chapters.

---

## Lifetime Bounds on Traits

You can specify lifetime bounds on trait implementations:

```rust
trait Summary {
    fn summarize(&self) -> String;
}

// This struct borrows data, so it has a lifetime parameter
struct Article<'a> {
    title: &'a str,
    content: &'a str,
}

// The impl must also specify the lifetime
impl<'a> Summary for Article<'a> {
    fn summarize(&self) -> String {
        format!("{}: {}...", self.title, &self.content[..20])
    }
}
```

### Combined lifetime and trait bounds

```rust
// T must implement Display AND live for at least 'a
fn print_ref<'a, T: std::fmt::Display + 'a>(value: &'a T) {
    println!("{}", value);
}
```

---

## Multiple Lifetime Parameters

Sometimes you need different lifetimes for different parameters:

```rust
// x and y can have different lifetimes
fn first_or_default<'a, 'b>(x: &'a str, default: &'b str) -> &'a str {
    if !x.is_empty() {
        x  // Returns reference with lifetime 'a
    } else {
        // default  // ❌ Can't return 'b where 'a is expected
        x  // Must return something with 'a lifetime
    }
}
```

In practice, you rarely need multiple lifetime parameters. The most common case is the single `'a` lifetime.

---

## Higher-Ranked Trait Bounds (HRTB)

This is an advanced concept you'll encounter in async Rust with closures and trait objects:

```rust
// "for any lifetime 'a, F takes &'a str and returns usize"
fn apply_to_string<F>(f: F, s: &str) -> usize
where
    F: for<'a> Fn(&'a str) -> usize,
{
    f(s)
}
```

You don't need to use HRTB often directly, but you'll see `for<'a>` in error messages when working with closures and async code. Just know that it means "this works for any lifetime."

---

## Things to Remember

1. **Every reference has a lifetime** — the compiler tracks them even when you don't write them.
2. **Lifetime annotations don't change how long things live** — they describe relationships the compiler can't infer.
3. **`'static` means "doesn't borrow from anything short-lived"** — owned values satisfy `'static`.
4. **Lifetime elision** handles most cases automatically (one input ref, or `&self`).
5. **Structs with references need lifetime parameters** — prefer owned data in async code.
6. **Async makes lifetimes harder** because data must survive across `.await` points.
7. **`tokio::spawn` requires `'static`** — this drives the use of `async move`, owned data, and `Arc`.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "`'static` means the value lives forever" | It means the value CAN live for the entire program — it doesn't borrow from anything with a limited scope. Owned types like `String` are `'static`. |
| "Lifetime annotations change when values are dropped" | They don't change anything — they describe constraints the compiler uses for checking. |
| "I should avoid lifetimes" | You should understand them. In practice, elision handles most cases. When you need annotations, they're telling you something important about your data flow. |
| "References in structs are fine" | They work but add complexity. For async code, prefer owned data unless you have a specific reason to borrow. |

---

## Interview Questions

**Q: What is a lifetime in Rust?**

> A lifetime is the scope during which a reference is valid. The compiler uses lifetimes to ensure references never outlive the data they point to, preventing dangling pointer bugs at compile time.

**Q: What does `'static` mean?**

> `'static` means the value doesn't contain any non-static references — it either owns all its data or only borrows from `'static` data (like string literals). Owned types like `String`, `Vec<T>`, `i32` all satisfy `'static`. In async Rust, `tokio::spawn` requires `'static` because spawned tasks may run for an arbitrary duration.

**Q: Why are lifetimes more challenging in async Rust?**

> In async Rust, tasks can be suspended at `.await` points and resumed later, possibly on a different thread. Any borrowed data must remain valid across these suspension points. This is why `tokio::spawn` requires `'static` — the spawned task's lifetime is unpredictable, so it must own all its data rather than borrowing from the spawning scope.

**Q: Explain lifetime elision rules.**

> There are three rules: (1) Each input reference gets its own lifetime parameter. (2) If there's exactly one input lifetime, it's applied to all output lifetimes. (3) If one parameter is `&self` or `&mut self`, its lifetime applies to all output lifetimes. If these rules don't resolve all output lifetimes, you must annotate manually.

---

> **Next**: [Chapter 4: Traits](./04_traits.md) — The foundation for polymorphism, Send, Sync, and Future
