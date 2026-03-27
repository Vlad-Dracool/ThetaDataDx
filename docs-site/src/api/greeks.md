# Greeks

Full Black-Scholes calculator ported from ThetaData's Java implementation. Computes 22 Greeks plus implied volatility.

For a tutorial-style guide, see [Greeks Calculator Guide](../guides/greeks-calculator.md).

## all_greeks

Compute IV from market price, then derive all 22 Greeks.

```rust
pub fn all_greeks(
    s: f64, x: f64, r: f64, q: f64, t: f64,
    option_price: f64, is_call: bool,
) -> GreeksResult
```

## implied_volatility

Bisection solver with up to 128 iterations.

```rust
pub fn implied_volatility(
    s: f64, x: f64, r: f64, q: f64, t: f64,
    option_price: f64, is_call: bool,
) -> (f64, f64)  // (iv, error)
```

Returns `(iv, error)` where error is the relative difference `(theoretical - market) / market`.

## Individual Greeks

All functions take: `s` (spot), `x` (strike), `v` (volatility), `r` (rate), `q` (dividend yield), `t` (time in years).

### First Order

| Function | Signature | Description |
|----------|-----------|-------------|
| `value` | `(s, x, v, r, q, t, is_call) -> f64` | Option theoretical value |
| `delta` | `(s, x, v, r, q, t, is_call) -> f64` | dV/dS |
| `theta` | `(s, x, v, r, q, t, is_call) -> f64` | dV/dt (daily, /365) |
| `vega` | `(s, x, v, r, q, t) -> f64` | dV/dv |
| `rho` | `(s, x, v, r, q, t, is_call) -> f64` | dV/dr |
| `epsilon` | `(s, x, v, r, q, t, is_call) -> f64` | dV/dq |
| `lambda` | `(s, x, v, r, q, t, is_call) -> f64` | Leverage ratio |

### Second Order

| Function | Signature | Description |
|----------|-----------|-------------|
| `gamma` | `(s, x, v, r, q, t) -> f64` | d2V/dS2 |
| `vanna` | `(s, x, v, r, q, t) -> f64` | d2V/dSdv |
| `charm` | `(s, x, v, r, q, t, is_call) -> f64` | d2V/dSdt |
| `vomma` | `(s, x, v, r, q, t) -> f64` | d2V/dv2 |
| `veta` | `(s, x, v, r, q, t) -> f64` | d2V/dvdt |

### Third Order

| Function | Signature | Description |
|----------|-----------|-------------|
| `speed` | `(s, x, v, r, q, t) -> f64` | d3V/dS3 |
| `zomma` | `(s, x, v, r, q, t) -> f64` | d3V/dS2dv |
| `color` | `(s, x, v, r, q, t) -> f64` | d3V/dS2dt |
| `ultima` | `(s, x, v, r, q, t) -> f64` | d3V/dv3 (clamped [-100, 100]) |

### Auxiliary

| Function | Signature | Description |
|----------|-----------|-------------|
| `dual_delta` | `(s, x, v, r, q, t, is_call) -> f64` | dV/dK |
| `dual_gamma` | `(s, x, v, r, q, t) -> f64` | d2V/dK2 |
| `d1` | `(s, x, v, r, q, t) -> f64` | Black-Scholes d1 |
| `d2` | `(s, x, v, r, q, t) -> f64` | Black-Scholes d2 |

## GreeksResult

```rust
pub struct GreeksResult {
    pub value: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub iv: f64,
    pub iv_error: f64,
    pub vanna: f64,
    pub charm: f64,
    pub vomma: f64,
    pub veta: f64,
    pub speed: f64,
    pub zomma: f64,
    pub color: f64,
    pub ultima: f64,
    pub d1: f64,
    pub d2: f64,
    pub dual_delta: f64,
    pub dual_gamma: f64,
    pub epsilon: f64,
    pub lambda: f64,
}
```

## Degenerate Input Handling

| Condition | Java Terminal | ThetaDataDx |
|-----------|--------------|-------------|
| `t = 0` | Returns `NaN`/`Inf` | Returns `0.0` (intrinsic for `value()`) |
| `v = 0` | Returns `NaN`/`Inf` | Returns `0.0` (intrinsic for `value()`) |

This is the mathematically correct limit for most Greeks at expiry.

## Implementation Notes

- `norm_cdf` uses a Horner-form Zelen & Severo approximation (~1e-7 accuracy) instead of Apache Commons Math's continued-fraction expansion
- `.exp()` is used instead of `E.powf(x)` for better precision (~1 ULP improvement)
- `all_greeks()` precomputes d1, d2, N(d1), N(d2) once and reuses them (~20x fewer transcendental function calls vs. computing each Greek independently)
