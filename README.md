# ThetaDataDx

No-JVM ThetaData Terminal — native Rust SDK for direct market data access.

[![build](https://github.com/userFRM/ThetaDataDx/actions/workflows/ci.yml/badge.svg)](https://github.com/userFRM/ThetaDataDx/actions/workflows/ci.yml)
[![Documentation](https://img.shields.io/docsrs/thetadatadx)](https://docs.rs/thetadatadx)
[![license](https://img.shields.io/github/license/userFRM/ThetaDataDx?color=blue)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/thetadatadx.svg)](https://crates.io/crates/thetadatadx)
[![PyPI](https://img.shields.io/pypi/v/thetadatadx)](https://pypi.org/project/thetadatadx)
[![Discord](https://img.shields.io/badge/join_Discord-community-5865F2.svg?logo=discord&logoColor=white)](https://discord.thetadata.us/)

## Overview

`thetadatadx` connects directly to ThetaData's upstream servers — MDDS for historical data and FPSS for real-time streaming — entirely in native Rust. No JVM terminal process, no local Java dependency, no subprocess management. Your application talks to ThetaData's infrastructure with the same wire protocol their own terminal uses.

> [!IMPORTANT]
> A valid [ThetaData](https://thetadata.us) subscription is required. This SDK authenticates against ThetaData's Nexus API using your account credentials.

## Features

- **Historical data** via MDDS/gRPC — EOD, OHLC, trades, quotes across stocks, options, and indices
- **Real-time streaming** via FPSS/TCP — live quotes, trades, open interest, and OHLC snapshots
- **Full Greeks calculator** — 22 Black-Scholes Greeks (first, second, and third order) plus IV solver
- **Zero-copy tick types** — `TradeTick`, `QuoteTick`, `OhlcTick`, `EodTick` with fixed-point `Price` encoding
- **Async/await** throughout — built on Tokio with concurrent gRPC streaming and background heartbeat tasks
- **Direct authentication** — handles the Nexus API auth flow, session management, and reconnection logic
- **FIT codec** — native decoder for ThetaData's nibble-encoded delta-compressed tick format
- **Multi-language SDKs** — Python (PyO3), Go (CGo), C++ (RAII), all powered by the Rust core

## Installation

### Rust

```toml
[dependencies]
thetadatadx = "1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Python

```sh
pip install thetadatadx
```

## Quick Start

> [!TIP]
> Create a `creds.txt` file with your ThetaData email on line 1 and password on line 2. This is the same format the Java terminal uses.

```rust
use thetadatadx::{DirectClient, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let client = DirectClient::connect(&creds, DirectConfig::production()).await?;

    // Fetch end-of-day stock data
    let eod = client.stock_history_eod("AAPL", "20240101", "20240301").await?;
    for tick in &eod {
        println!("{}: O={} H={} L={} C={} V={}",
            tick.date, tick.open_price(), tick.high_price(),
            tick.low_price(), tick.close_price(), tick.volume);
    }

    // List option expirations
    let exps = client.option_list_expirations("SPY").await?;
    println!("SPY expirations: {:?}", &exps[..5.min(exps.len())]);

    Ok(())
}
```

## Streaming Example

> [!NOTE]
> FPSS streaming connects to ThetaData's dedicated streaming servers via TLS/TCP. The client automatically sends heartbeat pings every 100ms as required by the protocol.

```rust
use thetadatadx::auth::Credentials;
use thetadatadx::fpss::{FpssClient, FpssEvent};
use thetadatadx::fpss::protocol::Contract;

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let (client, mut events) = FpssClient::connect(&creds, 1024).await?;

    let req_id = client.subscribe_quotes(&Contract::stock("AAPL")).await?;
    println!("Subscribed (req_id={req_id})");

    while let Some(event) = events.recv().await {
        match event {
            FpssEvent::QuoteData { payload } => {
                println!("Quote data: {} bytes", payload.len());
            }
            FpssEvent::TradeData { payload } => {
                println!("Trade data: {} bytes", payload.len());
            }
            FpssEvent::ContractAssigned { id, contract } => {
                println!("Contract {id} = {contract}");
            }
            _ => {}
        }
    }
    Ok(())
}
```

## Supported Endpoints

### Stock

| Category | Method | Description |
|----------|--------|-------------|
| List | `stock_list_symbols()` | All available stock symbols |
| History | `stock_history_eod()` | End-of-day data for a date range |
| History | `stock_history_ohlc()` | Intraday OHLC bars for a single date |
| History | `stock_history_ohlc_range()` | Intraday OHLC bars across a date range |
| History | `stock_history_trade()` | All trades on a given date |
| History | `stock_history_quote()` | NBBO quotes at a given interval |
| History | `stock_history_trade_quote()` | Combined trade + quote ticks |
| Snapshot | `stock_snapshot_quote()` | Latest NBBO quote for one or more symbols |
| Snapshot | `stock_snapshot_ohlc()` | Latest OHLC snapshot |
| Snapshot | `stock_snapshot_trade()` | Latest trade snapshot |

### Option

| Category | Method | Description |
|----------|--------|-------------|
| List | `option_list_symbols()` | All available option underlyings |
| List | `option_list_expirations()` | Expiration dates for an underlying |
| List | `option_list_strikes()` | Strike prices for a given expiration |
| History | `option_history_eod()` | End-of-day option data |
| History | `option_history_ohlc()` | Intraday option OHLC bars |
| History | `option_history_trade()` | Option trades on a given date |
| History | `option_history_quote()` | Option NBBO quotes |

### Index

| Category | Method | Description |
|----------|--------|-------------|
| List | `index_list_symbols()` | All available index symbols |
| History | `index_history_eod()` | End-of-day index data |

### Streaming (FPSS)

| Method | Description |
|--------|-------------|
| `subscribe_quotes()` | Real-time quote stream for a contract |
| `subscribe_trades()` | Real-time trade stream for a contract |
| `subscribe_open_interest()` | Real-time open interest for a contract |
| `subscribe_full_trades()` | All trades for a security type |
| `unsubscribe_quotes()` | Stop quote stream |
| `unsubscribe_trades()` | Stop trade stream |
| `unsubscribe_open_interest()` | Stop open interest stream |

### Greeks

| Function | Description |
|----------|-------------|
| `greeks::all_greeks()` | Compute all 22 Greeks + IV in one call |
| `greeks::implied_volatility()` | IV solver via bisection |
| `greeks::delta()` | First-order: delta |
| `greeks::gamma()` | Second-order: gamma |
| `greeks::theta()` | First-order: theta (daily) |
| `greeks::vega()` | First-order: vega |
| `greeks::rho()` | First-order: rho |
| `greeks::vanna()` | Second-order: vanna |
| `greeks::charm()` | Second-order: charm |
| `greeks::vomma()` | Second-order: vomma |
| `greeks::speed()` | Third-order: speed |
| `greeks::zomma()` | Third-order: zomma |
| `greeks::color()` | Third-order: color |
| `greeks::ultima()` | Third-order: ultima |

## Configuration

```rust
use thetadatadx::DirectConfig;

// Production (ThetaData NJ datacenter, gRPC over TLS)
let config = DirectConfig::production();

// Dev (same servers, shorter timeouts for faster iteration)
let config = DirectConfig::dev();

// Custom configuration (override specific fields)
let config = DirectConfig {
    fpss_timeout_ms: 5_000,
    reconnect_wait_ms: 2_000,
    ..DirectConfig::production()
};
```

> [!TIP]
> Credentials can be loaded from a file, environment variables, or constructed directly:
> ```rust
> let creds = Credentials::from_file("creds.txt")?;
> let creds = Credentials::new(std::env::var("THETA_EMAIL")?, std::env::var("THETA_PASS")?);
> ```

## Documentation

- **[API Reference](docs/api-reference.md)** — All client methods, tick types, enums, and configuration options
- **[Architecture](docs/architecture.md)** — System design, protocol specifications, wire formats, and auth flow
- **[JVM Deviations](docs/jvm-deviations.md)** — Intentional differences from the Java terminal
- **[Reverse-Engineering Guide](docs/reverse-engineering.md)** — How to decompile the terminal JAR and extract protocol definitions

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Disclaimer

> [!CAUTION]
> Theta Data, ThetaData, and Theta Terminal are trademarks of Theta Data, Inc. / AxiomX LLC. This project is **not affiliated with, endorsed by, or supported by Theta Data**.

ThetaDataDx is an independent, open-source project provided "as is", without warranty of any kind.

### How ThetaDataDx Was Built

ThetaDataDx was developed through independent analysis of the ThetaData Terminal JAR and its network protocol. The protocol implementation was built from scratch in Rust based on decompiled Java source and observed wire-level behavior. This approach is consistent with the principle of interoperability through protocol analysis — the same method used by projects like Samba (SMB/CIFS), open-source Exchange clients, and countless other third-party implementations of proprietary network protocols.

### Legal Considerations

> [!WARNING]
> - **No warranty.** ThetaDataDx is provided "as is", without warranty of any kind. See [LICENSE](./LICENSE) for full terms.
> - **Use at your own risk.** Users are solely responsible for ensuring their use complies with ThetaData's Terms of Service and any applicable laws or regulations. Using ThetaDataDx may carry risks including but not limited to account restriction or termination.
> - **Not financial software.** ThetaDataDx is a research and interoperability project. It is not intended as a replacement for officially supported ThetaData software in production trading environments. The authors accept no liability for financial losses, missed trades, or any other damages arising from the use of this software.
> - **Protocol stability.** ThetaDataDx relies on an undocumented protocol that ThetaData may change at any time without notice. There is no guarantee of continued functionality.

### EU Interoperability

For users and contributors in the European Union: Article 6 of the EU Software Directive (2009/24/EC) permits reverse engineering for the purpose of achieving interoperability with independently created software, provided that specific conditions are met. ThetaDataDx was developed with this legal framework in mind, enabling interoperability with ThetaData's market data infrastructure on platforms where the official Java-based Terminal cannot run (headless Linux, containers, embedded systems, native Rust/Go/C++ applications).

## License

GPL-3.0-or-later — see [LICENSE](./LICENSE).
