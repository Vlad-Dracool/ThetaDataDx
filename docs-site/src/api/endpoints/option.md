# Option Endpoints

34 methods covering the full option chain: list, snapshot, history, Greeks, and at-time queries.

All option-specific endpoints require these additional parameters:
- `symbol` -- underlying symbol (e.g., `"SPY"`)
- `expiration` -- expiration date as `YYYYMMDD` string
- `strike` -- strike price as scaled integer string (e.g., `"500000"` for $500.00)
- `right` -- `"C"` for call, `"P"` for put

## List (5)

### option_list_symbols

All available option underlying symbols.

**Rust:**
```rust
let symbols: Vec<String> = client.option_list_symbols().await?;
```

**Python:**
```python
symbols = client.option_list_symbols()
```

**CLI:**
```bash
tdx option list-symbols
```

---

### option_list_dates

Available dates for an option contract by request type.

**Rust:**
```rust
let dates: Vec<String> = client.option_list_dates(
    "EOD", "SPY", "20240419", "500000", "C"
).await?;
```

**CLI:**
```bash
tdx option list-dates EOD SPY 20240419 500000 C
```

---

### option_list_expirations

Expiration dates for an underlying. Returns `YYYYMMDD` strings.

**Rust:**
```rust
let exps: Vec<String> = client.option_list_expirations("SPY").await?;
```

**Python:**
```python
exps = client.option_list_expirations("SPY")
```

**CLI:**
```bash
tdx option expirations SPY
```

---

### option_list_strikes

Strike prices for a given expiration.

**Rust:**
```rust
let strikes: Vec<String> = client.option_list_strikes("SPY", "20240419").await?;
```

**Python:**
```python
strikes = client.option_list_strikes("SPY", "20240419")
```

**CLI:**
```bash
tdx option strikes SPY 20240419
```

---

### option_list_contracts

All option contracts for a symbol on a given date. Returns `DataTable` with contract details.

**Rust:**
```rust
let table: proto::DataTable = client.option_list_contracts("EOD", "SPY", "20240315").await?;
```

**CLI:**
```bash
tdx option list-contracts EOD SPY 20240315
```

## Snapshot (5)

### option_snapshot_ohlc / trade / quote / open_interest / market_value

Latest snapshot data for option contracts.

**Rust:**
```rust
let ohlc = client.option_snapshot_ohlc("SPY", "20240419", "500000", "C").await?;
let trades = client.option_snapshot_trade("SPY", "20240419", "500000", "C").await?;
let quotes = client.option_snapshot_quote("SPY", "20240419", "500000", "C").await?;
let oi = client.option_snapshot_open_interest("SPY", "20240419", "500000", "C").await?;
let mv = client.option_snapshot_market_value("SPY", "20240419", "500000", "C").await?;
```

**CLI:**
```bash
tdx option snapshot-ohlc SPY 20240419 500000 C
tdx option snapshot-trade SPY 20240419 500000 C
tdx option snapshot-quote SPY 20240419 500000 C
tdx option snapshot-open-interest SPY 20240419 500000 C
tdx option snapshot-market-value SPY 20240419 500000 C
```

## Snapshot Greeks (5)

### option_snapshot_greeks_implied_volatility

IV snapshot.

**Rust:**
```rust
let iv = client.option_snapshot_greeks_implied_volatility(
    "SPY", "20240419", "500000", "C"
).await?;
```

**CLI:**
```bash
tdx option snapshot-greeks-iv SPY 20240419 500000 C
```

---

### option_snapshot_greeks_all / first_order / second_order / third_order

Greeks snapshots by order.

**Rust:**
```rust
let all = client.option_snapshot_greeks_all("SPY", "20240419", "500000", "C").await?;
let first = client.option_snapshot_greeks_first_order("SPY", "20240419", "500000", "C").await?;
let second = client.option_snapshot_greeks_second_order("SPY", "20240419", "500000", "C").await?;
let third = client.option_snapshot_greeks_third_order("SPY", "20240419", "500000", "C").await?;
```

**CLI:**
```bash
tdx option snapshot-greeks-all SPY 20240419 500000 C
tdx option snapshot-greeks-first-order SPY 20240419 500000 C
tdx option snapshot-greeks-second-order SPY 20240419 500000 C
tdx option snapshot-greeks-third-order SPY 20240419 500000 C
```

## History (6)

### option_history_eod

End-of-day option data.

**Rust:**
```rust
let eod: Vec<EodTick> = client.option_history_eod(
    "SPY", "20240419", "500000", "C", "20240101", "20240301"
).await?;
```

