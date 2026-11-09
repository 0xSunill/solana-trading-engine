/// A maker's remaining quantity at one price level.
///
/// `remaining` is expressed in integer lots, not floating-point SOL.
/// For example, if one lot represents 0.01 SOL, then 100 SOL is 10,000 lots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Maker {
    pub id: String,
    pub remaining: u64,
}

impl Maker {
    pub fn new(id: impl Into<String>, remaining: u64) -> Self {
        Self {
            id: id.into(),
            remaining,
        }
    }
}

/// One maker's executed portion of an incoming order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub maker_id: String,
    pub quantity: u64,
}

/// Result of matching one incoming order at a single price level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub fills: Vec<Fill>,
    /// Quantity that could not be filled at this price level.
    pub unfilled: u64,
}

impl MatchResult {
    /// Returns the total quantity filled by this matching operation.
    pub fn total_filled(&self) -> u64 {
        self.fills.iter().map(|fill| fill.quantity).sum()
    }
}
