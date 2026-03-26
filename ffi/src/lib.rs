//! C FFI layer for `thetadatadx` — exposes the Rust SDK as `extern "C"` functions.
//!
//! This crate is compiled as both `cdylib` (shared library) and `staticlib` (archive).
//! It is consumed by the Go (CGo) and C++ SDKs.
//!
//! # Safety
//!
//! All `unsafe extern "C"` functions in this crate follow the same safety contract:
//!
//! - Pointer arguments must be either null (handled gracefully) or valid pointers
//!   obtained from a prior `tdx_*` call.
//! - `*const c_char` arguments must point to valid, NUL-terminated C strings.
//! - Returned `*mut` pointers are heap-allocated and must be freed with the
//!   corresponding `tdx_*_free` function.
//! - Functions are not thread-safe on the same handle; callers must synchronize.
//!
//! # Memory model
//!
//! - Opaque handles (`*mut TdxClient`, `*mut TdxCredentials`, etc.) are heap-allocated
//!   via `Box::into_raw` and freed via the corresponding `tdx_*_free` function.
//! - String results are returned as JSON (`*mut c_char`), freed with `tdx_string_free`.
//! - The caller MUST free every non-null pointer returned by this library.
//!
//! # Error handling
//!
//! Functions that can fail return a null pointer on error and set a thread-local
//! error string retrievable via `tdx_last_error`.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::OnceLock;

// ── Global tokio runtime (same pattern as the Python bindings) ──

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime for thetadatadx-ffi")
    })
}

// ── Thread-local error string ──

thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = const { std::cell::RefCell::new(None) };
}

fn set_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

/// Retrieve the last error message (or null if no error).
///
/// The returned pointer is valid until the next FFI call on the same thread.
/// Do NOT free this pointer.
#[no_mangle]
pub extern "C" fn tdx_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        let borrow = e.borrow();
        match borrow.as_ref() {
            Some(s) => s.as_ptr(),
            None => ptr::null(),
        }
    })
}

// ── Opaque handle types ──

/// Opaque credentials handle.
pub struct TdxCredentials {
    inner: thetadatadx::Credentials,
}

/// Opaque client handle.
pub struct TdxClient {
    inner: thetadatadx::DirectClient,
}

/// Opaque config handle.
pub struct TdxConfig {
    inner: thetadatadx::DirectConfig,
}

// ── Helper: C string to &str ──

unsafe fn cstr_to_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

/// Helper: serialize Vec<serde_json::Value> to a C string (JSON array).
fn json_to_cstring(val: &serde_json::Value) -> *mut c_char {
    match CString::new(val.to_string()) {
        Ok(cs) => cs.into_raw(),
        Err(_) => {
            set_error("JSON serialization produced invalid C string");
            ptr::null_mut()
        }
    }
}

// ── Credentials ──

