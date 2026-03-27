# Go SDK

Go SDK for ThetaData market data, powered by the `thetadatadx` Rust crate via CGo FFI.

**This is NOT a Go reimplementation.** Every call goes through compiled Rust via a C FFI layer. gRPC communication, protobuf parsing, zstd decompression, and TCP streaming all happen at native Rust speed. Go is just the interface.

## Prerequisites

- Go 1.21+
- Rust toolchain (for building the FFI library)
- C compiler (for CGo)

## Building

First, build the Rust FFI library:

```bash
git clone https://github.com/userFRM/ThetaDataDx.git
cd ThetaDataDx
cargo build --release -p thetadatadx-ffi
```

This produces `target/release/libthetadatadx_ffi.so` (Linux) or `libthetadatadx_ffi.dylib` (macOS).

Then use the Go module:

```bash
go get github.com/userFRM/ThetaDataDx/sdks/go
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"

    thetadatadx "github.com/userFRM/ThetaDataDx/sdks/go"
)

func main() {
    // Load credentials
    creds, err := thetadatadx.CredentialsFromFile("creds.txt")
    if err != nil {
        log.Fatal(err)
    }
    defer creds.Close()

    // Connect
    config := thetadatadx.ProductionConfig()
    defer config.Close()

    client, err := thetadatadx.Connect(creds, config)
    if err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    // Fetch EOD data
    eod, err := client.StockHistoryEOD("AAPL", "20240101", "20240301")
    if err != nil {
        log.Fatal(err)
    }
    for _, tick := range eod {
        fmt.Printf("%d: O=%.2f H=%.2f L=%.2f C=%.2f\n",
            tick.Date, tick.Open, tick.High, tick.Low, tick.Close)
    }
}
```

## API

### Credentials

```go
// From file (line 1 = email, line 2 = password)
creds, err := thetadatadx.CredentialsFromFile("creds.txt")
defer creds.Close()

// Direct construction
creds := thetadatadx.NewCredentials("email@example.com", "password")
defer creds.Close()
```

### Config

```go
config := thetadatadx.ProductionConfig()  // production servers
config := thetadatadx.DevConfig()         // dev servers
defer config.Close()
```

### Client

All data methods return typed Go structs (deserialized from JSON over FFI).

```go
client, err := thetadatadx.Connect(creds, config)
defer client.Close()
```

| Method | Returns | Description |
|--------|---------|-------------|
| `StockListSymbols()` | `[]string` | All stock symbols |
| `StockHistoryEOD(symbol, start, end)` | `[]EodTick` | EOD data |
| `StockHistoryOHLC(symbol, date, interval)` | `[]OhlcTick` | Intraday OHLC |
| `StockHistoryTrade(symbol, date)` | `[]TradeTick` | All trades |
| `StockHistoryQuote(symbol, date, interval)` | `[]QuoteTick` | NBBO quotes |
| `StockSnapshotQuote(symbols)` | `[]QuoteTick` | Live quote snapshot |
| `OptionListExpirations(symbol)` | `[]string` | Expiration dates |
| `OptionListStrikes(symbol, exp)` | `[]string` | Strike prices |
| `OptionListSymbols()` | `[]string` | Option underlyings |
| `IndexListSymbols()` | `[]string` | Index symbols |

### Greeks (Standalone Functions)

```go
// All 22 Greeks
g, err := thetadatadx.AllGreeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true)
fmt.Printf("IV=%.4f Delta=%.4f Gamma=%.6f\n", g.IV, g.Delta, g.Gamma)

// Just IV
iv, ivErr, err := thetadatadx.ImpliedVolatility(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true)
```

## FPSS Streaming

Real-time market data via ThetaData's FPSS servers:

```go
package main

import (
    "fmt"
    "log"

    thetadatadx "github.com/userFRM/ThetaDataDx/sdks/go"
)

func main() {
    creds, _ := thetadatadx.CredentialsFromFile("creds.txt")
    defer creds.Close()

    fpss, err := thetadatadx.FpssConnect(creds, 1024)
    if err != nil {
        log.Fatal(err)
    }
    defer fpss.Shutdown()

    // Subscribe to real-time quotes
    reqID, _ := fpss.SubscribeQuotes("AAPL", thetadatadx.SecTypeStock)
    fmt.Printf("Subscribed (req_id=%d)\n", reqID)

    // Poll for events
    for {
        event, err := fpss.NextEvent(5000)
        if err != nil {
            log.Println("Error:", err)
            break
        }
        if event == nil {
            continue // timeout
        }
        fmt.Printf("Event: %+v\n", event)
    }
}
```

### FpssClient API

| Method | Signature | Description |
|--------|-----------|-------------|
| `FpssConnect` | `(creds, bufSize) (*FpssClient, error)` | Connect and authenticate |
| `SubscribeQuotes` | `(root, secType) (int32, error)` | Subscribe to quotes |
| `SubscribeTrades` | `(root, secType) (int32, error)` | Subscribe to trades |
| `SubscribeOpenInterest` | `(root, secType) (int32, error)` | Subscribe to open interest |
| `NextEvent` | `(timeoutMs) (*FpssEvent, error)` | Poll next event |
| `Shutdown` | `() error` | Graceful shutdown |

### Security Type Constants

```go
thetadatadx.SecTypeStock   // 0
thetadatadx.SecTypeOption  // 1
thetadatadx.SecTypeIndex   // 2
thetadatadx.SecTypeRate    // 3
```

## Memory Management

All Go SDK objects that wrap FFI handles must be closed when no longer needed:

```go
creds, _ := thetadatadx.CredentialsFromFile("creds.txt")
defer creds.Close()  // frees the Rust-side allocation

config := thetadatadx.ProductionConfig()
defer config.Close()

client, _ := thetadatadx.Connect(creds, config)
defer client.Close()
```

## Architecture

```
Go code
    |  (CGo FFI)
    v
libthetadatadx_ffi.so / .a
    |  (Rust FFI crate)
    v
thetadatadx Rust crate
    |  (tonic gRPC / tokio TCP)
    v
ThetaData servers
```
