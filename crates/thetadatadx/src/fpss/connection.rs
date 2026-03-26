//! TLS TCP connection to FPSS servers.
//!
//! # Transport (from decompiled Java — `FPSSClient.java`)
//!
//! The Java terminal connects via `SSLSocket` (TLS over TCP) with:
//! - `TCP_NODELAY = true` (Nagle disabled for low latency)
//! - Connect timeout: 2 seconds
//! - Read timeout: 10 seconds
//! - Tries servers in order until one connects: `nj-a:20000`, `nj-a:20001`,
//!   `nj-b:20000`, `nj-b:20001`
//!
//! Source: `FPSSClient.connect()` and `FPSSClient.SERVERS` in decompiled terminal.
//!
//! # Rust implementation
//!
//! Uses `tokio::net::TcpStream` + `tokio-rustls` for the TLS layer,
//! matching the Java `SSLSocketFactory.createSocket()` behavior.

use std::sync::Arc;
use std::time::Duration;

use rustls::ClientConfig;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

use super::protocol::{CONNECT_TIMEOUT_MS, SERVERS};

/// Type alias for the TLS-wrapped TCP read half.
pub type FpssReader = ReadHalf<TlsStream<TcpStream>>;

/// Type alias for the TLS-wrapped TCP write half.
pub type FpssWriter = WriteHalf<TlsStream<TcpStream>>;

/// Establish a TLS connection to the first reachable FPSS server.
///
/// Tries each server in [`SERVERS`] in order. Returns the split read/write
/// halves on success, or the last error if all servers fail.
///
/// # Connection sequence (from `FPSSClient.connect()`)
///
/// 1. TCP connect with 2s timeout
/// 2. TLS handshake via system trust store
/// 3. Set `TCP_NODELAY = true`
/// 4. Split into read + write halves for concurrent I/O
///
/// Source: `FPSSClient.connect()` in decompiled terminal.
pub async fn connect() -> Result<(FpssReader, FpssWriter, String), crate::error::Error> {
    connect_to_servers(SERVERS).await
}

/// Connect to a specific server list (for testing or custom endpoints).
///
/// Same behavior as [`connect`] but accepts an arbitrary server list.
pub async fn connect_to_servers(
    servers: &[(&str, u16)],
) -> Result<(FpssReader, FpssWriter, String), crate::error::Error> {
    let mut last_err = None;
    let connect_timeout = Duration::from_millis(CONNECT_TIMEOUT_MS);

    for &(host, port) in servers {
        let addr = format!("{host}:{port}");
        tracing::debug!(server = %addr, "attempting FPSS connection");

        match try_connect(host, port, connect_timeout).await {
            Ok((reader, writer)) => {
                tracing::info!(server = %addr, "FPSS connected");
                return Ok((reader, writer, addr));
            }
            Err(e) => {
                tracing::warn!(server = %addr, error = %e, "FPSS connection failed");
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| crate::error::Error::Fpss("no servers configured".to_string())))
}

/// Build a shared rustls `ClientConfig` with the webpki root certificates.
fn tls_client_config() -> Arc<ClientConfig> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Arc::new(config)
}

/// Attempt a single TLS connection to one server.
///
/// # Steps (matching `FPSSClient.connect()`)
///
/// 1. `TcpStream::connect` with timeout — matches Java `socket.connect(addr, 2000)`
/// 2. `socket.setTcpNoDelay(true)` — matches Java `socket.setTcpNoDelay(true)`
/// 3. TLS handshake via rustls — matches Java `SSLSocketFactory.createSocket()`
/// 4. Split into read/write halves for concurrent async I/O
async fn try_connect(
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<(FpssReader, FpssWriter), crate::error::Error> {
    // Wrap the entire TCP connect + TLS handshake in a single timeout.
    // The Java terminal has a 2s connect timeout; TLS handshake should complete
    // within the same window, but we allow the full timeout for both phases.
    let addr = format!("{host}:{port}");
    let addr_clone = addr.clone();

    // Parse the server name for TLS verification before entering the timeout block,
    // so lifetime issues with `host` are avoided.
    let server_name = rustls::pki_types::ServerName::try_from(host.to_owned())
        .map_err(|e| crate::error::Error::Fpss(format!("invalid TLS server name '{host}': {e}")))?;

    let (reader, writer) = tokio::time::timeout(timeout, async {
        // TCP connect
        let tcp = TcpStream::connect(&addr_clone).await?;

        // TCP_NODELAY = true (matches Java: socket.setTcpNoDelay(true))
        tcp.set_nodelay(true)?;

        // TLS handshake using rustls with webpki root certificates.
        let connector = TlsConnector::from(tls_client_config());
        let tls_stream = connector.connect(server_name, tcp).await.map_err(|e| {
            crate::error::Error::Fpss(format!("TLS handshake with {addr_clone} failed: {e}"))
        })?;

        // Split for concurrent read/write
        Ok::<_, crate::error::Error>(tokio::io::split(tls_stream))
    })
    .await
    .map_err(|_| {
        crate::error::Error::Fpss(format!(
            "connection to {addr} timed out after {timeout:?} (TCP+TLS)"
        ))
    })??;

    Ok((reader, writer))
}

/// Connect to a specific server address (for testing or when the caller
/// already knows which server to use).
///
/// This bypasses the server rotation logic.
pub async fn connect_to(
    host: &str,
    port: u16,
) -> Result<(FpssReader, FpssWriter), crate::error::Error> {
    let timeout = Duration::from_millis(CONNECT_TIMEOUT_MS);
    try_connect(host, port, timeout).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_list_has_four_entries() {
        // Sanity check: the hardcoded server list from Java has 4 entries.
        assert_eq!(SERVERS.len(), 4);
        assert_eq!(SERVERS[0], ("nj-a.thetadata.us", 20000));
        assert_eq!(SERVERS[1], ("nj-a.thetadata.us", 20001));
        assert_eq!(SERVERS[2], ("nj-b.thetadata.us", 20000));
        assert_eq!(SERVERS[3], ("nj-b.thetadata.us", 20001));
    }

    #[test]
    fn connect_timeout_matches_java() {
        assert_eq!(CONNECT_TIMEOUT_MS, 2_000);
    }
}
