use crate::maker::{Fill, Maker, MatchResult};

/// Matches an incoming order using FIFO (first-in, first-out) priority.
///
/// The maker slice must already be ordered from oldest to newest at the same
/// price level. Better price levels must be processed before calling this
/// function; this function handles only one price level.
pub fn match_fifo(makers: &mut [Maker], incoming: u64) -> MatchResult {
    let mut remaining_order = incoming;
    let mut fills = Vec::new();

    for maker in makers.iter_mut() {
        if remaining_order == 0 {
            break;
        }

        if maker.remaining == 0 {
            continue;
        }

        // A maker cannot fill more than either its remaining quantity or
        // the quantity still requested by the taker.
        let fill_quantity = maker.remaining.min(remaining_order);

        maker.remaining -= fill_quantity;
        remaining_order -= fill_quantity;

        if fill_quantity > 0 {
            fills.push(Fill {
                maker_id: maker.id.clone(),
                quantity: fill_quantity,
            });
        }
    }

    MatchResult {
        fills,
        unfilled: remaining_order,
    }
}
