use std::collections::{BTreeMap, VecDeque};

use crate::matching::match_order;
use crate::types::{MatchResult, OrderBookError, OrderRequest, RestingOrder, Side};

#[derive(Debug, Clone)]
pub(crate) struct BookOrder {
    pub id: String,
    pub remaining: u64,
}

/// An order book with independent FIFO queues at every price level.
#[derive(Debug, Clone)]
pub struct OrderBook {
    tick_size: u64,
    bids: BTreeMap<u64, VecDeque<BookOrder>>,
    asks: BTreeMap<u64, VecDeque<BookOrder>>,
}

impl OrderBook {
    /// Creates an empty book. Prices are integer units and must be multiples
    /// of `tick_size`; a zero tick size is treated as one.
    pub fn new(tick_size: u64) -> Self {
        Self {
            tick_size: tick_size.max(1),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Matches an incoming order against the best eligible price levels.
    /// Better prices always trade first; orders at the same price trade FIFO.
    pub fn submit(&mut self, request: OrderRequest) -> Result<MatchResult, OrderBookError> {
        self.validate(&request)?;
        Ok(match_order(self, request))
    }

    /// Returns all resting orders, sorted best price first and FIFO within
    /// each level. Bids are ordered highest to lowest, then asks lowest to highest.
    pub fn snapshot(&self) -> Vec<RestingOrder> {
        let mut orders = Vec::new();
        Self::append_levels(&mut orders, &self.bids, Side::Bid, true);
        Self::append_levels(&mut orders, &self.asks, Side::Ask, false);
        orders
    }

    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }

    fn validate(&self, request: &OrderRequest) -> Result<(), OrderBookError> {
        if request.id.trim().is_empty() {
            return Err(OrderBookError::EmptyOrderId);
        }
        if request.quantity == 0 {
            return Err(OrderBookError::ZeroQuantity);
        }
        if let crate::types::OrderKind::Limit { price } = request.kind {
            if price == 0 || price % self.tick_size != 0 {
                return Err(OrderBookError::InvalidPrice {
                    price,
                    tick_size: self.tick_size,
                });
            }
        }
        if self.contains_order_id(&request.id) {
            return Err(OrderBookError::DuplicateOrderId(request.id.clone()));
        }
        Ok(())
    }

    fn contains_order_id(&self, id: &str) -> bool {
        self.bids
            .values()
            .chain(self.asks.values())
            .any(|level| level.iter().any(|order| order.id == id))
    }

    fn append_levels(
        output: &mut Vec<RestingOrder>,
        levels: &BTreeMap<u64, VecDeque<BookOrder>>,
        side: Side,
        descending: bool,
    ) {
        let prices: Box<dyn Iterator<Item = (&u64, &VecDeque<BookOrder>)>> = if descending {
            Box::new(levels.iter().rev())
        } else {
            Box::new(levels.iter())
        };

        for (&price, queue) in prices {
            for order in queue {
                output.push(RestingOrder {
                    id: order.id.clone(),
                    side,
                    price,
                    remaining: order.remaining,
                });
            }
        }
    }

    pub(crate) fn best_opposite_price(&self, incoming_side: Side) -> Option<u64> {
        match incoming_side {
            Side::Bid => self.asks.keys().next().copied(),
            Side::Ask => self.bids.keys().next_back().copied(),
        }
    }

    pub(crate) fn opposite_level_mut(
        &mut self,
        incoming_side: Side,
        price: u64,
    ) -> Option<&mut VecDeque<BookOrder>> {
        match incoming_side {
            Side::Bid => self.asks.get_mut(&price),
            Side::Ask => self.bids.get_mut(&price),
        }
    }

    pub(crate) fn remove_empty_opposite_level(&mut self, incoming_side: Side, price: u64) {
        match incoming_side {
            Side::Bid => {
                if self.asks.get(&price).is_some_and(VecDeque::is_empty) {
                    self.asks.remove(&price);
                }
            }
            Side::Ask => {
                if self.bids.get(&price).is_some_and(VecDeque::is_empty) {
                    self.bids.remove(&price);
                }
            }
        }
    }

    pub(crate) fn add_resting(&mut self, request: &OrderRequest, price: u64, quantity: u64) {
        let levels = match request.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };
        levels.entry(price).or_default().push_back(BookOrder {
            id: request.id.clone(),
            remaining: quantity,
        });
    }
}
