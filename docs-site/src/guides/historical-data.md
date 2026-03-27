# Historical Data

This guide covers fetching historical market data from ThetaData's MDDS servers. All historical data is accessed through `DirectClient`, which communicates over gRPC.

## Overview

Historical data endpoints fall into four categories:

| Category | Description | Example |
|----------|-------------|---------|
| **List** | Enumerate available symbols, dates, expirations, strikes | `stock_list_symbols()` |
| **Snapshot** | Latest value (real-time during market hours) | `stock_snapshot_quote(["AAPL"])` |
| **History** | Full tick-level or bar-level data for a date/range | `stock_history_eod("AAPL", "20240101", "20240301")` |
| **AtTime** | Value at a specific time of day across a date range | `stock_at_time_trade("AAPL", ...)` |

## Step 1: Connect

### Rust

```rust
use thetadatadx::{DirectClient, Credentials, DirectConfig};

let creds = Credentials::from_file("creds.txt")?;
let client = DirectClient::connect(&creds, DirectConfig::production()).await?;
```

### Python

```python
from thetadatadx import Credentials, Config, DirectClient

creds = Credentials.from_file("creds.txt")
client = DirectClient(creds, Config.production())
```

### CLI

```bash
# The CLI connects automatically using --creds (defaults to creds.txt)
tdx stock eod AAPL 20240101 20240301
```

## Step 2: End-of-Day Data

End-of-day data returns one record per trading day, with OHLC prices, volume, and closing bid/ask.

### Rust

```rust
let eod = client.stock_history_eod("AAPL", "20240101", "20240301").await?;
for tick in &eod {
    println!("{}: O={} H={} L={} C={} V={}",
        tick.date, tick.open_price(), tick.high_price(),
        tick.low_price(), tick.close_price(), tick.volume);
}
```

### Python

```python
eod = client.stock_history_eod("AAPL", "20240101", "20240301")
for tick in eod:
    print(f"{tick['date']}: O={tick['open']:.2f} C={tick['close']:.2f} V={tick['volume']}")

# Or as a DataFrame
df = client.stock_history_eod_df("AAPL", "20240101", "20240301")
print(df.describe())
```

### CLI

```bash
tdx stock eod AAPL 20240101 20240301
tdx stock eod AAPL 20240101 20240301 --format json
tdx stock eod AAPL 20240101 20240301 --format csv
```

## Step 3: Intraday OHLC Bars

Fetch aggregated bars at any interval. The `interval` parameter is in milliseconds.

| Interval | Milliseconds |
|----------|-------------|
| 1 second | `"1000"` |
| 1 minute | `"60000"` |
| 5 minutes | `"300000"` |
| 15 minutes | `"900000"` |
| 1 hour | `"3600000"` |

### Rust

```rust
// 1-minute bars for a single date
let bars = client.stock_history_ohlc("AAPL", "20240315", "60000").await?;
println!("{} bars", bars.len());

// 5-minute bars across a date range
let bars = client.stock_history_ohlc_range(
    "AAPL", "20240101", "20240301", "300000"
).await?;
```

### Python

```python
# 1-minute bars
bars = client.stock_history_ohlc("AAPL", "20240315", "60000")
print(f"{len(bars)} bars")

# As DataFrame
df = client.stock_history_ohlc_df("AAPL", "20240315", "60000")
```

### CLI

```bash
tdx stock ohlc AAPL 20240315 60000          # single date
tdx stock ohlc-range AAPL 20240101 20240301 60000  # date range
```

## Step 4: Tick-Level Trade Data

Retrieve every individual trade on a given date.

### Rust

```rust
let trades = client.stock_history_trade("AAPL", "20240315").await?;
for t in &trades {
    println!("{}ms: price={} size={} exchange={}",
        t.ms_of_day, t.get_price(), t.size, t.exchange);
}
```

For very large trade histories (millions of rows), use the streaming variant to avoid loading everything into memory:

```rust
client.stock_history_trade_stream("AAPL", "20240315", |chunk| {
    for t in &chunk {
        // process each trade
    }
    Ok(())
}).await?;
```

### Python

```python
trades = client.stock_history_trade("AAPL", "20240315")
print(f"{len(trades)} trades")
```

### CLI

```bash
tdx stock trade AAPL 20240315
```

## Step 5: Quote Data

Fetch NBBO quotes at a specified interval.

### Rust

```rust
// Quote every minute
let quotes = client.stock_history_quote("AAPL", "20240315", "60000").await?;
for q in &quotes {
    println!("{}ms: bid={} ask={} mid={}",
        q.ms_of_day, q.bid_price(), q.ask_price(), q.midpoint_price());
}

// Every quote change (use interval "0")
let all_quotes = client.stock_history_quote("AAPL", "20240315", "0").await?;
```

### Python

```python
quotes = client.stock_history_quote("AAPL", "20240315", "60000")
df = client.stock_history_quote_df("AAPL", "20240315", "0")
```

### CLI

```bash
tdx stock quote AAPL 20240315 60000
```

## Step 6: Snapshots

Get the latest real-time value for one or more symbols.

### Rust

```rust
// Multiple symbols in one call
let quotes = client.stock_snapshot_quote(&["AAPL", "MSFT", "GOOGL"]).await?;
for q in &quotes {
    println!("bid={} ask={}", q.bid_price(), q.ask_price());
}

let ohlc = client.stock_snapshot_ohlc(&["AAPL"]).await?;
let trades = client.stock_snapshot_trade(&["AAPL"]).await?;
```

### Python

```python
quotes = client.stock_snapshot_quote(["AAPL", "MSFT", "GOOGL"])
```

### CLI

```bash
tdx stock snapshot-quote AAPL,MSFT,GOOGL
tdx stock snapshot-ohlc AAPL,MSFT,GOOGL
tdx stock snapshot-trade AAPL
```

## Step 7: At-Time Queries

Get the trade or quote at a specific time of day, across a date range. The `time_of_day` parameter is milliseconds from midnight Eastern Time.

| Time (ET) | Milliseconds |
|-----------|-------------|
| 9:30 AM | `34200000` |
| 12:00 PM | `43200000` |
| 4:00 PM | `57600000` |

### Rust

```rust
// Trade at market open (9:30 AM) for each day in range
let trades = client.stock_at_time_trade(
    "AAPL", "20240101", "20240301", "34200000"
).await?;
```

### Python

```python
trades = client.stock_at_time_trade("AAPL", "20240101", "20240301", "34200000")
```

### CLI

```bash
tdx stock at-time-trade AAPL 20240101 20240301 34200000
```

## Date Format

All dates are `YYYYMMDD` strings: `"20240315"` for March 15, 2024. The SDK validates dates client-side before sending to the server.

## Streaming Large Responses

For endpoints that return millions of rows (e.g., full trade history for a liquid symbol), use `_stream` variants to avoid unbounded memory growth:

```rust
// Standard: loads everything into memory
let trades = client.stock_history_trade("AAPL", "20240315").await?;

// Streaming: processes chunk by chunk
client.stock_history_trade_stream("AAPL", "20240315", |chunk: Vec<TradeTick>| {
    println!("Got {} trades in this chunk", chunk.len());
    Ok(())
}).await?;
```

Available streaming variants:
- `stock_history_trade_stream`
- `stock_history_quote_stream`
- `option_history_trade_stream`
- `option_history_quote_stream`

## Empty Responses

When a query returns no data (e.g., a date with no trading), the SDK returns an empty collection rather than an error. Check `.is_empty()` or `len() == 0`.

`Error::NoData` is reserved for cases where the endpoint genuinely has no usable data (e.g., a symbol that does not exist).
