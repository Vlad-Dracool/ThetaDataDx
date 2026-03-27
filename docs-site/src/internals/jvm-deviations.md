# JVM Deviations

Intentional differences between ThetaDataDx and the decompiled Java terminal. Every deviation is a deliberate improvement -- zero wire protocol changes.

## Wire Protocol: No Deviations

Every byte sent and received is identical to the Java terminal. The gRPC requests, FPSS framing, FIT encoding, contract serialization, and auth handshake all match the Java implementation exactly.

## Client-Side Improvements

### Endpoint Generation: Macro vs Hand-Coded

| | Java | ThetaDataDx |
|---|---|---|
| **Approach** | 60 hand-coded handlers | `define_endpoint!` macro |
| **Impact** | Wire-identical requests |

Java duplicates boilerplate across 60 handlers. The Rust macro eliminates this: each endpoint is a single macro invocation. Adding a new endpoint requires one line instead of ~50.

### FPSS Dispatch: Disruptor Ring Buffer

| | Java | ThetaDataDx |
|---|---|---|
| **Approach** | LMAX Disruptor | `disruptor-rs` v4 |
| **Impact** | Matched Java's dispatch model |

Both use lock-free, bounded-latency, cache-line-padded ring buffers. The FPSS I/O thread is fully synchronous (no Tokio on the hot path).

### FIT Codec: Overflow Saturation

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | `int` wraps silently on overflow | `i64` accumulator, saturates to `i32::MAX/MIN` |
| **Trigger** | 10-digit values > 2,147,483,647 |

Java wrapping produces corrupt data silently. Saturation preserves the sign and makes overflow detectable. Real market data never triggers this.

### Greeks: Degenerate Input Guard

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | Returns `NaN`/`Inf` for `t=0` or `v=0` | Returns `0.0` (intrinsic for `value()`) |

NaN/Inf propagates silently through downstream calculations. Returning `0.0` is the mathematically correct limit.

### Greeks: Precomputed Intermediates

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | Each Greek calls `d1()`, `d2()` independently | `all_greeks()` precomputes once |
| **Impact** | Numerically identical, ~20x fewer transcendental calls |

### Greeks: `.exp()` vs `E.powf()`

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | `Math.pow(Math.E, x)` | `x.exp()` |
| **Impact** | ~1 ULP improvement |

`E.powf(x)` computes `e^(x * ln(e))` -- the `ln(e)` multiply introduces rounding. `.exp()` is a direct hardware operation.

### norm_cdf Implementation

| | Java | ThetaDataDx |
|---|---|---|
| **Approach** | Apache Commons Math continued-fraction | Horner-form Zelen & Severo (~1e-7 accuracy) |
| **Impact** | Fewer multiplications, no external dependency |

### Reconnection: Manual vs Automatic

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | Auto-reconnect | Manual (`reconnect()` method) |

**This is a deliberate, permanent deviation.** Auto-reconnect hides failures and makes debugging harder. Manual reconnection gives explicit control over retry policy, backoff, and circuit breaking. The `reconnect_delay()` helper provides Java-compatible delay calculation.

### Permanent Disconnect: More Reasons Treated as Fatal

| | Java | ThetaDataDx |
|---|---|---|
| **Permanent** | Only code 6 | 7 codes: 0, 1, 2, 6, 9, 17, 18 |

Java only stops for `AccountAlreadyConnected`, retrying forever on bad credentials (burning rate limits). ThetaDataDx treats all credential/account errors as permanent.

### Error Handling: Surfaced vs Silent

| | Java | ThetaDataDx |
|---|---|---|
| **CONTRACT parse failure** | Logged, silently dropped | Emits `FpssEvent::Error` |
| **REQ_RESPONSE parse failure** | Logged, silently dropped | Emits `FpssEvent::Error` |

Silent drops lose subscription state and server rejection details. Surfacing errors lets callers react.

### FpssEvent Split

| | Java | ThetaDataDx |
|---|---|---|
| **Approach** | Single monolithic event hierarchy | `FpssData` + `FpssControl` split |

Enables exhaustive `match` on data events without touching lifecycle events, and vice versa.

### Streaming `_stream` Variants

| | Java | ThetaDataDx |
|---|---|---|
| **Approach** | `ArrayBlockingQueue(2)` | Callback variants process without materializing |
| **Impact** | More flexible consumer model |

### Date Validation

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | Accepts any string, server rejects | Validates 8 ASCII digits client-side |

### Contract Root Length Validation

| | Java | ThetaDataDx |
|---|---|---|
| **Behavior** | No validation, `as byte` truncation | `assert!(root.len() <= 244)` |

## TLS Stack

| Component | Java | ThetaDataDx |
|-----------|------|-------------|
| MDDS | JSSE | `tonic` + `rustls` (ring backend) |
| FPSS | JSSE | `rustls` + `tokio-rustls` |
| Trust store | Java cacerts | `webpki-roots` (Mozilla roots) |

## What Is NOT Different

These are identical to the Java terminal:

- gRPC proto definitions (field numbers, types, service methods)
- FPSS message framing (1-byte len + 1-byte code + payload)
- FPSS auth handshake (CREDENTIALS -> METADATA/DISCONNECTED)
- FIT nibble encoding (digit values, separators, DATE marker, SPACING=5)
- FIT delta compression (first tick absolute, subsequent deltas)
- Contract binary serialization (stock vs option wire format)
- FPSS ping interval (100ms), payload (`[0x00]`), and 2000ms initial delay
- FPSS credential length read as unsigned (matches `readUnsignedShort()`)
- FPSS write buffer flushed only on PING (batched writes)
- FPSS ROW_SEP unconditionally resets field index to SPACING
- FPSS contract ID extracted via FIT decode
- FPSS delta state cleared on START/STOP signals
- `"client": "terminal"` in gRPC `query_parameters`
- Nexus auth URL, terminal key, request/response format
- Nexus 401/404 handling treated as invalid credentials
- MDDS gRPC endpoint (`mdds-01.thetadata.us:443`)
- FPSS server list (`nj-a:20000/20001`, `nj-b:20000/20001`)
- Price encoding formula (`value * 10^(type - 10)`)
- All enum codes (StreamMsgType, RemoveReason, SecType, DataType)
- Greeks formulas (operator precedence matched to Java)
- OHLCVC-from-trade derivation (server-seeded, then incremental)
- Vera (DataType code 166) in second-order Greeks
