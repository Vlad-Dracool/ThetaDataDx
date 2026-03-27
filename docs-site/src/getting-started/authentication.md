# Authentication

ThetaDataDx authenticates against ThetaData's Nexus API using your account email and password. There is no API key system -- your ThetaData account credentials are used directly.

## Credentials File

The simplest approach is a `creds.txt` file with your ThetaData email on line 1 and password on line 2. This is the same format the official Java terminal uses.

```text
your-email@example.com
your-password
```

### Rust

```rust
use thetadatadx::Credentials;

let creds = Credentials::from_file("creds.txt")?;
```

### Python

```python
from thetadatadx import Credentials

creds = Credentials.from_file("creds.txt")
```

### Go

```go
creds, err := thetadatadx.CredentialsFromFile("creds.txt")
if err != nil {
    log.Fatal(err)
}
defer creds.Close()
```

### C++

```cpp
auto creds = tdx::Credentials::from_file("creds.txt");
```

### CLI

```bash
tdx stock eod AAPL 20240101 20240301 --creds creds.txt
```

The `--creds` flag defaults to `creds.txt` in the current directory.

## Environment Variables

For containerized deployments or CI pipelines, use environment variables:

### Rust

```rust
use thetadatadx::Credentials;

let creds = Credentials::new(
    std::env::var("THETA_EMAIL")?,
    std::env::var("THETA_PASS")?,
);
```

### Python

```python
import os
from thetadatadx import Credentials

creds = Credentials(os.environ["THETA_EMAIL"], os.environ["THETA_PASS"])
```

### MCP Server

```bash
export THETA_EMAIL="you@example.com"
export THETA_PASSWORD="your-password"
thetadatadx-mcp
```

## Direct Construction

### Rust

```rust
let creds = Credentials::new("user@example.com", "hunter2");
```

### Rust (from string)

```rust
let creds = Credentials::parse("user@example.com\nhunter2")?;
```

Email is automatically lowercased and trimmed. Password is trimmed.

## Authentication Flow

When you create a `DirectClient`, the SDK:

1. Sends a POST request to `https://nexus-api.thetadata.us/identity/terminal/auth_user`
2. Includes your email and password in the request body
3. Receives a session UUID on success
4. Uses that UUID in all subsequent gRPC requests

```
Your App ──POST (email, password)──> Nexus API
Your App <──── session UUID ──────── Nexus API
Your App ──gRPC (session UUID)─────> MDDS Server
```

For FPSS streaming, authentication happens over the TCP connection:

```
Your App ──CREDENTIALS frame──> FPSS Server
Your App <──METADATA frame────── FPSS Server (success)
```

## Error Handling

| HTTP Status | Error | Meaning |
|-------------|-------|---------|
| 401 | `Error::Auth` | Invalid email or password |
| 404 | `Error::Auth` | Account not found |
| Other | `Error::Http` | Network or server issue |

The SDK matches the Java terminal's behavior: both 401 and 404 responses are treated as invalid credentials.

## Security Best Practices

- **Never commit credentials** to version control. Add `creds.txt` to your `.gitignore`.
- **Use environment variables** in production and CI environments.
- **Restrict file permissions** on `creds.txt`: `chmod 600 creds.txt`.
- The SDK sends credentials over TLS (HTTPS for Nexus, TLS/TCP for FPSS). Credentials are never transmitted in plaintext.
