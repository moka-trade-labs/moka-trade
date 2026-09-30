//! Fixed-point scales (spec CAP-07). Every integer quantity names its scale.

/// Prices are integers scaled by 1e6 (`mark_e6` in the wrapper).
pub const PRICE_SCALE_E6: u64 = 1_000_000;
/// Smallest mark the wrapper accepts (`mark_e6 ∈ [1, 1e12]`).
pub const MARK_E6_MIN: u64 = 1;
/// Largest mark the wrapper accepts.
pub const MARK_E6_MAX: u64 = 1_000_000_000_000;
/// USDC has 6 decimals.
pub const USDC_DECIMALS: u8 = 6;
/// Basis points per unit.
pub const BPS_DENOMINATOR: u64 = 10_000;

/// A mark price in 1e-6 USD per contract, guaranteed to be in the wrapper's
/// accepted range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MarkE6(u64);

impl MarkE6 {
    /// Returns `None` if `raw` is outside `[MARK_E6_MIN, MARK_E6_MAX]`.
    pub const fn new(raw: u64) -> Option<Self> {
        if raw >= MARK_E6_MIN && raw <= MARK_E6_MAX {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_range_is_inclusive() {
        assert_eq!(MarkE6::new(0), None);
        assert_eq!(MarkE6::new(1).map(MarkE6::get), Some(1));
        assert_eq!(MarkE6::new(MARK_E6_MAX).map(MarkE6::get), Some(MARK_E6_MAX));
        assert_eq!(MarkE6::new(MARK_E6_MAX + 1), None);
    }
}
