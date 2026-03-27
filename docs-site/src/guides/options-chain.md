# Options Chain

This guide walks through the complete options workflow: discovering expirations, listing strikes, fetching chain data, and computing Greeks.

## Step 1: List Available Underlyings

### Rust

```rust
let symbols = client.option_list_symbols().await?;
println!("{} option underlyings available", symbols.len());
// ["AAPL", "AMZN", "GOOGL", "MSFT", "SPY", ...]
```

### Python

```python
symbols = client.option_list_symbols()
print(f"{len(symbols)} option underlyings available")
```

### CLI

```bash
tdx option list-symbols
```

## Step 2: Get Expirations

### Rust

```rust
let expirations = client.option_list_expirations("SPY").await?;
for exp in &expirations {
    println!("Expiration: {}", exp);  // "20240419", "20240517", ...
}
```

### Python

```python
expirations = client.option_list_expirations("SPY")
print(expirations[:10])
```

### CLI

```bash
tdx option expirations SPY
```

## Step 3: Get Strikes for an Expiration

### Rust

```rust
let strikes = client.option_list_strikes("SPY", "20240419").await?;
println!("{} strikes available", strikes.len());
// Strikes are returned as strings of scaled integers
// e.g., "400000" = $400.00, "500000" = $500.00
```

### Python

```python
strikes = client.option_list_strikes("SPY", "20240419")
print(f"{len(strikes)} strikes for 2024-04-19")
```

### CLI

```bash
tdx option strikes SPY 20240419
```

## Step 4: List All Contracts

Get all option contracts for a symbol on a specific date:

### Rust

```rust
let contracts = client.option_list_contracts("EOD", "SPY", "20240315").await?;
```

### Python

```python
contracts = client.option_list_contracts("EOD", "SPY", "20240315")
```

### CLI

```bash
tdx option list-contracts EOD SPY 20240315
```

## Step 5: Fetch Option Data

### End-of-Day

The `right` parameter is `"C"` for call or `"P"` for put.

#### Rust

```rust
let eod = client.option_history_eod(
    "SPY",        // symbol
    "20240419",   // expiration
    "500000",     // strike ($500.00)
    "C",          // call
    "20240101",   // start date
    "20240301",   // end date
).await?;
for tick in &eod {
    println!("{}: O={} H={} L={} C={} V={}",
        tick.date, tick.open_price(), tick.high_price(),
        tick.low_price(), tick.close_price(), tick.volume);
}
```

#### Python

```python
eod = client.option_history_eod("SPY", "20240419", "500000", "C",
                                "20240101", "20240301")
for tick in eod:
    print(f"{tick['date']}: C={tick['close']:.2f} V={tick['volume']}")
```

#### CLI

```bash
tdx option eod SPY 20240419 500000 C 20240101 20240301
```

### Intraday OHLC

#### Rust

```rust
let bars = client.option_history_ohlc(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;
```

#### Python

```python
bars = client.option_history_ohlc("SPY", "20240419", "500000", "C",
                                  "20240315", "60000")
```

#### CLI

```bash
tdx option ohlc SPY 20240419 500000 C 20240315 60000
```

### Tick-Level Trades

#### Rust

```rust
let trades = client.option_history_trade(
    "SPY", "20240419", "500000", "C", "20240315"
).await?;
```

#### Python

```python
trades = client.option_history_trade("SPY", "20240419", "500000", "C", "20240315")
```

#### CLI

```bash
tdx option trade SPY 20240419 500000 C 20240315
```

## Step 6: Option Snapshots

Get the latest real-time data for an option contract.

### Rust

```rust
let quotes = client.option_snapshot_quote("SPY", "20240419", "500000", "C").await?;
let ohlc = client.option_snapshot_ohlc("SPY", "20240419", "500000", "C").await?;
let trades = client.option_snapshot_trade("SPY", "20240419", "500000", "C").await?;
let oi = client.option_snapshot_open_interest("SPY", "20240419", "500000", "C").await?;
```

### CLI

```bash
tdx option snapshot-quote SPY 20240419 500000 C
tdx option snapshot-ohlc SPY 20240419 500000 C
tdx option snapshot-trade SPY 20240419 500000 C
tdx option snapshot-open-interest SPY 20240419 500000 C
```

