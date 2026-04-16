## Why

Corporate networks often require traffic to flow through HTTPS proxies for security inspection and access control. Currently, `ylong_http_client` only supports HTTP proxies, limiting its use in enterprise environments where HTTPS proxy is the standard.

## What Changes

- Add support for HTTPS proxy connections where the proxy server itself uses TLS encryption
- Extend `Proxy` struct to hold proxy-specific TLS configuration (CA certificate, verification mode)
- Extend `ProxyBuilder` with `proxy_ca_file()` and `danger_accept_invalid_proxy_certs()` methods
- Modify `https_connect()` to establish proxy TLS handshake before sending CONNECT, then target TLS on top

## Capabilities

### New Capabilities
- `https-proxy`: Support for HTTPS proxy connections with configurable TLS settings, enabling clients to connect through TLS-encrypted corporate proxies

## Impact

- **Modified modules**: 
  - `ylong_http_client/src/util/config/settings.rs` - Public API for proxy TLS config
  - `ylong_http_client/src/util/proxy.rs` - Proxy struct with TLS fields
  - `ylong_http_client/src/async_impl/connector/mod.rs` - Connection establishment with proxy TLS
- **New dependencies**: None (reuses existing OpenSSL C FFI bindings)
- **No breaking changes**: Existing HTTP proxy functionality remains unchanged
