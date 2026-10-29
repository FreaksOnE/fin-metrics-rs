use rust_decimal::{Decimal, MathematicalOps};

use crate::calc_total_return;

/// Martin ratio (a.k.a. Ulcer Performance Index): total return per unit of
/// [Ulcer Index](https://www.tangotools.com/ui/ui.htm).
///
/// Matches the reference formula `UPI = (Total return - Risk-free return) /
/// UI` with the risk-free return assumed to be zero and the total return left
/// unannualized (the raw compounded return of the series), so results are
/// directly comparable only across series of equal length.
///
/// A positive-return series without drawdowns has an unbounded Martin ratio
/// and saturates at [`Decimal::MAX`]; a series with neither return nor
/// drawdowns is undefined and yields zero.
pub fn calc_martin_ratio(returns: &[Decimal]) -> Decimal {
    let total_return = calc_total_return(returns);
    let ulcer_index = calc_ulcer_index(returns);
    if ulcer_index.is_zero() {
        if total_return.is_zero() {
            return Decimal::ZERO;
        }
        return Decimal::MAX;
    }
    total_return / ulcer_index
}

/// Ulcer Index after Peter G. Martin (<https://www.tangotools.com/ui/ui.htm>):
/// the square root of the mean of the squared drawdowns of the equity curve
/// rebuilt from `returns` (equity compounds as `1 + r` per period, starting
/// at 1), each drawdown measured against the running peak.
///
/// Drawdowns are kept in the same fractional units as the inputs, so the
/// result is directly comparable with [`calc_total_return`]; multiply by 100
/// to get the "%" figure reported on the referenced page. An empty input or a
/// series without drawdowns returns zero.
pub fn calc_ulcer_index(returns: &[Decimal]) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }

    let mut equity = Decimal::ONE;
    let mut peak = equity;
    let mut sum_of_squares = Decimal::ZERO;
    for r in returns {
        equity *= Decimal::ONE + *r;
        if equity > peak {
            peak = equity;
        } else {
            let drawdown = equity / peak - Decimal::ONE;
            sum_of_squares += drawdown * drawdown;
        }
    }

    (sum_of_squares / Decimal::from(returns.len() as u64))
        .sqrt()
        .expect("mean of squared drawdowns is non-negative")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a fractional return, e.g. `pct(10)` = +10% = `0.10`.
    fn pct(whole: i64) -> Decimal {
        Decimal::new(whole, 2)
    }

    #[test]
    fn empty_input_has_no_drawdown() {
        assert_eq!(calc_ulcer_index(&[]), Decimal::ZERO);
    }

    #[test]
    fn drawdown_free_series_yields_zero() {
        let rising = [pct(5), pct(2), pct(3)];
        assert_eq!(calc_ulcer_index(&rising), Decimal::ZERO);
    }

    #[test]
    fn matches_reference_formula() {
        // equity: 1 -> 2 -> 1, i.e. a single -50% drawdown from the 2.0 peak
        // UI = sqrt(0.5^2 / 2) = sqrt(0.125) = 0.3535533905932737622...
        let returns = [Decimal::ONE, Decimal::new(-5, 1)];
        let expected = Decimal::new(3535533905932738, 16);
        let actual = calc_ulcer_index(&returns);
        assert!((actual - expected).abs() < Decimal::new(1, 12));
    }

    #[test]
    fn longer_drawdowns_raise_ui() {
        // Same -10% drawdown depth, held for one vs three periods
        let short = [pct(10), pct(-10)];
        let long = [pct(10), pct(-10), pct(0), pct(0)];
        assert!(calc_ulcer_index(&long) > calc_ulcer_index(&short));
    }

    #[test]
    fn deeper_drawdowns_raise_ui() {
        let shallow = [pct(10), pct(-5)];
        let deep = [pct(10), pct(-10)];
        assert!(calc_ulcer_index(&deep) > calc_ulcer_index(&shallow));
    }

    #[test]
    fn martin_ratio_is_total_return_over_ui() {
        // equity: 1 -> 2 -> 1.5; compounded total return 0.5
        // UI = sqrt(0.25^2 / 2) = sqrt(0.03125)
        // martin = 0.5 / sqrt(0.03125) = 2 * sqrt(2) = 2.8284271247461901...
        let returns = [Decimal::ONE, Decimal::new(-25, 2)];
        let expected = Decimal::new(28284271247461901, 16);
        let actual = calc_martin_ratio(&returns);
        assert!((actual - expected).abs() < Decimal::new(1, 12));
    }

    #[test]
    fn round_trip_has_zero_martin_ratio() {
        // equity: 1 -> 2 -> 1; no net growth despite the drawdown
        let returns = [Decimal::ONE, Decimal::new(-5, 1)];
        assert_eq!(calc_martin_ratio(&returns), Decimal::ZERO);
    }

    #[test]
    fn rising_series_without_drawdown_saturates_at_max() {
        let rising = [pct(5), pct(10)];
        assert_eq!(calc_martin_ratio(&rising), Decimal::MAX);
    }

    #[test]
    fn flat_series_is_undefined_and_yields_zero() {
        let flat = [Decimal::ZERO; 3];
        assert_eq!(calc_martin_ratio(&flat), Decimal::ZERO);
    }
}
