# Stock Endpoints

14 methods covering stocks: list, snapshot, history, and at-time queries.

## List (2)

### stock_list_symbols

All available stock symbols.

**Rust:**
```rust
let symbols: Vec<String> = client.stock_list_symbols().await?;
```

**Python:**
```python
symbols = client.stock_list_symbols()
```

**CLI:**
```bash
tdx stock list-symbols
```

**gRPC:** `GetStockListSymbols`

---

### stock_list_dates

Available dates for a stock by request type.

**Rust:**
```rust
let dates: Vec<String> = client.stock_list_dates("EOD", "AAPL").await?;
// request_type: "EOD", "TRADE", "QUOTE", etc.
```

**Python:**
```python
dates = client.stock_list_dates("EOD", "AAPL")
```

**CLI:**
```bash
tdx stock list-dates EOD AAPL
```

**gRPC:** `GetStockListDates`

## Snapshot (4)

### stock_snapshot_ohlc

Latest OHLC snapshot for one or more stocks.

**Rust:**
```rust
let ticks: Vec<OhlcTick> = client.stock_snapshot_ohlc(&["AAPL", "MSFT"]).await?;
```

**Python:**
```python
ticks = client.stock_snapshot_ohlc(["AAPL", "MSFT"])
```

**CLI:**
```bash
tdx stock snapshot-ohlc AAPL,MSFT,GOOGL
```

**gRPC:** `GetStockSnapshotOhlc`

---

### stock_snapshot_trade

Latest trade snapshot for one or more stocks.

**Rust:**
```rust
let ticks: Vec<TradeTick> = client.stock_snapshot_trade(&["AAPL"]).await?;
```

**Python:**
```python
ticks = client.stock_snapshot_trade(["AAPL"])
```

**CLI:**
```bash
tdx stock snapshot-trade AAPL
```

**gRPC:** `GetStockSnapshotTrade`

---

### stock_snapshot_quote

Latest NBBO quote snapshot for one or more stocks.

**Rust:**
```rust
let ticks: Vec<QuoteTick> = client.stock_snapshot_quote(&["AAPL", "MSFT"]).await?;
for q in &ticks {
    println!("bid={} ask={}", q.bid_price(), q.ask_price());
}
```

**Python:**
```python
ticks = client.stock_snapshot_quote(["AAPL", "MSFT"])
```

**CLI:**
```bash
tdx stock snapshot-quote AAPL,MSFT
```

**gRPC:** `GetStockSnapshotQuote`

---

### stock_snapshot_market_value

Latest market value snapshot.

**Rust:**
```rust
let table: proto::DataTable = client.stock_snapshot_market_value(&["AAPL"]).await?;
```

**CLI:**
```bash
tdx stock snapshot-market-value AAPL
```

**gRPC:** `GetStockSnapshotMarketValue`

## History (6)

### stock_history_eod

End-of-day stock data for a date range. Dates are `YYYYMMDD` strings.

**Rust:**
```rust
let ticks: Vec<EodTick> = client.stock_history_eod("AAPL", "20240101", "20240301").await?;
for t in &ticks {
    println!("{}: O={} H={} L={} C={} V={}",
        t.date, t.open_price(), t.high_price(),
        t.low_price(), t.close_price(), t.volume);
}
```

**Python:**
```python
eod = client.stock_history_eod("AAPL", "20240101", "20240301")
df = client.stock_history_eod_df("AAPL", "20240101", "20240301")
```

**CLI:**
```bash
tdx stock eod AAPL 20240101 20240301
tdx stock eod AAPL 20240101 20240301 --format json
tdx stock eod AAPL 20240101 20240301 --format csv
```

**gRPC:** `GetStockHistoryEod`

---

### stock_history_ohlc

Intraday OHLC bars for a single date. `interval` is milliseconds (e.g., `"60000"` for 1-minute bars).

**Rust:**
```rust
let bars: Vec<OhlcTick> = client.stock_history_ohlc("AAPL", "20240315", "60000").await?;
```

**Python:**
```python
bars = client.stock_history_ohlc("AAPL", "20240315", "60000")
```

**CLI:**
```bash
tdx stock ohlc AAPL 20240315 60000
```

**gRPC:** `GetStockHistoryOhlc`

---

### stock_history_ohlc_range

Intraday OHLC bars across a date range. Convenience wrapper -- uses `start_date`/`end_date` instead of single `date`.

**Rust:**
```rust
let bars: Vec<OhlcTick> = client.stock_history_ohlc_range(
    "AAPL", "20240101", "20240301", "300000"  // 5-min bars
).await?;
```

**Python:**
```python
bars = client.stock_history_ohlc_range("AAPL", "20240101", "20240301", "300000")
```

**CLI:**
```bash
tdx stock ohlc-range AAPL 20240101 20240301 300000
```

**gRPC:** `GetStockHistoryOhlc` (with start_date/end_date)

---

### stock_history_trade

All trades for a stock on a given date.

**Rust:**
```rust
let trades: Vec<TradeTick> = client.stock_history_trade("AAPL", "20240315").await?;

// Streaming variant for large results:
client.stock_history_trade_stream("AAPL", "20240315", |chunk| {
    println!("{} trades in chunk", chunk.len());
    Ok(())
}).await?;
```

**Python:**
```python
trades = client.stock_history_trade("AAPL", "20240315")
```

**CLI:**
```bash
tdx stock trade AAPL 20240315
```

**gRPC:** `GetStockHistoryTrade`

---

### stock_history_quote

NBBO quotes at a given interval. Use `"0"` for every quote change.

**Rust:**
```rust
let quotes: Vec<QuoteTick> = client.stock_history_quote("AAPL", "20240315", "60000").await?;
```

**Python:**
```python
quotes = client.stock_history_quote("AAPL", "20240315", "60000")
```

**CLI:**
```bash
tdx stock quote AAPL 20240315 60000
```

**gRPC:** `GetStockHistoryQuote`

---

### stock_history_trade_quote

Combined trade + quote ticks. Returns raw `DataTable`.

**Rust:**
```rust
let table: proto::DataTable = client.stock_history_trade_quote("AAPL", "20240315").await?;
```

**CLI:**
```bash
tdx stock trade-quote AAPL 20240315
```

**gRPC:** `GetStockHistoryTradeQuote`

## AtTime (2)

### stock_at_time_trade

Trade at a specific time of day across a date range. `time_of_day` is milliseconds from midnight ET (e.g., `"34200000"` for 9:30 AM).

**Rust:**
```rust
let trades: Vec<TradeTick> = client.stock_at_time_trade(
    "AAPL", "20240101", "20240301", "34200000"
).await?;
```

**Python:**
```python
trades = client.stock_at_time_trade("AAPL", "20240101", "20240301", "34200000")
```

**CLI:**
```bash
tdx stock at-time-trade AAPL 20240101 20240301 34200000
```

**gRPC:** `GetStockAtTimeTrade`

---

### stock_at_time_quote

Quote at a specific time of day across a date range.

**Rust:**
```rust
let quotes: Vec<QuoteTick> = client.stock_at_time_quote(
    "AAPL", "20240101", "20240301", "34200000"
).await?;
```

**Python:**
```python
quotes = client.stock_at_time_quote("AAPL", "20240101", "20240301", "34200000")
```

**CLI:**
```bash
tdx stock at-time-quote AAPL 20240101 20240301 34200000
```

**gRPC:** `GetStockAtTimeQuote`
