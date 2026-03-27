# Python SDK

Python SDK for ThetaData market data, powered by the `thetadatadx` Rust crate via PyO3.

**This is NOT a Python reimplementation.** Every call goes through compiled Rust -- gRPC communication, protobuf parsing, zstd decompression, FIT tick decoding, and TCP streaming all happen at native speed. Python is just the interface.

## Installation

```bash
pip install thetadatadx

# With DataFrame support
pip install thetadatadx[pandas]    # pandas
pip install thetadatadx[polars]    # polars
pip install thetadatadx[all]       # both
```

Requires Python 3.9+.

### Building from Source

```bash
pip install maturin
git clone https://github.com/userFRM/ThetaDataDx.git
cd ThetaDataDx/sdks/python
maturin develop --release
```

## Quick Start

```python
from thetadatadx import Credentials, Config, DirectClient

# Authenticate and connect
creds = Credentials.from_file("creds.txt")
client = DirectClient(creds, Config.production())

# Fetch end-of-day data
eod = client.stock_history_eod("AAPL", "20240101", "20240301")
for tick in eod:
    print(f"{tick['date']}: O={tick['open']:.2f} H={tick['high']:.2f} "
          f"L={tick['low']:.2f} C={tick['close']:.2f} V={tick['volume']}")

# Intraday 1-minute OHLC bars
bars = client.stock_history_ohlc("AAPL", "20240315", "60000")
print(f"{len(bars)} bars")

# Option chain
exps = client.option_list_expirations("SPY")
strikes = client.option_list_strikes("SPY", exps[0])
```

## Credentials

```python
# From file (line 1 = email, line 2 = password)
creds = Credentials.from_file("creds.txt")

# Direct construction
creds = Credentials("user@example.com", "password")
```

## Config

```python
config = Config.production()  # ThetaData NJ production servers
config = Config.dev()         # dev servers with shorter timeouts
```

## DirectClient

All 61 endpoints are available. Methods return lists of dicts.

```python
client = DirectClient(creds, Config.production())
```

### Stock Methods (14)

| Method | Description |
|--------|-------------|
| `stock_list_symbols()` | All stock symbols |
| `stock_list_dates(request_type, symbol)` | Available dates by request type |
| `stock_snapshot_ohlc(symbols)` | Latest OHLC snapshot |
| `stock_snapshot_trade(symbols)` | Latest trade snapshot |
| `stock_snapshot_quote(symbols)` | Latest NBBO quote snapshot |
| `stock_snapshot_market_value(symbols)` | Latest market value snapshot |
| `stock_history_eod(symbol, start, end)` | End-of-day data |
| `stock_history_ohlc(symbol, date, interval)` | Intraday OHLC bars |
| `stock_history_ohlc_range(symbol, start, end, interval)` | OHLC bars across date range |
| `stock_history_trade(symbol, date)` | All trades for a date |
| `stock_history_quote(symbol, date, interval)` | NBBO quotes |
| `stock_history_trade_quote(symbol, date)` | Combined trade+quote ticks |
| `stock_at_time_trade(symbol, start, end, time)` | Trade at specific time across dates |
| `stock_at_time_quote(symbol, start, end, time)` | Quote at specific time across dates |

### Option Methods (34)

| Method | Description |
|--------|-------------|
| `option_list_symbols()` | Option underlying symbols |
| `option_list_dates(request_type, symbol, exp, strike, right)` | Available dates for a contract |
| `option_list_expirations(symbol)` | Expiration dates |
| `option_list_strikes(symbol, exp)` | Strike prices |
| `option_list_contracts(request_type, symbol, date)` | All contracts for a date |
| `option_snapshot_ohlc(symbol, exp, strike, right)` | Latest OHLC snapshot |
| `option_snapshot_trade(symbol, exp, strike, right)` | Latest trade snapshot |
| `option_snapshot_quote(symbol, exp, strike, right)` | Latest quote snapshot |
| `option_snapshot_open_interest(symbol, exp, strike, right)` | Latest open interest |
| `option_snapshot_market_value(symbol, exp, strike, right)` | Latest market value |
| `option_snapshot_greeks_implied_volatility(symbol, exp, strike, right)` | IV snapshot |
| `option_snapshot_greeks_all(symbol, exp, strike, right)` | All Greeks snapshot |
| `option_snapshot_greeks_first_order(symbol, exp, strike, right)` | First-order Greeks |
| `option_snapshot_greeks_second_order(symbol, exp, strike, right)` | Second-order Greeks |
| `option_snapshot_greeks_third_order(symbol, exp, strike, right)` | Third-order Greeks |
| `option_history_eod(symbol, exp, strike, right, start, end)` | EOD option data |
| `option_history_ohlc(symbol, exp, strike, right, date, interval)` | Intraday OHLC bars |
| `option_history_trade(symbol, exp, strike, right, date)` | All trades |
| `option_history_quote(symbol, exp, strike, right, date, interval)` | NBBO quotes |
| `option_history_trade_quote(symbol, exp, strike, right, date)` | Combined trade+quote |
| `option_history_open_interest(symbol, exp, strike, right, date)` | Open interest history |
| `option_history_greeks_eod(symbol, exp, strike, right, start, end)` | EOD Greeks |
| `option_history_greeks_all(symbol, exp, strike, right, date, interval)` | All Greeks history |
| `option_history_trade_greeks_all(symbol, exp, strike, right, date)` | Greeks on each trade |
| `option_history_greeks_first_order(symbol, exp, strike, right, date, interval)` | First-order Greeks history |
| `option_history_trade_greeks_first_order(symbol, exp, strike, right, date)` | First-order on each trade |
| `option_history_greeks_second_order(symbol, exp, strike, right, date, interval)` | Second-order Greeks history |
| `option_history_trade_greeks_second_order(symbol, exp, strike, right, date)` | Second-order on each trade |
| `option_history_greeks_third_order(symbol, exp, strike, right, date, interval)` | Third-order Greeks history |
| `option_history_trade_greeks_third_order(symbol, exp, strike, right, date)` | Third-order on each trade |
| `option_history_greeks_implied_volatility(symbol, exp, strike, right, date, interval)` | IV history |
| `option_history_trade_greeks_implied_volatility(symbol, exp, strike, right, date)` | IV on each trade |
| `option_at_time_trade(symbol, exp, strike, right, start, end, time)` | Trade at specific time |
| `option_at_time_quote(symbol, exp, strike, right, start, end, time)` | Quote at specific time |

