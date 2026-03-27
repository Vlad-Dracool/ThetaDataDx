# Rate Endpoints

1 method for interest rate data.

## interest_rate_history_eod

End-of-day interest rate history.

**Rust:**
```rust
let table: proto::DataTable = client.interest_rate_history_eod(
    "SOFR", "20240101", "20240301"
).await?;
```

**Python:**
```python
result = client.interest_rate_history_eod("SOFR", "20240101", "20240301")
```

**CLI:**
```bash
tdx rate eod SOFR 20240101 20240301
```

**gRPC:** `GetInterestRateHistoryEod`

## Available Rate Symbols

Interest rate symbols available in ThetaData. These are also used for Greeks calculations via the `RateType` enum:

| Symbol | Description |
|--------|-------------|
| `SOFR` | Secured Overnight Financing Rate |
| `TREASURY_M1` | 1-month Treasury rate |
| `TREASURY_M3` | 3-month Treasury rate |
| `TREASURY_M6` | 6-month Treasury rate |
| `TREASURY_Y1` | 1-year Treasury rate |
| `TREASURY_Y2` | 2-year Treasury rate |
| `TREASURY_Y3` | 3-year Treasury rate |
| `TREASURY_Y5` | 5-year Treasury rate |
| `TREASURY_Y7` | 7-year Treasury rate |
| `TREASURY_Y10` | 10-year Treasury rate |
| `TREASURY_Y20` | 20-year Treasury rate |
| `TREASURY_Y30` | 30-year Treasury rate |

## Example: Building a Yield Curve

### Rust

```rust
let rates = ["TREASURY_M1", "TREASURY_M3", "TREASURY_M6",
             "TREASURY_Y1", "TREASURY_Y2", "TREASURY_Y5",
             "TREASURY_Y10", "TREASURY_Y30"];

for symbol in &rates {
    let table = client.interest_rate_history_eod(
        symbol, "20240301", "20240301"
    ).await?;
    println!("{}: {:?}", symbol, table);
}
```

### CLI

```bash
for rate in SOFR TREASURY_M3 TREASURY_Y1 TREASURY_Y2 TREASURY_Y5 TREASURY_Y10 TREASURY_Y30; do
    tdx rate eod "$rate" 20240301 20240301 --format json
done
```
