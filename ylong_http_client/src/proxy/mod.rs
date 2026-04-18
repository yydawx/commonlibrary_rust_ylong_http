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

//! Proxy module for HTTP/HTTPS proxy support.
//!
//! This module provides:
//! - [`Proxy`] for proxy configuration
//! - [`ProxyAuth`] trait for authentication
//! - [`Tunnel`] trait for tunnel establishment

pub mod auth;
pub mod error;
pub mod intercept;
pub mod tunnel;

pub use auth::{BasicAuth, NoAuth, ProxyAuth};
pub use error::TunnelError;
pub use intercept::{Intercept, NoProxy, Proxy, ProxyInfo};
pub use tunnel::{DefaultTunnelFactory, HttpConnectTunnel, Tunnel};

use crate::runtime::TcpStream;
use std::sync::Arc;

impl DefaultTunnelFactory {
    /// Creates an Arc-wrapped tunnel for use with ClientBuilder.
    pub fn to_arc(&self) -> Arc<dyn Tunnel<Stream = TcpStream>> {
        self.create_tcp_tunnel()
    }
}
