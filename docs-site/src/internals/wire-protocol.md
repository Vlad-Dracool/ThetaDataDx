# Wire Protocol

Complete specification of the FPSS (Feed Protocol Streaming Service) wire protocol.

## Connection Establishment

1. TCP connect to FPSS server (2s timeout)
2. TLS handshake (rustls + webpki-roots)
3. Set `TCP_NODELAY = true`
4. Split into read/write halves

Servers are tried in order until one connects:
- `nj-a.thetadata.us:20000`
- `nj-a.thetadata.us:20001`
- `nj-b.thetadata.us:20000`
- `nj-b.thetadata.us:20001`

## Frame Format

Every FPSS message uses the same wire framing:

```
+--------+--------+------------------+
| LEN(1) | CODE(1)| PAYLOAD (LEN B)  |
+--------+--------+------------------+
```

- **LEN** (u8): Payload length (0-255). Does NOT include the 2-byte header.
- **CODE** (u8): Message type (`StreamMsgType` enum).
- **PAYLOAD**: LEN bytes of message-specific data.

Total bytes per message on the wire = `LEN + 2`.

## Message Codes

| Code | Name | Direction | Description |
|------|------|-----------|-------------|
| 0x00 | CREDENTIALS | C->S | Auth: `[0x00] [user_len: u16 BE] [user bytes] [pass bytes]` |
| 0x01 | SESSION_TOKEN | C->S | Alternative session-based auth |
| 0x02 | INFO | S->C | Server info |
| 0x03 | METADATA | S->C | Login success, payload = permissions UTF-8 string |
| 0x04 | CONNECTED | S->C | Connection acknowledged |
| 0x0A | PING | C->S | Heartbeat: `[0x00]` every 100ms |
| 0x0B | ERROR | S->C | Error message (UTF-8 text) |
| 0x0C | DISCONNECTED | S->C | Disconnect reason: `[reason: i16 BE]` |
| 0x0D | RECONNECTED | S->C | Reconnection acknowledged |
| 0x14 | CONTRACT | S->C | Contract ID assignment: `[id: FIT-encoded i32] [contract bytes]` |
| 0x15 | QUOTE | Both | Subscribe (C->S) / data (S->C). FIT-encoded quote tick |
| 0x16 | TRADE | Both | Subscribe (C->S) / data (S->C). FIT-encoded trade tick |
| 0x17 | OPEN_INTEREST | Both | Subscribe (C->S) / data (S->C) |
| 0x18 | OHLCVC | S->C | FIT-encoded OHLC + volume + count snapshot |
| 0x1E | START | S->C | Market open signal |
| 0x1F | RESTART | S->C | Server restart signal |
| 0x20 | STOP | Both | Market close (S->C) / shutdown (C->S) |
| 0x28 | REQ_RESPONSE | S->C | Subscription result: `[req_id: i32 BE] [code: i32 BE]` |
| 0x33 | REMOVE_QUOTE | C->S | Unsubscribe quotes |
| 0x34 | REMOVE_TRADE | C->S | Unsubscribe trades |
| 0x35 | REMOVE_OI | C->S | Unsubscribe open interest |

## Authentication Handshake

```
Client                          Server
  |                                |
  |-- CREDENTIALS (0x00) -------->|
  |   [0x00][user_len:u16 BE]    |
  |   [email bytes][pass bytes]   |
  |                                |
  |<------ METADATA (0x03) -------|  (success: permissions string)
  |                                |
  |   OR                           |
  |                                |
  |<---- DISCONNECTED (0x0C) -----|  (failure: reason i16 BE)
```

After successful auth, the client waits 2000ms before sending the first PING (matching the Java terminal's initial delay).

## Heartbeat

After authentication, the client sends PING (code 0x0A) with payload `[0x00]` every 100ms. Failure to send pings causes the server to disconnect.

The write buffer is flushed only on PING sends, batching any intervening subscription messages.

## Subscription Flow

```
Client                          Server
  |                                |
  |-- QUOTE (0x15) -------------->|
  |   [req_id: i32 BE]           |
  |   [contract bytes]            |
  |                                |
  |<---- REQ_RESPONSE (0x28) -----|
  |   [req_id: i32 BE]           |
  |   [result: i32 BE]           |
  |   (0=OK, 1=ERR, 2=MAX, 3=PERMS)
  |                                |
  |<---- CONTRACT (0x14) ---------|
  |   [contract_id: FIT i32]     |
  |   [contract bytes]            |
  |   (assigns numeric ID)       |
  |                                |
  |<---- QUOTE (0x15) ------------|  (continuous)
  |   [FIT-encoded tick payload]  |
  |                                |
```

For full-type subscriptions (all trades for a security type): payload = `[req_id: i32 BE] [sec_type: u8]` (5 bytes = full type, longer = per-contract).

## Contract Binary Format

### Stock / Index / Rate

```
+------------+----------+---------------------+----------+
| total_size | root_len | root (ASCII)        | sec_type |
| (u8)       | (u8)     | (root_len bytes)    | (u8)     |
+------------+----------+---------------------+----------+
```

### Option

```
+------------+----------+--------+----------+----------+--------+----------+
| total_size | root_len | root   | sec_type | exp_date | is_call| strike   |
| (u8)       | (u8)     | (ASCII)| (u8=1)   | (i32 BE) | (u8)   | (i32 BE) |
+------------+----------+--------+----------+----------+--------+----------+
```

Security type codes: Stock=0, Option=1, Index=2, Rate=3.

## Disconnect Reason Codes

| Code | Name | Permanent? |
|------|------|-----------|
| -1 | Unspecified | No |
| 0 | InvalidCredentials | Yes |
| 1 | InvalidLoginValues | Yes |
| 2 | InvalidLoginSize | Yes |
| 3 | GeneralValidationError | No |
| 4 | TimedOut | No |
| 5 | ClientForcedDisconnect | No |
| 6 | AccountAlreadyConnected | Yes |
| 7 | SessionTokenExpired | No |
| 8 | InvalidSessionToken | No |
| 9 | FreeAccount | Yes |
| 12 | TooManyRequests | No (130s delay) |
| 13 | NoStartDate | No |
| 14 | LoginTimedOut | No |
| 15 | ServerRestarting | No |
| 16 | SessionTokenNotFound | No |
| 17 | ServerUserDoesNotExist | Yes |
| 18 | InvalidCredentialsNullUser | Yes |

## Price Encoding

Prices use a fixed-point `(value, type)` encoding:

```
real_price = value * 10^(type - 10)
```

See [Price](../api/price.md) for the full type table.

## FIE String Encoding

FIE (Feed Interchange Encoding) is used for building FPSS request payloads. It maps a 16-character alphabet to 4-bit nibbles:

| Character | Nibble |
|-----------|--------|
| `0`-`9` | 0-9 |
| `.` | 0xA |
| `,` | 0xB |
| `/` | 0xC |
| `n` | 0xD |
| `-` | 0xE |
| `e` | 0xF |

Characters are packed pairwise: `byte = (nibble(c1) << 4) | nibble(c2)`. Odd-length strings pad the last byte with `0xD`. Even-length strings append a `0xDD` terminator.
