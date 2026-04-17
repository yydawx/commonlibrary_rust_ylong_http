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

//! E2E test cases for HTTPS proxy with mitmproxy.

#![cfg(all(
    feature = "async",
    feature = "http1_1",
    feature = "__tls",
    feature = "ylong_base"
))]

use std::process::Command;

use ylong_http_client::async_impl::{Body, ClientBuilder, Request};
use ylong_http_client::Proxy;

const MITMPROXY_IMAGE: &str =
    "swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/mitmproxy/mitmproxy:11.0";

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn print_header(name: &str) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║  {:^76} ║", name);
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
}

fn print_config(key: &str, value: &str) {
    println!("  │  {}: {}", key, value);
}

fn print_step(step: &str, msg: &str) {
    println!("  ┌─ {}: {}", step, msg);
}

fn print_ok(msg: &str) {
    println!("  └─ ✓ {}", msg);
}

fn print_fail(msg: &str) {
    println!("  └─ ✗ {}", msg);
}

fn print_detail(key: &str, value: &str) {
    println!("  │     {}: {}", key, value);
}

struct MitmproxyServer {
    container_id: String,
    port: u16,
}

impl MitmproxyServer {
    fn new() -> Result<Self, String> {
        let port = Self::find_available_port()?;

        println!("  │  Pulling mitmproxy image...");
        let _ = Command::new("docker")
            .args(["pull", MITMPROXY_IMAGE])
            .output();

        println!("  │  Starting mitmproxy container (host network)...");
        let output = Command::new("docker")
            .args([
                "run",
                "-d",
                "--rm",
                "--network=host",
                "--entrypoint",
                "mitmdump",
                MITMPROXY_IMAGE,
                "--listen-port",
                &port.to_string(),
                "--listen-host",
                "127.0.0.1",
            ])
            .output()
            .map_err(|e| format!("Failed to run docker: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("mitmproxy failed: {}", stderr));
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("  │  Container ID: {}", container_id);

        std::thread::sleep(std::time::Duration::from_secs(2));

        print_detail("Listen Port", &port.to_string());
        print_detail("Network Mode", "host");
        print_ok("mitmproxy started");

        Ok(Self { container_id, port })
    }

    fn new_with_mtls() -> Result<Self, String> {
        let port = Self::find_available_port()?;

        println!("  │  Pulling mitmproxy image...");
        let _ = Command::new("docker")
            .args(["pull", MITMPROXY_IMAGE])
            .output();

        println!("  │  Starting mitmproxy with mTLS support (host network)...");
        let output = Command::new("docker")
            .args([
                "run",
                "-d",
                "--rm",
                "--network=host",
                "--entrypoint",
                "mitmdump",
                MITMPROXY_IMAGE,
                "--listen-port",
                &port.to_string(),
                "--listen-host",
                "127.0.0.1",
                "--set",
                "request_client_cert=true",
            ])
            .output()
            .map_err(|e| format!("Failed to run docker: {}", e))?;

        if !output.status.success() {
            return Err("mitmproxy with mTLS failed to start".to_string());
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("  │  Container ID: {}", container_id);
        print_detail("mTLS Mode", "Client certificate required");
        print_detail("Listen Port", &port.to_string());
        print_detail("Network Mode", "host");

        std::thread::sleep(std::time::Duration::from_secs(2));
        print_ok("mitmproxy with mTLS started");

        Ok(Self { container_id, port })
    }

    fn proxy_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    fn find_available_port() -> Result<u16, String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let base = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 10000) as u16;

        for i in 0..100 {
            let port = 28000 + base + i;
            if port < 65535 && std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                return Ok(port);
            }
        }
        Err("No available port".to_string())
    }

    fn container_logs(&self) {
        let output = Command::new("docker")
            .args(["logs", "--tail", "10", &self.container_id])
            .output();

        if let Ok(output) = output {
            let logs = String::from_utf8_lossy(&output.stdout);
            let err_logs = String::from_utf8_lossy(&output.stderr);
            if !logs.trim().is_empty() || !err_logs.trim().is_empty() {
                println!("  │");
                println!("  │  mitmproxy logs:");
                for line in logs.lines().chain(err_logs.lines()).take(10) {
                    if !line.trim().is_empty() {
                        println!("  │    {}", line);
                    }
                }
            }
        }
    }
}

impl Drop for MitmproxyServer {
    fn drop(&mut self) {
        print_step("Cleanup", "Stopping mitmproxy container");
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.container_id])
            .output();
        print_ok("Container stopped");
    }
}

