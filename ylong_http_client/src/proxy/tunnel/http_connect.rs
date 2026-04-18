// Copyright (c) 2023 Huawei Device Co., Ltd.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! HTTP CONNECT tunnel implementation.

use std::fmt::Debug;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;

use crate::proxy::auth::ProxyAuth;
use crate::proxy::error::TunnelError;
use crate::proxy::tunnel::tunnel_trait::Tunnel;
use crate::runtime::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const DEFAULT_BUFFER_SIZE: usize = 8192;

/// HTTP CONNECT tunnel.
pub struct HttpConnectTunnel<S> {
    buffer_size: usize,
    _marker: std::marker::PhantomData<S>,
}

impl<S> Clone for HttpConnectTunnel<S> {
    fn clone(&self) -> Self {
        Self {
            buffer_size: self.buffer_size,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S> HttpConnectTunnel<S> {
    /// Creates a new HttpConnectTunnel with default buffer size.
    pub fn new() -> Self {
        Self {
            buffer_size: DEFAULT_BUFFER_SIZE,
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the buffer size for reading proxy response headers.
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Parse proxy CONNECT response.
    pub fn parse_response(buf: &[u8]) -> Result<(), TunnelError> {
        if buf.starts_with(b"HTTP/1.1 200") || buf.starts_with(b"HTTP/1.0 200") {
            Ok(())
        } else if buf.starts_with(b"HTTP/1.1 407") {
            Err(TunnelError::AuthenticationRequired)
        } else {
            Err(TunnelError::tunnel_failed("unexpected proxy response"))
        }
    }

    fn build_connect_request(
        &self,
        target: &str,
        port: u16,
        auth: Option<&dyn ProxyAuth>,
    ) -> Vec<u8> {
        let mut req = Vec::new();
        let target_host = if target.contains(':') && !target.starts_with('[') {
            format!("[{}]", target)
        } else {
            target.to_string()
        };
        let _ = write!(&mut req, "CONNECT {}:{} HTTP/1.1\r\n", target_host, port);
        let _ = write!(&mut req, "Host: {}:{}\r\n", target_host, port);
        if let Some(auth) = auth {
            if let Some(header) = auth.authorization_header() {
                if let Ok(header_str) = header.to_string() {
                    let _ = write!(&mut req, "Proxy-Authorization: {}\r\n", header_str);
                }
            }
        }
        req.extend_from_slice(b"\r\n");
        req
    }
}

impl<S> Default for HttpConnectTunnel<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Debug> Debug for HttpConnectTunnel<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpConnectTunnel")
            .field("buffer_size", &self.buffer_size)
            .finish()
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send + Sync + Debug + 'static> Tunnel
    for HttpConnectTunnel<S>
{
    type Stream = S;

    fn establish(
        &self,
        mut stream: Self::Stream,
        target: &str,
        port: u16,
        auth: Option<&dyn ProxyAuth>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, TunnelError>> + Send + '_>> {
        let req = self.build_connect_request(target, port, auth);
        let buffer_size = self.buffer_size;

        Box::pin(async move {
            // Send CONNECT request
            stream.write_all(&req).await?;

            // Read response
            let mut buf = vec![0u8; buffer_size];
            let mut pos = 0;

            loop {
                let buf_slice = &mut buf[pos..];
                match Pin::new(&mut stream).read(buf_slice).await? {
                    0 => return Err(TunnelError::UnexpectedEof),
                    n => pos += n,
                }

                let resp = &buf[..pos];
                if resp.ends_with(b"\r\n\r\n") {
                    Self::parse_response(resp)?;
                    return Ok(stream);
                }
                if resp.starts_with(b"HTTP/1.1 407") {
                    return Err(TunnelError::AuthenticationRequired);
                }
                if pos == buffer_size {
                    return Err(TunnelError::HeadersTooLong);
                }
            }
        })
    }

    fn name(&self) -> &'static str {
        "http_connect"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::auth::BasicAuth;
    use crate::proxy::auth::NoAuth;

    #[test]
    fn test_http_connect_tunnel_new() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        assert_eq!(tunnel.name(), "http_connect");
    }

    #[test]
    fn test_http_connect_tunnel_default() {
        let tunnel: HttpConnectTunnel<crate::runtime::TcpStream> = Default::default();
        assert_eq!(tunnel.name(), "http_connect");
    }

    #[test]
    fn test_http_connect_tunnel_with_buffer_size() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new().with_buffer_size(16384);
        assert_eq!(tunnel.name(), "http_connect");
    }

    #[test]
    fn test_parse_response_http10_success() {
        let resp = b"HTTP/1.0 200 Connection Established\r\n\r\n";
        assert!(HttpConnectTunnel::<crate::runtime::TcpStream>::parse_response(resp).is_ok());
    }

    #[test]
    fn test_parse_response_with_reason_phrase() {
        let resp = b"HTTP/1.1 200 OK\r\n\r\n";
        assert!(HttpConnectTunnel::<crate::runtime::TcpStream>::parse_response(resp).is_ok());
    }

    #[test]
    fn test_parse_response_407_with_reason() {
        let resp = b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n";
        assert!(matches!(
            HttpConnectTunnel::<crate::runtime::TcpStream>::parse_response(resp),
            Err(TunnelError::AuthenticationRequired)
        ));
    }

    #[test]
    fn test_parse_response_500() {
        let resp = b"HTTP/1.1 500 Internal Server Error\r\n\r\n";
        assert!(matches!(
            HttpConnectTunnel::<crate::runtime::TcpStream>::parse_response(resp),
            Err(TunnelError::TunnelFailed(_))
        ));
    }

    #[test]
    fn test_parse_response_502() {
        let resp = b"HTTP/1.1 502 Bad Gateway\r\n\r\n";
        assert!(matches!(
            HttpConnectTunnel::<crate::runtime::TcpStream>::parse_response(resp),
            Err(TunnelError::TunnelFailed(_))
        ));
    }

    #[test]
    fn test_parse_response_504() {
        let resp = b"HTTP/1.1 504 Gateway Timeout\r\n\r\n";
        assert!(matches!(
            HttpConnectTunnel::<crate::runtime::TcpStream>::parse_response(resp),
            Err(TunnelError::TunnelFailed(_))
        ));
    }

    #[test]
    fn test_build_connect_request_no_auth() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        let req = tunnel.build_connect_request("example.com", 443, None);
        let req_str = String::from_utf8_lossy(&req);

        assert!(req_str.contains("CONNECT example.com:443 HTTP/1.1"));
        assert!(req_str.contains("Host: example.com:443"));
        assert!(!req_str.contains("Proxy-Authorization"));
    }

    #[test]
    fn test_build_connect_request_with_basic_auth() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        let auth = BasicAuth::new("user", "pass");
        let req = tunnel.build_connect_request("example.com", 443, Some(&auth));
        let req_str = String::from_utf8_lossy(&req);

        assert!(req_str.contains("CONNECT example.com:443 HTTP/1.1"));
        assert!(req_str.contains("Host: example.com:443"));
        assert!(req_str.contains("Proxy-Authorization"));
        assert!(req_str.contains("Basic "));
    }

    #[test]
    fn test_build_connect_request_with_no_auth() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        let auth = NoAuth;
        let req = tunnel.build_connect_request("example.com", 443, Some(&auth));
        let req_str = String::from_utf8_lossy(&req);

        assert!(!req_str.contains("Proxy-Authorization"));
    }

