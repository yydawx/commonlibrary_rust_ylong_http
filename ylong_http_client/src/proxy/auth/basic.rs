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

//! Basic authentication for proxy.

use std::fmt::Debug;

use ylong_http::headers::HeaderValue;

use crate::proxy::auth::proxy_auth::ProxyAuth;
use crate::util::base64::encode;

/// Basic authentication credentials.
#[derive(Clone)]
pub struct BasicAuth {
    credentials: HeaderValue,
}

impl Debug for BasicAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicAuth")
            .field("credentials", &"<redacted>")
            .finish()
    }
}

impl BasicAuth {
    /// Creates a new BasicAuth with the given username and password.
    pub fn new(username: &str, password: &str) -> Self {
        let encoded = encode(format!("{username}:{password}").as_bytes());
        let encoded_str = String::from_utf8_lossy(&encoded);
        let credentials = HeaderValue::from_bytes(format!("Basic {encoded_str}").as_bytes())
            .unwrap_or_else(|_| HeaderValue::from_bytes(b"").unwrap());
        Self { credentials }
    }

    /// Creates a BasicAuth from an already-encoded credentials string.
    pub fn from_encoded(encoded: &str) -> Self {
        let credentials = HeaderValue::from_bytes(format!("Basic {}", encoded).as_bytes())
            .unwrap_or_else(|_| HeaderValue::from_bytes(b"").unwrap());
        Self { credentials }
    }
}

impl ProxyAuth for BasicAuth {
    fn scheme(&self) -> &str {
        "Basic"
    }

    fn authorization_header(&self) -> Option<HeaderValue> {
        let mut header = self.credentials.clone();
        header.set_sensitive(true);
        Some(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_new() {
        let auth = BasicAuth::new("user", "pass");
        assert_eq!(auth.scheme(), "Basic");
        assert!(auth.authorization_header().is_some());
    }

    #[test]
    fn test_basic_auth_credentials() {
        let auth = BasicAuth::new("Aladdin", "open sesame");
        let header = auth.authorization_header().unwrap();
        let header_str = header.to_string().unwrap();
        assert!(header_str.starts_with("Basic "));
        assert!(header_str.contains("QWxhZGRpbjpvcGVuIHNlc2FtZQ"));
    }

    #[test]
    fn test_basic_auth_sensitive() {
        let auth = BasicAuth::new("user", "pass");
        let header = auth.authorization_header().unwrap();
        assert!(header.is_sensitive());
    }

    #[test]
    fn test_basic_auth_empty_password() {
        let auth = BasicAuth::new("user", "");
        let header = auth.authorization_header().unwrap();
        let header_str = header.to_string().unwrap();
        assert!(header_str.starts_with("Basic "));
        assert!(header_str.contains("dXNlcjo=")); // "user:"
    }

    #[test]
    fn test_basic_auth_empty_username() {
        let auth = BasicAuth::new("", "password");
        let header = auth.authorization_header().unwrap();
        let header_str = header.to_string().unwrap();
        assert!(header_str.starts_with("Basic "));
        assert!(header_str.contains("OnBhc3N3b3Jk")); // ":password"
    }

    #[test]
    fn test_basic_auth_special_characters() {
        let auth = BasicAuth::new("user@domain", "p@ss:word");
        let header = auth.authorization_header().unwrap();
        let header_str = header.to_string().unwrap();
        assert!(header_str.starts_with("Basic "));
    }

    #[test]
    fn test_basic_auth_unicode() {
        let auth = BasicAuth::new("用户", "密码");
        let header = auth.authorization_header().unwrap();
        let header_str = header.to_string().unwrap();
        assert!(header_str.starts_with("Basic "));
    }

    #[test]
    fn test_basic_auth_debug() {
        let auth = BasicAuth::new("user", "pass");
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("BasicAuth"));
        assert!(debug_str.contains("redacted"));
        assert!(!debug_str.contains("pass"));
    }

    #[test]
    fn test_basic_auth_clone() {
        let auth1 = BasicAuth::new("user", "pass");
        let auth2 = auth1.clone();
        assert_eq!(auth1.scheme(), auth2.scheme());
    }

    #[test]
    fn test_basic_auth_long_credentials() {
        let long_user = "a".repeat(100);
        let long_pass = "b".repeat(100);
        let auth = BasicAuth::new(&long_user, &long_pass);
        assert!(auth.authorization_header().is_some());
    }
}