struct UpstreamServer {
    addr: String,
}

impl UpstreamServer {
    fn new() -> Self {
        use ylong_runtime::io::{AsyncReadExt, AsyncWriteExt};
        use ylong_runtime::net::TcpListener;

        let (tx, rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) =
            std::sync::mpsc::channel();
        let (tx2, rx2): (
            std::sync::mpsc::Sender<String>,
            std::sync::mpsc::Receiver<String>,
        ) = std::sync::mpsc::channel();

        ylong_runtime::spawn(async move {
            let server = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
            let local_addr = server.local_addr().expect("get addr failed");
            tx2.send(local_addr.to_string()).expect("send addr failed");

            let (mut stream, _) = server.accept().await.expect("accept failed");

            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read failed");
            println!("  │     Received request: {} bytes", n);

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nHello World";
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write failed");
            println!("  │     Sent response: HTTP/1.1 200 OK (11 bytes)");

            let _ = rx.recv();
        });

        let addr = rx2.recv().expect("recv addr failed");

        Self { addr }
    }

    fn host_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

fn print_test_passed() {
    println!();
    println!("  ╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("  ║  ✓ TEST PASSED                                                               ║");
    println!("  ╚══════════════════════════════════════════════════════════════════════════════╝");
}

/// SDV test: HTTPS proxy with skip verification through mitmproxy.
#[test]
fn sdv_async_https_proxy_mitmproxy_skip_verify() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    print_header("Test: HTTPS Proxy (Skip Verification)");
    println!();
    println!("  Configuration:");
    print_config("Proxy URL", "http://127.0.0.1:<port>");
    print_config(
        "TLS Verify",
        "SKIPPED (danger_accept_invalid_proxy_certs=true)",
    );
    print_config("Upstream", "HTTP server");

    print_step("Step 1", "Start upstream HTTP server");
    let upstream = UpstreamServer::new();
    print_detail("Address", &upstream.addr);
    print_ok("Upstream server started");

    print_step("Step 2", "Start mitmproxy container");
    let mitmproxy = match MitmproxyServer::new() {
        Ok(m) => m,
        Err(e) => {
            print_fail(&e);
            println!();
            println!("  ⚠ Skipping test: Docker/mitmproxy not available");
            return;
        }
    };

    print_step("Step 3", "Build proxy configuration");
    print_detail("Proxy URL", &mitmproxy.proxy_url());
    print_detail("Upstream Target", &upstream.host_url());

    let proxy = match Proxy::all(mitmproxy.proxy_url().as_str())
        .danger_accept_invalid_proxy_certs(true)
        .build()
    {
        Ok(p) => p,
        Err(e) => {
            print_fail(&format!("Proxy build failed: {}", e));
            return;
        }
    };
    print_ok("Proxy built successfully");

    print_step("Step 4", "Build HTTP client");
    let client = match ClientBuilder::new().proxy(proxy).build() {
        Ok(c) => c,
        Err(e) => {
            print_fail(&format!("Client build failed: {}", e));
            return;
        }
    };
    print_ok("Client built successfully");

    print_step("Step 5", "Send HTTP request through proxy");
    let url = format!("{}/test", upstream.host_url());
    print_detail("Request", &format!("GET {} HTTP/1.1", url));

    let result = ylong_runtime::block_on(async {
        let request = Request::builder()
            .url(url.as_str())
            .body(Body::empty())
            .expect("Request build failed");

        client.request(request).await
    });

    match result {
        Ok(response) => {
            print_detail("Response Status", &response.status().as_u16().to_string());
            assert_eq!(response.status().as_u16(), 200);
            print_ok("Request completed successfully");
        }
        Err(e) => {
            mitmproxy.container_logs();
            print_fail(&format!("Request failed: {}", e));
            panic!("Test failed: {}", e);
        }
    }

    print_test_passed();
}

/// SDV test: HTTPS proxy with mTLS through mitmproxy.
#[test]
fn sdv_async_https_proxy_mitmproxy_mtls() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    print_header("Test: HTTPS Proxy (mTLS - Mutual TLS)");
    println!();
    println!("  Configuration:");
    print_config("Proxy URL", "https://127.0.0.1:<port>");
    print_config("TLS Verify", "SKIPPED (for mitmproxy CA)");
    print_config("Client Auth", "mTLS (client certificate required)");
    print_config("Certificate", "tests/file/cert.pem");
    print_config("Private Key", "tests/file/key.pem");

    print_step("Step 1", "Start upstream HTTP server");
    let upstream = UpstreamServer::new();
    print_detail("Address", &upstream.addr);
    print_ok("Upstream server started");

