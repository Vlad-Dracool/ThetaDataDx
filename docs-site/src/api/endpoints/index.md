# Index Endpoints

9 methods covering market indices: list, snapshot, history, and at-time queries.

## List (2)

### index_list_symbols

All available index symbols.

**Rust:**
```rust
let symbols: Vec<String> = client.index_list_symbols().await?;
// ["SPX", "NDX", "RUT", "VIX", ...]
```

**Python:**
```python
symbols = client.index_list_symbols()
```

**CLI:**
```bash
tdx index list-symbols
```

**gRPC:** `GetIndexListSymbols`

---

### index_list_dates

Available dates for an index symbol.

**Rust:**
```rust
let dates: Vec<String> = client.index_list_dates("SPX").await?;
```

**Python:**
```python
dates = client.index_list_dates("SPX")
```

**CLI:**
```bash
tdx index list-dates SPX
```

**gRPC:** `GetIndexListDates`

## Snapshot (3)

### index_snapshot_ohlc

Latest OHLC snapshot for one or more indices.

**Rust:**
```rust
let ticks: Vec<OhlcTick> = client.index_snapshot_ohlc(&["SPX", "NDX"]).await?;
```

**Python:**
```python
ticks = client.index_snapshot_ohlc(["SPX", "NDX"])
```

**CLI:**
```bash
tdx index snapshot-ohlc SPX,NDX,RUT
```

**gRPC:** `GetIndexSnapshotOhlc`

---

### index_snapshot_price

Latest price snapshot for one or more indices.

**Rust:**
```rust
let table: proto::DataTable = client.index_snapshot_price(&["SPX", "NDX"]).await?;
```

**CLI:**
```bash
tdx index snapshot-price SPX,NDX,RUT
```

**gRPC:** `GetIndexSnapshotPrice`

---

### index_snapshot_market_value

Latest market value snapshot.

**Rust:**
```rust
let table: proto::DataTable = client.index_snapshot_market_value(&["SPX"]).await?;
```

**CLI:**
```bash
tdx index snapshot-market-value SPX,NDX,RUT
```

**gRPC:** `GetIndexSnapshotMarketValue`

## History (3)

### index_history_eod

End-of-day index data for a date range.

**Rust:**
```rust
let eod: Vec<EodTick> = client.index_history_eod("SPX", "20240101", "20240301").await?;
for tick in &eod {
    println!("{}: O={} H={} L={} C={}",
        tick.date, tick.open_price(), tick.high_price(),
        tick.low_price(), tick.close_price());
}
```

**Python:**
```python
eod = client.index_history_eod("SPX", "20240101", "20240301")
df = client.index_history_eod_df("SPX", "20240101", "20240301")
```

**CLI:**
```bash
tdx index eod SPX 20240101 20240301
```

**gRPC:** `GetIndexHistoryEod`

---

### index_history_ohlc

Intraday OHLC bars for an index.

**Rust:**
```rust
let bars: Vec<OhlcTick> = client.index_history_ohlc(
    "SPX", "20240101", "20240301", "60000"
).await?;
```

**Python:**
```python
bars = client.index_history_ohlc("SPX", "20240101", "20240301", "60000")
```

**CLI:**
```bash
tdx index ohlc SPX 20240101 20240301 60000
```

**gRPC:** `GetIndexHistoryOhlc`

---

### index_history_price

Intraday price history for an index.

**Rust:**
```rust
let table: proto::DataTable = client.index_history_price("SPX", "20240315", "60000").await?;
```

**CLI:**
```bash
tdx index price SPX 20240315 60000
```

**gRPC:** `GetIndexHistoryPrice`

## AtTime (1)

### index_at_time_price

Index price at a specific time of day across a date range.

**Rust:**
```rust
let table: proto::DataTable = client.index_at_time_price(
    "SPX", "20240101", "20240301", "34200000"  // 9:30 AM ET
).await?;
```

**Python:**
```python
result = client.index_at_time_price("SPX", "20240101", "20240301", "34200000")
```

**CLI:**
```bash
tdx index at-time-price SPX 20240101 20240301 34200000
```

**gRPC:** `GetIndexAtTimePrice`