/// Create credentials from email and password strings.
///
/// Returns null on invalid input (check `tdx_last_error()`).
#[no_mangle]
pub unsafe extern "C" fn tdx_credentials_new(
    email: *const c_char,
    password: *const c_char,
) -> *mut TdxCredentials {
    let email = match unsafe { cstr_to_str(email) } {
        Some(s) => s,
        None => {
            set_error("email is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let password = match unsafe { cstr_to_str(password) } {
        Some(s) => s,
        None => {
            set_error("password is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let creds = thetadatadx::Credentials::new(email, password);
    Box::into_raw(Box::new(TdxCredentials { inner: creds }))
}

/// Load credentials from a file (line 1 = email, line 2 = password).
///
/// Returns null on error (check `tdx_last_error()`).
#[no_mangle]
pub unsafe extern "C" fn tdx_credentials_from_file(path: *const c_char) -> *mut TdxCredentials {
    let path = match unsafe { cstr_to_str(path) } {
        Some(s) => s,
        None => {
            set_error("path is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    match thetadatadx::Credentials::from_file(path) {
        Ok(creds) => Box::into_raw(Box::new(TdxCredentials { inner: creds })),
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Free a credentials handle.
#[no_mangle]
pub unsafe extern "C" fn tdx_credentials_free(creds: *mut TdxCredentials) {
    if !creds.is_null() {
        drop(unsafe { Box::from_raw(creds) });
    }
}

// ── Config ──

/// Create a production config (ThetaData NJ datacenter).
#[no_mangle]
pub extern "C" fn tdx_config_production() -> *mut TdxConfig {
    Box::into_raw(Box::new(TdxConfig {
        inner: thetadatadx::DirectConfig::production(),
    }))
}

/// Create a dev config (shorter timeouts).
#[no_mangle]
pub extern "C" fn tdx_config_dev() -> *mut TdxConfig {
    Box::into_raw(Box::new(TdxConfig {
        inner: thetadatadx::DirectConfig::dev(),
    }))
}

/// Free a config handle.
#[no_mangle]
pub unsafe extern "C" fn tdx_config_free(config: *mut TdxConfig) {
    if !config.is_null() {
        drop(unsafe { Box::from_raw(config) });
    }
}

// ── Client ──

/// Connect to ThetaData servers (authenticates via Nexus API).
///
/// Returns null on connection/auth failure (check `tdx_last_error()`).
#[no_mangle]
pub unsafe extern "C" fn tdx_client_connect(
    creds: *const TdxCredentials,
    config: *const TdxConfig,
) -> *mut TdxClient {
    if creds.is_null() {
        set_error("credentials handle is null");
        return ptr::null_mut();
    }
    if config.is_null() {
        set_error("config handle is null");
        return ptr::null_mut();
    }
    let creds = unsafe { &*creds };
    let config = unsafe { &*config };
    match runtime().block_on(thetadatadx::DirectClient::connect(
        &creds.inner,
        config.inner.clone(),
    )) {
        Ok(client) => Box::into_raw(Box::new(TdxClient { inner: client })),
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Free a client handle.
#[no_mangle]
pub unsafe extern "C" fn tdx_client_free(client: *mut TdxClient) {
    if !client.is_null() {
        drop(unsafe { Box::from_raw(client) });
    }
}

// ── String free ──

/// Free a string returned by any `tdx_*` function.
///
/// MUST be called for every non-null `*mut c_char` returned by this library.
#[no_mangle]
pub unsafe extern "C" fn tdx_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}

// ── Stock endpoints ──

/// List all available stock symbols.
///
/// Returns a JSON array of strings, e.g. `["AAPL","MSFT",...]`.
/// Caller must free the result with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_stock_list_symbols(client: *const TdxClient) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    match runtime().block_on(client.inner.stock_list_symbols()) {
        Ok(symbols) => {
            let json = serde_json::Value::Array(
                symbols.into_iter().map(serde_json::Value::String).collect(),
            );
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Helper macro for EOD tick serialization.
fn eod_tick_to_json(t: &thetadatadx::types::tick::EodTick) -> serde_json::Value {
    serde_json::json!({
        "ms_of_day": t.ms_of_day,
        "open": t.open_price().to_f64(),
        "high": t.high_price().to_f64(),
        "low": t.low_price().to_f64(),
        "close": t.close_price().to_f64(),
        "volume": t.volume,
        "count": t.count,
        "bid": t.bid_price().to_f64(),
        "ask": t.ask_price().to_f64(),
        "date": t.date,
    })
}

fn ohlc_tick_to_json(t: &thetadatadx::types::tick::OhlcTick) -> serde_json::Value {
    serde_json::json!({
        "ms_of_day": t.ms_of_day,
        "open": t.open_price().to_f64(),
        "high": t.high_price().to_f64(),
        "low": t.low_price().to_f64(),
        "close": t.close_price().to_f64(),
        "volume": t.volume,
        "count": t.count,
        "date": t.date,
    })
}

fn trade_tick_to_json(t: &thetadatadx::types::tick::TradeTick) -> serde_json::Value {
    serde_json::json!({
        "ms_of_day": t.ms_of_day,
        "sequence": t.sequence,
        "condition": t.condition,
        "size": t.size,
        "exchange": t.exchange,
        "price": t.get_price().to_f64(),
        "price_raw": t.price,
        "price_type": t.price_type,
        "condition_flags": t.condition_flags,
        "price_flags": t.price_flags,
        "volume_type": t.volume_type,
        "records_back": t.records_back,
        "date": t.date,
    })
}

fn quote_tick_to_json(t: &thetadatadx::types::tick::QuoteTick) -> serde_json::Value {
    serde_json::json!({
        "ms_of_day": t.ms_of_day,
        "bid_size": t.bid_size,
        "bid_exchange": t.bid_exchange,
        "bid": t.bid_price().to_f64(),
        "bid_condition": t.bid_condition,
        "ask_size": t.ask_size,
        "ask_exchange": t.ask_exchange,
        "ask": t.ask_price().to_f64(),
        "ask_condition": t.ask_condition,
        "date": t.date,
    })
}

/// Fetch stock end-of-day history.
///
/// Returns a JSON array of EOD tick objects.
/// Caller must free the result with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_stock_history_eod(
    client: *const TdxClient,
    symbol: *const c_char,
    start_date: *const c_char,
    end_date: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let symbol = match unsafe { cstr_to_str(symbol) } {
        Some(s) => s,
        None => {
            set_error("symbol is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let start = match unsafe { cstr_to_str(start_date) } {
        Some(s) => s,
        None => {
            set_error("start_date is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let end = match unsafe { cstr_to_str(end_date) } {
        Some(s) => s,
        None => {
            set_error("end_date is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match runtime().block_on(client.inner.stock_history_eod(symbol, start, end)) {
        Ok(ticks) => {
            let json = serde_json::Value::Array(ticks.iter().map(eod_tick_to_json).collect());
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Fetch stock intraday OHLC bars for a single date.
///
/// `interval` is in milliseconds as a string (e.g. "60000" for 1 min).
/// Returns a JSON array. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_stock_history_ohlc(
    client: *const TdxClient,
    symbol: *const c_char,
    date: *const c_char,
    interval: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let symbol = match unsafe { cstr_to_str(symbol) } {
        Some(s) => s,
        None => {
            set_error("symbol is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let date = match unsafe { cstr_to_str(date) } {
        Some(s) => s,
        None => {
            set_error("date is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let interval = match unsafe { cstr_to_str(interval) } {
        Some(s) => s,
        None => {
            set_error("interval is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match runtime().block_on(client.inner.stock_history_ohlc(symbol, date, interval)) {
        Ok(ticks) => {
            let json = serde_json::Value::Array(ticks.iter().map(ohlc_tick_to_json).collect());
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Fetch all trades on a given date.
///
/// Returns a JSON array. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_stock_history_trade(
    client: *const TdxClient,
    symbol: *const c_char,
    date: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let symbol = match unsafe { cstr_to_str(symbol) } {
        Some(s) => s,
        None => {
            set_error("symbol is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let date = match unsafe { cstr_to_str(date) } {
        Some(s) => s,
        None => {
            set_error("date is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match runtime().block_on(client.inner.stock_history_trade(symbol, date)) {
        Ok(ticks) => {
            let json = serde_json::Value::Array(ticks.iter().map(trade_tick_to_json).collect());
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Fetch NBBO quotes at a given interval.
///
/// Returns a JSON array. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_stock_history_quote(
    client: *const TdxClient,
    symbol: *const c_char,
    date: *const c_char,
    interval: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let symbol = match unsafe { cstr_to_str(symbol) } {
        Some(s) => s,
        None => {
            set_error("symbol is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let date = match unsafe { cstr_to_str(date) } {
        Some(s) => s,
        None => {
            set_error("date is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let interval = match unsafe { cstr_to_str(interval) } {
        Some(s) => s,
        None => {
            set_error("interval is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match runtime().block_on(client.inner.stock_history_quote(symbol, date, interval)) {
        Ok(ticks) => {
            let json = serde_json::Value::Array(ticks.iter().map(quote_tick_to_json).collect());
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Fetch latest NBBO quote snapshot for multiple symbols.
///
/// `symbols_json` is a JSON array of strings, e.g. `["AAPL","MSFT"]`.
/// Returns a JSON array. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_stock_snapshot_quote(
    client: *const TdxClient,
    symbols_json: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let json_str = match unsafe { cstr_to_str(symbols_json) } {
        Some(s) => s,
        None => {
            set_error("symbols_json is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let symbols: Vec<String> = match serde_json::from_str(json_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(&format!("invalid symbols JSON: {}", e));
            return ptr::null_mut();
        }
    };
    let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();

    match runtime().block_on(client.inner.stock_snapshot_quote(&refs)) {
        Ok(ticks) => {
            let json = serde_json::Value::Array(ticks.iter().map(quote_tick_to_json).collect());
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

// ── Option endpoints ──

/// List expiration dates for an underlying.
///
/// Returns a JSON array of date strings. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_option_list_expirations(
    client: *const TdxClient,
    symbol: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let symbol = match unsafe { cstr_to_str(symbol) } {
        Some(s) => s,
        None => {
            set_error("symbol is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match runtime().block_on(client.inner.option_list_expirations(symbol)) {
        Ok(exps) => {
            let json =
                serde_json::Value::Array(exps.into_iter().map(serde_json::Value::String).collect());
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// List strike prices for a given expiration.
///
/// Returns a JSON array of strings. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_option_list_strikes(
    client: *const TdxClient,
    symbol: *const c_char,
    expiration: *const c_char,
) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };
    let symbol = match unsafe { cstr_to_str(symbol) } {
        Some(s) => s,
        None => {
            set_error("symbol is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };
    let expiration = match unsafe { cstr_to_str(expiration) } {
        Some(s) => s,
        None => {
            set_error("expiration is null or invalid UTF-8");
            return ptr::null_mut();
        }
    };

    match runtime().block_on(client.inner.option_list_strikes(symbol, expiration)) {
        Ok(strikes) => {
            let json = serde_json::Value::Array(
                strikes.into_iter().map(serde_json::Value::String).collect(),
            );
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// List all option underlyings.
///
/// Returns a JSON array of strings. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_option_list_symbols(client: *const TdxClient) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };

    match runtime().block_on(client.inner.option_list_symbols()) {
        Ok(symbols) => {
            let json = serde_json::Value::Array(
                symbols.into_iter().map(serde_json::Value::String).collect(),
            );
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

// ── Index endpoints ──

/// List all index symbols.
///
/// Returns a JSON array of strings. Caller must free with `tdx_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tdx_index_list_symbols(client: *const TdxClient) -> *mut c_char {
    if client.is_null() {
        set_error("client handle is null");
        return ptr::null_mut();
    }
    let client = unsafe { &*client };

    match runtime().block_on(client.inner.index_list_symbols()) {
        Ok(symbols) => {
            let json = serde_json::Value::Array(
                symbols.into_iter().map(serde_json::Value::String).collect(),
            );
            json_to_cstring(&json)
        }
        Err(e) => {
            set_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

// ── Greeks ──

/// Compute all 22 Black-Scholes Greeks + IV.
///
/// Returns a JSON object with all greek values.
/// Caller must free the result with `tdx_string_free`.
#[no_mangle]
pub extern "C" fn tdx_all_greeks(
    spot: f64,
    strike: f64,
    rate: f64,
    div_yield: f64,
    tte: f64,
    option_price: f64,
    is_call: i32,
) -> *mut c_char {
    let g = thetadatadx::greeks::all_greeks(
        spot,
        strike,
        rate,
        div_yield,
        tte,
        option_price,
        is_call != 0,
    );
    let json = serde_json::json!({
        "value": g.value,
        "delta": g.delta,
        "gamma": g.gamma,
        "theta": g.theta,
        "vega": g.vega,
        "rho": g.rho,
        "iv": g.iv,
        "iv_error": g.iv_error,
        "vanna": g.vanna,
        "charm": g.charm,
        "vomma": g.vomma,
        "veta": g.veta,
        "speed": g.speed,
        "zomma": g.zomma,
        "color": g.color,
        "ultima": g.ultima,
        "d1": g.d1,
        "d2": g.d2,
        "dual_delta": g.dual_delta,
        "dual_gamma": g.dual_gamma,
        "epsilon": g.epsilon,
        "lambda": g.lambda,
    });
    json_to_cstring(&json)
}

/// Compute implied volatility via bisection.
///
/// Returns IV in `*out_iv` and error in `*out_error`.
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn tdx_implied_volatility(
    spot: f64,
    strike: f64,
    rate: f64,
    div_yield: f64,
    tte: f64,
    option_price: f64,
    is_call: i32,
    out_iv: *mut f64,
    out_error: *mut f64,
) -> i32 {
    if out_iv.is_null() || out_error.is_null() {
        set_error("output pointers must not be null");
        return -1;
    }
    let (iv, err) = thetadatadx::greeks::implied_volatility(
        spot,
        strike,
        rate,
        div_yield,
        tte,
        option_price,
        is_call != 0,
    );
    unsafe {
        *out_iv = iv;
        *out_error = err;
    }
    0
}
