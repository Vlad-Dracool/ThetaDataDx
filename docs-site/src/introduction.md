# ThetaDataDx

**No-JVM ThetaData Terminal -- native Rust SDK for direct market data access.**

[![build](https://github.com/userFRM/ThetaDataDx/actions/workflows/ci.yml/badge.svg)](https://github.com/userFRM/ThetaDataDx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/thetadatadx.svg)](https://crates.io/crates/thetadatadx)
[![PyPI](https://img.shields.io/pypi/v/thetadatadx)](https://pypi.org/project/thetadatadx)
[![license](https://img.shields.io/github/license/userFRM/ThetaDataDx?color=blue)](https://github.com/userFRM/ThetaDataDx/blob/main/LICENSE)

## What is ThetaDataDx?

ThetaDataDx connects directly to ThetaData's upstream servers -- MDDS for historical data and FPSS for real-time streaming -- entirely in native Rust. No JVM terminal process, no local Java dependency, no subprocess management. Your application talks to ThetaData's infrastructure with the same wire protocol their own terminal uses.

> **A valid [ThetaData](https://thetadata.us) subscription is required.** This SDK authenticates against ThetaData's Nexus API using your account credentials.

## Key Features

- **Historical data** via MDDS/gRPC -- EOD, OHLC, trades, quotes across stocks, options, and indices
- **Real-time streaming** via FPSS/TCP -- live quotes, trades, open interest, and OHLC snapshots
- **Full Greeks calculator** -- 22 Black-Scholes Greeks (first, second, and third order) plus IV solver
- **Zero-copy tick types** -- `TradeTick`, `QuoteTick`, `OhlcTick`, `EodTick` with fixed-point `Price` encoding
- **Async/await** throughout -- built on Tokio with concurrent gRPC streaming and background heartbeat tasks
- **Direct authentication** -- handles the Nexus API auth flow, session management, and reconnection logic
- **FIT codec** -- native decoder for ThetaData's nibble-encoded delta-compressed tick format
- **Multi-language SDKs** -- Python (PyO3), Go (CGo), C++ (RAII), all powered by the Rust core
- **pandas/polars DataFrame support** -- `to_dataframe()` and `_df` convenience methods in the Python SDK

## 61 Endpoints, 4 Languages

| Category | Endpoints |
|----------|-----------|
| Stock | 14 methods (list, snapshot, history, at-time) |
| Option | 34 methods (list, snapshot, history, Greeks, at-time) |
| Index | 9 methods |
| Rate | 1 method |
| Calendar | 3 methods |
| Streaming | 7 subscription methods (FPSS) |
| Greeks | `all_greeks()` + 20 individual functions |

Every endpoint is available in Rust, Python, Go, and C++. The CLI tool (`tdx`) exposes all endpoints from the command line.

## Architecture at a Glance

```
Your Application (Rust / Python / Go / C++)
    |
    +-- DirectClient (MDDS) ---> ThetaData gRPC servers (historical)
    |
    +-- FpssClient (FPSS) ----> ThetaData TCP servers (real-time)
    |
    +-- Greeks calculator -----> Local computation (no network)
```

No Java runtime. No JVM terminal process. No subprocess. Direct wire-protocol access.

## Getting Started

Head to the [Installation](getting-started/installation.md) page to set up ThetaDataDx, or jump straight to the [Quick Start](getting-started/quick-start.md) guide.

## Disclaimer

> Theta Data, ThetaData, and Theta Terminal are trademarks of Theta Data, Inc. / AxiomX LLC. This project is **not affiliated with, endorsed by, or supported by Theta Data**.

ThetaDataDx is an independent, open-source project provided "as is", without warranty of any kind. See the [LICENSE](https://github.com/userFRM/ThetaDataDx/blob/main/LICENSE) for full terms.

### Legal Considerations

- **No warranty.** ThetaDataDx is provided "as is", without warranty of any kind.
- **Use at your own risk.** Users are solely responsible for ensuring their use complies with ThetaData's Terms of Service and any applicable laws or regulations.
- **Not financial software.** ThetaDataDx is a research and interoperability project. It is not intended as a replacement for officially supported ThetaData software in production trading environments.
- **Protocol stability.** ThetaDataDx relies on an undocumented protocol that ThetaData may change at any time without notice.

### EU Interoperability

For users and contributors in the European Union: Article 6 of the EU Software Directive (2009/24/EC) permits reverse engineering for the purpose of achieving interoperability with independently created software, provided that specific conditions are met. ThetaDataDx was developed with this legal framework in mind, enabling interoperability with ThetaData's market data infrastructure on platforms where the official Java-based Terminal cannot run (headless Linux, containers, embedded systems, native Rust/Go/C++ applications).

## License

GPL-3.0-or-later
