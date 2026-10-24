use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
}