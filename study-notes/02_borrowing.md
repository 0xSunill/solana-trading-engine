# Chapter 2: Borrowing

> **Connection to the bigger picture**: Ownership is great for safety, but if every function call consumed ownership, you'd have to clone everything constantly. Borrowing solves this — it lets you temporarily lend access to data without giving up ownership. In concurrent and async Rust, borrowing rules prevent data races at compile time. The borrow checker is why Rust can guarantee thread safety without a runtime.

---

## What Is Borrowing?

Borrowing is Rust's way of letting code access data it doesn't own. Instead of taking ownership, you take a **reference** to the data.

Think of it like a library book:
- **Ownership** = you bought the book. It's yours.
- **Borrowing** = you borrowed the book from the library. You can read it, but you must return it. The library still owns it.

In Rust:
- **Ownership** = you have the value. You can do anything with it.
- **Immutable borrow** (`&T`) = you can look at the value but not change it.
- **Mutable borrow** (`&mut T`) = you can look at AND change the value, but nobody else can access it while you're changing it.

---

## Immutable References: `&T`

An immutable reference lets you read data without taking ownership.

```rust
fn print_length(s: &String) {
    println!("Length of '{}' is {}", s, s.len());
}  // s (the reference) goes out of scope, but the String is NOT dropped

fn main() {
    let name = String::from("Sunil");
    print_length(&name);  // Lend an immutable reference
    println!("I still own: {}", name);  // ✅ name is still valid
}
```

**What happens internally:**

```
Stack                              Heap
┌─────────────────┐
│ main's frame:   │
│ name: String    │
│  ┌───────────┐  │               ┌──────────────┐
│  │ ptr ──────┼──┼──────────────▶│ S u n i l    │
│  │ len: 5    │  │               └──────────────┘
│  │ cap: 5    │  │
│  └───────────┘  │
└─────────────────┘

┌──────────────────┐
│ print_length's   │
│ frame:           │
│ s: &String       │
│  ┌───────────┐   │
│  │ ptr ──────┼───┼──▶ (points to name on main's stack)
│  └───────────┘   │
└──────────────────┘
```

The reference `s` is a pointer to `name`. It does NOT own the data. When `print_length` returns, only the reference is cleaned up — the actual String data is untouched.

### Multiple immutable references

You can have as many immutable references as you want simultaneously:

```rust
fn main() {
    let name = String::from("Sunil");
    
    let r1 = &name;
    let r2 = &name;
    let r3 = &name;
    
    println!("{}, {}, {}", r1, r2, r3);  // ✅ All fine
}
```

This is safe because nobody can modify the data while these references exist. If the data can't change, reading it from multiple places simultaneously is perfectly safe.

> **Why this matters for concurrency**: This is the same principle that makes shared immutable data safe across threads. If 10 threads all have `&T` references to the same data, and nobody can modify it, there's no data race.

---

## Mutable References: `&mut T`

A mutable reference lets you modify borrowed data.

```rust
fn add_greeting(s: &mut String) {
    s.push_str(", welcome!");
}

fn main() {
    let mut name = String::from("Sunil");  // Must be declared `mut`
    add_greeting(&mut name);               // Lend a mutable reference
    println!("{}", name);                  // "Sunil, welcome!"
}
```

**Key requirement**: The variable itself must be declared `mut` to create a mutable reference to it.

### The One Mutable Reference Rule

You can only have ONE mutable reference to a value at a time:

```rust
fn main() {
    let mut name = String::from("Sunil");
    
    let r1 = &mut name;
    let r2 = &mut name;  // COMPILE ERROR
    
    println!("{}, {}", r1, r2);
}
```

**Compiler error:**

```
error[E0499]: cannot borrow `name` as mutable more than once at a time
 --> src/main.rs:5:14
  |
4 |     let r1 = &mut name;
  |              --------- first mutable borrow occurs here
5 |     let r2 = &mut name;
  |              ^^^^^^^^^ second mutable borrow occurs here
6 |
7 |     println!("{}, {}", r1, r2);
  |                        -- first borrow later used here
```

**Why does this rule exist?**

Imagine two mutable references to the same `Vec`:

```
r1 ──▶ Vec [1, 2, 3]  ◀── r2
```

If `r1` pushes an element, the Vec might reallocate its internal buffer to a new memory location:

```
r1 ──▶ Vec [1, 2, 3, 4]  (new location in memory)
r2 ──▶ (old, freed memory)  ← DANGLING POINTER!
```

With one mutable reference, this can't happen because nobody else is looking at the data.

> **Why this matters for concurrency**: A data race occurs when two threads access the same data, at least one is writing, and there's no synchronization. Rust's "one mutable reference OR many immutable references" rule prevents data races at compile time. In concurrent code, `Mutex` and `RwLock` enforce this rule at runtime.

---

