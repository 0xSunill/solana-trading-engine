// use sqlx::postgres::PgPoolOptions;

// #[tokio::main]
// async fn main() -> Result<(), sqlx::Error> {
//     let database_url = std::env::var("DATABASE_URL")
//         .expect("DATABASE_URL must be set");

//     let pool = PgPoolOptions::new()
//         .max_connections(5)
//         .connect(&database_url)
//         .await?;

//     let value: i32 = sqlx::query_scalar("SELECT 1")
//         .fetch_one(&pool)
//         .await?;

//     println!("Database connected! Result: {value}");

//     Ok(())
// }

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Connected to PostgreSQL!");
    let state = AppState { pool };


    

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind port");

    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("Server failed");
}
