/// The side of an order in the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

/// Whether an incoming order has a price limit or can trade at any price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderKind {
    Limit { price: u64 },
    Market,
}

/// What to do with the part of an order that does not immediately trade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    /// Keep any unfilled limit quantity in the book.
    Gtc,
    /// Cancel any unfilled quantity after attempting to trade.
    Ioc,
}

/// An incoming order submitted to an [`OrderBook`](crate::OrderBook).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderRequest {
    pub id: String,
    pub side: Side,
    pub kind: OrderKind,
    pub quantity: u64,
    pub time_in_force: TimeInForce,
}

impl OrderRequest {
    pub fn limit(id: impl Into<String>, side: Side, price: u64, quantity: u64) -> Self {
        Self {
            id: id.into(),
            side,
            kind: OrderKind::Limit { price },
            quantity,
            time_in_force: TimeInForce::Gtc,
        }
    }

    pub fn market(id: impl Into<String>, side: Side, quantity: u64) -> Self {
        Self {
            id: id.into(),
            side,
            kind: OrderKind::Market,
            quantity,
            time_in_force: TimeInForce::Ioc,
        }
    }
}

/// A trade created while matching an incoming order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub maker_id: String,
    pub taker_id: String,
    pub price: u64,
    pub quantity: u64,
}

/// The outcome of submitting one order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub fills: Vec<Fill>,
    /// Quantity that did not trade. For a GTC limit order, this may be resting.
    pub unfilled: u64,
    /// Quantity added to the book after matching.
    pub resting_quantity: u64,
}

impl MatchResult {
    pub fn total_filled(&self) -> u64 {
        self.fills.iter().map(|fill| fill.quantity).sum()
    }
}

/// One order currently resting at a price level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestingOrder {
    pub id: String,
    pub side: Side,
    pub price: u64,
    pub remaining: u64,
}

/// Errors returned when an order cannot be accepted by the book.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderBookError {
    ZeroQuantity,
    EmptyOrderId,
    DuplicateOrderId(String),
    InvalidPrice { price: u64, tick_size: u64 },
}

impl std::fmt::Display for OrderBookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroQuantity => write!(f, "order quantity must be greater than zero"),
            Self::EmptyOrderId => write!(f, "order id must not be empty"),
            Self::DuplicateOrderId(id) => write!(f, "order id {id} is already resting"),
            Self::InvalidPrice { price, tick_size } => write!(
                f,
                "price {price} must be positive and divisible by tick size {tick_size}"
            ),
        }
    }
}

impl std::error::Error for OrderBookError {}
