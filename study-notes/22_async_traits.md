# Chapter 22: Async Traits

> **Connection to the bigger picture**: Async functions in traits are essential for defining abstract interfaces in async code — like a `DataSource` trait with async `fetch` methods. Rust now supports `async fn` in traits natively (stabilized in Rust 1.75), making this much simpler than before.

---

## Native `async fn` in Traits (Rust 1.75+)

```rust
trait DataSource {
    async fn fetch(&self, key: &str) -> Result<String, Box<dyn std::error::Error>>;
}

struct DatabaseSource {
    connection_string: String,
}

impl DataSource for DatabaseSource {
    async fn fetch(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Async database query
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(format!("value for {}", key))
    }
}
```

### The `Send` bound issue

By default, the future returned by an async trait method is NOT guaranteed to be `Send`. This can cause issues with `tokio::spawn`:

```rust
async fn use_source(source: impl DataSource) {
    // This might not work with tokio::spawn if the future isn't Send
    let result = source.fetch("key").await;
    println!("{:?}", result);
}
```

### Adding `Send` bounds

```rust
// Use the trait_variant::make macro or manually specify:
trait DataSource: Send + Sync {
    fn fetch(&self, key: &str) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send;
}

// Or use the #[trait_variant::make] attribute (from trait-variant crate):
// #[trait_variant::make(SendDataSource: Send)]
// trait DataSource { ... }
```

### Using `async-trait` crate (pre-1.75 pattern, still widely used)

```rust
use async_trait::async_trait;

#[async_trait]
trait DataSource {
    async fn fetch(&self, key: &str) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
impl DataSource for DatabaseSource {
    async fn fetch(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(format!("value for {}", key))
    }
}
```

The `async_trait` macro transforms async methods into methods returning `Pin<Box<dyn Future>>`. This adds a heap allocation per call but makes the future `Send` by default.

---

## Object Safety Considerations

Async trait methods with `impl Future` return types are NOT object-safe by default:

```rust
trait Handler {
    async fn handle(&self);
}

// This won't work without async_trait:
// fn dispatch(handler: &dyn Handler) { ... }
```

Use `async_trait` or `Box<dyn Future>` for trait objects.

---

## Things to Remember

1. **Rust 1.75+ supports native async fn in traits.**
2. **`async-trait` crate** is still widely used for compatibility and trait objects.
3. **`Send` bounds** matter when using async traits with `tokio::spawn`.
4. **Trait objects with async methods** need heap allocation (Box<dyn Future>).
5. **Prefer native async traits** for new code when possible.

---

> **Next**: [Chapter 23: Common Async Compiler Errors](./23_async_compiler_errors.md)
