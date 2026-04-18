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

//! Tunnel factory implementation.

use std::fmt::Debug;
use std::sync::Arc;

use crate::proxy::tunnel::http_connect::HttpConnectTunnel;
use crate::proxy::tunnel::tunnel_trait::Tunnel;
use crate::runtime::TcpStream;

/// Default tunnel factory implementation.
#[derive(Clone, Debug, Default)]
pub struct DefaultTunnelFactory {
    buffer_size: usize,
}

impl DefaultTunnelFactory {
    /// Creates a new DefaultTunnelFactory.
    pub fn new() -> Self {
        Self { buffer_size: 8192 }
    }

    /// Sets the buffer size for tunnel operations.
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Creates a new HTTP CONNECT tunnel for TcpStream.
    pub fn create_tcp_tunnel(&self) -> Arc<dyn Tunnel<Stream = TcpStream>> {
        Arc::new(HttpConnectTunnel::<TcpStream>::new().with_buffer_size(self.buffer_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_factory() {
        let factory = DefaultTunnelFactory::new();
        let tunnel = factory.create_tcp_tunnel();
        assert_eq!(tunnel.name(), "http_connect");
    }

    #[test]
    fn test_factory_with_buffer_size() {
        let factory = DefaultTunnelFactory::new().with_buffer_size(16384);
        let tunnel = factory.create_tcp_tunnel();
        assert_eq!(tunnel.name(), "http_connect");
    }

    #[test]
    fn test_factory_default() {
        let factory = DefaultTunnelFactory::default();
        let tunnel = factory.create_tcp_tunnel();
        assert_eq!(tunnel.name(), "http_connect");
    }

    #[test]
    fn test_factory_clone() {
        let factory1 = DefaultTunnelFactory::new();
        let factory2 = factory1.clone();
        assert_eq!(factory1.buffer_size, factory2.buffer_size);
    }

    #[test]
    fn test_factory_debug() {
        let factory = DefaultTunnelFactory::new();
        let debug_str = format!("{:?}", factory);
        assert!(debug_str.contains("DefaultTunnelFactory"));
    }

    #[test]
    fn test_factory_send() {
        fn assert_send<T: Send>() {}
        assert_send::<DefaultTunnelFactory>();
    }

    #[test]
    fn test_factory_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<DefaultTunnelFactory>();
    }

    #[test]
    fn test_factory_different_buffer_sizes() {
        let factory_small = DefaultTunnelFactory::new().with_buffer_size(4096);
        let factory_large = DefaultTunnelFactory::new().with_buffer_size(65536);
        assert_ne!(factory_small.buffer_size, factory_large.buffer_size);
    }
}