## The Fundamental Borrowing Rule

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   At any given time, you can have EITHER:                   │
│                                                             │
│   • ONE mutable reference (&mut T)                          │
│                                                             │
│   OR                                                        │
│                                                             │
│   • ANY number of immutable references (&T)                 │
│                                                             │
│   But NEVER both at the same time.                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

This is sometimes called the **aliasing XOR mutation** rule:

- **Aliasing**: Multiple references to the same data (multiple `&T`).
- **Mutation**: Changing the data (`&mut T`).
- **XOR**: You can have one or the other, but not both simultaneously.

### Why aliasing + mutation is dangerous

```rust
// This is the kind of bug Rust prevents:
fn main() {
    let mut v = vec![1, 2, 3];
    
    let first = &v[0];     // Immutable reference to the first element
    
    v.push(4);             // Mutable operation on the Vec
    // push() might reallocate the Vec's internal buffer
    // If it does, `first` now points to freed memory!
    
    println!("{}", first); // Would be a use-after-free bug in C/C++
}
```

**Compiler error:**

```
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:6:5
  |
4 |     let first = &v[0];
  |                  - immutable borrow occurs here
5 |
6 |     v.push(4);
  |     ^^^^^^^^^ mutable borrow occurs here
7 |
8 |     println!("{}", first);
  |                    ----- immutable borrow later used here
```

**The fix:**

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    
    // Option 1: Use the reference before mutating
    let first = &v[0];
    println!("{}", first);  // Use it here
    // first is no longer used after this point
    
    v.push(4);  // ✅ Now safe to mutate
    
    // Option 2: Clone the value
    let first = v[0];  // i32 is Copy, so this copies the value
    v.push(4);
    println!("{}", first);  // ✅ first has its own copy
}
```

---

## Non-Lexical Lifetimes (NLL)

Modern Rust (since 2018 edition) uses **Non-Lexical Lifetimes**. This means a borrow ends when the reference is last used, not at the end of the scope.

```rust
fn main() {
    let mut name = String::from("Sunil");
    
    let r = &name;           // Immutable borrow starts
    println!("{}", r);       // Last use of r
    // Immutable borrow ends here (NLL)
    
    let r_mut = &mut name;   // ✅ Mutable borrow is allowed now
    r_mut.push_str("!");
    println!("{}", r_mut);
}
```

Before NLL (Rust 2015), this would have been an error because `r` would have been considered borrowed until the end of the scope. NLL makes the borrow checker smarter and more ergonomic.

---

## Borrowing Rules in Practice

### Pattern 1: Read-only functions (most common)

```rust
fn calculate_total(prices: &[f64]) -> f64 {
    prices.iter().sum()
}

fn main() {
    let prices = vec![9.99, 24.95, 3.50];
    let total = calculate_total(&prices);
    println!("Total: ${:.2}", total);
    println!("Items: {:?}", prices);  // ✅ prices is still valid
}
```

### Pattern 2: Mutation through mutable reference

```rust
fn apply_discount(prices: &mut Vec<f64>, discount: f64) {
    for price in prices.iter_mut() {
        *price *= (1.0 - discount);
    }
}

fn main() {
    let mut prices = vec![100.0, 200.0, 300.0];
    apply_discount(&mut prices, 0.10);  // 10% discount
    println!("{:?}", prices);  // [90.0, 180.0, 270.0]
}
```

### Pattern 3: Returning a reference (requires lifetimes)

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    let result;
    let a = String::from("hello");
    {
        let b = String::from("hi");
        result = longest(&a, &b);
        println!("Longest: {}", result);  // Must use result while b is alive
    }
    // Can't use result here - b is dropped
}
```