### Index Methods (9)

| Method | Description |
|--------|-------------|
| `index_list_symbols()` | All index symbols |
| `index_list_dates(symbol)` | Available dates for an index |
| `index_snapshot_ohlc(symbol)` | Latest OHLC snapshot |
| `index_snapshot_price(symbol)` | Latest price snapshot |
| `index_snapshot_market_value(symbol)` | Latest market value snapshot |
| `index_history_eod(symbol, start, end)` | End-of-day index data |
| `index_history_ohlc(symbol, start, end, interval)` | Intraday OHLC bars |
| `index_history_price(symbol, date, interval)` | Intraday price history |
| `index_at_time_price(symbol, start, end, time)` | Price at specific time |

### Calendar Methods (3)

| Method | Description |
|--------|-------------|
| `calendar_open_today()` | Is the market open today? |
| `calendar_on_date(date)` | Calendar info for a date |
| `calendar_year(year)` | Calendar for an entire year |

### Rate Methods (1)

| Method | Description |
|--------|-------------|
| `interest_rate_history_eod(symbol, start, end)` | Interest rate EOD history |

## DataFrame Support

### to_dataframe

Convert any result to a pandas DataFrame:

```python
from thetadatadx import Credentials, Config, DirectClient, to_dataframe

creds = Credentials.from_file("creds.txt")
client = DirectClient(creds, Config.production())

eod = client.stock_history_eod("AAPL", "20240101", "20240301")
df = to_dataframe(eod)
print(df.head())
print(df.describe())
```

### _df Convenience Methods

All 61 data methods have `_df` variants that return DataFrames directly:

```python
df = client.stock_history_eod_df("AAPL", "20240101", "20240301")
df = client.stock_history_ohlc_df("AAPL", "20240315", "60000")
df = client.option_list_expirations_df("SPY")
```

Requires `pip install thetadatadx[pandas]`.

## Greeks Calculator

```python
from thetadatadx import all_greeks, implied_volatility

# All 22 Greeks at once
g = all_greeks(
    spot=450.0, strike=455.0, rate=0.05,
    div_yield=0.015, tte=30/365, option_price=8.50, is_call=True
)
print(f"IV={g['iv']:.4f} Delta={g['delta']:.4f} Gamma={g['gamma']:.6f}")
print(f"Theta={g['theta']:.4f} Vega={g['vega']:.4f} Rho={g['rho']:.4f}")

# Just IV
iv, err = implied_volatility(450.0, 455.0, 0.05, 0.015, 30/365, 8.50, True)
print(f"IV: {iv:.4f}, Error: {err:.6f}")
```

## FPSS Streaming

Real-time market data via ThetaData's FPSS servers:

```python
from thetadatadx import Credentials, FpssClient

creds = Credentials.from_file("creds.txt")
fpss = FpssClient(creds, buffer_size=1024)

# Subscribe to real-time data
fpss.subscribe("AAPL", "QUOTE")
fpss.subscribe("SPY", "TRADE")

# Poll for events
while True:
    event = fpss.next_event(timeout_ms=5000)
    if event is None:
        break  # timeout
    if event["type"] == "quote":
        print(f"Quote: {event['contract']} bid={event['bid']} ask={event['ask']}")
    elif event["type"] == "trade":
        print(f"Trade: {event['contract']} price={event['price']} size={event['size']}")

fpss.shutdown()
```

### FpssClient API

| Method | Description |
|--------|-------------|
| `FpssClient(creds, buffer_size=1024)` | Connect and authenticate |
| `subscribe(symbol, data_type)` | Subscribe (`"QUOTE"`, `"TRADE"`, `"OI"`) |
| `next_event(timeout_ms=5000)` | Poll next event (dict or None) |
| `shutdown()` | Graceful shutdown |

## Architecture

```
Python code
    |  (PyO3 FFI)
    v
thetadatadx Rust crate
    |  (tonic gRPC / TLS TCP)
    v
ThetaData servers
```

No HTTP middleware, no Java terminal, no subprocess. Direct wire protocol access at Rust speed.
