# fin-metrics-rs

Financial performance metrics computed over a series of period returns, using
exact [`rust_decimal::Decimal`](https://crates.io/crates/rust_decimal)
arithmetic — no floating point.

Every metric takes a slice of fractional period returns (`0.10` means +10%) and
returns a `Decimal` in the same fractional units.

## Metrics

| Function             | Description                                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------------- |
| `calc_total_return`  | Compounded total return of the series                                                                         |
| `calc_profit_factor` | Log-scaled ratio of gross wins to gross losses                                                                |
| `calc_martin_ratio`  | Total return per unit of [Ulcer Index](https://www.tangotools.com/ui/ui.htm) (a.k.a. Ulcer Performance Index) |
| `calc_ulcer_index`   | Ulcer Index: RMS drawdown of the compounded equity curve                                                      |

## Installation

```toml
[dependencies]
fin-metrics-rs = "0.1.0"
rust_decimal = "1.43.0"
```

## Usage

### `calc_total_return`

Compounds the period returns (`1 + r` per period) and subtracts 1. The result
is floored at `-1` (total loss); an empty series yields `0`.

```rust
use fin_metrics_rs::calc_total_return;
use rust_decimal::Decimal;

/// Builds a fractional return, e.g. `pct(10)` = +10% = `0.10`.
fn pct(whole: i64) -> Decimal {
    Decimal::new(whole, 2)
}

fn main() {
    let returns = [pct(10), pct(20)];

    // 1.10 * 1.20 - 1 = 0.32 (compounded — not the 0.30 a naive sum gives)
    let total_return = calc_total_return(&returns);
    println!("total return: {total_return}"); // 0.32
}
```

### `calc_profit_factor`

Natural log of gross wins divided by gross losses (losses accumulated as a
positive magnitude). Zero returns are ignored. Unbounded results saturate: a
series without losses yields `Decimal::MAX`, one without wins yields
`Decimal::MIN`, and an empty or all-zero series is undefined and yields `0`.

```rust
use fin_metrics_rs::calc_profit_factor;
use rust_decimal::Decimal;

fn pct(whole: i64) -> Decimal {
    Decimal::new(whole, 2)
}

fn main() {
    let returns = [pct(10), pct(-5), pct(5), Decimal::ZERO];

    // wins 0.15, losses 0.05 -> ln(0.15 / 0.05) = ln(3) ≈ 1.09861228866811
    let profit_factor = calc_profit_factor(&returns);
    println!("profit factor: {profit_factor}");
}
```

### `calc_martin_ratio`

Total return per unit of drawdown risk: `calc_total_return(returns) /
calc_ulcer_index(returns)`. Higher is better — more return for the same depth
and duration of drawdowns. A positive-return series without drawdowns is
unbounded and saturates at `Decimal::MAX`; a series with neither return nor
drawdowns is undefined and yields `0`.

This follows the [spec](https://www.tangotools.com/ui/ui.htm)'s
`UPI = (Total return - Risk-free return) / UI` with the risk-free return
assumed to be zero and the total return left unannualized (the raw compounded
return of the series) — compare only series of equal length.

```rust
use fin_metrics_rs::calc_martin_ratio;
use rust_decimal::Decimal;

fn pct(whole: i64) -> Decimal {
    Decimal::new(whole, 2)
}

fn main() {
    // equity: 1 -> 2 -> 1.5
    let returns = [pct(100), pct(-25)];

    // total return 0.5, ulcer index sqrt(0.03125)
    // -> 0.5 / sqrt(0.03125) = 2 * sqrt(2) ≈ 2.82842712474619
    let martin_ratio = calc_martin_ratio(&returns);
    println!("martin ratio: {martin_ratio}");
}
```

The underlying `calc_ulcer_index` is also exported, if you need the drawdown
measure on its own. It follows Martin's algorithm (running peak, squared
percentage drawdowns, root of the mean) with the running peak seeded at the
initial equity of 1, so a loss in the first period counts as a drawdown:

```rust
use fin_metrics_rs::calc_ulcer_index;
use rust_decimal::Decimal;

fn main() {
    // equity: 1 -> 2 -> 1, a single -50% drawdown from the 2.0 peak
    let returns = [Decimal::ONE, Decimal::new(-5, 1)];

    // sqrt(0.5^2 / 2) = sqrt(0.125) ≈ 0.3535533905932738
    let ulcer_index = calc_ulcer_index(&returns);
    println!("ulcer index: {ulcer_index}");
}
```

## License

MIT
