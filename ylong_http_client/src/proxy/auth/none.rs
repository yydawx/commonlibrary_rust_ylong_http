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

//! No authentication.

use std::fmt::Debug;

use crate::proxy::auth::proxy_auth::ProxyAuth;

/// No authentication (placeholder).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoAuth;

impl ProxyAuth for NoAuth {
    fn scheme(&self) -> &str {
        "none"
    }

    fn authorization_header(&self) -> Option<ylong_http::headers::HeaderValue> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth_scheme() {
        let auth = NoAuth;
        assert_eq!(auth.scheme(), "none");
    }

    #[test]
    fn test_no_auth_header() {
        let auth = NoAuth;
        assert!(auth.authorization_header().is_none());
    }

    #[test]
    fn test_no_auth_default() {
        let auth = NoAuth::default();
        assert_eq!(auth.scheme(), "none");
        assert!(auth.authorization_header().is_none());
    }

    #[test]
    fn test_no_auth_clone() {
        let auth1 = NoAuth;
        let _auth2 = auth1.clone();
    }

    #[test]
    fn test_no_auth_copy() {
        let auth1 = NoAuth;
        let auth2 = auth1;
        assert_eq!(auth1.scheme(), auth2.scheme());
    }

    #[test]
    fn test_no_auth_debug() {
        let auth = NoAuth;
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("NoAuth"));
    }

    #[test]
    fn test_no_auth_send() {
        fn assert_send<T: Send>() {}
        assert_send::<NoAuth>();
    }

    #[test]
    fn test_no_auth_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<NoAuth>();
    }
}
