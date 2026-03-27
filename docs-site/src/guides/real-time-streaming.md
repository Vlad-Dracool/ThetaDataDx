# Real-Time Streaming

This guide covers real-time market data streaming via ThetaData's FPSS (Feed Protocol Streaming Service) servers. FPSS delivers live quotes, trades, open interest, and OHLC snapshots over a persistent TLS/TCP connection.

## Overview

FPSS is a separate system from the historical data gRPC (MDDS) interface. It uses a custom binary protocol over TLS/TCP with:

- **FIT-encoded tick data** (nibble-based variable-length integers with delta compression)
- **100ms heartbeat** (client must send pings to keep the connection alive)
- **Lock-free event dispatch** via a disruptor ring buffer
- **Synchronous I/O thread** (no Tokio on the streaming hot path)

## Step 1: Connect

### Rust

```rust
use thetadatadx::auth::Credentials;
use thetadatadx::fpss::{FpssClient, FpssData, FpssControl, FpssEvent};
use thetadatadx::fpss::protocol::Contract;

let creds = Credentials::from_file("creds.txt")?;
let client = FpssClient::connect(&creds, 1024, |event: &FpssEvent| {
    match event {
        FpssEvent::Data(FpssData::Quote { contract_id, bid, ask, .. }) => {
            println!("Quote: contract={contract_id} bid={bid} ask={ask}");
        }
        FpssEvent::Data(FpssData::Trade { contract_id, price, size, .. }) => {
            println!("Trade: contract={contract_id} price={price} size={size}");
        }
        FpssEvent::Control(FpssControl::ContractAssigned { id, contract }) => {
            println!("Contract {id} = {contract}");
        }
        _ => {}
    }
})?;
```

The `1024` parameter is the ring buffer size for event dispatch.

### Python

```python
from thetadatadx import Credentials, FpssClient

creds = Credentials.from_file("creds.txt")
fpss = FpssClient(creds, buffer_size=1024)
```

### Go

```go
creds, _ := thetadatadx.CredentialsFromFile("creds.txt")
defer creds.Close()

fpss, err := thetadatadx.FpssConnect(creds, 1024)
if err != nil {
    log.Fatal(err)
}
defer fpss.Shutdown()
```

### C++

```cpp
auto creds = tdx::Credentials::from_file("creds.txt");
auto fpss = tdx::FpssClient::connect(creds, 1024);
```

## Step 2: Subscribe

Subscribe to data streams for specific contracts.

### Rust

```rust
// Stock quotes
let req_id = client.subscribe_quotes(&Contract::stock("AAPL"))?;
println!("Subscribed (req_id={req_id})");

// Stock trades
client.subscribe_trades(&Contract::stock("MSFT"))?;

// Option quotes
let opt = Contract::option("SPY", 20261218, true, 60000); // call, strike 600
client.subscribe_quotes(&opt)?;

// Open interest
client.subscribe_open_interest(&Contract::stock("AAPL"))?;

// All trades for a security type
client.subscribe_full_trades(SecType::Stock)?;
```

### Python

```python
fpss.subscribe("AAPL", "QUOTE")
fpss.subscribe("MSFT", "TRADE")
fpss.subscribe("SPY", "OI")
```

### Go

```go
reqID, _ := fpss.SubscribeQuotes("AAPL", thetadatadx.SecTypeStock)
fmt.Printf("Subscribed (req_id=%d)\n", reqID)

fpss.SubscribeTrades("MSFT", thetadatadx.SecTypeStock)
fpss.SubscribeOpenInterest("AAPL", thetadatadx.SecTypeStock)
```

### C++

```cpp
int32_t req_id = fpss.subscribe_quotes("AAPL", tdx::SecType::Stock);
fpss.subscribe_trades("MSFT", tdx::SecType::Stock);
```

## Step 3: Receive Events

### Rust (callback-based)

The Rust API uses a callback that fires on the ring buffer's consumer thread:

```rust
let client = FpssClient::connect(&creds, 1024, |event: &FpssEvent| {
    match event {
        FpssEvent::Data(FpssData::Quote {
            contract_id, ms_of_day, bid, ask, bid_size, ask_size, price_type, ..
        }) => {
            // price_type is needed to decode the integer price values
            println!("Quote: id={contract_id} bid={bid} ask={ask}");
        }
        FpssEvent::Data(FpssData::Trade {
            contract_id, price, size, ..
        }) => {
            println!("Trade: id={contract_id} price={price} size={size}");
        }
        FpssEvent::Data(FpssData::OpenInterest {
            contract_id, open_interest, ..
        }) => {
            println!("OI: id={contract_id} oi={open_interest}");
        }
        FpssEvent::Data(FpssData::Ohlcvc {
            contract_id, open, high, low, close, volume, count, ..
        }) => {
            println!("OHLCVC: id={contract_id} O={open} H={high} L={low} C={close}");
        }
        FpssEvent::Control(FpssControl::LoginSuccess { permissions }) => {
            println!("Logged in: {permissions}");
        }
        FpssEvent::Control(FpssControl::ContractAssigned { id, contract }) => {
            println!("Contract {id} assigned: {contract}");
        }
        FpssEvent::Control(FpssControl::ReqResponse { req_id, result }) => {
            println!("Request {req_id}: {:?}", result);
        }
        FpssEvent::Control(FpssControl::MarketOpen) => {
            println!("Market opened");
        }
        FpssEvent::Control(FpssControl::MarketClose) => {
            println!("Market closed");
        }
        FpssEvent::Control(FpssControl::Disconnected { reason }) => {
            println!("Disconnected: {:?}", reason);
        }
        _ => {}
    }
})?;

// Block until shutdown
std::thread::park();
```

