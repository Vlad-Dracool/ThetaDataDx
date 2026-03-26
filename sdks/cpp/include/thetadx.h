/**
 * thetadatadx C FFI header.
 *
 * This header declares the C interface to the thetadatadx Rust SDK.
 * Used by both the C++ wrapper and any other C-compatible language.
 *
 * Memory model:
 * - Opaque handles (TdxCredentials*, TdxClient*, TdxConfig*) are heap-allocated
 *   by the Rust side and MUST be freed with the corresponding tdx_*_free function.
 * - String results (char*) are heap-allocated JSON and MUST be freed with tdx_string_free.
 * - Functions that can fail return NULL and set a thread-local error string
 *   retrievable via tdx_last_error().
 */

#ifndef THETADX_H
#define THETADX_H

#ifdef __cplusplus
extern "C" {
#endif

/* ── Opaque handle types ── */
typedef struct TdxCredentials TdxCredentials;
typedef struct TdxClient TdxClient;
typedef struct TdxConfig TdxConfig;

/* ── Error ── */

/** Retrieve the last error message (or NULL if no error).
 *  The returned pointer is valid until the next FFI call on the same thread.
 *  Do NOT free this pointer. */
const char* tdx_last_error(void);

/* ── Credentials ── */

/** Create credentials from email and password. Returns NULL on error. */
TdxCredentials* tdx_credentials_new(const char* email, const char* password);

/** Load credentials from a file (line 1 = email, line 2 = password). Returns NULL on error. */
TdxCredentials* tdx_credentials_from_file(const char* path);

/** Free a credentials handle. */
void tdx_credentials_free(TdxCredentials* creds);

/* ── Config ── */

/** Create a production config (ThetaData NJ datacenter). */
TdxConfig* tdx_config_production(void);

/** Create a dev config (shorter timeouts). */
TdxConfig* tdx_config_dev(void);

/** Free a config handle. */
void tdx_config_free(TdxConfig* config);

/* ── Client ── */

/** Connect to ThetaData servers. Returns NULL on connection/auth failure. */
TdxClient* tdx_client_connect(const TdxCredentials* creds, const TdxConfig* config);

/** Free a client handle. */
void tdx_client_free(TdxClient* client);

/* ── String free ── */

/** Free a string returned by any tdx_* function. */
void tdx_string_free(char* s);

/* ── Stock endpoints ── */

/** List all stock symbols. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_stock_list_symbols(const TdxClient* client);

/** Fetch stock EOD history. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_stock_history_eod(const TdxClient* client, const char* symbol,
                            const char* start_date, const char* end_date);

/** Fetch stock intraday OHLC. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_stock_history_ohlc(const TdxClient* client, const char* symbol,
                             const char* date, const char* interval);

/** Fetch all trades on a date. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_stock_history_trade(const TdxClient* client, const char* symbol, const char* date);

/** Fetch NBBO quotes. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_stock_history_quote(const TdxClient* client, const char* symbol,
                              const char* date, const char* interval);

/** Fetch live quote snapshot. symbols_json is a JSON array of strings.
 *  Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_stock_snapshot_quote(const TdxClient* client, const char* symbols_json);

/* ── Option endpoints ── */

/** List expiration dates. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_option_list_expirations(const TdxClient* client, const char* symbol);

/** List strikes for an expiration. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_option_list_strikes(const TdxClient* client, const char* symbol,
                              const char* expiration);

/** List all option underlyings. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_option_list_symbols(const TdxClient* client);

/* ── Index endpoints ── */

/** List all index symbols. Returns JSON array. Caller must free with tdx_string_free. */
char* tdx_index_list_symbols(const TdxClient* client);

/* ── Greeks ── */

/** Compute all 22 Greeks + IV. Returns JSON object. Caller must free with tdx_string_free. */
char* tdx_all_greeks(double spot, double strike, double rate, double div_yield,
                     double tte, double option_price, int is_call);

/** Compute implied volatility. Returns 0 on success, -1 on failure. */
int tdx_implied_volatility(double spot, double strike, double rate, double div_yield,
                           double tte, double option_price, int is_call,
                           double* out_iv, double* out_error);

#ifdef __cplusplus
}
#endif

#endif /* THETADX_H */
