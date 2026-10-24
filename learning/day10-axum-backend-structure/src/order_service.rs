use crate::models::{CreateOrderRequest, Order};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

#[derive(Default)]
pub struct OrderService {
    next_id: AtomicU64,
    orders: Mutex<HashMap<u64, Order>>,
}

impl OrderService {
    pub fn create_order(
        &self,
        input: CreateOrderRequest,
    ) -> Result<Order, String> {
        if input.symbol.trim().is_empty() {
            return Err("Symbol cannot be empty".into());
        }

        if input.side != "buy" && input.side != "sell" {
            return Err("Side must be buy or sell".into());
        }

        if !input.quantity.is_finite() || input.quantity <= 0.0 {
            return Err("Quantity must be positive and finite".into());
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;

        let order = Order {
            id,
            symbol: input.symbol,
            side: input.side,
            quantity: input.quantity,
        };

        self.orders
            .lock()
            .expect("Order storage lock poisoned")
            .insert(id, order.clone());

        Ok(order)
    }

    pub fn get_order(&self, id: u64) -> Option<Order> {
        self.orders
            .lock()
            .expect("Order storage lock poisoned")
            .get(&id)
            .cloned()
    }
}