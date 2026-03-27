# REST Server

The `thetadatadx-server` binary is a drop-in replacement for the ThetaData Java Terminal. It runs a local HTTP REST server (default port 25510) and WebSocket server (default port 25520) that expose the same API as the Java terminal. Existing clients (Python SDK, Excel add-ins, curl scripts, browsers) connect without any code changes.

## Installation

```bash
# From source
cargo install --path crates/thetadatadx-server

# Or build from the workspace root
cargo build --release -p thetadatadx-server
# binary at target/release/thetadatadx-server
```

## Quick Start

```bash
thetadatadx-server --creds creds.txt
```

This starts:
- **HTTP REST API** on `http://127.0.0.1:25510` (all `/v2/...` routes)
- **WebSocket server** on `ws://127.0.0.1:25520/v1/events`

## CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--creds <path>` | `creds.txt` | Credentials file (email line 1, password line 2) |
| `--http-port <port>` | `25510` | HTTP REST API port |
| `--ws-port <port>` | `25520` | WebSocket server port |
| `--bind <addr>` | `127.0.0.1` | Bind address for both servers |
| `--log-level <level>` | `info` | Log level (e.g. `debug`, `trace`, `thetadatadx=trace`) |

## Architecture

```
External apps (Python SDK, Excel, browsers)
    |
    |--- HTTP REST :25510 (/v2/...)
    |--- WebSocket :25520 (/v1/events)
    |
thetadatadx-server (this binary)
    |
    |--- DirectClient (MDDS gRPC) for historical data
    |--- FpssClient (FPSS TCP) for real-time streaming
    |
ThetaData upstream servers
```

## REST API

All routes are auto-generated from the endpoint registry, giving full coverage of all 61 DirectClient endpoints (61 data routes + 3 system routes = 64 total HTTP routes). Routes follow the Java terminal's URL patterns.

### URL Patterns

| Pattern | Example |
|---------|---------|
| `/v2/list/roots/{sec_type}` | `/v2/list/roots/stock` |
| `/v2/list/dates/{sec_type}` | `/v2/list/dates/stock?request_type=EOD&symbol=AAPL` |
| `/v2/list/{what}` | `/v2/list/expirations?root=SPY` |
| `/v2/hist/{sec_type}/{what}` | `/v2/hist/stock/eod?root=AAPL&start_date=20240101&end_date=20240301` |
| `/v2/hist/{sec_type}/greeks/{what}` | `/v2/hist/option/greeks/all?root=SPY&exp=20240419&strike=500000&right=C` |
| `/v2/hist/{sec_type}/trade_greeks/{what}` | `/v2/hist/option/trade_greeks/all?root=SPY&exp=20240419&strike=500000&right=C` |
| `/v2/snapshot/{sec_type}/{what}` | `/v2/snapshot/stock/quote?root=AAPL` |
| `/v2/snapshot/{sec_type}/greeks/{what}` | `/v2/snapshot/option/greeks/all?root=SPY&exp=20240419&strike=500000&right=C` |
| `/v2/at_time/{sec_type}/{what}` | `/v2/at_time/stock/trade?root=AAPL&start_date=...&end_date=...&time_of_day=34200000` |
| `/v2/calendar/{what}` | `/v2/calendar/open_today` |
| `/v2/system/{what}` | `/v2/system/mdds/status` |

### All 61 Data Routes

**Stock (14)**:
`/v2/list/roots/stock`, `/v2/list/dates/stock`, `/v2/snapshot/stock/ohlc`, `/v2/snapshot/stock/trade`, `/v2/snapshot/stock/quote`, `/v2/snapshot/stock/market_value`, `/v2/hist/stock/eod`, `/v2/hist/stock/ohlc`, `/v2/hist/stock/ohlc_range`, `/v2/hist/stock/trade`, `/v2/hist/stock/quote`, `/v2/hist/stock/trade_quote`, `/v2/at_time/stock/trade`, `/v2/at_time/stock/quote`

**Option (34)**:
`/v2/list/roots/option`, `/v2/list/dates/option`, `/v2/list/expirations`, `/v2/list/strikes`, `/v2/list/contracts`, `/v2/snapshot/option/ohlc`, `/v2/snapshot/option/trade`, `/v2/snapshot/option/quote`, `/v2/snapshot/option/open_interest`, `/v2/snapshot/option/market_value`, `/v2/snapshot/option/greeks/implied_volatility`, `/v2/snapshot/option/greeks/all`, `/v2/snapshot/option/greeks/first_order`, `/v2/snapshot/option/greeks/second_order`, `/v2/snapshot/option/greeks/third_order`, `/v2/hist/option/eod`, `/v2/hist/option/ohlc`, `/v2/hist/option/trade`, `/v2/hist/option/quote`, `/v2/hist/option/trade_quote`, `/v2/hist/option/open_interest`, `/v2/hist/option/greeks/eod`, `/v2/hist/option/greeks/all`, `/v2/hist/option/trade_greeks/all`, `/v2/hist/option/greeks/first_order`, `/v2/hist/option/trade_greeks/first_order`, `/v2/hist/option/greeks/second_order`, `/v2/hist/option/trade_greeks/second_order`, `/v2/hist/option/greeks/third_order`, `/v2/hist/option/trade_greeks/third_order`, `/v2/hist/option/greeks/implied_volatility`, `/v2/hist/option/trade_greeks/implied_volatility`, `/v2/at_time/option/trade`, `/v2/at_time/option/quote`

**Index (9)**:
`/v2/list/roots/index`, `/v2/list/dates/index`, `/v2/snapshot/index/ohlc`, `/v2/snapshot/index/price`, `/v2/snapshot/index/market_value`, `/v2/hist/index/eod`, `/v2/hist/index/ohlc`, `/v2/hist/index/price`, `/v2/at_time/index/price`

**Calendar (3)**:
`/v2/calendar/open_today`, `/v2/calendar/on_date`, `/v2/calendar/year`

**Rate (1)**:
`/v2/hist/rate/eod`

### System Routes (3)

| Route | Description |
|-------|-------------|
| `/v2/system/mdds/status` | MDDS gRPC connection status |
| `/v2/system/fpss/status` | FPSS streaming connection status |
| `/v2/system/shutdown` | Graceful shutdown |

### Response Envelope

Every response follows the Java terminal's JSON envelope:

```json
{
    "header": { "format": "json", "error_type": "null" },
    "response": [ ... ]
}
```

Error responses:

```json
{
    "header": { "format": "json", "error_type": "server_error" },
    "error": { "message": "..." }
}
```

## WebSocket API

The WebSocket server at `/v1/events` replicates the Java terminal's streaming protocol:

- Only one WebSocket client is allowed at a time
- STATUS heartbeat every second
- JSON event types: QUOTE, TRADE, OHLC, OPEN_INTEREST, STATUS
- Client subscribe/unsubscribe commands forwarded to FPSS

## Security

- CORS is restricted to the same-origin (the bind address and HTTP port)
- Both servers bind to `127.0.0.1` by default (localhost only)

## Key Design

1. **Single client instance** -- authenticates once at startup, shares `DirectClient` across all handlers
2. **Registry-driven routes** -- all 61 endpoints auto-registered from `endpoint_registry::ENDPOINTS`
3. **FPSS bridge** -- real-time streaming events forwarded from `FpssClient` to WebSocket clients
4. **Java terminal compatibility** -- identical JSON envelope format, URL patterns, and WebSocket protocol
