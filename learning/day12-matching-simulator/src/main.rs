use matching_engine::fifo::match_fifo;
use matching_engine::maker::Maker;
use matching_engine::pro_rata::match_pro_rata;

fn print_result(label: &str, makers: &[Maker], result: &matching_engine::maker::MatchResult) {
    println!("\n{label}");
    println!("Fills:");

    if result.fills.is_empty() {
        println!("  (no fills)");
    } else {
        for fill in &result.fills {
            println!("  Maker {}: {} lots", fill.maker_id, fill.quantity);
        }
    }

    println!("Total filled: {} lots", result.total_filled());
    println!("Unfilled: {} lots", result.unfilled);
    println!("Remaining maker liquidity:");

    for maker in makers {
        println!("  Maker {}: {} lots", maker.id, maker.remaining);
    }
}

fn main() {
    let original_makers = vec![
        Maker::new("A", 50),
        Maker::new("B", 100),
        Maker::new("C", 100),
    ];

    // FIFO example: maker order represents arrival order (oldest first).
    let mut fifo_makers = original_makers.clone();
    let fifo_result = match_fifo(&mut fifo_makers, 200);
    print_result("FIFO: incoming order = 200 lots", &fifo_makers, &fifo_result);

    // Pro-rata example: all makers are at the same price level.
    let mut pro_rata_makers = original_makers.clone();
    let pro_rata_result = match_pro_rata(&mut pro_rata_makers, 200);
    print_result(
        "Pro-rata: incoming order = 200 lots",
        &pro_rata_makers,
        &pro_rata_result,
    );

    // A second taker trades against the liquidity remaining after the first fill.
    let second_result = match_pro_rata(&mut pro_rata_makers, 30);
    print_result(
        "Pro-rata: second incoming order = 30 lots",
        &pro_rata_makers,
        &second_result,
    );
}