## Step 7: Greeks from ThetaData

Fetch server-computed Greeks for option contracts.

### Snapshot Greeks

#### Rust

```rust
// All Greeks at once
let greeks = client.option_snapshot_greeks_all(
    "SPY", "20240419", "500000", "C"
).await?;

// By order
let first = client.option_snapshot_greeks_first_order("SPY", "20240419", "500000", "C").await?;
let second = client.option_snapshot_greeks_second_order("SPY", "20240419", "500000", "C").await?;
let third = client.option_snapshot_greeks_third_order("SPY", "20240419", "500000", "C").await?;

// Just IV
let iv = client.option_snapshot_greeks_implied_volatility(
    "SPY", "20240419", "500000", "C"
).await?;
```

#### CLI

```bash
tdx option snapshot-greeks-all SPY 20240419 500000 C
tdx option snapshot-greeks-first-order SPY 20240419 500000 C
tdx option snapshot-greeks-iv SPY 20240419 500000 C
```

### Historical Greeks

#### Rust

```rust
// EOD Greeks over a date range
let greeks_eod = client.option_history_greeks_eod(
    "SPY", "20240419", "500000", "C", "20240101", "20240301"
).await?;

// Intraday Greeks at 1-minute intervals
let greeks_all = client.option_history_greeks_all(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;

// Greeks computed on each individual trade
let trade_greeks = client.option_history_trade_greeks_all(
    "SPY", "20240419", "500000", "C", "20240315"
).await?;
```

#### CLI

```bash
tdx option greeks-eod SPY 20240419 500000 C 20240101 20240301
tdx option greeks-all SPY 20240419 500000 C 20240315 60000
tdx option trade-greeks-all SPY 20240419 500000 C 20240315
```

## Step 8: Local Greeks Calculation

Compute Greeks locally without any server call, using the built-in Black-Scholes calculator.

### Rust

```rust
use thetadatadx::greeks;

let result = greeks::all_greeks(
    450.0,            // spot price
    455.0,            // strike price
    0.05,             // risk-free rate
    0.015,            // dividend yield
    30.0 / 365.0,     // time to expiration (years)
    8.50,             // market price of the option
    true,             // is_call
);
println!("IV: {:.4}, Delta: {:.4}, Gamma: {:.6}, Theta: {:.4}",
    result.iv, result.delta, result.gamma, result.theta);
```

### Python

```python
from thetadatadx import all_greeks

g = all_greeks(
    spot=450.0, strike=455.0, rate=0.05,
    div_yield=0.015, tte=30/365, option_price=8.50, is_call=True
)
print(f"IV={g['iv']:.4f} Delta={g['delta']:.4f}")
```

### CLI

```bash
tdx greeks 450 455 0.05 0.015 0.082 8.5 call
tdx iv 450 455 0.05 0.015 0.082 8.5 call
```

## Full Chain Scan Example

Here is a complete example that scans an entire option chain:

### Rust

```rust
use thetadatadx::{DirectClient, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let client = DirectClient::connect(&creds, DirectConfig::production()).await?;

    // Get nearest expiration
    let exps = client.option_list_expirations("SPY").await?;
    let nearest_exp = &exps[0];
    println!("Scanning expiration: {}", nearest_exp);

    // Get all strikes
    let strikes = client.option_list_strikes("SPY", nearest_exp).await?;
    println!("{} strikes available", strikes.len());

    // Fetch snapshot quotes for each strike
    for strike in &strikes {
        let call_quotes = client.option_snapshot_quote(
            "SPY", nearest_exp, strike, "C"
        ).await?;
        let put_quotes = client.option_snapshot_quote(
            "SPY", nearest_exp, strike, "P"
        ).await?;

        if let (Some(c), Some(p)) = (call_quotes.first(), put_quotes.first()) {
            println!("Strike {}: Call bid={} ask={} | Put bid={} ask={}",
                strike, c.bid_price(), c.ask_price(),
                p.bid_price(), p.ask_price());
        }
    }

    Ok(())
}
```