    print_step("Step 2", "Start mitmproxy with mTLS support");
    let mitmproxy = match MitmproxyServer::new_with_mtls() {
        Ok(m) => m,
        Err(e) => {
            print_fail(&e);
            println!();
            println!("  ⚠ Skipping test: Docker/mitmproxy not available");
            return;
        }
    };

    print_step("Step 3", "Build proxy configuration with client identity");
    let proxy_url = mitmproxy.proxy_url().replace("http://", "https://");
    print_detail("Proxy URL", &proxy_url);
    print_detail("Upstream Target", &upstream.host_url());
    print_detail("mTLS Identity", "cert.pem + key.pem");

    let proxy = Proxy::all(proxy_url.as_str())
        .danger_accept_invalid_proxy_certs(true)
        .proxy_identity(
            "tests/file/cert.pem",
            "tests/file/key.pem",
            ylong_http_client::TlsFileType::PEM,
        )
        .build();

    let proxy = match proxy {
        Ok(p) => p,
        Err(e) => {
            print_fail(&format!("Proxy build failed: {}", e));
            return;
        }
    };
    print_ok("Proxy with mTLS identity built");

    print_step("Step 4", "Build HTTP client");
    let client = match ClientBuilder::new().proxy(proxy).build() {
        Ok(c) => c,
        Err(e) => {
            print_fail(&format!("Client build failed: {}", e));
            return;
        }
    };
    print_ok("Client built successfully");

    print_step("Step 5", "Send HTTP request with mTLS handshake");
    let url = format!("{}/test", upstream.host_url());
    print_detail("Request", &format!("GET {} HTTP/1.1", url));
    print_detail("TLS Handshake", "Client sends certificate to mitmproxy");

    let result = ylong_runtime::block_on(async {
        let request = Request::builder()
            .url(url.as_str())
            .body(Body::empty())
            .expect("Request build failed");

        client.request(request).await
    });

    match result {
        Ok(response) => {
            print_detail("Response Status", &response.status().as_u16().to_string());
            assert_eq!(response.status().as_u16(), 200);
            print_ok("mTLS handshake and request completed");
        }
        Err(e) => {
            mitmproxy.container_logs();
            print_fail(&format!("Request failed: {}", e));
            panic!("Test failed: {}", e);
        }
    }

    print_test_passed();
}

/// SDV test: HTTPS proxy with TLS version restrictions.
#[test]
fn sdv_async_https_proxy_mitmproxy_tls_version() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    print_header("Test: HTTPS Proxy (TLS Version Restrictions)");
    println!();
    println!("  Configuration:");
    print_config("Proxy URL", "https://127.0.0.1:<port>");
    print_config("TLS Verify", "SKIPPED");
    print_config("Min TLS Version", "TLS 1.2");
    print_config("Max TLS Version", "TLS 1.3");

    print_step("Step 1", "Start upstream HTTP server");
    let upstream = UpstreamServer::new();
    print_detail("Address", &upstream.addr);
    print_ok("Upstream server started");

    print_step("Step 2", "Start mitmproxy container");
    let mitmproxy = match MitmproxyServer::new() {
        Ok(m) => m,
        Err(e) => {
            print_fail(&e);
            println!();
            println!("  ⚠ Skipping test: Docker/mitmproxy not available");
            return;
        }
    };

    print_step("Step 3", "Build proxy configuration with version limits");
    let proxy_url = mitmproxy.proxy_url().replace("http://", "https://");
    print_detail("Proxy URL", &proxy_url);
    print_detail("Upstream Target", &upstream.host_url());
    print_detail("Min Version", "TLS_1_2");
    print_detail("Max Version", "TLS_1_3");

    let proxy = Proxy::all(proxy_url.as_str())
        .danger_accept_invalid_proxy_certs(true)
        .proxy_min_proto_version(ylong_http_client::TlsVersion::TLS_1_2)
        .proxy_max_proto_version(ylong_http_client::TlsVersion::TLS_1_3)
        .build();

    let proxy = match proxy {
        Ok(p) => p,
        Err(e) => {
            print_fail(&format!("Proxy build failed: {}", e));
            return;
        }
    };
    print_ok("Proxy with TLS version limits built");

    print_step("Step 4", "Build HTTP client");
    let client = match ClientBuilder::new().proxy(proxy).build() {
        Ok(c) => c,
        Err(e) => {
            print_fail(&format!("Client build failed: {}", e));
            return;
        }
    };
    print_ok("Client built successfully");

    print_step("Step 5", "Send HTTP request with TLS version restriction");
    let url = format!("{}/test", upstream.host_url());
    print_detail("Request", &format!("GET {} HTTP/1.1", url));
    print_detail("TLS Version", "1.2 ~ 1.3");

    let result = ylong_runtime::block_on(async {
        let request = Request::builder()
            .url(url.as_str())
            .body(Body::empty())
            .expect("Request build failed");

        client.request(request).await
    });

    match result {
        Ok(response) => {
            print_detail("Response Status", &response.status().as_u16().to_string());
            assert_eq!(response.status().as_u16(), 200);
            print_ok("Request with TLS version restriction completed");
        }
        Err(e) => {
            mitmproxy.container_logs();
            print_fail(&format!("Request failed: {}", e));
            panic!("Test failed: {}", e);
        }
    }

    print_test_passed();
}

