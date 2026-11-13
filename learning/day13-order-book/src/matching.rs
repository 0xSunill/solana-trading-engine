use crate::order_book::OrderBook;
use crate::types::{Fill, MatchResult, OrderKind, OrderRequest, Side, TimeInForce};

pub(crate) fn match_order(book: &mut OrderBook, request: OrderRequest) -> MatchResult {
    let mut remaining = request.quantity;
    let mut fills = Vec::new();

    while remaining > 0 {
        let Some(price) = book.best_opposite_price(request.side) else {
            break;
        };
        if !price_is_eligible(request.side, request.kind, price) {
            break;
        }

        let mut maker_filled = false;
        if let Some(level) = book.opposite_level_mut(request.side, price) {
            if let Some(maker) = level.front_mut() {
                let quantity = maker.remaining.min(remaining);
                maker.remaining -= quantity;
                remaining -= quantity;
                fills.push(Fill {
                    maker_id: maker.id.clone(),
                    taker_id: request.id.clone(),
                    price,
                    quantity,
                });
                maker_filled = maker.remaining == 0;
            }

            if maker_filled {
                level.pop_front();
            }
        }

        book.remove_empty_opposite_level(request.side, price);
    }

    let mut resting_quantity = 0;
    if remaining > 0
        && request.time_in_force == TimeInForce::Gtc
        && let OrderKind::Limit { price } = request.kind
    {
        book.add_resting(&request, price, remaining);
        resting_quantity = remaining;
    }

    MatchResult {
        fills,
        unfilled: remaining,
        resting_quantity,
    }
}

fn price_is_eligible(side: Side, kind: OrderKind, maker_price: u64) -> bool {
    match (side, kind) {
        (_, OrderKind::Market) => true,
        (Side::Bid, OrderKind::Limit { price }) => maker_price <= price,
        (Side::Ask, OrderKind::Limit { price }) => maker_price >= price,
    }
}