    #[test]
    fn test_build_connect_request_different_ports() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();

        let req = tunnel.build_connect_request("example.com", 80, None);
        let req_str = String::from_utf8_lossy(&req);
        assert!(req_str.contains("CONNECT example.com:80 HTTP/1.1"));

        let req = tunnel.build_connect_request("example.com", 8080, None);
        let req_str = String::from_utf8_lossy(&req);
        assert!(req_str.contains("CONNECT example.com:8080 HTTP/1.1"));
    }

    #[test]
    fn test_build_connect_request_ipv6() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        let req = tunnel.build_connect_request("::1", 443, None);
        let req_str = String::from_utf8_lossy(&req);
        assert!(req_str.contains("CONNECT [::1]:443 HTTP/1.1"));
    }

    #[test]
    fn test_http_connect_tunnel_debug() {
        let tunnel = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        let debug_str = format!("{:?}", tunnel);
        assert!(debug_str.contains("HttpConnectTunnel"));
        assert!(debug_str.contains("buffer_size"));
    }

    #[test]
    fn test_http_connect_tunnel_clone() {
        let tunnel1 = HttpConnectTunnel::<crate::runtime::TcpStream>::new();
        let tunnel2 = tunnel1.clone();
        assert_eq!(tunnel1.name(), tunnel2.name());
    }

    #[test]
    fn test_http_connect_tunnel_send() {
        fn assert_send<T: Send>() {}
        assert_send::<HttpConnectTunnel<crate::runtime::TcpStream>>();
    }

    #[test]
    fn test_http_connect_tunnel_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<HttpConnectTunnel<crate::runtime::TcpStream>>();
    }
}
