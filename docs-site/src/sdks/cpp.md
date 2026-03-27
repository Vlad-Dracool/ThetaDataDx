# C++ SDK

C++ SDK for ThetaData market data, powered by the `thetadatadx` Rust crate via C FFI.

**This is NOT a C++ reimplementation.** Every call goes through compiled Rust via a C FFI layer. gRPC communication, protobuf parsing, zstd decompression, and TCP streaming all happen at native Rust speed. C++ is just the interface.

## Prerequisites

- C++17 compiler
- CMake 3.16+
- Rust toolchain (for building the FFI library)

## Building

First, build the Rust FFI library:

```bash
git clone https://github.com/userFRM/ThetaDataDx.git
cd ThetaDataDx
cargo build --release -p thetadatadx-ffi
```

Then build the C++ SDK:

```bash
cd sdks/cpp
mkdir build && cd build
cmake ..
make
```

Run the example:

```bash
./thetadatadx_example
```

## Quick Start

```cpp
#include "thetadatadx.hpp"
#include <iostream>

int main() {
    // Load credentials
    auto creds = tdx::Credentials::from_file("creds.txt");

    // Connect
    auto client = tdx::Client::connect(creds, tdx::Config::production());

    // Fetch EOD data
    auto eod = client.stock_history_eod("AAPL", "20240101", "20240301");
    for (auto& tick : eod) {
        std::cout << tick.date << ": O=" << tick.open
                  << " H=" << tick.high << " L=" << tick.low
                  << " C=" << tick.close << std::endl;
    }

    // Greeks (no server connection needed)
    auto g = tdx::all_greeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true);
    std::cout << "IV=" << g.iv << " Delta=" << g.delta << std::endl;
}
```

## API

### Credentials

```cpp
// From file (line 1 = email, line 2 = password)
auto creds = tdx::Credentials::from_file("creds.txt");

// Direct construction
auto creds = tdx::Credentials::from_email("email@example.com", "password");
```

### Config

```cpp
auto config = tdx::Config::production();  // production servers
auto config = tdx::Config::dev();         // dev servers
```

### Client

RAII class. All methods throw `std::runtime_error` on failure.

```cpp
auto client = tdx::Client::connect(creds, tdx::Config::production());
```

| Method | Returns | Description |
|--------|---------|-------------|
| `stock_list_symbols()` | `vector<string>` | All stock symbols |
| `stock_history_eod(symbol, start, end)` | `vector<EodTick>` | EOD data |
| `stock_history_ohlc(symbol, date, interval)` | `vector<OhlcTick>` | Intraday OHLC |
| `stock_history_trade(symbol, date)` | `vector<TradeTick>` | All trades |
| `stock_history_quote(symbol, date, interval)` | `vector<QuoteTick>` | NBBO quotes |
| `stock_snapshot_quote(symbols)` | `vector<QuoteTick>` | Live quote snapshot |
| `option_list_expirations(symbol)` | `vector<string>` | Expiration dates |
| `option_list_strikes(symbol, exp)` | `vector<string>` | Strike prices |
| `option_list_symbols()` | `vector<string>` | Option underlyings |
| `index_list_symbols()` | `vector<string>` | Index symbols |

### Standalone Functions

```cpp
// All 22 Greeks
auto g = tdx::all_greeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true);
std::cout << "IV=" << g.iv << " Delta=" << g.delta << " Gamma=" << g.gamma;

// Just IV
auto [iv, err] = tdx::implied_volatility(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true);
```

## FPSS Streaming

Real-time market data:

```cpp
#include "thetadatadx.hpp"
#include <iostream>

int main() {
    auto creds = tdx::Credentials::from_file("creds.txt");
    auto fpss = tdx::FpssClient::connect(creds, 1024);

    // Subscribe to real-time quotes
    int32_t req_id = fpss.subscribe_quotes("AAPL", tdx::SecType::Stock);
    std::cout << "Subscribed (req_id=" << req_id << ")" << std::endl;

    // Subscribe to trades
    fpss.subscribe_trades("MSFT", tdx::SecType::Stock);

    // Poll for events
    while (auto event = fpss.next_event(5000)) {
        std::cout << "Event type: " << event->type() << std::endl;
        if (event->type() == tdx::FpssEventType::Quote) {
            std::cout << "Quote: " << event->contract()
                      << " bid=" << event->bid()
                      << " ask=" << event->ask() << std::endl;
        } else if (event->type() == tdx::FpssEventType::Trade) {
            std::cout << "Trade: " << event->contract()
                      << " price=" << event->price()
                      << " size=" << event->size() << std::endl;
        }
    }

    fpss.shutdown();
}
```

### FpssClient API

| Method | Signature | Description |
|--------|-----------|-------------|
| `connect` | `(creds, buf_size) -> FpssClient` | Static factory, connect + auth |
| `subscribe_quotes` | `(root, sec_type) -> int32_t` | Subscribe to quotes |
| `subscribe_trades` | `(root, sec_type) -> int32_t` | Subscribe to trades |
| `subscribe_open_interest` | `(root, sec_type) -> int32_t` | Subscribe to open interest |
| `next_event` | `(timeout_ms) -> unique_ptr<FpssEvent>` | Poll next event (nullptr on timeout) |
| `shutdown` | `() -> void` | Graceful shutdown |

### Security Type Enum

```cpp
tdx::SecType::Stock   // 0
tdx::SecType::Option  // 1
tdx::SecType::Index   // 2
tdx::SecType::Rate    // 3
```

## Memory Management

The C++ SDK uses RAII wrappers around the C FFI handles. All objects automatically free their underlying resources when destroyed. No manual memory management required.

```cpp
{
    auto client = tdx::Client::connect(creds, tdx::Config::production());
    // ... use client ...
}  // client automatically freed here
```

## Architecture

```
C++ code
    |  (RAII wrappers)
    v
thetadatadx.h (C FFI)
    |
    v
libthetadatadx_ffi.so / .a
    |  (Rust FFI crate)
    v
thetadatadx Rust crate
    |  (tonic gRPC / tokio TCP)
    v
ThetaData servers
```

## C FFI Layer

The raw C interface can be used directly from any language with C interop (Swift, Zig, Nim, etc.):

| Category | Functions |
|----------|-----------|
| Lifecycle | `tdx_credentials_new`, `tdx_credentials_from_file`, `tdx_credentials_free` |
| Config | `tdx_config_production`, `tdx_config_dev`, `tdx_config_free` |
| Client | `tdx_client_connect`, `tdx_client_free` |
| Greeks | `tdx_all_greeks`, `tdx_implied_volatility` |
| FPSS | `tdx_fpss_connect`, `tdx_fpss_subscribe_*`, `tdx_fpss_next_event`, `tdx_fpss_shutdown`, `tdx_fpss_free` |
| Memory | `tdx_string_free`, `tdx_last_error` |

Results are returned as JSON strings (`*mut c_char`) that must be freed with `tdx_string_free`.
