package thetadatadx

/*
#include <stdlib.h>

// Forward declarations (already defined in thetadatadx.go, but CGo needs them per file).
typedef void TdxCredentials;
typedef void TdxClient;
typedef void TdxConfig;

extern TdxClient* tdx_client_connect(const TdxCredentials* creds, const TdxConfig* config);
extern void tdx_client_free(TdxClient* client);
extern char* tdx_stock_list_symbols(const TdxClient* client);
extern char* tdx_stock_history_eod(const TdxClient* client, const char* symbol, const char* start_date, const char* end_date);
extern char* tdx_stock_history_ohlc(const TdxClient* client, const char* symbol, const char* date, const char* interval);
extern char* tdx_stock_history_trade(const TdxClient* client, const char* symbol, const char* date);
extern char* tdx_stock_history_quote(const TdxClient* client, const char* symbol, const char* date, const char* interval);
extern char* tdx_stock_snapshot_quote(const TdxClient* client, const char* symbols_json);
extern char* tdx_option_list_expirations(const TdxClient* client, const char* symbol);
extern char* tdx_option_list_strikes(const TdxClient* client, const char* symbol, const char* expiration);
extern char* tdx_option_list_symbols(const TdxClient* client);
extern char* tdx_index_list_symbols(const TdxClient* client);
extern char* tdx_all_greeks(double spot, double strike, double rate, double div_yield, double tte, double option_price, int is_call);
extern int tdx_implied_volatility(double spot, double strike, double rate, double div_yield, double tte, double option_price, int is_call, double* out_iv, double* out_error);
extern void tdx_string_free(char* s);
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"
)

// ── Tick types ──

// EodTick represents an end-of-day tick.
type EodTick struct {
	MsOfDay int     `json:"ms_of_day"`
	Open    float64 `json:"open"`
	High    float64 `json:"high"`
	Low     float64 `json:"low"`
	Close   float64 `json:"close"`
	Volume  int     `json:"volume"`
	Count   int     `json:"count"`
	Bid     float64 `json:"bid"`
	Ask     float64 `json:"ask"`
	Date    int     `json:"date"`
}

// OhlcTick represents an OHLC bar.
type OhlcTick struct {
	MsOfDay int     `json:"ms_of_day"`
	Open    float64 `json:"open"`
	High    float64 `json:"high"`
	Low     float64 `json:"low"`
	Close   float64 `json:"close"`
	Volume  int     `json:"volume"`
	Count   int     `json:"count"`
	Date    int     `json:"date"`
}

// TradeTick represents a trade.
type TradeTick struct {
	MsOfDay        int     `json:"ms_of_day"`
	Sequence       int     `json:"sequence"`
	Condition      int     `json:"condition"`
	Size           int     `json:"size"`
	Exchange       int     `json:"exchange"`
	Price          float64 `json:"price"`
	PriceRaw       int     `json:"price_raw"`
	PriceType      int     `json:"price_type"`
	ConditionFlags int     `json:"condition_flags"`
	PriceFlags     int     `json:"price_flags"`
	VolumeType     int     `json:"volume_type"`
	RecordsBack    int     `json:"records_back"`
	Date           int     `json:"date"`
}

// QuoteTick represents an NBBO quote.
type QuoteTick struct {
	MsOfDay      int     `json:"ms_of_day"`
	BidSize      int     `json:"bid_size"`
	BidExchange  int     `json:"bid_exchange"`
	Bid          float64 `json:"bid"`
	BidCondition int     `json:"bid_condition"`
	AskSize      int     `json:"ask_size"`
	AskExchange  int     `json:"ask_exchange"`
	Ask          float64 `json:"ask"`
	AskCondition int     `json:"ask_condition"`
	Date         int     `json:"date"`
}

// Greeks holds all 22 Black-Scholes Greeks plus IV.
type Greeks struct {
	Value     float64 `json:"value"`
	Delta     float64 `json:"delta"`
	Gamma     float64 `json:"gamma"`
	Theta     float64 `json:"theta"`
	Vega      float64 `json:"vega"`
	Rho       float64 `json:"rho"`
	IV        float64 `json:"iv"`
	IVError   float64 `json:"iv_error"`
	Vanna     float64 `json:"vanna"`
	Charm     float64 `json:"charm"`
	Vomma     float64 `json:"vomma"`
	Veta      float64 `json:"veta"`
	Speed     float64 `json:"speed"`
	Zomma     float64 `json:"zomma"`
	Color     float64 `json:"color"`
	Ultima    float64 `json:"ultima"`
	D1        float64 `json:"d1"`
	D2        float64 `json:"d2"`
	DualDelta float64 `json:"dual_delta"`
	DualGamma float64 `json:"dual_gamma"`
	Epsilon   float64 `json:"epsilon"`
	Lambda    float64 `json:"lambda"`
}

// ── Client ──

// Client is a high-level ThetaData client.
type Client struct {
	handle *C.TdxClient
}

// Connect authenticates and connects to ThetaData servers.
func Connect(creds *Credentials, config *Config) (*Client, error) {
	if creds == nil || creds.handle == nil {
		return nil, fmt.Errorf("thetadatadx: credentials handle is nil")
	}
	if config == nil || config.handle == nil {
		return nil, fmt.Errorf("thetadatadx: config handle is nil")
	}
	h := C.tdx_client_connect(creds.handle, config.handle)
	if h == nil {
		return nil, fmt.Errorf("thetadatadx: %s", lastError())
	}
	return &Client{handle: h}, nil
}

// Close disconnects and frees the client handle.
func (c *Client) Close() {
	if c.handle != nil {
		C.tdx_client_free(c.handle)
		c.handle = nil
	}
}

// ── Stock endpoints ──

// StockListSymbols returns all available stock symbols.
func (c *Client) StockListSymbols() ([]string, error) {
	raw, err := callJSON(C.tdx_stock_list_symbols(c.handle))
	if err != nil {
		return nil, err
	}
	var result []string
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse symbols: %w", err)
	}
	return result, nil
}

// StockHistoryEOD fetches end-of-day stock data for a date range.
// Dates are YYYYMMDD strings.
func (c *Client) StockHistoryEOD(symbol, startDate, endDate string) ([]EodTick, error) {
	cSymbol := C.CString(symbol)
	cStart := C.CString(startDate)
	cEnd := C.CString(endDate)
	defer C.free(unsafe.Pointer(cSymbol))
	defer C.free(unsafe.Pointer(cStart))
	defer C.free(unsafe.Pointer(cEnd))

	raw, err := callJSON(C.tdx_stock_history_eod(c.handle, cSymbol, cStart, cEnd))
	if err != nil {
		return nil, err
	}
	var result []EodTick
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse EOD ticks: %w", err)
	}
	return result, nil
}

// StockHistoryOHLC fetches intraday OHLC bars for a single date.
// Interval is in milliseconds as a string (e.g. "60000" for 1 min).
func (c *Client) StockHistoryOHLC(symbol, date, interval string) ([]OhlcTick, error) {
	cSymbol := C.CString(symbol)
	cDate := C.CString(date)
	cInterval := C.CString(interval)
	defer C.free(unsafe.Pointer(cSymbol))
	defer C.free(unsafe.Pointer(cDate))
	defer C.free(unsafe.Pointer(cInterval))

	raw, err := callJSON(C.tdx_stock_history_ohlc(c.handle, cSymbol, cDate, cInterval))
	if err != nil {
		return nil, err
	}
	var result []OhlcTick
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse OHLC ticks: %w", err)
	}
	return result, nil
}

// StockHistoryTrade fetches all trades on a given date.
func (c *Client) StockHistoryTrade(symbol, date string) ([]TradeTick, error) {
	cSymbol := C.CString(symbol)
	cDate := C.CString(date)
	defer C.free(unsafe.Pointer(cSymbol))
	defer C.free(unsafe.Pointer(cDate))

	raw, err := callJSON(C.tdx_stock_history_trade(c.handle, cSymbol, cDate))
	if err != nil {
		return nil, err
	}
	var result []TradeTick
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse trade ticks: %w", err)
	}
	return result, nil
}

// StockHistoryQuote fetches NBBO quotes at a given interval.
func (c *Client) StockHistoryQuote(symbol, date, interval string) ([]QuoteTick, error) {
	cSymbol := C.CString(symbol)
	cDate := C.CString(date)
	cInterval := C.CString(interval)
	defer C.free(unsafe.Pointer(cSymbol))
	defer C.free(unsafe.Pointer(cDate))
	defer C.free(unsafe.Pointer(cInterval))

	raw, err := callJSON(C.tdx_stock_history_quote(c.handle, cSymbol, cDate, cInterval))
	if err != nil {
		return nil, err
	}
	var result []QuoteTick
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse quote ticks: %w", err)
	}
	return result, nil
}

// StockSnapshotQuote fetches the latest NBBO quote snapshot for the given symbols.
func (c *Client) StockSnapshotQuote(symbols []string) ([]QuoteTick, error) {
	symbolsJSON, err := json.Marshal(symbols)
	if err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to serialize symbols: %w", err)
	}
	cJSON := C.CString(string(symbolsJSON))
	defer C.free(unsafe.Pointer(cJSON))

	raw, jsonErr := callJSON(C.tdx_stock_snapshot_quote(c.handle, cJSON))
	if jsonErr != nil {
		return nil, jsonErr
	}
	var result []QuoteTick
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse quote ticks: %w", err)
	}
	return result, nil
}

// ── Option endpoints ──

// OptionListExpirations returns expiration dates for an underlying.
func (c *Client) OptionListExpirations(symbol string) ([]string, error) {
	cSymbol := C.CString(symbol)
	defer C.free(unsafe.Pointer(cSymbol))

	raw, err := callJSON(C.tdx_option_list_expirations(c.handle, cSymbol))
	if err != nil {
		return nil, err
	}
	var result []string
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse expirations: %w", err)
	}
	return result, nil
}

// OptionListStrikes returns strike prices for a given expiration.
func (c *Client) OptionListStrikes(symbol, expiration string) ([]string, error) {
	cSymbol := C.CString(symbol)
	cExp := C.CString(expiration)
	defer C.free(unsafe.Pointer(cSymbol))
	defer C.free(unsafe.Pointer(cExp))

	raw, err := callJSON(C.tdx_option_list_strikes(c.handle, cSymbol, cExp))
	if err != nil {
		return nil, err
	}
	var result []string
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse strikes: %w", err)
	}
	return result, nil
}

// OptionListSymbols returns all option underlyings.
func (c *Client) OptionListSymbols() ([]string, error) {
	raw, err := callJSON(C.tdx_option_list_symbols(c.handle))
	if err != nil {
		return nil, err
	}
	var result []string
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse symbols: %w", err)
	}
	return result, nil
}

// ── Index endpoints ──

// IndexListSymbols returns all index symbols.
func (c *Client) IndexListSymbols() ([]string, error) {
	raw, err := callJSON(C.tdx_index_list_symbols(c.handle))
	if err != nil {
		return nil, err
	}
	var result []string
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse symbols: %w", err)
	}
	return result, nil
}

// ── Greeks ──

// AllGreeks computes all 22 Black-Scholes Greeks + IV in one call.
func AllGreeks(spot, strike, rate, divYield, tte, optionPrice float64, isCall bool) (*Greeks, error) {
	callFlag := C.int(0)
	if isCall {
		callFlag = 1
	}
	cstr := C.tdx_all_greeks(
		C.double(spot), C.double(strike), C.double(rate),
		C.double(divYield), C.double(tte), C.double(optionPrice),
		callFlag,
	)
	raw, err := callJSON(cstr)
	if err != nil {
		return nil, err
	}
	var g Greeks
	if err := json.Unmarshal(raw, &g); err != nil {
		return nil, fmt.Errorf("thetadatadx: failed to parse greeks: %w", err)
	}
	return &g, nil
}

// ImpliedVolatility computes implied volatility via bisection.
// Returns (iv, error_bound).
func ImpliedVolatility(spot, strike, rate, divYield, tte, optionPrice float64, isCall bool) (float64, float64, error) {
	callFlag := C.int(0)
	if isCall {
		callFlag = 1
	}
	var outIV, outErr C.double
	rc := C.tdx_implied_volatility(
		C.double(spot), C.double(strike), C.double(rate),
		C.double(divYield), C.double(tte), C.double(optionPrice),
		callFlag, &outIV, &outErr,
	)
	if rc != 0 {
		return 0, 0, fmt.Errorf("thetadatadx: %s", lastError())
	}
	return float64(outIV), float64(outErr), nil
}
