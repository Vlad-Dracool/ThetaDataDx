# FpssClient

Real-time streaming client for ThetaData's FPSS (Feed Protocol Streaming Service) servers. Connects over TLS/TCP and delivers live market data via a lock-free disruptor ring buffer.

## Construction

### Rust

```rust
pub fn connect(
    creds: &Credentials,
    event_buffer: usize,
    callback: impl Fn(&FpssEvent) + Send + 'static,
) -> Result<Self, Error>
```

Establishes a TLS connection, authenticates, and starts the background reader and heartbeat threads.

```rust
use thetadatadx::fpss::{FpssClient, FpssEvent, FpssData, FpssControl};
use thetadatadx::fpss::protocol::Contract;

let client = FpssClient::connect(&creds, 1024, |event: &FpssEvent| {
    match event {
        FpssEvent::Data(data) => { /* handle market data */ }
        FpssEvent::Control(ctrl) => { /* handle lifecycle */ }
        _ => {}
    }
})?;
```

### Python

```python
from thetadatadx import Credentials, FpssClient

creds = Credentials.from_file("creds.txt")
fpss = FpssClient(creds, buffer_size=1024)
```

### Go

```go
fpss, err := thetadatadx.FpssConnect(creds, 1024)
```

### C++

```cpp
auto fpss = tdx::FpssClient::connect(creds, 1024);
```

## Subscription Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `subscribe_quotes` | `(&self, &Contract) -> Result<i32, Error>` | Subscribe to quote data |
| `subscribe_trades` | `(&self, &Contract) -> Result<i32, Error>` | Subscribe to trade data |
| `subscribe_open_interest` | `(&self, &Contract) -> Result<i32, Error>` | Subscribe to open interest |
| `subscribe_full_trades` | `(&self, SecType) -> Result<i32, Error>` | Subscribe to all trades for a security type |
| `unsubscribe_quotes` | `(&self, &Contract) -> Result<i32, Error>` | Unsubscribe quotes |
| `unsubscribe_trades` | `(&self, &Contract) -> Result<i32, Error>` | Unsubscribe trades |
| `unsubscribe_open_interest` | `(&self, &Contract) -> Result<i32, Error>` | Unsubscribe open interest |

All subscription methods return a request ID. The server confirms via a `ReqResponse` event.

## State Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_authenticated` | `(&self) -> bool` | Check if connection is live |
| `server_addr` | `(&self) -> &str` | Get connected server address |
| `contract_map` | `(&self) -> HashMap<i32, Contract>` | Server-assigned contract IDs |
| `shutdown` | `(&mut self) -> Result<(), Error>` | Send STOP and shut down tasks |

## FpssEvent

Events are split into three categories:

```rust
pub enum FpssEvent {
    Data(FpssData),
    Control(FpssControl),
    RawData { code: u8, payload: Vec<u8> },
}
```

### FpssData

Market data events:

```rust
pub enum FpssData {
    Quote {
        contract_id: i32, ms_of_day: i32,
        bid_size: i32, bid_exchange: i32, bid: i32, bid_condition: i32,
        ask_size: i32, ask_exchange: i32, ask: i32, ask_condition: i32,
        price_type: i32, date: i32,
    },
    Trade {
        contract_id: i32, ms_of_day: i32, sequence: i32,
        ext_condition1: i32, ext_condition2: i32,
        ext_condition3: i32, ext_condition4: i32,
        condition: i32, size: i32, exchange: i32, price: i32,
        condition_flags: i32, price_flags: i32,
        volume_type: i32, records_back: i32, price_type: i32, date: i32,
    },
    OpenInterest {
        contract_id: i32, ms_of_day: i32, open_interest: i32, date: i32,
    },
    Ohlcvc {
        contract_id: i32, ms_of_day: i32,
        open: i32, high: i32, low: i32, close: i32,
        volume: i32, count: i32, price_type: i32, date: i32,
    },
}
```

### FpssControl

Lifecycle events:

```rust
pub enum FpssControl {
    LoginSuccess { permissions: String },
    ContractAssigned { id: i32, contract: Contract },
    ReqResponse { req_id: i32, result: StreamResponseType },
    MarketOpen,
    MarketClose,
    ServerError { message: String },
    Disconnected { reason: RemoveReason },
    Error { message: String },
}
```

## Contract

Contracts identify the security being subscribed to.

```rust
pub struct Contract {
    pub root: String,
    pub sec_type: SecType,
    pub exp_date: Option<i32>,    // YYYYMMDD for options
    pub is_call: Option<bool>,    // true=call, false=put
    pub strike: Option<i32>,      // scaled integer
}
```

### Constructors

```rust
Contract::stock("AAPL")
Contract::index("SPX")
Contract::rate("SOFR")
Contract::option("SPY", 20261218, true, 60000)  // call, strike $600
```

### Serialization

```rust
let bytes = contract.to_bytes();                           // serialize for wire
let (contract, consumed) = Contract::from_bytes(&bytes)?;  // deserialize
```

## Reconnection

```rust
pub async fn reconnect(
    creds: &Credentials,
    previous_subs: Vec<(SubscriptionKind, Contract)>,
    delay_ms: u64,
    event_buffer: usize,
) -> Result<(FpssClient, mpsc::Receiver<FpssEvent>), Error>
```

```rust
pub fn reconnect_delay(reason: RemoveReason) -> Option<u64>
```

Returns:
- `None` for permanent errors (bad credentials) -- do NOT retry
- `Some(130_000)` for `TooManyRequests`
- `Some(2_000)` for everything else

## OhlcvcAccumulator

OHLCVC bars are derived from trade ticks via the internal accumulator. It is per-contract and only begins emitting `FpssData::Ohlcvc` events after receiving a server-seeded initial OHLCVC bar. Subsequent trades update incrementally.

## FFI Functions

7 `extern "C"` functions for FPSS lifecycle:

| Function | Description |
|----------|-------------|
| `thetadatadx_fpss_connect` | Connect and authenticate |
| `thetadatadx_fpss_subscribe_quotes` | Subscribe to quotes |
| `thetadatadx_fpss_subscribe_trades` | Subscribe to trades |
| `thetadatadx_fpss_subscribe_open_interest` | Subscribe to OI |
| `thetadatadx_fpss_next_event` | Poll next event |
| `thetadatadx_fpss_shutdown` | Graceful shutdown |
| `thetadatadx_fpss_free_event` | Free an event |
