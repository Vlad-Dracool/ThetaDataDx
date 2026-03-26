/**
 * thetadatadx C++ SDK.
 *
 * RAII wrappers around the C FFI layer. Provides idiomatic C++ access to
 * ThetaData market data with automatic resource management.
 */

#ifndef THETADX_HPP
#define THETADX_HPP

#include "thetadatadx.h"

#include <memory>
#include <string>
#include <vector>
#include <utility>

namespace tdx {

// ── Tick types ──

struct EodTick {
    int ms_of_day;
    double open;
    double high;
    double low;
    double close;
    int volume;
    int count;
    double bid;
    double ask;
    int date;
};

struct OhlcTick {
    int ms_of_day;
    double open;
    double high;
    double low;
    double close;
    int volume;
    int count;
    int date;
};

struct TradeTick {
    int ms_of_day;
    int sequence;
    int condition;
    int size;
    int exchange;
    double price;
    int price_raw;
    int price_type;
    int condition_flags;
    int price_flags;
    int volume_type;
    int records_back;
    int date;
};

struct QuoteTick {
    int ms_of_day;
    int bid_size;
    int bid_exchange;
    double bid;
    int bid_condition;
    int ask_size;
    int ask_exchange;
    double ask;
    int ask_condition;
    int date;
};

struct Greeks {
    double value;
    double delta;
    double gamma;
    double theta;
    double vega;
    double rho;
    double iv;
    double iv_error;
    double vanna;
    double charm;
    double vomma;
    double veta;
    double speed;
    double zomma;
    double color;
    double ultima;
    double d1;
    double d2;
    double dual_delta;
    double dual_gamma;
    double epsilon;
    double lambda;
};

// ── RAII deleters ──

struct CredentialsDeleter {
    void operator()(TdxCredentials* p) const { if (p) tdx_credentials_free(p); }
};

struct ConfigDeleter {
    void operator()(TdxConfig* p) const { if (p) tdx_config_free(p); }
};

struct ClientDeleter {
    void operator()(TdxClient* p) const { if (p) tdx_client_free(p); }
};

// ── Credentials ──

class Credentials {
public:
    /** Load credentials from a file (line 1 = email, line 2 = password). */
    static Credentials from_file(const std::string& path);

    /** Create credentials from email and password. */
    static Credentials from_email(const std::string& email, const std::string& password);

    /** Get the raw handle (for passing to Client::connect). */
    TdxCredentials* get() const { return handle_.get(); }

private:
    explicit Credentials(TdxCredentials* h) : handle_(h) {}
    std::unique_ptr<TdxCredentials, CredentialsDeleter> handle_;
};

// ── Config ──

class Config {
public:
    /** Production config (ThetaData NJ datacenter). */
    static Config production();

    /** Dev config (shorter timeouts). */
    static Config dev();

    /** Get the raw handle. */
    TdxConfig* get() const { return handle_.get(); }

private:
    explicit Config(TdxConfig* h) : handle_(h) {}
    std::unique_ptr<TdxConfig, ConfigDeleter> handle_;
};

// ── Client ──

class Client {
public:
    /** Connect to ThetaData servers. Throws on failure. */
    static Client connect(const Credentials& creds, const Config& config);

    // Stock endpoints
    std::vector<std::string> stock_list_symbols() const;
    std::vector<EodTick> stock_history_eod(const std::string& symbol,
                                           const std::string& start_date,
                                           const std::string& end_date) const;
    std::vector<OhlcTick> stock_history_ohlc(const std::string& symbol,
                                             const std::string& date,
                                             const std::string& interval) const;
    std::vector<TradeTick> stock_history_trade(const std::string& symbol,
                                               const std::string& date) const;
    std::vector<QuoteTick> stock_history_quote(const std::string& symbol,
                                               const std::string& date,
                                               const std::string& interval) const;
    std::vector<QuoteTick> stock_snapshot_quote(const std::vector<std::string>& symbols) const;

    // Option endpoints
    std::vector<std::string> option_list_expirations(const std::string& symbol) const;
    std::vector<std::string> option_list_strikes(const std::string& symbol,
                                                  const std::string& expiration) const;
    std::vector<std::string> option_list_symbols() const;

    // Index endpoints
    std::vector<std::string> index_list_symbols() const;

private:
    explicit Client(TdxClient* h) : handle_(h) {}
    std::unique_ptr<TdxClient, ClientDeleter> handle_;
};

// ── Standalone Greeks functions ──

/** Compute all 22 Greeks + IV. Throws on failure. */
Greeks all_greeks(double spot, double strike, double rate, double div_yield,
                  double tte, double option_price, bool is_call);

/** Compute implied volatility. Returns (iv, error). Throws on failure. */
std::pair<double, double> implied_volatility(double spot, double strike,
                                              double rate, double div_yield,
                                              double tte, double option_price,
                                              bool is_call);

} // namespace tdx

#endif /* THETADX_HPP */
