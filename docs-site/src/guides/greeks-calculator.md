# Greeks Calculator

ThetaDataDx includes a full Black-Scholes calculator with 22 Greeks -- first, second, and third order -- plus an implied volatility solver. The calculator runs entirely locally with no server connection required.

## Overview

The calculator is a faithful port of ThetaData's Java implementation, with two improvements:

1. **Degenerate input guard** -- returns `0.0` instead of `NaN`/`Inf` for `t=0` or `v=0` inputs
2. **Precomputed intermediates** -- `all_greeks()` computes d1, d2, N(d1), N(d2) once and reuses them across all 22 Greeks

## Parameters

All Greek functions accept the same base parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| `s` | `f64` | Spot price (underlying) |
| `x` | `f64` | Strike price |
| `v` | `f64` | Volatility (sigma) |
| `r` | `f64` | Risk-free rate |
| `q` | `f64` | Dividend yield |
| `t` | `f64` | Time to expiration (years) |
| `is_call` | `bool` | `true` for call, `false` for put |

## All Greeks at Once

The most common usage: compute IV from the market price, then derive all 22 Greeks.

### Rust

```rust
use thetadatadx::greeks;

let result = greeks::all_greeks(
    450.0,            // spot
    455.0,            // strike
    0.05,             // risk-free rate
    0.015,            // dividend yield
    30.0 / 365.0,     // time to expiry (years)
    8.50,             // market option price
    true,             // is_call
);

println!("Implied Volatility: {:.4}", result.iv);
println!("IV Error:           {:.6}", result.iv_error);
println!();
println!("--- First Order ---");
println!("Value:    {:.4}", result.value);
println!("Delta:    {:.4}", result.delta);
println!("Theta:    {:.4} (daily)", result.theta);
println!("Vega:     {:.4}", result.vega);
println!("Rho:      {:.4}", result.rho);
println!("Epsilon:  {:.4}", result.epsilon);
println!("Lambda:   {:.4}", result.lambda);
println!();
println!("--- Second Order ---");
println!("Gamma:    {:.6}", result.gamma);
println!("Vanna:    {:.6}", result.vanna);
println!("Charm:    {:.6}", result.charm);
println!("Vomma:    {:.6}", result.vomma);
println!("Veta:     {:.6}", result.veta);
println!();
println!("--- Third Order ---");
println!("Speed:    {:.8}", result.speed);
println!("Zomma:    {:.8}", result.zomma);
println!("Color:    {:.8}", result.color);
println!("Ultima:   {:.6}", result.ultima);
println!();
println!("--- Auxiliary ---");
println!("d1:         {:.6}", result.d1);
println!("d2:         {:.6}", result.d2);
println!("Dual Delta: {:.6}", result.dual_delta);
println!("Dual Gamma: {:.6}", result.dual_gamma);
```

### Python

```python
from thetadatadx import all_greeks

g = all_greeks(
    spot=450.0, strike=455.0, rate=0.05,
    div_yield=0.015, tte=30/365, option_price=8.50, is_call=True
)

print(f"IV:    {g['iv']:.4f}")
print(f"Delta: {g['delta']:.4f}")
print(f"Gamma: {g['gamma']:.6f}")
print(f"Theta: {g['theta']:.4f}")
print(f"Vega:  {g['vega']:.4f}")
```

### Go

```go
g, _ := thetadatadx.AllGreeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true)
fmt.Printf("IV=%.4f Delta=%.4f Gamma=%.6f\n", g.IV, g.Delta, g.Gamma)
```

### C++

```cpp
auto g = tdx::all_greeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true);
std::cout << "IV=" << g.iv << " Delta=" << g.delta << std::endl;
```

### CLI

```bash
tdx greeks 450 455 0.05 0.015 0.082 8.5 call
#         spot strike rate div_yield tte price is_call
```

## Implied Volatility Only

If you only need IV without the full Greek suite:

### Rust

```rust
let (iv, error) = greeks::implied_volatility(
    450.0,   // spot
    455.0,   // strike
    0.05,    // rate
    0.015,   // dividend yield
    30.0 / 365.0, // time to expiry
    8.50,    // market price
    true,    // is_call
);
println!("IV: {:.4}, Error: {:.6}", iv, error);
```

