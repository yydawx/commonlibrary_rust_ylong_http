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

//! Tunnel trait definition.

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::proxy::auth::ProxyAuth;
use crate::proxy::error::TunnelError;
use crate::runtime::{AsyncRead, AsyncWrite};

/// Trait for tunnel establishment.
///
/// Implementors provide the ability to establish a tunnel through a proxy
/// to a target host.
pub trait Tunnel: Debug + Send + Sync + 'static {
    /// The stream type produced by this tunnel.
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + Sync + Debug + 'static;

    /// Establishes a tunnel through the proxy to the target host.
    ///
    /// # Arguments
    /// * `stream` - The underlying connection to the proxy
    /// * `target` - The target hostname
    /// * `port` - The target port
    /// * `auth` - Optional authentication credentials
    fn establish(
        &self,
        stream: Self::Stream,
        target: &str,
        port: u16,
        auth: Option<&dyn ProxyAuth>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, TunnelError>> + Send + Sync + '_>>;

    /// Returns the name of this tunnel type for registration.
    fn name(&self) -> &'static str {
        "tunnel"
    }
}
