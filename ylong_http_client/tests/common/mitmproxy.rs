// Copyright (c) 2023 Huawei Device Device Co., Ltd.
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

//! mitmproxy Docker container manager for E2E testing.
//!
//! This module provides utilities to start/stop mitmproxy containers for testing
//! HTTPS proxy with various TLS configurations.

use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Mitmproxy container handle for E2E testing.
pub struct MitmproxyHandle {
    container_id: String,
    proxy_port: u16,
}

impl MitmproxyHandle {
    /// Returns the proxy address (host:port).
    pub fn proxy_addr(&self) -> String {
        format!("127.0.0.1:{}", self.proxy_port)
    }

    /// Returns the proxy URL.
    pub fn proxy_url(&self) -> String {
        format!("http://{}", self.proxy_addr())
    }

    /// Returns the HTTPS proxy URL.
    pub fn https_proxy_url(&self) -> String {
        format!("https://{}", self.proxy_addr())
    }
}

impl Drop for MitmproxyHandle {
    fn drop(&mut self) {
        let _ = stop_mitmproxy(&self.container_id);
    }
}

fn check_docker_available() -> Result<(), String> {
    let output = Command::new("docker")
        .args(["version"])
        .output()
        .map_err(|e| format!("Failed to run docker: {}", e))?;

    if !output.status.success() {
        return Err("Docker is not available".to_string());
    }
    Ok(())
}

fn stop_mitmproxy(container_id: &str) -> Result<(), String> {
    let _ = Command::new("docker")
        .args(["rm", "-f", container_id])
        .output();

    Ok(())
}

/// Starts a mitmproxy container as an HTTP/HTTPS proxy.
///
/// # Arguments
/// * `image` - Docker image name for mitmproxy
/// * `upstream_port` - Port for the upstream HTTP server (mitmproxy listens on this + 1)
/// * `extra_args` - Additional mitmproxy command line arguments
///
/// # Returns
/// * `MitmproxyHandle` - Handle to the running container
pub fn start_mitmproxy(
    image: &str,
    upstream_port: u16,
    extra_args: &[&str],
) -> Result<MitmproxyHandle, String> {
    check_docker_available()?;

    let proxy_port = find_available_port()?;

    let port_mapping = format!("{}:8080", proxy_port);
    let mut args = vec!["run", "-d", "--rm", "--privileged", "-p", &port_mapping];

    args.extend_from_slice(&["--entrypoint", "mitmdump"]);
    args.push(image);
    args.extend(["--listen-port", "8080", "--listen-host", "0.0.0.0"]);
    args.extend_from_slice(extra_args);

    let output = Command::new("docker")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to start mitmproxy: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mitmproxy container failed to start: {}", stderr));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    std::thread::sleep(Duration::from_secs(2));

    Ok(MitmproxyHandle {
        container_id,
        proxy_port,
    })
}

/// Starts a mitmproxy container with mTLS support (requesting client certificates).
pub fn start_mitmproxy_with_mtls(image: &str) -> Result<MitmproxyHandle, String> {
    start_mitmproxy(image, 0, &["--set", "request_client_cert=true"])
}

/// Starts a mitmproxy container with custom cipher suites.
pub fn start_mitmproxy_with_ciphers(image: &str, ciphers: &str) -> Result<MitmproxyHandle, String> {
    start_mitmproxy(image, 0, &["--set", &format!("ciphers_client={}", ciphers)])
}

/// Starts a mitmproxy container with TLS version restrictions.
pub fn start_mitmproxy_with_tls_version(
    image: &str,
    min_version: &str,
    max_version: &str,
) -> Result<MitmproxyHandle, String> {
    start_mitmproxy(
        image,
        0,
        &[
            "--set",
            &format!("tls_version_client_min={}", min_version),
            "--set",
            &format!("tls_version_client_max={}", max_version),
        ],
    )
}

fn find_available_port() -> Result<u16, String> {
    for port in 18080..18100 {
        if is_port_available(port) {
            return Ok(port);
        }
    }
    Err("No available port found".to_string())
}

fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

/// Manages mitmproxy lifecycle for tests.
///
/// # Example
/// ```rust,ignore
/// {
///     let _guard = MitmproxyGuard::new("mitmproxy/mitmproxy:latest").unwrap();
///     // mitmproxy is running
///     // when _guard is dropped, mitmproxy is stopped
/// }
/// ```
pub struct MitmproxyGuard {
    handle: Option<MitmproxyHandle>,
}

impl MitmproxyGuard {
    /// Creates a new mitmproxy container.
    pub fn new(image: &str) -> Result<Self, String> {
        let handle = start_mitmproxy(image, 0, &[])?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Creates mitmproxy with mTLS support.
    pub fn with_mtls(image: &str) -> Result<Self, String> {
        let handle = start_mitmproxy_with_mtls(image)?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Creates mitmproxy with custom cipher suites.
    pub fn with_ciphers(image: &str, ciphers: &str) -> Result<Self, String> {
        let handle = start_mitmproxy_with_ciphers(image, ciphers)?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Creates mitmproxy with TLS version restrictions.
    pub fn with_tls_version(
        image: &str,
        min_version: &str,
        max_version: &str,
    ) -> Result<Self, String> {
        let handle = start_mitmproxy_with_tls_version(image, min_version, max_version)?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Returns the proxy address.
    pub fn proxy_addr(&self) -> String {
        self.handle
            .as_ref()
            .map(|h| h.proxy_addr())
            .unwrap_or_default()
    }

    /// Returns the proxy URL.
    pub fn proxy_url(&self) -> String {
        self.handle
            .as_ref()
            .map(|h| h.proxy_url())
            .unwrap_or_default()
    }
}

impl Drop for MitmproxyGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = stop_mitmproxy(&handle.container_id);
        }
    }
}