**Python:**
```python
eod = client.option_history_eod("SPY", "20240419", "500000", "C",
                                "20240101", "20240301")
```

**CLI:**
```bash
tdx option eod SPY 20240419 500000 C 20240101 20240301
```

---

### option_history_ohlc

Intraday option OHLC bars.

**Rust:**
```rust
let bars: Vec<OhlcTick> = client.option_history_ohlc(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;
```

**CLI:**
```bash
tdx option ohlc SPY 20240419 500000 C 20240315 60000
```

---

### option_history_trade

Option trades on a given date.

**Rust:**
```rust
let trades: Vec<TradeTick> = client.option_history_trade(
    "SPY", "20240419", "500000", "C", "20240315"
).await?;

// Streaming variant
client.option_history_trade_stream(
    "SPY", "20240419", "500000", "C", "20240315",
    |chunk| { Ok(()) }
).await?;
```

**CLI:**
```bash
tdx option trade SPY 20240419 500000 C 20240315
```

---

### option_history_quote

Option NBBO quotes.

**Rust:**
```rust
let quotes: Vec<QuoteTick> = client.option_history_quote(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;
```

**CLI:**
```bash
tdx option quote SPY 20240419 500000 C 20240315 60000
```

---

### option_history_trade_quote

Combined trade + quote ticks.

**Rust:**
```rust
let table = client.option_history_trade_quote(
    "SPY", "20240419", "500000", "C", "20240315"
).await?;
```

**CLI:**
```bash
tdx option trade-quote SPY 20240419 500000 C 20240315
```

---

### option_history_open_interest

Open interest history for an option contract.

**Rust:**
```rust
let table = client.option_history_open_interest(
    "SPY", "20240419", "500000", "C", "20240315"
).await?;
```

**CLI:**
```bash
tdx option open-interest SPY 20240419 500000 C 20240315
```

## History Greeks (6)

### option_history_greeks_eod

EOD Greeks history for an option contract.

**Rust:**
```rust
let table = client.option_history_greeks_eod(
    "SPY", "20240419", "500000", "C", "20240101", "20240301"
).await?;
```

**CLI:**
```bash
tdx option greeks-eod SPY 20240419 500000 C 20240101 20240301
```

---

### option_history_greeks_all / first_order / second_order / third_order / implied_volatility

Intraday Greeks history sampled by interval.

**Rust:**
```rust
let all = client.option_history_greeks_all(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;
let first = client.option_history_greeks_first_order(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;
let iv = client.option_history_greeks_implied_volatility(
    "SPY", "20240419", "500000", "C", "20240315", "60000"
).await?;
```

**CLI:**
```bash
tdx option greeks-all SPY 20240419 500000 C 20240315 60000
tdx option greeks-first-order SPY 20240419 500000 C 20240315 60000
tdx option greeks-second-order SPY 20240419 500000 C 20240315 60000
tdx option greeks-third-order SPY 20240419 500000 C 20240315 60000
tdx option greeks-iv SPY 20240419 500000 C 20240315 60000
```

## History Trade Greeks (5)

Greeks computed on each individual trade.

### option_history_trade_greeks_all / first_order / second_order / third_order / implied_volatility

**Rust:**
```rust
let all = client.option_history_trade_greeks_all(
    "SPY", "20240419", "500000", "C", "20240315"
).await?;
```

**CLI:**
```bash
tdx option trade-greeks-all SPY 20240419 500000 C 20240315
tdx option trade-greeks-first-order SPY 20240419 500000 C 20240315
tdx option trade-greeks-second-order SPY 20240419 500000 C 20240315
tdx option trade-greeks-third-order SPY 20240419 500000 C 20240315
tdx option trade-greeks-iv SPY 20240419 500000 C 20240315
```

## AtTime (2)

### option_at_time_trade / option_at_time_quote

Trade or quote at a specific time of day across a date range.

**Rust:**
```rust
let trades: Vec<TradeTick> = client.option_at_time_trade(
    "SPY", "20240419", "500000", "C",
    "20240101", "20240301", "34200000"  // 9:30 AM ET
).await?;

let quotes: Vec<QuoteTick> = client.option_at_time_quote(
    "SPY", "20240419", "500000", "C",
    "20240101", "20240301", "34200000"
).await?;
```

**CLI:**
```bash
tdx option at-time-trade SPY 20240419 500000 C 20240101 20240301 34200000
tdx option at-time-quote SPY 20240419 500000 C 20240101 20240301 34200000
```