### Python (polling)

```python
while True:
    event = fpss.next_event(timeout_ms=5000)
    if event is None:
        continue  # timeout, no event
    if event["type"] == "quote":
        print(f"Quote: {event['contract']} bid={event['bid']} ask={event['ask']}")
    elif event["type"] == "trade":
        print(f"Trade: {event['contract']} price={event['price']} size={event['size']}")
```

### Go (polling)

```go
for {
    event, err := fpss.NextEvent(5000) // 5s timeout
    if err != nil {
        log.Println("Error:", err)
        break
    }
    if event == nil {
        continue // timeout
    }
    fmt.Printf("Event: %+v\n", event)
}
```

### C++ (polling)

```cpp
while (auto event = fpss.next_event(5000)) {
    if (event->type() == tdx::FpssEventType::Quote) {
        std::cout << "Quote: " << event->contract()
                  << " bid=" << event->bid()
                  << " ask=" << event->ask() << std::endl;
    }
}
```

## Step 4: Unsubscribe

### Rust

```rust
client.unsubscribe_quotes(&Contract::stock("AAPL"))?;
client.unsubscribe_trades(&Contract::stock("MSFT"))?;
client.unsubscribe_open_interest(&Contract::stock("AAPL"))?;
```

## Step 5: Shutdown

### Rust

```rust
client.shutdown()?;
```

### Python

```python
fpss.shutdown()
```

### Go

```go
fpss.Shutdown()
```

### C++

```cpp
fpss.shutdown();
```

## Event Types

Events are split into data events and control events:

### Data Events (`FpssData`)

| Event | Fields |
|-------|--------|
| `Quote` | contract_id, ms_of_day, bid_size, bid_exchange, bid, bid_condition, ask_size, ask_exchange, ask, ask_condition, price_type, date |
| `Trade` | contract_id, ms_of_day, sequence, ext_condition1-4, condition, size, exchange, price, condition_flags, price_flags, volume_type, records_back, price_type, date |
| `OpenInterest` | contract_id, ms_of_day, open_interest, date |
| `Ohlcvc` | contract_id, ms_of_day, open, high, low, close, volume, count, price_type, date |

### Control Events (`FpssControl`)

| Event | Fields |
|-------|--------|
| `LoginSuccess` | permissions (string) |
| `ContractAssigned` | id, contract |
| `ReqResponse` | req_id, result (Subscribed/Error/MaxStreamsReached/InvalidPerms) |
| `MarketOpen` | (none) |
| `MarketClose` | (none) |
| `ServerError` | message |
| `Disconnected` | reason (RemoveReason enum) |
| `Error` | message |

## OHLCVC Bar Derivation

The FPSS server sends initial OHLCVC bars for subscribed contracts. After receiving the initial bar, the client internally derives updated OHLCVC bars from subsequent trades. This matches the Java terminal's behavior:

1. The accumulator is **not active** until the server sends an initial bar
2. Each incoming trade updates open/high/low/close/volume/count
3. Derived bars are emitted as `FpssData::Ohlcvc` events

## Reconnection

ThetaDataDx uses **manual reconnection** (unlike the Java terminal's auto-reconnect). When the server disconnects, you receive an `FpssControl::Disconnected` event with a reason code.

### Reconnection Logic

```rust
use thetadatadx::fpss::FpssClient;
use thetadatadx::types::RemoveReason;

// Check if reconnection is appropriate
match FpssClient::reconnect_delay(reason) {
    None => {
        // Permanent error (bad credentials, etc.) -- do NOT retry
        eprintln!("Permanent disconnect: {:?}", reason);
    }
    Some(delay_ms) => {
        // Reconnect after delay
        let (new_client, new_events) = FpssClient::reconnect(
            &creds,
            previous_subscriptions,
            delay_ms,
            1024,
        ).await?;
    }
}
```

### Disconnect Reason Categories

| Category | Codes | Action |
|----------|-------|--------|
| Permanent | 0, 1, 2, 6, 9, 17, 18 | Do NOT reconnect |
| Rate-limited | 12 | Wait 130 seconds, then reconnect |
| Transient | All others | Wait 2 seconds, then reconnect |

Permanent reasons: InvalidCredentials, InvalidLoginValues, InvalidLoginSize, AccountAlreadyConnected, FreeAccount, ServerUserDoesNotExist, InvalidCredentialsNullUser.
