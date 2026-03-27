# CLI Tool (tdx)

Command-line interface for querying ThetaData market data. No JVM required.

## Installation

```bash
# From source
cargo install --path crates/thetadatadx-cli

# Or build from the workspace root
cargo build --release -p thetadatadx-cli
# binary at target/release/tdx
```

## Setup

Create a `creds.txt` file with your ThetaData credentials:

```
your-email@example.com
your-password
```

## Global Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--creds <path>` | `creds.txt` | Credentials file |
| `--config <preset>` | `production` | `production` or `dev` |
| `--format <fmt>` | `table` | `table`, `json`, or `csv` |

## Output Formats

```bash
tdx stock history_eod AAPL 20240101 20240301                  # pretty table (default)
tdx stock history_eod AAPL 20240101 20240301 --format json     # JSON array
tdx stock history_eod AAPL 20240101 20240301 --format csv      # CSV
```

## Commands

### Authentication

```bash
tdx auth --creds creds.txt
```

### Stock (14 subcommands)

```bash
# List
tdx stock list_symbols
tdx stock list_dates EOD AAPL

# History
tdx stock history_eod AAPL 20240101 20240301
tdx stock history_ohlc AAPL 20240315 60000              # 1-min bars
tdx stock history_ohlc_range AAPL 20240101 20240301 60000
tdx stock history_trade AAPL 20240315
tdx stock history_quote AAPL 20240315 60000
tdx stock history_trade_quote AAPL 20240315

# Snapshots
tdx stock snapshot_ohlc AAPL,MSFT,GOOGL
tdx stock snapshot_trade AAPL,MSFT,GOOGL
tdx stock snapshot_quote AAPL,MSFT,GOOGL
tdx stock snapshot_market_value AAPL,MSFT

# AtTime
tdx stock at_time_trade AAPL 20240101 20240301 34200000   # 9:30 AM
tdx stock at_time_quote AAPL 20240101 20240301 34200000
```

### Options (34 subcommands)

```bash
# List
tdx option list_symbols
tdx option list_expirations SPY
tdx option list_strikes SPY 20240419
tdx option list_dates EOD SPY 20240419 500000 C
tdx option list_contracts EOD SPY 20240315

# History
tdx option history_eod SPY 20240419 500000 C 20240101 20240301
tdx option history_ohlc SPY 20240419 500000 C 20240315 60000
tdx option history_trade SPY 20240419 500000 C 20240315
tdx option history_quote SPY 20240419 500000 C 20240315 60000
tdx option history_trade_quote SPY 20240419 500000 C 20240315
tdx option history_open_interest SPY 20240419 500000 C 20240315

# Snapshots
tdx option snapshot_ohlc SPY 20240419 500000 C
tdx option snapshot_trade SPY 20240419 500000 C
tdx option snapshot_quote SPY 20240419 500000 C
tdx option snapshot_open_interest SPY 20240419 500000 C
tdx option snapshot_market_value SPY 20240419 500000 C

# Snapshot Greeks
tdx option snapshot_greeks_implied_volatility SPY 20240419 500000 C
tdx option snapshot_greeks_all SPY 20240419 500000 C
tdx option snapshot_greeks_first_order SPY 20240419 500000 C
tdx option snapshot_greeks_second_order SPY 20240419 500000 C
tdx option snapshot_greeks_third_order SPY 20240419 500000 C

# History Greeks
tdx option history_greeks_eod SPY 20240419 500000 C 20240101 20240301
tdx option history_greeks_all SPY 20240419 500000 C 20240315 60000
tdx option history_greeks_first_order SPY 20240419 500000 C 20240315 60000
tdx option history_greeks_second_order SPY 20240419 500000 C 20240315 60000
tdx option history_greeks_third_order SPY 20240419 500000 C 20240315 60000
tdx option history_greeks_implied_volatility SPY 20240419 500000 C 20240315 60000

# Trade Greeks
tdx option history_trade_greeks_all SPY 20240419 500000 C 20240315
tdx option history_trade_greeks_first_order SPY 20240419 500000 C 20240315
tdx option history_trade_greeks_second_order SPY 20240419 500000 C 20240315
tdx option history_trade_greeks_third_order SPY 20240419 500000 C 20240315
tdx option history_trade_greeks_implied_volatility SPY 20240419 500000 C 20240315

# AtTime
tdx option at_time_trade SPY 20240419 500000 C 20240101 20240301 34200000
tdx option at_time_quote SPY 20240419 500000 C 20240101 20240301 34200000
```

