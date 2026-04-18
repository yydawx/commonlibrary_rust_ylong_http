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

//! Proxy error types.

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Tunnel establishment errors.
#[derive(Debug, PartialEq, Clone)]
pub enum TunnelError {
    /// Connection to proxy failed.
    ConnectFailed(String),
    /// Tunnel establishment failed with message.
    TunnelFailed(String),
    /// Proxy headers exceed buffer limit.
    HeadersTooLong,
    /// Proxy authentication required (407 response).
    AuthenticationRequired,
    /// Proxy authentication failed.
    AuthenticationFailed(String),
    /// Connection closed unexpectedly.
    UnexpectedEof,
}

impl TunnelError {
    pub(crate) fn connect_failed<E: Into<String>>(e: E) -> Self {
        Self::ConnectFailed(e.into())
    }

    pub(crate) fn tunnel_failed<S: Into<String>>(msg: S) -> Self {
        Self::TunnelFailed(msg.into())
    }

    pub(crate) fn auth_failed<S: Into<String>>(msg: S) -> Self {
        Self::AuthenticationFailed(msg.into())
    }
}

impl Display for TunnelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectFailed(msg) => write!(f, "connect failed: {}", msg),
            Self::TunnelFailed(msg) => write!(f, "tunnel failed: {}", msg),
            Self::HeadersTooLong => write!(f, "proxy headers too long for tunnel"),
            Self::AuthenticationRequired => write!(f, "proxy authentication required"),
            Self::AuthenticationFailed(msg) => write!(f, "proxy authentication failed: {}", msg),
            Self::UnexpectedEof => write!(f, "unexpected EOF during tunnel"),
        }
    }
}

impl Error for TunnelError {}

impl From<std::io::Error> for TunnelError {
    fn from(e: std::io::Error) -> Self {
        Self::ConnectFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_error_connect_failed_display() {
        let err = TunnelError::ConnectFailed("connection refused".to_string());
        let display = err.to_string();
        assert!(display.contains("connect failed"));
        assert!(display.contains("connection refused"));
    }

    #[test]
    fn test_tunnel_error_tunnel_failed_display() {
        let err = TunnelError::TunnelFailed("timeout".to_string());
        let display = err.to_string();
        assert!(display.contains("tunnel failed"));
        assert!(display.contains("timeout"));
    }

    #[test]
    fn test_tunnel_error_headers_too_long_display() {
        let err = TunnelError::HeadersTooLong;
        let display = err.to_string();
        assert!(display.contains("too long"));
    }

    #[test]
    fn test_tunnel_error_authentication_required_display() {
        let err = TunnelError::AuthenticationRequired;
        let display = err.to_string();
        assert!(display.contains("authentication required"));
    }

    #[test]
    fn test_tunnel_error_authentication_failed_display() {
        let err = TunnelError::AuthenticationFailed("invalid credentials".to_string());
        let display = err.to_string();
        assert!(display.contains("authentication failed"));
        assert!(display.contains("invalid credentials"));
    }

    #[test]
    fn test_tunnel_error_unexpected_eof_display() {
        let err = TunnelError::UnexpectedEof;
        let display = err.to_string();
        assert!(display.contains("EOF"));
    }

    #[test]
    fn test_tunnel_error_debug() {
        let err = TunnelError::HeadersTooLong;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("HeadersTooLong"));
    }

    #[test]
    fn test_tunnel_error_debug_all_variants() {
        let variants = [
            TunnelError::ConnectFailed("test".to_string()),
            TunnelError::TunnelFailed("test".to_string()),
            TunnelError::HeadersTooLong,
            TunnelError::AuthenticationRequired,
            TunnelError::AuthenticationFailed("test".to_string()),
            TunnelError::UnexpectedEof,
        ];

        for variant in variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_tunnel_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tunnel_err: TunnelError = io_err.into();

        match tunnel_err {
            TunnelError::ConnectFailed(msg) => {
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected ConnectFailed"),
        }
    }

    #[test]
    fn test_tunnel_error_source() {
        let err = TunnelError::ConnectFailed("test".to_string());
        let source = err.source();
        assert!(source.is_none());
    }

    #[test]
    fn test_tunnel_error_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TunnelError>();
    }

    #[test]
    fn test_tunnel_error_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TunnelError>();
    }

    #[test]
    fn test_tunnel_error_clone() {
        let err = TunnelError::ConnectFailed("test".to_string());
        let cloned = err.clone();
        assert!(matches!(cloned, TunnelError::ConnectFailed(_)));
    }

    #[test]
    fn test_tunnel_error_partial_eq() {
        let err1 = TunnelError::HeadersTooLong;
        let err2 = TunnelError::HeadersTooLong;
        let err3 = TunnelError::UnexpectedEof;

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_tunnel_error_empty_string() {
        let err = TunnelError::ConnectFailed(String::new());
        let display = err.to_string();
        assert!(display.contains("connect failed"));
    }

    #[test]
    fn test_tunnel_error_long_message() {
        let long_msg = "x".repeat(1000);
        let err = TunnelError::TunnelFailed(long_msg.clone());
        let display = err.to_string();
        assert!(display.contains(&long_msg[..100]));
    }
}
