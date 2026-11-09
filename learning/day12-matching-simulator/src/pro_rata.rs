use crate::maker::{Fill, Maker, MatchResult};

/// Matches an incoming order using pro-rata allocation at one price level.
///
/// Each maker receives a share proportional to its available quantity:
///
///     ideal_i = target_quantity * maker_quantity_i / total_liquidity
///
/// This implementation uses integer arithmetic and the largest-remainder
/// method to distribute leftover lots. If two makers have equal remainders,
/// the earlier maker in the input slice wins the tie. That tie-break rule is
/// for this educational simulator; it is not a claim about Archer's exact
/// on-chain remainder-allocation policy.
///
/// The maker slice represents all eligible makers at the same price level.
/// Better price levels must be processed before calling this function.
pub fn match_pro_rata(makers: &mut [Maker], incoming: u64) -> MatchResult {
    // Use u128 for the sum because many u64 maker quantities can add up to
    // more than u64::MAX.
    let total_liquidity: u128 = makers
        .iter()
        .map(|maker| u128::from(maker.remaining))
        .sum();

    if incoming == 0 || total_liquidity == 0 {
        return MatchResult {
            fills: Vec::new(),
            unfilled: incoming,
        };
    }

    // Never allocate more than the taker requested or the makers supplied.
    // Since incoming is u64, the target always fits in u64.
    let target = u128::from(incoming).min(total_liquidity) as u64;

    // Calculate every allocation against the same pre-trade snapshot.
    let mut allocations = vec![0_u64; makers.len()];
    let mut remainders = vec![0_u128; makers.len()];
    let mut allocated_total = 0_u64;

    for (index, maker) in makers.iter().enumerate() {
        if maker.remaining == 0 {
            continue;
        }

        // The product of two u64 values fits in u128.
        let numerator = u128::from(target) * u128::from(maker.remaining);
        let base_allocation = (numerator / total_liquidity) as u64;
        let remainder = numerator % total_liquidity;

        allocations[index] = base_allocation;
        remainders[index] = remainder;
        allocated_total += base_allocation;
    }

    // Lots not allocated by the floor calculations.
    let leftover = target - allocated_total;

    if leftover > 0 {
        // Largest-remainder method: highest fractional remainder first.
        // Equal remainders are resolved by original input order.
        let mut candidate_indices: Vec<usize> = (0..makers.len())
            .filter(|&index| {
                remainders[index] > 0
                    && allocations[index] < makers[index].remaining
            })
            .collect();

        candidate_indices.sort_by(|&left, &right| {
            remainders[right]
                .cmp(&remainders[left])
                .then_with(|| left.cmp(&right))
        });

        // In largest-remainder allocation, each selected maker receives at
        // most one residual lot.
        for &index in candidate_indices.iter().take(leftover as usize) {
            allocations[index] += 1;
        }
    }

    // Apply allocations in original maker order for deterministic output.
    let mut fills = Vec::new();

    for (index, maker) in makers.iter_mut().enumerate() {
        let fill_quantity = allocations[index];
        debug_assert!(fill_quantity <= maker.remaining);
        maker.remaining -= fill_quantity;

        if fill_quantity > 0 {
            fills.push(Fill {
                maker_id: maker.id.clone(),
                quantity: fill_quantity,
            });
        }
    }

    let filled_total: u64 = fills.iter().map(|fill| fill.quantity).sum();
    let unfilled = incoming - filled_total;

    MatchResult { fills, unfilled }
}
