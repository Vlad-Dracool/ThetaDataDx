//! Response formatting that matches the Java terminal's JSON output exactly.
//!
//! Uses `sonic_rs` (SIMD-accelerated) instead of `serde_json` for all
//! serialization. The Java terminal wraps every REST response in:
//!
//! ```json
//! {
//!     "header": { "format": "json", "error_type": "null" },
//!     "response": [ ... ]
//! }
//! ```

use sonic_rs::prelude::*;
use thetadatadx::proto;
use thetadatadx::types::price::Price;
use thetadatadx::types::tick::*;

// ---------------------------------------------------------------------------
//  JSON envelope
// ---------------------------------------------------------------------------

/// Wrap a response array in the Java terminal's standard envelope.
pub fn ok_envelope(response: Vec<sonic_rs::Value>) -> sonic_rs::Value {
    sonic_rs::json!({
        "header": {
            "format": "json",
            "error_type": "null"
        },
        "response": response
    })
}

/// Error envelope matching the Java terminal's error format.
pub fn error_envelope(error_type: &str, message: &str) -> sonic_rs::Value {
    sonic_rs::json!({
        "header": {
            "format": "json",
            "error_type": error_type
        },
        "error": {
            "message": message
        }
    })
}

/// Wrap a list of strings in the envelope (for list endpoints).
pub fn list_envelope(items: &[String]) -> sonic_rs::Value {
    let response: Vec<sonic_rs::Value> = items
        .iter()
        .map(|s| sonic_rs::Value::from(s.as_str()))
        .collect();
    ok_envelope(response)
}

// ---------------------------------------------------------------------------
//  Price formatting -- matches Java PriceCalcUtils exactly
// ---------------------------------------------------------------------------

/// Format a price value to f64.
fn fmt_price(value: i32, price_type: i32) -> f64 {
    Price::new(value, price_type).to_f64()
}

// ---------------------------------------------------------------------------
//  Tick -> sonic_rs::Value conversions
// ---------------------------------------------------------------------------

/// Convert EOD ticks to JSON array matching the Java terminal format.
pub fn eod_ticks_to_json(ticks: &[EodTick]) -> Vec<sonic_rs::Value> {
    ticks
        .iter()
        .map(|t| {
            sonic_rs::json!({
                "ms_of_day": t.ms_of_day,
                "ms_of_day2": t.ms_of_day2,
                "open": fmt_price(t.open, t.price_type),
                "high": fmt_price(t.high, t.price_type),
                "low": fmt_price(t.low, t.price_type),
                "close": fmt_price(t.close, t.price_type),
                "volume": t.volume,
                "count": t.count,
                "bid_size": t.bid_size,
                "bid_exchange": t.bid_exchange,
                "bid": fmt_price(t.bid, t.price_type),
                "bid_condition": t.bid_condition,
                "ask_size": t.ask_size,
                "ask_exchange": t.ask_exchange,
                "ask": fmt_price(t.ask, t.price_type),
                "ask_condition": t.ask_condition,
                "date": t.date
            })
        })
        .collect()
}

/// Convert OHLC ticks to JSON array.
pub fn ohlc_ticks_to_json(ticks: &[OhlcTick]) -> Vec<sonic_rs::Value> {
    ticks
        .iter()
        .map(|t| {
            sonic_rs::json!({
                "ms_of_day": t.ms_of_day,
                "open": fmt_price(t.open, t.price_type),
                "high": fmt_price(t.high, t.price_type),
                "low": fmt_price(t.low, t.price_type),
                "close": fmt_price(t.close, t.price_type),
                "volume": t.volume,
                "count": t.count,
                "date": t.date
            })
        })
        .collect()
}

/// Convert trade ticks to JSON array.
pub fn trade_ticks_to_json(ticks: &[TradeTick]) -> Vec<sonic_rs::Value> {
    ticks
        .iter()
        .map(|t| {
            sonic_rs::json!({
                "ms_of_day": t.ms_of_day,
                "sequence": t.sequence,
                "size": t.size,
                "condition": t.condition,
                "price": fmt_price(t.price, t.price_type),
                "exchange": t.exchange,
                "date": t.date
            })
        })
        .collect()
}

/// Convert quote ticks to JSON array.
pub fn quote_ticks_to_json(ticks: &[QuoteTick]) -> Vec<sonic_rs::Value> {
    ticks
        .iter()
        .map(|t| {
            sonic_rs::json!({
                "ms_of_day": t.ms_of_day,
                "bid_size": t.bid_size,
                "bid_exchange": t.bid_exchange,
                "bid": fmt_price(t.bid, t.price_type),
                "bid_condition": t.bid_condition,
                "ask_size": t.ask_size,
                "ask_exchange": t.ask_exchange,
                "ask": fmt_price(t.ask, t.price_type),
                "ask_condition": t.ask_condition,
                "date": t.date
            })
        })
        .collect()
}

/// Convert a raw `DataTable` to a JSON array of objects.
///
/// Proto `DataValue.data_type` oneof variants:
/// - `Text(String)`, `Number(i64)`, `Price(Price)`, `Timestamp(ZonedDateTime)`, `NullValue(i32)`
pub fn data_table_to_json(table: &proto::DataTable) -> Vec<sonic_rs::Value> {
    table
        .data_table
        .iter()
        .map(|row| {
            let mut obj = sonic_rs::Object::new();
            for (i, header) in table.headers.iter().enumerate() {
                if let Some(val) = row.values.get(i) {
                    use proto::data_value::DataType;
                    match &val.data_type {
                        Some(DataType::Number(n)) => {
                            obj.insert(header, sonic_rs::Value::from(*n));
                        }
                        Some(DataType::Text(s)) => {
                            obj.insert(header, sonic_rs::Value::from(s.as_str()));
                        }
                        Some(DataType::Price(p)) => {
                            let f = fmt_price(p.value, p.r#type);
                            let v = sonic_rs::Value::new_f64(f)
                                .unwrap_or_else(|| sonic_rs::Value::from(0i64));
                            obj.insert(header, v);
                        }
                        Some(DataType::Timestamp(ts)) => {
                            obj.insert(header, sonic_rs::Value::from(ts.epoch_ms));
                        }
                        Some(DataType::NullValue(_)) | None => {
                            obj.insert(header, sonic_rs::Value::default());
                        }
                    }
                }
            }
            sonic_rs::Value::from(obj)
        })
        .collect()
}

// ---------------------------------------------------------------------------
//  CSV formatting
// ---------------------------------------------------------------------------

/// Convert a JSON response array to CSV with headers.
///
/// Returns `None` if the response is empty. Each object's keys become CSV
/// column headers (order taken from the first row).
pub fn json_to_csv(response: &[sonic_rs::Value]) -> Option<String> {
    let first = response.first()?;
    let obj = first.as_object()?;
    let null_val = sonic_rs::Value::default();
    let keys: Vec<&str> = obj
        .iter()
        .map(|(k, _)| k)
        .collect();
    if keys.is_empty() {
        return None;
    }

    let mut out = String::with_capacity(response.len() * 128);
    // Header row
    for (i, k) in keys.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(k);
    }
    out.push('\n');

    // Data rows
    for row in response {
        if let Some(row_obj) = row.as_object() {
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let val = row_obj.get(k).unwrap_or(&null_val);
                if val.is_str() {
                    if let Some(s) = val.as_str() {
                        out.push_str(s);
                    }
                } else if val.is_null() {
                    // empty cell
                } else {
                    let rendered = sonic_rs::to_string(val).unwrap_or_default();
                    out.push_str(&rendered);
                }
            }
            out.push('\n');
        }
    }

    Some(out)
}
