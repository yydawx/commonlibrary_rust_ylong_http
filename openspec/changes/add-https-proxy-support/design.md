## Context

The client currently supports HTTP proxies via CONNECT tunneling. When connecting to an HTTPS target through an HTTP proxy, the flow is:

1. TCP connects to proxy
2. Send HTTP CONNECT request
3. Proxy responds 200
4. Establish TLS with target server (through the proxy tunnel)

HTTPS proxies differ: the client must first establish TLS with the proxy server, then send the CONNECT request through the encrypted proxy tunnel.

### Current Architecture

- `Proxy` struct holds intercept rules, no_proxy, basic_auth
- `ConnectorConfig` holds `proxies: Proxies` and `tls: TlsConfig` (target TLS)
- `https_connect()` handles target TLS via `config.ssl_new()`
- `tunnel()` establishes CONNECT tunnel, returns raw `TcpStream`

### Constraints

- Must reuse existing OpenSSL C FFI bindings (no new native dependencies)
- Private key loading is not yet implemented (Phase 2), so only one-way TLS verification is supported
- Existing HTTP proxy behavior must remain unchanged
- Async connector only (tokio_base / ylong_base)

## Goals / Non-Goals

**Goals:**
- Support HTTPS proxies with configurable CA certificate and verification mode
- Enable enterprise deployments requiring TLS inspection at proxy level
- Maintain backward compatibility with existing HTTP proxy users

**Non-Goals:**
- mTLS support (requires Phase 2 private key loading)
- Proxy authentication over TLS (HTTP Basic Auth over proxy TLS is already supported)
- HTTP/3 proxy support
- Sync connector implementation (async only for this phase)

## Decisions

### 1. Store proxy TLS config inside Proxy struct

**Decision:** Add `proxy_ca: Option<Cert>` and `danger_accept_invalid_proxy: bool` fields directly to the `Proxy` struct.

**Rationale:** Each proxy instance may have different TLS requirements (different CAs, some requiring verification bypass). The proxy is already the unit of configuration and is accessible from `ConnectorConfig.proxies` during connection establishment.

**Alternative considered:** Global `proxy_tls: TlsConfig` in `ConnectorConfig`. Rejected because it prevents per-proxy TLS configuration.

### 2. Follow existing ProxyBuilder pattern for API

**Decision:** Add `proxy_ca_file()` and `danger_accept_invalid_proxy_certs()` methods to `ProxyBuilder`.

**Rationale:** Consistent with existing API style (`no_proxy()`, `basic_auth()`). Mirrors `TlsConfigBuilder`'s `ca_file()` and `danger_accept_invalid_certs()`.

```rust
let proxy = Proxy::all("https://proxy.example.com:443")
    .proxy_ca_file("proxy-ca.pem")
    .danger_accept_invalid_proxy_certs(false)
    .build()?;
```

### 3. Detect HTTPS proxy by scheme in proxy URL

**Decision:** Parse proxy URL scheme and check if `scheme == "https"`.

**Rationale:** Natural mapping - `Proxy::all("https://proxy:443")` means HTTPS proxy. No new enum or flag needed.

### 4. Establish proxy TLS before CONNECT

**Decision:** In `https_connect()`, if proxy is HTTPS:
1. Build proxy TLS config from `Proxy.proxy_ca` and `danger_accept_invalid_proxy`
2. Perform TLS handshake with proxy server
3. Send CONNECT through the encrypted proxy connection
4. Then establish target TLS on top

**Rationale:** This matches the HTTPS proxy protocol specification. The CONNECT request must travel through the proxy TLS tunnel.

```
Client ──── TLS (proxy) ──── Proxy
        │
        └── CONNECT (encrypted) ──── Proxy
                  │
                  └── TLS (target) ──── Target
```

### 5. Nested TLS via AsyncSslStream composition

**Decision:** Layer `AsyncSslStream<AsyncSslStream<TcpStream>>` for dual TLS.

**Rationale:** `AsyncSslStream<S>` only requires `S: AsyncRead + AsyncWrite`. Since `AsyncSslStream<TcpStream>` implements these traits, the nested composition works naturally.

**Implementation Note:** Consider creating a `ProxyTunnelStream` wrapper type to encapsulate the nested TLS layers for improved readability. This is optional—if inline implementation is clearer, prefer simplicity.

```rust
// Optional wrapper (implementation detail)
pub struct ProxyTunnelStream {
    proxy_tls: AsyncSslStream<TcpStream>,
    target_tls: Option<AsyncSslStream<AsyncSslStream<TcpStream>>>,
}
```

### 6. Error Type Design

**Decision:** Define explicit error variants to distinguish proxy TLS from target TLS errors.

**Error Types:**

| Error Variant | Description | Context Included |
|--------------|-------------|------------------|
| `ProxyTlsHandshakeError` | TLS handshake failure with proxy server | Proxy URL, SSL error code |
| `ProxyConnectError` | CONNECT request/response failure | Proxy URL, HTTP status |
| `TargetTlsHandshakeError` | TLS handshake failure with target server | Target URL, SSL error code |
| `TargetConnectError` | Connection failure to target through tunnel | Target URL, IO error |

**Error Message Convention:**
- Prefix all proxy-related errors with `[proxy]`
- Prefix all target-related errors with `[target]`
- Include hostname for debugging

**Example:**
```rust
Err(Error::ProxyTlsHandshake(
    "failed to connect to proxy.example.com:443".to_string(),
    ssl_error_code
))
```

## Risks / Trade-offs

[Risk] Proxy TLS verification failure due to hostname mismatch
→ **Mitigation:** Extract hostname from proxy URL for SNI. Existing `set_verify_hostname()` on Ssl supports this.

[Risk] Mixed proxy modes (some HTTPS, some HTTP in same config)
→ **Mitigation:** `Proxies::match_proxy()` returns correct `Proxy` instance. Each proxy's TLS is handled independently.

[Risk] Async-only implementation may limit adoption
→ **Mitigation:** Sync implementation can be added in a follow-up change if needed.

[Risk] Performance overhead from double TLS handshake
→ **Mitigation:** Expected behavior for HTTPS proxies. TLS handshakes are amortized over connection reuse.

## Migration Plan

1. Add fields to `Proxy` struct (backward-compatible addition)
2. Add builder methods (backward-compatible addition)
3. Modify `https_connect()` with conditional proxy TLS logic
4. Add unit tests for new proxy TLS paths
5. Add integration test with real HTTPS proxy

No migration needed for existing users - default behavior unchanged.

## Open Questions

1. Should `danger_accept_invalid_proxy_certs` default to `false` or `true`?
   - Recommendation: `false` (secure by default)