(We'll cover lifetimes in detail in the next chapter.)

---

## The Borrow Checker

The borrow checker is the part of the Rust compiler that enforces borrowing rules. It runs at compile time and has zero runtime cost.

**What the borrow checker tracks:**

1. Which variables are currently borrowed
2. Whether borrows are mutable or immutable
3. When borrows begin and end (using NLL)
4. Whether any reference could outlive the data it points to

**Common borrow checker errors and what they mean:**

| Error | Meaning |
|-------|---------|
| `cannot borrow as mutable because it is also borrowed as immutable` | You're trying to mutate data while something else has a read reference |
| `cannot borrow as mutable more than once` | You're trying to create two `&mut` references at the same time |
| `does not live long enough` | A reference would outlive the data it points to |
| `cannot move out of borrowed content` | You're trying to take ownership of something you only have a reference to |

### Fighting the borrow checker vs understanding it

When learning Rust, the borrow checker can feel like an adversary. But every error it produces corresponds to a real bug it's preventing:

| Borrow checker error | Bug it prevents |
|---------------------|-----------------|
| "two mutable borrows" | Data race |
| "immutable and mutable borrow" | Iterator invalidation, use-after-free |
| "does not live long enough" | Dangling pointer |
| "cannot move out of borrowed content" | Double free |

> **Real-world use**: In a trading system, imagine two async tasks both modifying the same order book through mutable references. Without the borrow checker, one task might read stale data while another is updating it, leading to incorrect trades. Rust catches this at compile time.

---

## Borrowing and Slices

Slices are one of the most common types of borrows in Rust.

```rust
fn main() {
    let text = String::from("hello world");
    
    let hello: &str = &text[0..5];    // Slice borrowing part of the String
    let world: &str = &text[6..11];   // Another slice
    
    println!("{} {}", hello, world);
    
    // text is still the owner
    println!("Full: {}", text);
}
```

**What a slice looks like in memory:**

```
Stack                           Heap
┌──────────────────┐
│ text: String     │
│  ptr ────────────┼──────────▶┌──────────────────────┐
│  len: 11         │           │ h e l l o   w o r l d │
│  cap: 11         │           └──────────────────────┘
└──────────────────┘                ▲         ▲
                                    │         │
┌──────────────────┐                │         │
│ hello: &str      │                │         │
│  ptr ────────────┼────────────────┘         │
│  len: 5          │                          │
└──────────────────┘                          │
                                              │
┌──────────────────┐                          │
│ world: &str      │                          │
│  ptr ────────────┼──────────────────────────┘
│  len: 5          │
└──────────────────┘
```

Slices borrow a portion of the original data. They don't own anything — they're just a pointer and a length.

---

## `&str` vs `&String`

This is a common source of confusion:

```rust
// &String: a reference to a String value
fn takes_string_ref(s: &String) {
    println!("{}", s);
}

// &str: a string slice - more general
fn takes_str(s: &str) {
    println!("{}", s);
}

fn main() {
    let owned = String::from("hello");
    let literal = "world";  // This is already a &str
    
    takes_string_ref(&owned);   // ✅
    // takes_string_ref(literal); // ❌ literal is &str, not &String
    
    takes_str(&owned);    // ✅ String auto-derefs to &str
    takes_str(literal);   // ✅ Already a &str
}
```

> **Best practice**: Use `&str` as function parameter type, not `&String`. It's more flexible — it accepts both `&String` and string literals.

---

## Reborrowing

You can create a shorter-lived reference from a longer-lived one:

```rust
fn main() {
    let mut data = String::from("hello");
    
    let r = &mut data;  // Mutable borrow
    
    // You can create an immutable reference from a mutable one
    let r2: &String = &*r;  // Reborrow as immutable
    println!("{}", r2);
    
    // Or more commonly, it happens implicitly:
    print_it(r);  // r is &mut String, but print_it takes &String
    // Rust automatically reborrows
    
    r.push_str(" world");  // ✅ r is still usable
}

fn print_it(s: &String) {
    println!("{}", s);
}
```

---

## Things to Remember

1. **`&T` is read-only**, `&mut T` allows modification.
2. **Many `&T` OR one `&mut T`** — never both.
3. **References never own data.** When a reference goes out of scope, the data is not dropped.
4. **Use `&str` over `&String`** for function parameters.
5. **NLL means borrows end at last use**, not at end of scope.
6. **The borrow checker prevents real bugs** — data races, dangling pointers, iterator invalidation.
7. **Borrowing is the preferred way to share data** — prefer `&T` before resorting to `Clone`, `Rc`, or `Arc`.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "The borrow checker is too strict" | Every rejection corresponds to a potential bug. As you get better, you'll agree with the compiler more often. |
| "I should avoid references and just clone" | References are zero-cost; cloning allocates memory. Use references when possible. |
| "`&mut` means the reference itself is mutable" | `&mut T` means you can mutate the data T through the reference. The reference itself is always a fixed pointer. |
| "Borrowing and lifetimes are the same thing" | Borrowing is about access rules. Lifetimes are about how long references are valid. They're related but distinct. |

---

## Interview Questions

**Q: What is the fundamental borrowing rule in Rust?**

> At any given time, you can have either one mutable reference or any number of immutable references to a value, but not both. This prevents data races and aliasing bugs at compile time.

**Q: Why can't you have a mutable and immutable reference simultaneously?**

> If you have a `&T` (read-only view) and a `&mut T` (write access) at the same time, the mutable reference could change the data while the immutable reference assumes it hasn't changed. This can cause iterator invalidation, buffer reallocation, and other bugs.

**Q: How does the borrow checker relate to thread safety?**

> The borrow checker's "aliasing XOR mutation" rule is the compile-time equivalent of a read-write lock. Multiple readers (`&T`) are safe, a single writer (`&mut T`) is safe, but mixing them is not. In threaded code, `RwLock` enforces the same rule at runtime.

---

> **Next**: [Chapter 3: References and Lifetimes](./03_references_and_lifetimes.md) — How Rust ensures references are always valid
