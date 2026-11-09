use crate::fifo::match_fifo;
use crate::maker::{Fill, Maker};
use crate::pro_rata::match_pro_rata;

fn makers_50_100_100() -> Vec<Maker> {
    vec![
        Maker::new("A", 50),
        Maker::new("B", 100),
        Maker::new("C", 100),
    ]
}

fn fill(id: &str, quantity: u64) -> Fill {
    Fill {
        maker_id: id.to_string(),
        quantity,
    }
}

#[test]
fn fifo_fills_oldest_orders_first() {
    let mut makers = makers_50_100_100();
    let result = match_fifo(&mut makers, 200);

    assert_eq!(result.fills, vec![fill("A", 50), fill("B", 100), fill("C", 50)]);
    assert_eq!(result.unfilled, 0);
    assert_eq!(makers[0].remaining, 0);
    assert_eq!(makers[1].remaining, 0);
    assert_eq!(makers[2].remaining, 50);
}

#[test]
fn fifo_reports_unfilled_quantity_when_liquidity_is_insufficient() {
    let mut makers = makers_50_100_100();
    let result = match_fifo(&mut makers, 300);

    assert_eq!(result.total_filled(), 250);
    assert_eq!(result.unfilled, 50);
    assert!(makers.iter().all(|maker| maker.remaining == 0));
}

#[test]
fn fifo_zero_incoming_order_does_not_change_makers() {
    let mut makers = makers_50_100_100();
    let result = match_fifo(&mut makers, 0);

    assert!(result.fills.is_empty());
    assert_eq!(result.unfilled, 0);
    assert_eq!(makers, makers_50_100_100());
}

#[test]
fn pro_rata_allocates_200_as_40_80_80() {
    let mut makers = makers_50_100_100();
    let result = match_pro_rata(&mut makers, 200);

    assert_eq!(result.fills, vec![fill("A", 40), fill("B", 80), fill("C", 80)]);
    assert_eq!(result.unfilled, 0);
    assert_eq!(makers[0].remaining, 10);
    assert_eq!(makers[1].remaining, 20);
    assert_eq!(makers[2].remaining, 20);
}

#[test]
fn pro_rata_recalculates_from_remaining_liquidity_for_second_trade() {
    let mut makers = makers_50_100_100();
    let first = match_pro_rata(&mut makers, 200);
    assert_eq!(first.total_filled(), 200);

    let second = match_pro_rata(&mut makers, 30);
    assert_eq!(second.fills, vec![fill("A", 6), fill("B", 12), fill("C", 12)]);
    assert_eq!(second.unfilled, 0);
    assert_eq!(makers[0].remaining, 4);
    assert_eq!(makers[1].remaining, 8);
    assert_eq!(makers[2].remaining, 8);
}

#[test]
fn pro_rata_largest_remainder_distributes_rounding_lots() {
    let mut makers = vec![
        Maker::new("A", 5),
        Maker::new("B", 10),
        Maker::new("C", 10),
    ];
    let result = match_pro_rata(&mut makers, 2);

    // Ideal shares are 0.4, 0.8 and 0.8. The residual lots go to B and C.
    assert_eq!(result.fills, vec![fill("B", 1), fill("C", 1)]);
    assert_eq!(result.unfilled, 0);
    assert_eq!(makers[0].remaining, 5);
    assert_eq!(makers[1].remaining, 9);
    assert_eq!(makers[2].remaining, 9);
}

#[test]
fn pro_rata_ties_are_broken_by_original_maker_order() {
    let mut makers = vec![
        Maker::new("A", 1),
        Maker::new("B", 1),
        Maker::new("C", 1),
    ];
    let result = match_pro_rata(&mut makers, 1);

    // All ideal shares are 1/3. This simulator gives the tied residual lot to A.
    assert_eq!(result.fills, vec![fill("A", 1)]);
    assert_eq!(result.unfilled, 0);
}

#[test]
fn pro_rata_fills_all_available_liquidity_if_taker_requests_too_much() {
    let mut makers = makers_50_100_100();
    let result = match_pro_rata(&mut makers, 300);

    assert_eq!(result.fills, vec![fill("A", 50), fill("B", 100), fill("C", 100)]);
    assert_eq!(result.total_filled(), 250);
    assert_eq!(result.unfilled, 50);
    assert!(makers.iter().all(|maker| maker.remaining == 0));
}

#[test]
fn pro_rata_zero_incoming_order_does_not_change_makers() {
    let mut makers = makers_50_100_100();
    let result = match_pro_rata(&mut makers, 0);

    assert!(result.fills.is_empty());
    assert_eq!(result.unfilled, 0);
    assert_eq!(makers, makers_50_100_100());
}

#[test]
fn pro_rata_handles_empty_maker_list() {
    let mut makers: Vec<Maker> = Vec::new();
    let result = match_pro_rata(&mut makers, 10);

    assert!(result.fills.is_empty());
    assert_eq!(result.unfilled, 10);
}

#[test]
fn pro_rata_uses_wide_intermediate_arithmetic() {
    let mut makers = vec![
        Maker::new("A", u64::MAX),
        Maker::new("B", u64::MAX),
    ];
    let result = match_pro_rata(&mut makers, u64::MAX);

    assert_eq!(result.total_filled(), u64::MAX);
    assert_eq!(result.unfilled, 0);
    // The tie-break gives the extra lot to A.
    assert_eq!(makers[0].remaining, u64::MAX / 2);
    assert_eq!(makers[1].remaining, u64::MAX / 2 + 1);
}

#[test]
fn pro_rata_ignores_zero_liquidity_makers() {
    let mut makers = vec![
        Maker::new("A", 0),
        Maker::new("B", 100),
        Maker::new("C", 100),
    ];
    let result = match_pro_rata(&mut makers, 50);

    assert_eq!(result.fills, vec![fill("B", 25), fill("C", 25)]);
    assert_eq!(result.unfilled, 0);
    assert_eq!(makers[0].remaining, 0);
}
