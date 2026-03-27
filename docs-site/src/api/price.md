# Price

Fixed-point price with variable decimal precision. All prices in ThetaData use this encoding.

## Encoding

The real price is calculated as:

```
real_price = value * 10^(price_type - 10)
```

## Struct

```rust
pub struct Price {
    pub value: i32,
    pub price_type: i32,
}
```

## Construction

```rust
Price::new(15025, 8)    // 150.25
Price::new(100, 10)     // 100.0
Price::ZERO             // 0.0
Price::from_proto(&proto_price)
```

## Methods

| Method | Return | Description |
|--------|--------|-------------|
| `to_f64()` | `f64` | Lossy float conversion |
| `is_zero()` | `bool` | True if value == 0 or price_type == 0 |
| `to_proto()` | `proto::Price` | Convert to protobuf |

## Traits

- `Display` -- formats with correct decimal places: `"150.25"`, `"0.005"`, `"500.0"`
- `Debug` -- `Price(150.25)`
- `Eq, Ord, PartialEq, PartialOrd` -- compares across different price_type values by normalizing to a common base
- `Copy, Clone, Default`

## Price Type Table

| price_type | Decimal Places | Multiplier | Example |
|------------|----------------|------------|---------|
| 0 | Zero price | 0 | `(0, 0)` = `0.0` |
| 6 | 4 decimals | 0.0001 | `(1502500, 6)` = `150.2500` |
| 7 | 3 decimals | 0.001 | `(5, 7)` = `0.005` |
| 8 | 2 decimals | 0.01 | `(15025, 8)` = `150.25` |
| 10 | 0 decimals | 1.0 | `(100, 10)` = `100.0` |
| 12 | -2 decimals | 100.0 | `(5, 12)` = `500.0` |

## Usage with Tick Types

All tick types store prices as raw `i32` values. Use the `*_price()` accessor methods to get a `Price` with proper decimal handling:

```rust
let tick: TradeTick = /* ... */;
let price: Price = tick.get_price();
println!("Trade price: {}", price);       // "150.25"
println!("As float: {}", price.to_f64()); // 150.25

let quote: QuoteTick = /* ... */;
let bid: Price = quote.bid_price();
let ask: Price = quote.ask_price();
let mid: Price = quote.midpoint_price();
println!("Spread: {} - {} (mid: {})", bid, ask, mid);
```

## Comparison Across Types

Prices with different `price_type` values can be compared directly. The comparison normalizes both to a common base:

```rust
let a = Price::new(15025, 8);    // 150.25
let b = Price::new(1502500, 6);  // 150.2500
assert_eq!(a, b);                // true -- same real value
```

## FPSS Streaming Prices

In FPSS streaming events, prices are raw `i32` values with a separate `price_type` field. Construct a `Price` to interpret them:

```rust
match event {
    FpssEvent::Data(FpssData::Quote { bid, ask, price_type, .. }) => {
        let bid_price = Price::new(bid, price_type);
        let ask_price = Price::new(ask, price_type);
        println!("Bid: {}, Ask: {}", bid_price, ask_price);
    }
    _ => {}
}
```
