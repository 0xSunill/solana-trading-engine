# Chapter 7: OS Threads

> **Connection to the bigger picture**: OS threads are Rust's most direct concurrency primitive. Understanding threads deeply is essential because: (1) Tokio's runtime uses OS threads internally, (2) `spawn_blocking` creates OS threads for CPU-bound work, (3) the ownership transfer into threads (`move`) is the same pattern used with `async move`, and (4) the `Send` and `Sync` traits — central to async Rust — exist because of thread safety.

---

## Creating Threads

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        println!("Hello from a new thread!");
        42  // Return value
    });
    
    println!("Hello from the main thread!");
    
    let result = handle.join().unwrap();  // Wait for thread to finish
    println!("Thread returned: {}", result);
}
```

**What happens step by step:**

1. `thread::spawn(closure)` asks the OS to create a new thread.
2. The OS allocates a new stack (typically 2-8 MB) and begins executing the closure.
3. `spawn` returns a `JoinHandle<T>` immediately — the main thread continues.
4. Both threads run concurrently (and possibly in parallel).
5. `handle.join()` blocks the current thread until the spawned thread finishes.
6. `join()` returns `Result<T, Box<dyn Any + Send>>` — `Ok(value)` if the thread completed, `Err(panic_payload)` if the thread panicked.

```
Main thread:  [spawn]──────────[join()─blocks─]──[continues]
                 │                     ▲
                 ▼                     │
New thread:   [───────running──────────┘]
```

---

## The `move` Keyword

The most common pattern with threads is using `move` closures.

### The problem without `move`

```rust
use std::thread;

fn main() {
    let name = String::from("Sunil");
    
    let handle = thread::spawn(|| {
        println!("Hello, {}!", name);  // ❌ COMPILE ERROR
    });
    
    handle.join().unwrap();
}
```

**Compiler error:**

```
error[E0373]: closure may outlive the current function, but it borrows `name`,
which is owned by the current function
 --> src/main.rs:5:32
  |
5 |     let handle = thread::spawn(|| {
  |                                ^^ may outlive borrowed value `name`
6 |         println!("Hello, {}!", name);
  |                                ---- `name` is borrowed here
  |
  = note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `name` (and any other
referenced variables), use the `move` keyword
```

**Why this happens:**

The closure tries to **borrow** `name` by default. But the thread might run after `main` has finished — which means `name` would be dropped while the thread still has a reference to it. That would be a dangling pointer.

```
Main thread:  [creates name]──[spawns thread]──[might drop name here]
                                    │
Thread:        [───────────────────uses name───]  ← name might be dead!
```

The compiler requires `'static` for thread closures because it can't guarantee the spawned thread will finish before the current function returns.

### The solution: `move`

```rust
use std::thread;

fn main() {
    let name = String::from("Sunil");
    
    let handle = thread::spawn(move || {
        println!("Hello, {}!", name);  // ✅ name was MOVED into the closure
    });
    
    // println!("{}", name);  // ❌ name was moved — can't use here
    
    handle.join().unwrap();
}
```

**What `move` does:**

`move` makes the closure take **ownership** of all captured variables (instead of borrowing). The `String` is moved into the closure, which is then moved into the new thread.

```
Before move:
  main thread owns: name → "Sunil"

After thread::spawn(move || ...):
  main thread owns: (nothing - name was moved)
  new thread owns: name → "Sunil"
```

Since the thread now **owns** the data, there's no dangling reference problem.

> **Key insight**: This is exactly the same pattern as `tokio::spawn(async move { ... })`. The `move` keyword works the same way for both thread closures and async blocks.

---

## Multiple Threads

```rust
use std::thread;

fn main() {
    let mut handles = vec![];
    
    for i in 0..5 {
        let handle = thread::spawn(move || {
            println!("Thread {} starting", i);
            thread::sleep(std::time::Duration::from_millis(100));
            println!("Thread {} done", i);
            i * 10  // Return value
        });
        handles.push(handle);
    }
    
    // Collect results from all threads
    let results: Vec<i32> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    println!("Results: {:?}", results);
}
```

**Why `i` doesn't need `move` explained:**

Actually, `i` does get moved — `i32` is `Copy`, so `move` copies it into each closure. Each thread gets its own independent copy of `i`. This is safe because `i32` is a stack value that can be trivially copied.

For heap-allocated data, you'd need to clone:

```rust
use std::thread;

fn main() {
    let shared_data = String::from("base");
    let mut handles = vec![];
    
    for i in 0..5 {
        let data = shared_data.clone();  // Clone for each thread
        
        let handle = thread::spawn(move || {
            println!("Thread {}: {}", i, data);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

---

## Thread Lifecycle

```
┌──────────┐     spawn()     ┌──────────┐     closure     ┌──────────┐
│  Created  │ ──────────────▶ │  Running  │ ──completes──▶ │ Finished │
└──────────┘                 └──────────┘                 └──────────┘
                                  │                            │
                                  │ panic!()                   │
                                  ▼                            │
                             ┌──────────┐                     │
                             │ Panicked │                     │
                             └──────────┘                     │
                                  │                            │
                                  └──────────────┬─────────────┘
                                                 ▼
                                         join() returns:
                                         Ok(value) or Err(panic)
```

### Panics inside threads

A panic in a spawned thread does NOT crash the whole program (unlike panics on the main thread):

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        panic!("Thread crashed!");
    });
    
    match handle.join() {
        Ok(value) => println!("Thread returned: {:?}", value),
        Err(panic_info) => println!("Thread panicked: {:?}", panic_info),
    }
    
    println!("Main thread continues!");  // ✅ This runs
}
```

Output:
```
thread '<unnamed>' panicked at 'Thread crashed!'
Thread panicked: Any { .. }
Main thread continues!
```

### What happens if you don't join?

```rust
fn main() {
    thread::spawn(|| {
        thread::sleep(std::time::Duration::from_secs(5));
        println!("Thread done");  // This might never print!
    });
    
    println!("Main thread done");
}  // Main thread exits → process terminates → spawned thread is killed
```

If the main thread finishes before the spawned thread, the process exits and the spawned thread is forcibly terminated. Always `join()` threads you care about.

---

## Thread Builder

For more control over thread creation:

```rust
use std::thread;

fn main() {
    let builder = thread::Builder::new()
        .name("worker-1".into())     // Thread name (for debugging)
        .stack_size(4 * 1024 * 1024); // 4 MB stack
    
    let handle = builder.spawn(|| {
        println!("Thread name: {:?}", thread::current().name());
    }).expect("Failed to spawn thread");
    
    handle.join().unwrap();
}
```

---

## Sharing Data Between Threads

### Moving ownership (simple case)

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        sum
    });
    
    let result = handle.join().unwrap();
    println!("Sum: {}", result);  // 15
}
```

### Sharing read-only data with `Arc`

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];
    
    for i in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: sum = {}", i, data.iter().sum::<i32>());
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

### Sharing mutable data with `Arc<Mutex<T>>`

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
            // MutexGuard is dropped here → lock released
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final: {}", *counter.lock().unwrap());  // 10
}
```

