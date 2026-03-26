/**
 * thetadatadx C++ RAII wrapper.
 *
 * Wraps the C FFI handles in RAII classes with unique_ptr-based ownership.
 * All data methods return parsed C++ types (vectors of structs) from JSON.
 *
 * This file is a single-header-ish implementation. Include "thetadatadx.hpp" (below)
 * or compile this .cpp with the include/ header for the C types.
 */

#include "thetadatadx.hpp"

#include <stdexcept>
#include <sstream>

namespace tdx {

// ── JSON parsing helpers ──
// Minimal JSON parsing using string manipulation (no external dependency).
// For production use, consider nlohmann/json or simdjson.

namespace detail {

static std::string last_ffi_error() {
    const char* err = tdx_last_error();
    return err ? std::string(err) : "unknown error";
}

// Managed C string from FFI: auto-frees on destruction.
struct FfiString {
    char* ptr;
    FfiString(char* p) : ptr(p) {}
    ~FfiString() { if (ptr) tdx_string_free(ptr); }
    FfiString(const FfiString&) = delete;
    FfiString& operator=(const FfiString&) = delete;

    std::string str() const { return ptr ? std::string(ptr) : ""; }
    bool ok() const { return ptr != nullptr; }
};

// Simple JSON value extraction (numbers and strings from objects).
// This is intentionally minimal. For a real project, use a JSON library.

static double json_double(const std::string& json, const std::string& key) {
    std::string needle = "\"" + key + "\":";
    auto pos = json.find(needle);
    if (pos == std::string::npos) return 0.0;
    pos += needle.size();
    // Skip whitespace
    while (pos < json.size() && (json[pos] == ' ' || json[pos] == '\t')) ++pos;
    return std::stod(json.substr(pos));
}

static int json_int(const std::string& json, const std::string& key) {
    std::string needle = "\"" + key + "\":";
    auto pos = json.find(needle);
    if (pos == std::string::npos) return 0;
    pos += needle.size();
    while (pos < json.size() && (json[pos] == ' ' || json[pos] == '\t')) ++pos;
    return std::stoi(json.substr(pos));
}

// Split a JSON array of objects into individual object strings.
static std::vector<std::string> split_json_array(const std::string& json) {
    std::vector<std::string> result;
    int depth = 0;
    size_t start = 0;
    bool in_string = false;
    bool escaped = false;

    for (size_t i = 0; i < json.size(); ++i) {
        char c = json[i];
        if (escaped) { escaped = false; continue; }
        if (c == '\\') { escaped = true; continue; }
        if (c == '"') { in_string = !in_string; continue; }
        if (in_string) continue;

        if (c == '{') {
            if (depth == 0) start = i;
            ++depth;
        } else if (c == '}') {
            --depth;
            if (depth == 0) {
                result.push_back(json.substr(start, i - start + 1));
            }
        }
    }
    return result;
}

// Parse a JSON array of strings: ["a","b","c"]
static std::vector<std::string> parse_string_array(const std::string& json) {
    std::vector<std::string> result;
    bool in_string = false;
    bool escaped = false;
    std::string current;

    for (size_t i = 0; i < json.size(); ++i) {
        char c = json[i];
        if (escaped) { current += c; escaped = false; continue; }
        if (c == '\\') { escaped = true; continue; }
        if (c == '"') {
            if (in_string) {
                result.push_back(current);
                current.clear();
            }
            in_string = !in_string;
            continue;
        }
        if (in_string) current += c;
    }
    return result;
}

static EodTick parse_eod_tick(const std::string& obj) {
    return EodTick{
        json_int(obj, "ms_of_day"),
        json_double(obj, "open"),
        json_double(obj, "high"),
        json_double(obj, "low"),
        json_double(obj, "close"),
        json_int(obj, "volume"),
        json_int(obj, "count"),
        json_double(obj, "bid"),
        json_double(obj, "ask"),
        json_int(obj, "date"),
    };
}

static OhlcTick parse_ohlc_tick(const std::string& obj) {
    return OhlcTick{
        json_int(obj, "ms_of_day"),
        json_double(obj, "open"),
        json_double(obj, "high"),
        json_double(obj, "low"),
        json_double(obj, "close"),
        json_int(obj, "volume"),
        json_int(obj, "count"),
        json_int(obj, "date"),
    };
}

static TradeTick parse_trade_tick(const std::string& obj) {
    return TradeTick{
        json_int(obj, "ms_of_day"),
        json_int(obj, "sequence"),
        json_int(obj, "condition"),
        json_int(obj, "size"),
        json_int(obj, "exchange"),
        json_double(obj, "price"),
        json_int(obj, "price_raw"),
        json_int(obj, "price_type"),
        json_int(obj, "condition_flags"),
        json_int(obj, "price_flags"),
        json_int(obj, "volume_type"),
        json_int(obj, "records_back"),
        json_int(obj, "date"),
    };
}

static QuoteTick parse_quote_tick(const std::string& obj) {
    return QuoteTick{
        json_int(obj, "ms_of_day"),
        json_int(obj, "bid_size"),
        json_int(obj, "bid_exchange"),
        json_double(obj, "bid"),
        json_int(obj, "bid_condition"),
        json_int(obj, "ask_size"),
        json_int(obj, "ask_exchange"),
        json_double(obj, "ask"),
        json_int(obj, "ask_condition"),
        json_int(obj, "date"),
    };
}

} // namespace detail

// ── Credentials ──

Credentials Credentials::from_file(const std::string& path) {
    TdxCredentials* h = tdx_credentials_from_file(path.c_str());
    if (!h) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return Credentials(h);
}

Credentials Credentials::from_email(const std::string& email, const std::string& password) {
    TdxCredentials* h = tdx_credentials_new(email.c_str(), password.c_str());
    if (!h) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return Credentials(h);
}

// ── Config ──

Config Config::production() {
    return Config(tdx_config_production());
}

Config Config::dev() {
    return Config(tdx_config_dev());
}

// ── Client ──

Client Client::connect(const Credentials& creds, const Config& config) {
    TdxClient* h = tdx_client_connect(creds.get(), config.get());
    if (!h) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return Client(h);
}

// Stock endpoints

std::vector<std::string> Client::stock_list_symbols() const {
    detail::FfiString result(tdx_stock_list_symbols(handle_.get()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return detail::parse_string_array(result.str());
}

std::vector<EodTick> Client::stock_history_eod(
    const std::string& symbol, const std::string& start_date, const std::string& end_date) const
{
    detail::FfiString result(tdx_stock_history_eod(
        handle_.get(), symbol.c_str(), start_date.c_str(), end_date.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());

    auto objects = detail::split_json_array(result.str());
    std::vector<EodTick> ticks;
    ticks.reserve(objects.size());
    for (auto& obj : objects) ticks.push_back(detail::parse_eod_tick(obj));
    return ticks;
}

std::vector<OhlcTick> Client::stock_history_ohlc(
    const std::string& symbol, const std::string& date, const std::string& interval) const
{
    detail::FfiString result(tdx_stock_history_ohlc(
        handle_.get(), symbol.c_str(), date.c_str(), interval.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());

    auto objects = detail::split_json_array(result.str());
    std::vector<OhlcTick> ticks;
    ticks.reserve(objects.size());
    for (auto& obj : objects) ticks.push_back(detail::parse_ohlc_tick(obj));
    return ticks;
}

std::vector<TradeTick> Client::stock_history_trade(
    const std::string& symbol, const std::string& date) const
{
    detail::FfiString result(tdx_stock_history_trade(
        handle_.get(), symbol.c_str(), date.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());

    auto objects = detail::split_json_array(result.str());
    std::vector<TradeTick> ticks;
    ticks.reserve(objects.size());
    for (auto& obj : objects) ticks.push_back(detail::parse_trade_tick(obj));
    return ticks;
}

std::vector<QuoteTick> Client::stock_history_quote(
    const std::string& symbol, const std::string& date, const std::string& interval) const
{
    detail::FfiString result(tdx_stock_history_quote(
        handle_.get(), symbol.c_str(), date.c_str(), interval.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());

    auto objects = detail::split_json_array(result.str());
    std::vector<QuoteTick> ticks;
    ticks.reserve(objects.size());
    for (auto& obj : objects) ticks.push_back(detail::parse_quote_tick(obj));
    return ticks;
}

std::vector<QuoteTick> Client::stock_snapshot_quote(const std::vector<std::string>& symbols) const {
    // Build JSON array manually
    std::string json = "[";
    for (size_t i = 0; i < symbols.size(); ++i) {
        if (i > 0) json += ",";
        json += "\"" + symbols[i] + "\"";
    }
    json += "]";

    detail::FfiString result(tdx_stock_snapshot_quote(handle_.get(), json.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());

    auto objects = detail::split_json_array(result.str());
    std::vector<QuoteTick> ticks;
    ticks.reserve(objects.size());
    for (auto& obj : objects) ticks.push_back(detail::parse_quote_tick(obj));
    return ticks;
}

// Option endpoints

std::vector<std::string> Client::option_list_expirations(const std::string& symbol) const {
    detail::FfiString result(tdx_option_list_expirations(handle_.get(), symbol.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return detail::parse_string_array(result.str());
}

std::vector<std::string> Client::option_list_strikes(
    const std::string& symbol, const std::string& expiration) const
{
    detail::FfiString result(tdx_option_list_strikes(
        handle_.get(), symbol.c_str(), expiration.c_str()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return detail::parse_string_array(result.str());
}

std::vector<std::string> Client::option_list_symbols() const {
    detail::FfiString result(tdx_option_list_symbols(handle_.get()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return detail::parse_string_array(result.str());
}

// Index endpoints

std::vector<std::string> Client::index_list_symbols() const {
    detail::FfiString result(tdx_index_list_symbols(handle_.get()));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return detail::parse_string_array(result.str());
}

// Greeks

Greeks all_greeks(double spot, double strike, double rate, double div_yield,
                  double tte, double option_price, bool is_call)
{
    detail::FfiString result(tdx_all_greeks(
        spot, strike, rate, div_yield, tte, option_price, is_call ? 1 : 0));
    if (!result.ok()) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());

    auto s = result.str();
    return Greeks{
        detail::json_double(s, "value"),
        detail::json_double(s, "delta"),
        detail::json_double(s, "gamma"),
        detail::json_double(s, "theta"),
        detail::json_double(s, "vega"),
        detail::json_double(s, "rho"),
        detail::json_double(s, "iv"),
        detail::json_double(s, "iv_error"),
        detail::json_double(s, "vanna"),
        detail::json_double(s, "charm"),
        detail::json_double(s, "vomma"),
        detail::json_double(s, "veta"),
        detail::json_double(s, "speed"),
        detail::json_double(s, "zomma"),
        detail::json_double(s, "color"),
        detail::json_double(s, "ultima"),
        detail::json_double(s, "d1"),
        detail::json_double(s, "d2"),
        detail::json_double(s, "dual_delta"),
        detail::json_double(s, "dual_gamma"),
        detail::json_double(s, "epsilon"),
        detail::json_double(s, "lambda"),
    };
}

std::pair<double, double> implied_volatility(
    double spot, double strike, double rate, double div_yield,
    double tte, double option_price, bool is_call)
{
    double iv = 0.0, err = 0.0;
    int rc = tdx_implied_volatility(
        spot, strike, rate, div_yield, tte, option_price,
        is_call ? 1 : 0, &iv, &err);
    if (rc != 0) throw std::runtime_error("thetadatadx: " + detail::last_ffi_error());
    return {iv, err};
}

} // namespace tdx
