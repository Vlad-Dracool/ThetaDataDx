# Server Mode

ThetaDataDx can run as a persistent server, exposing market data to other applications via a drop-in REST+WebSocket server, MCP (Model Context Protocol), or as a CLI-driven query tool. This guide covers all approaches.

## REST + WebSocket Server (Java Terminal Replacement)

The `thetadatadx-server` binary is a drop-in replacement for the ThetaData Java Terminal. It runs local HTTP REST (`:25510`) and WebSocket (`:25520`) servers with identical API compatibility.

```bash
thetadatadx-server --creds creds.txt
```

Existing Python SDK scripts, Excel add-ins, and curl commands work without changes. See the [REST Server reference](../tools/rest-server.md) for full documentation.

## MCP Server

The MCP server (`thetadatadx-mcp`) exposes all 64 ThetaDataDx tools to any LLM or MCP-compatible client via JSON-RPC 2.0 over stdio.

### Architecture

```
LLM (Claude / Codex / Gemini / Cursor)
    |  JSON-RPC 2.0 over stdio
    v
thetadatadx-mcp (long-running process)
    |  Single DirectClient, authenticated once at startup
    v
ThetaData servers (MDDS gRPC + FPSS TCP)
```

The server authenticates **once** at startup. Subsequent tool calls execute instantly with zero per-request auth overhead.

### Installation

```bash
cargo install thetadatadx-mcp --git https://github.com/userFRM/ThetaDataDx
```

### Configuration for Claude Code

Add to `.claude/settings.json`:

```json
{
  "mcpServers": {
    "thetadata": {
      "command": "thetadatadx-mcp",
      "env": {
        "THETA_EMAIL": "you@example.com",
        "THETA_PASSWORD": "your-password"
      }
    }
  }
}
```

Or with a credentials file:

```json
{
  "mcpServers": {
    "thetadata": {
      "command": "thetadatadx-mcp",
      "args": ["--creds", "/path/to/creds.txt"]
    }
  }
}
```

### Configuration for Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "thetadata": {
      "command": "thetadatadx-mcp",
      "env": {
        "THETA_EMAIL": "you@example.com",
        "THETA_PASSWORD": "your-password"
      }
    }
  }
}
```

### Available Tools (64 total)

| Category | Count | Tools |
|----------|-------|-------|
| Meta | 1 | `ping` |
| Offline Greeks | 2 | `all_greeks`, `implied_volatility` |
| Stock | 14 | list, snapshot, history, at-time |
| Option | 34 | list, snapshot, history, Greeks, at-time |
| Index | 9 | list, snapshot, history, at-time |
| Calendar & Rates | 4 | `calendar_open_today`, `calendar_on_date`, `calendar_year`, `interest_rate_history_eod` |

### Offline Mode

If no credentials are provided, the server starts in offline mode. Only three tools are available:
- `ping` -- server health check
- `all_greeks` -- compute 22 Black-Scholes Greeks
- `implied_volatility` -- IV solver

### Example Tool Calls

**List tools:**
```json
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```

**Fetch AAPL end-of-day data:**
```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
  "name":"stock_history_eod",
  "arguments":{"symbol":"AAPL","start_date":"20240101","end_date":"20240301"}
}}
```

**Compute Greeks offline:**
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
  "name":"all_greeks",
  "arguments":{
    "spot":150.0,"strike":155.0,"rate":0.05,
    "dividend_yield":0.01,"time_to_expiry":0.25,
    "option_price":5.50,"is_call":true
  }
}}
```

### Logging

```bash
RUST_LOG=debug thetadatadx-mcp       # verbose
RUST_LOG=warn thetadatadx-mcp        # quiet
RUST_LOG=thetadatadx=debug thetadatadx-mcp  # just the library
```

All logs go to **stderr** (stdout is reserved for JSON-RPC).

## CLI as a Query Server

The `tdx` CLI tool can be used as an ad-hoc query tool in shell scripts and pipelines:

```bash
# JSON output for programmatic consumption
tdx stock eod AAPL 20240101 20240301 --format json | jq '.[0]'

# CSV for spreadsheet import
tdx stock eod AAPL 20240101 20240301 --format csv > aapl_eod.csv

# Chain queries in a script
for symbol in AAPL MSFT GOOGL AMZN; do
    tdx stock snapshot-quote "$symbol" --format json
done

# Pipe to other tools
tdx option expirations SPY --format json | jq -r '.[]' | head -5
```

See the [CLI reference](../tools/cli.md) for complete documentation.