---

## Scoped Threads

Scoped threads (stabilized in Rust 1.63) let you borrow data from the parent scope without `move`:

```rust
fn main() {
    let mut data = vec![1, 2, 3, 4, 5];
    
    std::thread::scope(|s| {
        // These threads can borrow from the outer scope
        s.spawn(|| {
            println!("Thread 1: {:?}", &data);  // ✅ Borrows data
        });
        
        s.spawn(|| {
            println!("Thread 2: {:?}", &data);  // ✅ Also borrows data
        });
    });
    // All scoped threads are joined here automatically
    
    // We can mutate data again since all threads have finished
    data.push(6);
    println!("After: {:?}", data);
}
```

**Why scoped threads don't need `'static`:** The `thread::scope` function guarantees that all threads are joined before it returns. Since the threads can't outlive the scope, borrowing is safe.

---

## Exercises

### Exercise 1: Parallel Sum

Split a vector into chunks and sum each chunk in a separate thread:

```rust
use std::thread;

fn parallel_sum(data: &[i32], num_threads: usize) -> i32 {
    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    
    thread::scope(|s| {
        let mut handles = vec![];
        
        for chunk in data.chunks(chunk_size) {
            handles.push(s.spawn(|| -> i32 {
                chunk.iter().sum()
            }));
        }
        
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

fn main() {
    let data: Vec<i32> = (1..=100).collect();
    let result = parallel_sum(&data, 4);
    println!("Sum: {}", result);  // 5050
}
```

---

## Things to Remember

1. **`thread::spawn` requires `'static`** — use `move` to transfer ownership.
2. **Always `join()` threads you care about** — otherwise they die when main exits.
3. **Panics in threads don't crash the program** — they're caught by `join()`.
4. **`move` transfers ownership** — the spawning scope can no longer use moved data.
5. **Use `Arc` for shared read-only data** across threads.
6. **Use `Arc<Mutex<T>>` for shared mutable data** across threads.
7. **Scoped threads** (`thread::scope`) allow borrowing from the parent scope.
8. **Thread creation is expensive** — don't create thousands. Use thread pools or async tasks for high concurrency.

---

## Interview Questions

**Q: Why does `thread::spawn` require the closure to be `'static`?**

> Because the spawned thread might outlive the function that created it. If the closure borrowed local variables, those variables could be dropped while the thread is still running, creating dangling pointers. `'static` ensures the closure owns all its data or only references data that lives for the entire program.

**Q: What is the difference between `thread::spawn` and `thread::scope`?**

> `thread::spawn` creates threads that can outlive their creator, requiring `'static` closures. `thread::scope` creates threads that are guaranteed to finish before the scope ends, allowing them to borrow from the parent scope without `move` or `Arc`.

**Q: How many threads should you create?**

> For CPU-bound work: one per CPU core (use `num_cpus` crate). For I/O-bound work: consider using async tasks instead. Creating thousands of threads wastes memory (2-8 MB stack each) and CPU time (context switching).

---

> **Next**: [Chapter 8: Send and Sync](./08_send_and_sync.md) — The traits that make thread safety a compile-time guarantee