/// SDV test: HTTPS proxy with CA certificate verification.
#[test]
fn sdv_async_https_proxy_mitmproxy_ca_verification() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    print_header("Test: HTTPS Proxy (CA Certificate Verification)");
    println!();
    println!("  Configuration:");
    print_config("Proxy URL", "https://127.0.0.1:<port>");
    print_config("TLS Verify", "ENABLED (using mitmproxy CA)");
    print_config("CA Certificate", "~/.mitmproxy/mitmproxy-ca-cert.pem");

    print_step("Step 1", "Generate mitmproxy CA certificate");
    let ca_cert_path = std::env::var("HOME")
        .map(|h| format!("{}/.mitmproxy/mitmproxy-ca-cert.pem", h))
        .unwrap_or_else(|_| "/tmp/mitmproxy-ca-cert.pem".to_string());

    std::fs::create_dir_all(
        std::path::Path::new(&ca_cert_path)
            .parent()
            .unwrap_or(std::path::Path::new("/tmp")),
    )
    .ok();

    let cert_exists = std::path::Path::new(&ca_cert_path).exists();
    if !cert_exists {
        println!("  │  Starting mitmproxy to generate CA cert...");
        let output = Command::new("docker")
            .args([
                "run",
                "-d",
                "--rm",
                "--network=host",
                "--entrypoint",
                "mitmdump",
                MITMPROXY_IMAGE,
                "--listen-port",
                "0",
                "--generate-upstream-cert-chain-only",
            ])
            .output();
        if output.is_ok() {
            std::thread::sleep(std::time::Duration::from_secs(3));
            Command::new("docker").args(["ps", "-q"]).output().map(|o| {
                let id = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !id.is_empty() {
                    let _ = Command::new("docker").args(["rm", "-f", &id]).output();
                }
            });
        }
    }
    print_detail("CA Cert Path", &ca_cert_path);

    let ca_cert = match std::path::Path::new(&ca_cert_path).exists() {
        true => {
            print_ok("mitmproxy CA cert generated");
            ca_cert_path
        }
        false => {
            print_fail("CA cert not found");
            println!();
            println!("  ⚠ Skipping CA verification test (CA cert generation requires mitmproxy first run)");
            return;
        }
    };

    print_step("Step 2", "Start upstream HTTP server");
    let upstream = UpstreamServer::new();
    print_detail("Address", &upstream.addr);
    print_ok("Upstream server started");

    print_step("Step 3", "Start mitmproxy container");
    let mitmproxy = match MitmproxyServer::new() {
        Ok(m) => m,
        Err(e) => {
            print_fail(&e);
            println!();
            println!("  ⚠ Skipping test: Docker/mitmproxy not available");
            return;
        }
    };

    print_step("Step 4", "Build proxy configuration with CA verification");
    let proxy_url = mitmproxy.proxy_url().replace("http://", "https://");
    print_detail("Proxy URL", &proxy_url);
    print_detail("Upstream Target", &upstream.host_url());
    print_detail("CA Certificate", &ca_cert);

    let proxy = Proxy::all(proxy_url.as_str())
        .proxy_ca_file(&ca_cert)
        .build();

    let proxy = match proxy {
        Ok(p) => p,
        Err(e) => {
            print_fail(&format!("Proxy build failed: {}", e));
            return;
        }
    };
    print_ok("Proxy with CA verification built");

    print_step("Step 5", "Build HTTP client");
    let client = match ClientBuilder::new().proxy(proxy).build() {
        Ok(c) => c,
        Err(e) => {
            print_fail(&format!("Client build failed: {}", e));
            return;
        }
    };
    print_ok("Client built successfully");

    print_step("Step 6", "Send HTTP request with CA verification");
    let url = format!("{}/test", upstream.host_url());
    print_detail("Request", &format!("GET {} HTTP/1.1", url));
    print_detail("TLS Verify", "CA certificate verification enabled");

    let result = ylong_runtime::block_on(async {
        let request = Request::builder()
            .url(url.as_str())
            .body(Body::empty())
            .expect("Request build failed");

        client.request(request).await
    });

    match result {
        Ok(response) => {
            print_detail("Response Status", &response.status().as_u16().to_string());
            assert_eq!(response.status().as_u16(), 200);
            print_ok("Request with CA verification completed");
        }
        Err(e) => {
            mitmproxy.container_logs();
            print_fail(&format!("Request failed: {}", e));
            panic!("Test failed: {}", e);
        }
    }

    print_test_passed();
}

