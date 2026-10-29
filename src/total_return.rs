use std::cmp::max;

use rust_decimal::Decimal;

/// Compounded total return of a series of period returns: the growth of an
/// equity curve that compounds as `1 + r` per period from 1.0 to minus 1
///
/// The result is floored at `-1` (total loss)
pub fn calc_total_return(returns: &[Decimal]) -> Decimal {
    max(
        returns.iter().map(|r| Decimal::ONE + *r).product::<Decimal>()
            - Decimal::ONE,
        Decimal::NEGATIVE_ONE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a fractional return, e.g. `pct(10)` = +10% = `0.10`.
    fn pct(whole: i64) -> Decimal {
        Decimal::new(whole, 2)
    }

    #[test]
    fn empty_input_has_no_return() {
        assert_eq!(calc_total_return(&[]), Decimal::ZERO);
    }

    #[test]
    fn compounds_period_returns() {
        // 1.10 * 1.20 - 1 = 0.32 (not the 0.30 a naive sum would give)
        let returns = [pct(10), pct(20)];
        assert_eq!(calc_total_return(&returns), Decimal::new(32, 2));
    }

    #[test]
    fn losing_series_has_negative_return() {
        // 0.90 * 0.80 - 1 = -0.28
        let returns = [pct(-10), pct(-20)];
        assert_eq!(calc_total_return(&returns), Decimal::new(-28, 2));
    }

    #[test]
    fn round_trip_yields_zero() {
        let returns = [pct(100), pct(-50)];
        assert_eq!(calc_total_return(&returns), Decimal::ZERO);
    }

    #[test]
    fn total_loss_floors_at_negative_one() {
        let gap_down = [pct(-150)]; // -150%
        assert_eq!(calc_total_return(&gap_down), Decimal::NEGATIVE_ONE);
    }
}
