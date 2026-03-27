# Quick Start

This guide walks you through your first ThetaDataDx program in each supported language.

## Prerequisites

1. A valid [ThetaData](https://thetadata.us) subscription
2. A `creds.txt` file with your email (line 1) and password (line 2)
3. ThetaDataDx installed (see [Installation](installation.md))

## Rust

```rust
use thetadatadx::{DirectClient, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    // 1. Load credentials
    let creds = Credentials::from_file("creds.txt")?;

    // 2. Connect to ThetaData (authenticates automatically)
    let client = DirectClient::connect(&creds, DirectConfig::production()).await?;

    // 3. Fetch end-of-day stock data
    let eod = client.stock_history_eod("AAPL", "20240101", "20240301").await?;
    for tick in &eod {
        println!("{}: O={} H={} L={} C={} V={}",
            tick.date, tick.open_price(), tick.high_price(),
            tick.low_price(), tick.close_price(), tick.volume);
    }

    // 4. List option expirations
    let exps = client.option_list_expirations("SPY").await?;
    println!("SPY expirations: {:?}", &exps[..5.min(exps.len())]);

    // 5. Compute Greeks (offline, no server call)
    let greeks = thetadatadx::greeks::all_greeks(
        450.0,        // spot
        455.0,        // strike
        0.05,         // risk-free rate
        0.015,        // dividend yield
        30.0 / 365.0, // time to expiry (years)
        8.50,         // option market price
        true,         // is_call
    );
    println!("IV: {:.4}, Delta: {:.4}, Gamma: {:.6}",
        greeks.iv, greeks.delta, greeks.gamma);

    Ok(())
}
```

## Python

```python
from thetadatadx import Credentials, Config, DirectClient, all_greeks

# 1. Load credentials
creds = Credentials.from_file("creds.txt")

# 2. Connect to ThetaData
client = DirectClient(creds, Config.production())

# 3. Fetch end-of-day stock data
eod = client.stock_history_eod("AAPL", "20240101", "20240301")
for tick in eod:
    print(f"{tick['date']}: O={tick['open']:.2f} H={tick['high']:.2f} "
          f"L={tick['low']:.2f} C={tick['close']:.2f} V={tick['volume']}")

# 4. List option expirations
exps = client.option_list_expirations("SPY")
print(f"SPY expirations: {exps[:5]}")

# 5. Compute Greeks (offline)
g = all_greeks(
    spot=450.0, strike=455.0, rate=0.05,
    div_yield=0.015, tte=30/365, option_price=8.50, is_call=True
)
print(f"IV={g['iv']:.4f} Delta={g['delta']:.4f} Gamma={g['gamma']:.6f}")
```

### With pandas DataFrames

```python
from thetadatadx import Credentials, Config, DirectClient, to_dataframe

creds = Credentials.from_file("creds.txt")
client = DirectClient(creds, Config.production())

# Option 1: explicit conversion
eod = client.stock_history_eod("AAPL", "20240101", "20240301")
df = to_dataframe(eod)
print(df.head())

# Option 2: _df convenience methods
df = client.stock_history_eod_df("AAPL", "20240101", "20240301")
df = client.stock_history_ohlc_df("AAPL", "20240315", "60000")
```

Requires `pip install thetadatadx[pandas]`.

## Go

```go
package main

import (
    "fmt"
    "log"

    thetadatadx "github.com/userFRM/ThetaDataDx/sdks/go"
)

func main() {
    // 1. Load credentials
    creds, err := thetadatadx.CredentialsFromFile("creds.txt")
    if err != nil {
        log.Fatal(err)
    }
    defer creds.Close()

    // 2. Connect to ThetaData
    config := thetadatadx.ProductionConfig()
    defer config.Close()

    client, err := thetadatadx.Connect(creds, config)
    if err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    // 3. Fetch end-of-day stock data
    eod, err := client.StockHistoryEOD("AAPL", "20240101", "20240301")
    if err != nil {
        log.Fatal(err)
    }
    for _, tick := range eod {
        fmt.Printf("%d: O=%.2f H=%.2f L=%.2f C=%.2f\n",
            tick.Date, tick.Open, tick.High, tick.Low, tick.Close)
    }

    // 4. Compute Greeks (offline)
    g, err := thetadatadx.AllGreeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("IV=%.4f Delta=%.4f Gamma=%.6f\n", g.IV, g.Delta, g.Gamma)
}
```

## C++

```cpp
#include "thetadatadx.hpp"
#include <iostream>
#include <iomanip>

int main() {
    // 1. Load credentials
    auto creds = tdx::Credentials::from_file("creds.txt");

    // 2. Connect to ThetaData
    auto client = tdx::Client::connect(creds, tdx::Config::production());

    // 3. Fetch end-of-day stock data
    auto eod = client.stock_history_eod("AAPL", "20240101", "20240301");
    for (auto& tick : eod) {
        std::cout << tick.date << ": O=" << std::fixed << std::setprecision(2)
                  << tick.open << " H=" << tick.high
                  << " L=" << tick.low << " C=" << tick.close << std::endl;
    }

    // 4. Compute Greeks (offline)
    auto g = tdx::all_greeks(450.0, 455.0, 0.05, 0.015, 30.0/365.0, 8.50, true);
    std::cout << "IV=" << g.iv << " Delta=" << g.delta
              << " Gamma=" << g.gamma << std::endl;
}
```

## CLI

```bash
# Test authentication
tdx auth --creds creds.txt

# Fetch end-of-day data
tdx stock eod AAPL 20240101 20240301

# As JSON
tdx stock eod AAPL 20240101 20240301 --format json

# As CSV
tdx stock eod AAPL 20240101 20240301 --format csv

# List option expirations
tdx option expirations SPY

# Compute Greeks (no server connection needed)
tdx greeks 450 455 0.05 0.015 0.082 8.5 call
```

## What's Next?

- [Configuration](configuration.md) -- customize timeouts, concurrency, and server settings
- [Historical Data Guide](../guides/historical-data.md) -- deep dive into all historical data endpoints
- [Real-Time Streaming Guide](../guides/real-time-streaming.md) -- set up live market data feeds
- [Options Chain Guide](../guides/options-chain.md) -- walk through the full option chain workflow