### Indices (9 subcommands)

```bash
tdx index list_symbols
tdx index list_dates SPX
tdx index history_eod SPX 20240101 20240301
tdx index history_ohlc SPX 20240101 20240301 60000
tdx index history_price SPX 20240315 60000
tdx index snapshot_ohlc SPX,NDX,RUT
tdx index snapshot_price SPX,NDX,RUT
tdx index snapshot_market_value SPX,NDX,RUT
tdx index at_time_price SPX 20240101 20240301 34200000
```

### Interest Rates (1 subcommand)

```bash
tdx rate history_eod SOFR 20240101 20240301
```

### Calendar (3 subcommands)

```bash
tdx calendar open_today
tdx calendar year 2024
tdx calendar on_date 20240315
```

### Offline Greeks (2 subcommands)

No server connection needed for these commands.

```bash
# All 22 Greeks (parameters: spot strike rate div_yield tte price is_call)
tdx greeks 450 450 0.05 0.015 0.082 8.5 call

# Just IV
tdx iv 450 450 0.05 0.015 0.082 8.5 call
```

## Endpoint Coverage

All 61 DirectClient endpoints plus 2 offline commands:

| Category | Count | Subcommands |
|----------|-------|-------------|
| Stock | 14 | `list_symbols`, `list_dates`, `history_eod`, `history_ohlc`, `history_ohlc_range`, `history_trade`, `history_quote`, `history_trade_quote`, `snapshot_ohlc`, `snapshot_trade`, `snapshot_quote`, `snapshot_market_value`, `at_time_trade`, `at_time_quote` |
| Option | 34 | `list_symbols`, `list_dates`, `list_expirations`, `list_strikes`, `list_contracts`, `snapshot_ohlc`, `snapshot_trade`, `snapshot_quote`, `snapshot_open_interest`, `snapshot_market_value`, `snapshot_greeks_implied_volatility`, `snapshot_greeks_all`, `snapshot_greeks_first_order`, `snapshot_greeks_second_order`, `snapshot_greeks_third_order`, `history_eod`, `history_ohlc`, `history_trade`, `history_quote`, `history_trade_quote`, `history_open_interest`, `history_greeks_eod`, `history_greeks_all`, `history_trade_greeks_all`, `history_greeks_first_order`, `history_trade_greeks_first_order`, `history_greeks_second_order`, `history_trade_greeks_second_order`, `history_greeks_third_order`, `history_trade_greeks_third_order`, `history_greeks_implied_volatility`, `history_trade_greeks_implied_volatility`, `at_time_trade`, `at_time_quote` |
| Index | 9 | `list_symbols`, `list_dates`, `history_eod`, `history_ohlc`, `history_price`, `snapshot_ohlc`, `snapshot_price`, `snapshot_market_value`, `at_time_price` |
| Rate | 1 | `history_eod` |
| Calendar | 3 | `open_today`, `on_date`, `year` |
| Offline | 2 | `greeks`, `iv` |

## Scripting Examples

### Export EOD data to CSV

```bash
tdx stock history_eod AAPL 20240101 20240301 --format csv > aapl_eod.csv
```

### Scan multiple symbols

```bash
for symbol in AAPL MSFT GOOGL AMZN META; do
    echo "=== $symbol ==="
    tdx stock snapshot_quote "$symbol" --format json
done
```

### Get option chain for nearest expiration

```bash
# Get first expiration
EXP=$(tdx option list_expirations SPY --format json | jq -r '.[0]')
echo "Expiration: $EXP"

# Get strikes
tdx option list_strikes SPY "$EXP" --format json | jq '.'
```

### Daily data pipeline

```bash
#!/bin/bash
DATE=$(date +%Y%m%d)
tdx stock history_eod AAPL "$DATE" "$DATE" --format csv >> aapl_daily.csv
tdx stock history_eod MSFT "$DATE" "$DATE" --format csv >> msft_daily.csv
```
