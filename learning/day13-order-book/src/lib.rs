//! A small price-time-priority order book built on integer price ticks.

mod matching;
mod order_book;
mod types;

pub use order_book::OrderBook;
pub use types::{
    Fill, MatchResult, OrderBookError, OrderKind, OrderRequest, RestingOrder, Side, TimeInForce,
};
