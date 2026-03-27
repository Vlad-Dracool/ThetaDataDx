# Types

All tick types are `Copy + Clone + Debug` structs with `i32` fields. Prices are stored in fixed-point encoding -- use the `*_price()` methods to get `Price` values with proper decimal handling.

## TradeTick

16 fields representing a single trade.

```rust
pub struct TradeTick {
    pub ms_of_day: i32,        // Milliseconds since midnight ET
    pub sequence: i32,          // Sequence number
    pub ext_condition1: i32,    // Extended condition code 1
    pub ext_condition2: i32,    // Extended condition code 2
    pub ext_condition3: i32,    // Extended condition code 3
    pub ext_condition4: i32,    // Extended condition code 4
    pub condition: i32,         // Trade condition code
    pub size: i32,              // Trade size (shares)
    pub exchange: i32,          // Exchange code
    pub price: i32,             // Price (fixed-point, use get_price())
    pub condition_flags: i32,   // Condition flags bitmap
    pub price_flags: i32,       // Price flags bitmap
    pub volume_type: i32,       // 0 = incremental, 1 = cumulative
    pub records_back: i32,      // Records back count
    pub price_type: i32,        // Decimal type for price decoding
    pub date: i32,              // Date as YYYYMMDD integer
}
```

### Methods

| Method | Return | Description |
|--------|--------|-------------|
| `get_price()` | `Price` | Trade price with decimal handling |
| `is_cancelled()` | `bool` | Condition code 40-44 |
| `trade_condition_no_last()` | `bool` | Condition flags bit 0 |
| `price_condition_set_last()` | `bool` | Price flags bit 0 |
| `is_incremental_volume()` | `bool` | `volume_type == 0` |
| `regular_trading_hours()` | `bool` | 9:30 AM - 4:00 PM ET |
| `is_seller()` | `bool` | `ext_condition1 == 12` |

## QuoteTick

11 fields representing an NBBO quote.

```rust
pub struct QuoteTick {
    pub ms_of_day: i32,
    pub bid_size: i32,
    pub bid_exchange: i32,
    pub bid: i32,
    pub bid_condition: i32,
    pub ask_size: i32,
    pub ask_exchange: i32,
    pub ask: i32,
    pub ask_condition: i32,
    pub price_type: i32,
    pub date: i32,
}
```

### Methods

| Method | Return | Description |
|--------|--------|-------------|
| `bid_price()` | `Price` | Bid price with decimal handling |
| `ask_price()` | `Price` | Ask price with decimal handling |
| `midpoint_value()` | `i32` | Integer midpoint `(bid + ask) / 2` |
| `midpoint_price()` | `Price` | Midpoint as Price |

## OhlcTick

9 fields representing an aggregated bar.

```rust
pub struct OhlcTick {
    pub ms_of_day: i32,
    pub open: i32,
    pub high: i32,
    pub low: i32,
    pub close: i32,
    pub volume: i32,
    pub count: i32,
    pub price_type: i32,
    pub date: i32,
}
```

### Methods

`open_price()`, `high_price()`, `low_price()`, `close_price()` -- all return `Price`.

## EodTick

18 fields -- full end-of-day snapshot with OHLC + quote data.

```rust
pub struct EodTick {
    pub ms_of_day: i32,
    pub ms_of_day2: i32,
    pub open: i32,
    pub high: i32,
    pub low: i32,
    pub close: i32,
    pub volume: i32,
    pub count: i32,
    pub bid_size: i32,
    pub bid_exchange: i32,
    pub bid: i32,
    pub bid_condition: i32,
    pub ask_size: i32,
    pub ask_exchange: i32,
    pub ask: i32,
    pub ask_condition: i32,
    pub price_type: i32,
    pub date: i32,
}
```

### Methods

`open_price()`, `high_price()`, `low_price()`, `close_price()`, `bid_price()`, `ask_price()`, `midpoint_value()` -- all use the shared `price_type`.

## OpenInterestTick

```rust
pub struct OpenInterestTick {
    pub ms_of_day: i32,
    pub open_interest: i32,
    pub date: i32,
}
```

## SnapshotTradeTick

7-field abbreviated trade for snapshots.

```rust
pub struct SnapshotTradeTick {
    pub ms_of_day: i32,
    pub sequence: i32,
    pub size: i32,
    pub condition: i32,
    pub price: i32,
    pub price_type: i32,
    pub date: i32,
}
```

Methods: `get_price() -> Price`

## TradeQuoteTick

25-field combined trade + quote tick.

```rust
pub struct TradeQuoteTick {
    // Trade portion (14 fields)
    pub ms_of_day: i32,
    pub sequence: i32,
    pub ext_condition1: i32,
    pub ext_condition2: i32,
    pub ext_condition3: i32,
    pub ext_condition4: i32,
    pub condition: i32,
    pub size: i32,
    pub exchange: i32,
    pub price: i32,
    pub condition_flags: i32,
    pub price_flags: i32,
    pub volume_type: i32,
    pub records_back: i32,
    // Quote portion (9 fields)
    pub quote_ms_of_day: i32,
    pub bid_size: i32,
    pub bid_exchange: i32,
    pub bid: i32,
    pub bid_condition: i32,
    pub ask_size: i32,
    pub ask_exchange: i32,
    pub ask: i32,
    pub ask_condition: i32,
    // Shared
    pub price_type: i32,
    pub date: i32,
}
```

Methods: `trade_price()`, `bid_price()`, `ask_price()` -- all return `Price`.

## Enums

### SecType

| Variant | Code | String |
|---------|------|--------|
| `Stock` | 0 | `"STOCK"` |
| `Option` | 1 | `"OPTION"` |
| `Index` | 2 | `"INDEX"` |
| `Rate` | 3 | `"RATE"` |

### StreamResponseType

| Variant | Code | Meaning |
|---------|------|---------|
| `Subscribed` | 0 | Success |
| `Error` | 1 | General error |
| `MaxStreamsReached` | 2 | Subscription limit hit |
| `InvalidPerms` | 3 | Insufficient permissions |

### RemoveReason

Disconnect reason codes (i16). See [Wire Protocol](../internals/wire-protocol.md) for the full table.

### Right

Option right: `Call`, `Put`.

Methods: `from_char(char) -> Option<Self>` (accepts `C/c/P/p`), `as_char() -> char`

### DataType

80+ data field type codes. Key categories:

| Category | Codes |
|----------|-------|
| Core | Date(0), MsOfDay(1), PriceType(4) |
| Quote | BidSize(101), Bid(103), AskSize(105), Ask(107), Midpoint(111) |
| Trade | Sequence(131), Size(132), Price(134), Exchange(135), Volume(141) |
| First-Order Greeks | Theta(151), Vega(152), Delta(153), Rho(154) |
| Second-Order Greeks | Gamma(161), Vanna(162), Charm(163), Vomma(164), Vera(166) |
| Third-Order Greeks | Speed(171), Zomma(172), Color(173), Ultima(174) |
| OHLC | Open(191), High(192), Low(193), Close(194) |
| IV | ImpliedVol(201), BidImpliedVol(202), AskImpliedVol(203) |

### ReqType

Request type codes for historical data queries: `Eod(1)`, `Quote(101)`, `Trade(201)`, `Greeks(203)`, and many more.

### StreamMsgType

FPSS wire message codes (u8). See [Wire Protocol](../internals/wire-protocol.md) for details.
