# FIT Codec

FIT (Feed Interchange Transport) is ThetaData's nibble-based variable-length integer encoding with delta compression, used for FPSS tick data.

## Nibble Encoding

Each byte contains two 4-bit nibbles: `byte = (high << 4) | low`.

| Nibble | Meaning |
|--------|---------|
| 0-9 | Decimal digit, accumulated left-to-right into current integer |
| 0xB | FIELD_SEPARATOR -- flush integer to output, advance to next field |
| 0xC | ROW_SEPARATOR -- flush, zero-fill fields up to index 4, jump to index 5 |
| 0xD | END -- flush current integer, terminate row, return field count |
| 0xE | NEGATIVE -- next flushed integer is negated |

### Encoding Example

The value sequence `[34200000, 1, 0, 0, 0, 100, 4, 15025]` encodes as:

```
34200000 COMMA 1 SLASH 100 COMMA 4 COMMA 15025 END
```

Where SLASH (ROW_SEP) zero-fills fields 2-4 (ext_condition slots), jumping directly to field index 5.

## Delta Compression

- **First tick** per contract: absolute values (no delta applied)
- **Subsequent ticks**: each field is a delta added to the previous tick's value
- Fields not present in the delta row carry forward from the previous tick
- **Delta state is cleared** when the server sends START (market open) or STOP (market close) -- the next tick after these signals is treated as absolute

### Example

```
First tick (absolute):  [34200000, 1, 0, 0, 0, 100, 4, 15025]
Delta row:              [500,      1, 0, 0, 0,  50, 0,    -3]
Resolved tick 2:        [34200500, 2, 0, 0, 0, 150, 4, 15022]
```

## Special Markers

### DATE Marker

If the first byte of a row is `0xCE` (DATE marker), the entire row is consumed until an END nibble is found, and `read_changes` returns 0. This signals a date boundary in the stream.

### SPACING Constant

The ROW_SEPARATOR nibble (0xC) unconditionally resets the field index to SPACING (5). This skips the ext_condition fields (indices 2-4) which are often zero, providing compact encoding for common tick patterns.

## Overflow Handling

| Implementation | Behavior |
|----------------|----------|
| Java terminal | `int` wraps silently on overflow |
| ThetaDataDx | `i64` accumulator, saturates to `i32::MAX/MIN` |

Real market data never has values exceeding `i32` range. The saturation behavior preserves the sign and makes overflow detectable, unlike Java's silent wrapping which produces corrupt data.

## Tick Field Layouts

### Quote Tick (12 fields)

| Index | Field |
|-------|-------|
| 0 | contract_id |
| 1 | ms_of_day |
| 2 | bid_size |
| 3 | bid_exchange |
| 4 | bid |
| 5 | bid_condition |
| 6 | ask_size |
| 7 | ask_exchange |
| 8 | ask |
| 9 | ask_condition |
| 10 | price_type |
| 11 | date |

### Trade Tick (17 fields)

| Index | Field |
|-------|-------|
| 0 | contract_id |
| 1 | ms_of_day |
| 2 | sequence |
| 3 | ext_condition1 |
| 4 | ext_condition2 |
| 5 | ext_condition3 |
| 6 | ext_condition4 |
| 7 | condition |
| 8 | size |
| 9 | exchange |
| 10 | price |
| 11 | condition_flags |
| 12 | price_flags |
| 13 | volume_type |
| 14 | records_back |
| 15 | price_type |
| 16 | date |

### Open Interest Tick (4 fields)

| Index | Field |
|-------|-------|
| 0 | contract_id |
| 1 | ms_of_day |
| 2 | open_interest |
| 3 | date |

### OHLCVC Tick (10 fields)

| Index | Field |
|-------|-------|
| 0 | contract_id |
| 1 | ms_of_day |
| 2 | open |
| 3 | high |
| 4 | low |
| 5 | close |
| 6 | volume |
| 7 | count |
| 8 | price_type |
| 9 | date |
