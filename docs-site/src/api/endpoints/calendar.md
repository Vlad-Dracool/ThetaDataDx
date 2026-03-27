# Calendar Endpoints

3 methods for market calendar information.

## calendar_open_today

Whether the market is open today.

**Rust:**
```rust
let table: proto::DataTable = client.calendar_open_today().await?;
```

**Python:**
```python
result = client.calendar_open_today()
```

**CLI:**
```bash
tdx calendar today
```

**gRPC:** `GetCalendarOpenToday`

---

## calendar_on_date

Calendar information for a specific date (market hours, holidays, early close).

**Rust:**
```rust
let table: proto::DataTable = client.calendar_on_date("20240315").await?;
```

**Python:**
```python
result = client.calendar_on_date("20240315")
```

**CLI:**
```bash
tdx calendar date 20240315
```

**gRPC:** `GetCalendarOnDate`

---

## calendar_year

Calendar information for an entire year. Returns all trading days with market hours and holiday information.

**Rust:**
```rust
let table: proto::DataTable = client.calendar_year("2024").await?;
```

**Python:**
```python
result = client.calendar_year("2024")
```

**CLI:**
```bash
tdx calendar year 2024
```

**gRPC:** `GetCalendarYear`

## Example: Check if Today is a Trading Day

### Rust

```rust
let cal = client.calendar_open_today().await?;
// Parse the DataTable to determine market status
```

### CLI

```bash
# Quick check from the command line
tdx calendar today --format json | jq '.'
```

## Example: Find All Holidays in a Year

### CLI

```bash
tdx calendar year 2024 --format json | jq '[.[] | select(.is_holiday == true)]'
```
