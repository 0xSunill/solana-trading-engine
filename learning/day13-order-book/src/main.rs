use day13_order_book::{OrderBook, OrderKind, OrderRequest, Side, TimeInForce};

fn main() {
    // Prices are integer ticks, so the book does not use floating-point values.
    let mut book = OrderBook::new(1);

    // Two makers share the best price. FIFO fills ask-1 before ask-2, then
    // matching continues to the next price level (102).
    for (id, price, quantity) in [
        ("ask-1", 101, 3),
        ("ask-2", 101, 4),
        ("ask-3", 102, 8),
        ("ask-4", 104, 6),
    ] {
        book.submit(OrderRequest::limit(id, Side::Ask, price, quantity))
            .unwrap();
    }

    let result = book
        .submit(OrderRequest {
            id: "taker-1".into(),
            side: Side::Bid,
            kind: OrderKind::Limit { price: 102 },
            quantity: 9,
            time_in_force: TimeInForce::Gtc,
        })
        .unwrap();

    println!("fills:");
    for fill in &result.fills {
        println!(
            "  {} bought {} lots from {} at price {}",
            fill.taker_id, fill.quantity, fill.maker_id, fill.price
        );
    }
    println!("total filled: {} lots", result.total_filled());
    println!("unfilled: {} lots", result.unfilled);
    println!("resting quantity: {} lots", result.resting_quantity);
    println!("book snapshot (best price first):");
    for order in book.snapshot() {
        println!(
            "  {:?} {} lots at {} (order {})",
            order.side, order.remaining, order.price, order.id
        );
    }
}
