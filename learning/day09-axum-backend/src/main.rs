
// use axum::{
//     extract::{Path, Query},
//     routing::get,
//     Json, Router,
// };
// use serde::{Deserialize, Serialize};

// #[derive(Serialize)]
// struct Order {
//     id: u64,
//     symbol: String,
//     side: String,
//     quantity: f64,
// }

// #[derive(Deserialize)]
// struct CreateOrder {
//     symbol: String,
//     side: String,
//     quantity: f64,
// }

// #[derive(Deserialize)]
// struct OrderFilters {
//     symbol: Option<String>,
// }

// async fn get_order(Path(id): Path<u64>) -> Json<Order> {
//     Json(Order {
//         id,
//         symbol: "SOL".to_string(),
//         side: "buy".to_string(),
//         quantity: 1.5,
//     })
// }

// async fn list_orders(
//     Query(filters): Query<OrderFilters>,
// ) -> String {
//     match filters.symbol {
//         Some(symbol) => format!("Filtering orders for {symbol}"),
//         None => "Returning all orders".to_string(),
//     }
// }

// async fn create_order(
//     Json(payload): Json<CreateOrder>,
// ) -> Json<Order> {
//     Json(Order {
//         id: 1001,
//         symbol: payload.symbol,
//         side: payload.side,
//         quantity: payload.quantity,
//     })
// }

// #[tokio::main]
// async fn main() {
//     let app = Router::new()
//         .route("/orders", get(list_orders).post(create_order))
//         .route("/orders/{id}", get(get_order));

//     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//         .await
//         .unwrap();

//     println!("Server running on port 3000");

//     axum::serve(listener, app).await.unwrap();
// }








use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    quantity: f64,
}

#[derive(Deserialize)]
struct CreateOrder {
    symbol: String,
    side: String,
    quantity: f64,
}

async fn get_order(
    Path(id): Path<u64>,
) -> Result<Json<Order>, (StatusCode, String)> {
    if id == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Order ID must be greater than zero".to_string(),
        ));
    }

    if id != 123 {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Order {id} was not found"),
        ));
    }

    Ok(Json(Order {
        id: 123,
        symbol: "SOL".to_string(),
        side: "buy".to_string(),
        quantity: 1.5,
    }))
}

async fn create_order(
    Json(payload): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
    if payload.quantity <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Quantity must be greater than zero".to_string(),
        ));
    }

    if payload.side != "buy" && payload.side != "sell" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Side must be buy or sell".to_string(),
        ));
    }

    let order = Order {
        id: 1001,
        symbol: payload.symbol,
        side: payload.side,
        quantity: payload.quantity,
    };

    Ok((StatusCode::CREATED, Json(order)))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/orders/{id}", get(get_order))
        .route("/orders", axum::routing::post(create_order));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}










