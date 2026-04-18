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

//! Proxy authentication module.

mod basic;
mod none;

pub use basic::BasicAuth;
pub use none::NoAuth;
pub use proxy_auth::ProxyAuth;

mod proxy_auth {
    use std::fmt::Debug;

    use ylong_http::headers::HeaderValue;

    /// Trait for proxy authentication mechanisms.
    pub trait ProxyAuth: Debug + Send + Sync + 'static {
        /// Returns the authentication scheme name.
        fn scheme(&self) -> &str;

        /// Returns the authorization header value if authentication is needed.
        fn authorization_header(&self) -> Option<HeaderValue>;
    }
}