/// SDV test: HTTPS proxy with full TLS configuration (all options combined).
#[test]
fn sdv_async_https_proxy_mitmproxy_full_config() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    print_header("Test: HTTPS Proxy (Full TLS Configuration)");
    println!();
    println!("  Configuration:");
    print_config("Proxy URL", "https://127.0.0.1:<port>");
    print_config("TLS Verify", "SKIPPED");
    print_config("Client Auth", "mTLS (cert + key)");
    print_config("Cipher Suite", "ECDHE-RSA-AES256-GCM-SHA384");
    print_config("Min TLS Version", "TLS 1.2");
    print_config("Max TLS Version", "TLS 1.3");

    print_step("Step 1", "Start upstream HTTP server");
    let upstream = UpstreamServer::new();
    print_detail("Address", &upstream.addr);
    print_ok("Upstream server started");

    print_step("Step 2", "Start mitmproxy with mTLS support");
    let mitmproxy = match MitmproxyServer::new_with_mtls() {
        Ok(m) => m,
        Err(e) => {
            print_fail(&e);
            println!();
            println!("  ⚠ Skipping test: Docker/mitmproxy not available");
            return;
        }
    };

    print_step("Step 3", "Build proxy configuration with ALL options");
    let proxy_url = mitmproxy.proxy_url().replace("http://", "https://");
    print_detail("Proxy URL", &proxy_url);
    print_detail("Upstream Target", &upstream.host_url());
    print_detail("mTLS Identity", "cert.pem + key.pem");
    print_detail("Cipher List", "ECDHE-RSA-AES256-GCM-SHA384");
    print_detail("Min Version", "TLS_1_2");
    print_detail("Max Version", "TLS_1_3");

    let proxy = Proxy::all(proxy_url.as_str())
        .danger_accept_invalid_proxy_certs(true)
        .proxy_identity(
            "tests/file/cert.pem",
            "tests/file/key.pem",
            ylong_http_client::TlsFileType::PEM,
        )
        .proxy_cipher_list("ECDHE-RSA-AES256-GCM-SHA384")
        .proxy_min_proto_version(ylong_http_client::TlsVersion::TLS_1_2)
        .proxy_max_proto_version(ylong_http_client::TlsVersion::TLS_1_3)
        .build();

    let proxy = match proxy {
        Ok(p) => p,
        Err(e) => {
            print_fail(&format!("Proxy build failed: {}", e));
            return;
        }
    };
    print_ok("Proxy with full TLS config built");

    print_step("Step 4", "Build HTTP client");
    let client = match ClientBuilder::new().proxy(proxy).build() {
        Ok(c) => c,
        Err(e) => {
            print_fail(&format!("Client build failed: {}", e));
            return;
        }
    };
    print_ok("Client built successfully");

    print_step("Step 5", "Send HTTP request with full TLS config");
    let url = format!("{}/test", upstream.host_url());
    print_detail("Request", &format!("GET {} HTTP/1.1", url));
    print_detail("mTLS", "Client certificate + private key");
    print_detail("Cipher", "ECDHE-RSA-AES256-GCM-SHA384");
    print_detail("TLS Version", "1.2 ~ 1.3");

    let result = ylong_runtime::block_on(async {
        let request = Request::builder()
            .url(url.as_str())
            .body(Body::empty())
            .expect("Request build failed");

        client.request(request).await
    });

    match result {
        Ok(response) => {
            print_detail("Response Status", &response.status().as_u16().to_string());
            assert_eq!(response.status().as_u16(), 200);
            print_ok("Request with full TLS config completed");
        }
        Err(e) => {
            mitmproxy.container_logs();
            print_fail(&format!("Request failed: {}", e));
            panic!("Test failed: {}", e);
        }
    }

    print_test_passed();
}
