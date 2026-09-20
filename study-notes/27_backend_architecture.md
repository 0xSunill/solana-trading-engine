# Chapter 27: Backend Architecture in Rust

> **Connection to the bigger picture**: A well-structured backend is maintainable, testable, and scalable. This chapter covers the layered architecture pattern that separates concerns — routing, business logic, data access, and configuration — which is essential for production Solana trading backends.

---

## Production Structure

```
src/
├── main.rs              ← Entry: builds runtime, router, starts server
├── config/
│   └── mod.rs           ← Configuration loading (env vars, files)
├── state.rs             ← AppState: shared across all handlers
├── routes/
│   └── mod.rs           ← Router definition, groups routes
├── handlers/
│   ├── mod.rs
│   ├── orders.rs        ← HTTP handler functions
│   └── health.rs        ← Health check endpoint
├── services/
│   ├── mod.rs
│   └── trading.rs       ← Business logic, orchestration
├── repositories/
│   ├── mod.rs
│   └── order_repo.rs    ← Database queries (SQL)
├── models/
│   ├── mod.rs
│   └── order.rs         ← Data structures, domain types
├── middleware/
│   └── auth.rs          ← Authentication, rate limiting
├── errors/
│   └── mod.rs           ← Custom error types
└── telemetry.rs         ← Logging, tracing setup
```

---

## Layer Responsibilities

```
Request ──▶ Router ──▶ Handler ──▶ Service ──▶ Repository ──▶ Database
                          │           │            │
                     Validates     Business     SQL queries
                     input,        logic,       data access
                     extracts      rules
                     data
```

| Layer | Responsibility | Knows about |
|-------|---------------|-------------|
| **Router** | Maps URL paths to handlers | Handlers, middleware |
| **Handler** | Parses HTTP, returns HTTP | Services, extractors |
| **Service** | Business logic, rules | Repositories, models |
| **Repository** | Database access (SQL) | Models, database pool |
| **Model** | Data structures | Nothing (pure data) |

### Why layers matter

Each layer has a single responsibility and can be tested independently:

- **Handlers** can be tested with mock services
- **Services** can be tested with mock repositories
- **Repositories** can be tested with test databases

---

## Application State

```rust
use sqlx::PgPool;
use std::sync::Arc;

pub struct AppState {
    pub db: PgPool,                                          // Database pool
    pub config: AppConfig,                                    // Configuration
    pub price_cache: tokio::sync::RwLock<std::collections::HashMap<String, f64>>,  // Cached prices
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<SharedState> {
        let db = PgPool::connect(&config.database_url).await?;
        
        Ok(Arc::new(Self {
            db,
            config,
            price_cache: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }))
    }
}
```

---

## Configuration

```rust
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub solana_rpc_url: String,
    pub solana_ws_url: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            server_host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()?,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")?,
            solana_ws_url: std::env::var("SOLANA_WS_URL")?,
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
        })
    }
}
```

---

## Dependency Injection

Rust uses **constructor injection** and the type system instead of DI frameworks:

```rust
// Service depends on repository (injected via constructor)
pub struct TradingService {
    order_repo: OrderRepository,
    price_client: PriceClient,
}

impl TradingService {
    pub fn new(order_repo: OrderRepository, price_client: PriceClient) -> Self {
        Self { order_repo, price_client }
    }
    
    pub async fn place_order(&self, request: OrderRequest) -> Result<Order, AppError> {
        let price = self.price_client.get_price(&request.symbol).await?;
        let order = Order::new(request, price);
        self.order_repo.save(&order).await?;
        Ok(order)
    }
}
```

For testing, you can use trait objects or generics to inject mock implementations.

---

## Things to Remember

1. **Separate concerns into layers** — handler, service, repository.
2. **AppState holds shared resources** — DB pool, config, caches.
3. **Configuration from environment** — use `dotenvy` and `std::env`.
4. **Inject dependencies** through constructors and the type system.
5. **Each layer is independently testable.**

---

> **Next**: [Chapter 28: Tokio + Axum + Database](./28_tokio_axum_database.md)
