## 1. Proxy Struct Extension

- [x] 1.1 Add `proxy_ca: Option<Cert>` field to `proxy::Proxy` struct
- [x] 1.2 Add `danger_accept_invalid_proxy: bool` field to `proxy::Proxy` struct
- [x] 1.3 Add `set_proxy_ca()` mutator method to `proxy::Proxy`
- [x] 1.4 Add `set_danger_accept_invalid_proxy()` mutator method to `proxy::Proxy`

## 2. ProxyBuilder Public API

- [x] 2.1 Add `proxy_ca_file(path: &str)` method to `ProxyBuilder` in `settings.rs`
- [x] 2.2 Add `danger_accept_invalid_proxy_certs(skip: bool)` method to `ProxyBuilder`
- [x] 2.3 Wire builder methods to set fields on inner `proxy::Proxy`

## 3. Async Connector Implementation

- [x] 3.1 Add `proxy_tls_handshake()` helper function to establish TLS with proxy
- [x] 3.2 Modify `https_connect()` to detect HTTPS proxy from proxy URL scheme
- [x] 3.3 Implement proxy TLS establishment before CONNECT when proxy is HTTPS
- [x] 3.4 Ensure CONNECT is sent through the proxy TLS tunnel
- [x] 3.5 Handle nested TLS: proxy TLS wraps target TLS
- [x] 3.6 Add `MixStream::HttpsProxy` variant for nested TLS support
- [ ] 3.7 (Optional) Create `ProxyTunnelStream` wrapper type for nested TLS encapsulation

## 4. Error Handling

- [x] 4.1 Distinguish proxy TLS errors from target TLS errors in error messages
- [x] 4.2 Return appropriate error kinds for proxy TLS handshake failures

## 5. Testing

- [x] 5.1 Add unit tests for `proxy_ca_file()` builder method
- [x] 5.2 Add unit tests for `danger_accept_invalid_proxy_certs()` builder method
- [x] 5.3 Add unit test for HTTPS proxy detection by scheme
- [ ] 5.4 Add unit test for `proxy_tls_handshake()` function (mock proxy)

## 6. Compilation Fixes (April 2026)

- [x] 6.1 Fix `Wrapper` re-export visibility in `ssl_stream/mod.rs`
- [x] 6.2 Fix nested `Wrapper<Wrapper<S>>` unwrapping in `c_ssl_stream.rs::into_inner()`
- [x] 6.3 Add `MixStream::HttpsProxy` variant for HTTPS proxy connections
- [x] 6.4 Add missing `timeout` field to `ConnectorConfig` in `sync_impl/client.rs`
- [x] 6.5 Fix `pool.rs` to pass required arguments to `Pool::get()`
- [x] 6.6 Fix `SpeedConfig` import path in `sync_impl/pool.rs`
- [x] 6.7 Remove unused imports (`Proxy`, `Wrapper`, `TlsConfigBuilder`, etc.)
- [x] 6.8 Prefix unused `base_config` parameter with underscore
