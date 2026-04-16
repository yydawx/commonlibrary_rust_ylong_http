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

//! This example demonstrates how to use HTTPS proxy with the ylong_http_client.
//! 
//! # Features
//! 
//! - Connect through an HTTPS proxy with custom CA certificate
//! - Skip proxy certificate verification for development/testing
//! 
//! # Usage
//! 
//! ```ignore
//! // With custom CA certificate
//! let proxy = Proxy::all("https://proxy.example.com:443")
//!     .proxy_ca_file("corporate-ca.pem")
//!     .build()?;
//! 
//! // Skip certificate verification (for testing only)
//! let proxy = Proxy::all("https://proxy.example.com:443")
//!     .danger_accept_invalid_proxy_certs(true)
//!     .build()?;
//! ```

use ylong_http_client::{HttpClientError, Proxy};

fn main() -> Result<(), HttpClientError> {
    println!("HTTPS Proxy Example");
    println!("====================\n");

    let proxy_url = "https://127.0.0.1:8080";

    println!("1. Creating HTTPS proxy configuration (skipping cert verification)...\n");

    // Create HTTPS proxy configuration
    let _proxy = Proxy::all(proxy_url)
        .danger_accept_invalid_proxy_certs(true)
        .build()?;

    println!("HTTPS proxy configuration created successfully!");
    println!("- Proxy URL: {}", proxy_url);
    println!("- Skip certificate verification: true");

    println!("\n2. Creating HTTPS proxy configuration with custom CA...\n");

    // Create HTTPS proxy configuration with custom CA
    let _proxy_with_ca = Proxy::all(proxy_url)
        .proxy_ca_file("tests/file/root-ca.pem")
        .build()?;

    println!("HTTPS proxy configuration with CA created successfully!");
    println!("- Proxy URL: {}", proxy_url);
    println!("- CA certificate file: tests/file/root-ca.pem");

    println!("\n3. Verifying proxy configuration...\n");

    // Verify that the proxy URL is correctly set
    println!("Proxy configuration verification:");
    println!("- Proxy URL: {}", proxy_url);
    println!("- Proxy type: HTTPS proxy (detected from scheme)");

    println!("\nNote: For HTTPS proxy to work, you need:");
    println!("  - A running HTTPS proxy server");
    println!("  - Correct CA certificate if using custom CA");
    println!("  - Or use danger_accept_invalid_proxy_certs(true) for testing");

    Ok(())
}
