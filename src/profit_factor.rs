use rust_decimal::{Decimal, MathematicalOps};

/// Log-scaled profit factor: `ln(sum of winning returns / sum of losing
/// returns)`, with losses accumulated as a positive magnitude.
///
/// Unbounded results saturate: a series without losses yields
/// [`Decimal::MAX`] and a series without wins yields [`Decimal::MIN`]; an
/// empty or all-zero series is undefined and yields zero.
pub fn calc_profit_factor(returns: &[Decimal]) -> Decimal {
    let gross_win: Decimal =
        returns.iter().filter(|r| **r > Decimal::ZERO).sum();
    let gross_loss: Decimal =
        returns.iter().filter(|r| **r < Decimal::ZERO).sum::<Decimal>().abs();

    if gross_win.is_zero() && gross_loss.is_zero() {
        return Decimal::ZERO;
    }
    if gross_loss.is_zero() {
        return Decimal::MAX;
    }
    if gross_win.is_zero() {
        return Decimal::MIN;
    }
    (gross_win / gross_loss).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a fractional return, e.g. `pct(10)` = +10% = `0.10`.
    fn pct(whole: i64) -> Decimal {
        Decimal::new(whole, 2)
    }

    #[test]
    fn empty_input_is_undefined_and_yields_zero() {
        assert_eq!(calc_profit_factor(&[]), Decimal::ZERO);
    }

    #[test]
    fn all_zero_returns_are_undefined_and_yield_zero() {
        let flat = [Decimal::ZERO; 3];
        assert_eq!(calc_profit_factor(&flat), Decimal::ZERO);
    }

    #[test]
    fn series_without_losses_saturates_at_max() {
        let wins = [pct(5), pct(3)];
        assert_eq!(calc_profit_factor(&wins), Decimal::MAX);
    }

    #[test]
    fn series_without_wins_saturates_at_min() {
        let losses = [pct(-5), pct(-3)];
        assert_eq!(calc_profit_factor(&losses), Decimal::MIN);
    }

    #[test]
    fn matches_reference_formula() {
        // wins: 0.10 + 0.05 = 0.15; losses: |-0.05| = 0.05
        // pf = ln(0.15 / 0.05) = ln(3) = 1.09861228866811...
        let returns = [pct(10), pct(-5), pct(5), Decimal::ZERO];
        let expected = Decimal::new(109861228866811, 14);
        let actual = calc_profit_factor(&returns);
        assert!((actual - expected).abs() < Decimal::new(1, 12));
    }

    #[test]
    fn equal_wins_and_losses_yield_zero() {
        // pf = ln(1) = 0
        let returns = [pct(10), pct(-10)];
        assert_eq!(calc_profit_factor(&returns), Decimal::ZERO);
    }

    #[test]
    fn zero_returns_are_ignored() {
        let with_zero = [pct(10), Decimal::ZERO, pct(-10)];
        let without_zero = [pct(10), pct(-10)];
        assert_eq!(
            calc_profit_factor(&with_zero),
            calc_profit_factor(&without_zero)
        );
    }
}