The solver uses bisection with up to 128 iterations. The `error` return value is the relative difference `(theoretical - market) / market`.

### Python

```python
from thetadatadx import implied_volatility

iv, err = implied_volatility(450.0, 455.0, 0.05, 0.015, 30/365, 8.50, True)
print(f"IV: {iv:.4f}, Error: {err:.6f}")
```

### CLI

```bash
tdx iv 450 455 0.05 0.015 0.082 8.5 call
```

## Individual Greeks

For targeted computation when you only need one or two Greeks:

### Rust

```rust
use thetadatadx::greeks;

let s = 450.0;  // spot
let x = 455.0;  // strike
let v = 0.20;   // volatility
let r = 0.05;   // rate
let q = 0.015;  // dividend yield
let t = 30.0 / 365.0;  // time

// First order
let delta = greeks::delta(s, x, v, r, q, t, true);
let theta = greeks::theta(s, x, v, r, q, t, true);  // daily (divided by 365)
let vega  = greeks::vega(s, x, v, r, q, t);
let rho   = greeks::rho(s, x, v, r, q, t, true);

// Second order
let gamma = greeks::gamma(s, x, v, r, q, t);
let vanna = greeks::vanna(s, x, v, r, q, t);
let charm = greeks::charm(s, x, v, r, q, t, true);
let vomma = greeks::vomma(s, x, v, r, q, t);

// Third order
let speed  = greeks::speed(s, x, v, r, q, t);
let zomma  = greeks::zomma(s, x, v, r, q, t);
let color  = greeks::color(s, x, v, r, q, t);
let ultima = greeks::ultima(s, x, v, r, q, t);  // clamped to [-100, 100]
```

## Complete Greek Reference

### First Order

| Greek | Description | Notes |
|-------|-------------|-------|
| `delta` | Rate of change of option value with respect to underlying price | Call: 0 to 1, Put: -1 to 0 |
| `theta` | Time decay per day | Divided by 365 |
| `vega` | Sensitivity to volatility | Same for calls and puts |
| `rho` | Sensitivity to interest rate | |
| `epsilon` | Sensitivity to dividend yield | |
| `lambda` | Leverage ratio (delta * S / V) | |

### Second Order

| Greek | Description | Notes |
|-------|-------------|-------|
| `gamma` | Rate of change of delta | Same for calls and puts |
| `vanna` | Cross-sensitivity: delta to vol, vega to spot | |
| `charm` | Delta decay (rate of change of delta with time) | |
| `vomma` | Sensitivity of vega to volatility (vol-of-vol) | |
| `veta` | Sensitivity of vega to time | |

### Third Order

| Greek | Description | Notes |
|-------|-------------|-------|
| `speed` | Rate of change of gamma with respect to price | |
| `zomma` | Rate of change of gamma with respect to vol | |
| `color` | Rate of change of gamma with respect to time | |
| `ultima` | Sensitivity of vomma to volatility | Clamped to [-100, 100] |

### Auxiliary

| Greek | Description |
|-------|-------------|
| `d1` | Black-Scholes d1 intermediate |
| `d2` | Black-Scholes d2 intermediate |
| `dual_delta` | Sensitivity to strike price |
| `dual_gamma` | Second derivative with respect to strike |

## GreeksResult Struct

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

## Edge Cases

| Condition | Behavior |
|-----------|----------|
| `t = 0` (at expiry) | Returns `0.0` for most Greeks; `value()` returns intrinsic value |
| `v = 0` (zero vol) | Returns `0.0` for most Greeks; `value()` returns intrinsic value |
| Deep ITM/OTM | IV solver may return high error; check `iv_error` field |
| `ultima` overflow | Clamped to [-100, 100] range |

The Java terminal returns `NaN` or `Inf` for degenerate inputs. ThetaDataDx returns `0.0` instead, which is the mathematically correct limit for most Greeks as `t -> 0` or `v -> 0`.
