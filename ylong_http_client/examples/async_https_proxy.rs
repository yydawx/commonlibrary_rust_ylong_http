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
//! - Client certificate authentication (mTLS) with proxy
//! - Custom cipher suites and protocol versions
//!
//! # Usage
//!
//! ```ignore
//! // With custom CA certificate (单向认证)
//! let proxy = Proxy::all("https://proxy.example.com:443")
//!     .proxy_ca_file("corporate-ca.pem")
//!     .build()?;
//!
//! // Skip certificate verification (for testing only)
//! let proxy = Proxy::all("https://proxy.example.com:443")
//!     .danger_accept_invalid_proxy_certs(true)
//!     .build()?;
//!
//! // With client certificate (双向认证/mTLS)
//! let proxy = Proxy::all("https://proxy.example.com:443")
//!     .proxy_ca_file("corporate-ca.pem")
//!     .proxy_identity("client-cert.pem", "client-key.pem", TlsFileType::PEM)
//!     .build()?;
//!
//! // Custom cipher suites and protocol versions
//! let proxy = Proxy::all("https://proxy.example.com:443")
//!     .proxy_cipher_list("ECDHE-RSA-AES256-GCM-SHA384")
//!     .proxy_min_proto_version(TlsVersion::TLS_1_2)
//!     .proxy_max_proto_version(TlsVersion::TLS_1_3)
//!     .build()?;
//! ```

use ylong_http_client::{HttpClientError, Proxy, TlsFileType, TlsVersion};

fn main() -> Result<(), HttpClientError> {
    println!("HTTPS Proxy TLS Configuration Examples");
    println!("=====================================\n");

    println!("1. Basic HTTPS proxy with CA certificate...\n");
    let proxy_with_ca = Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .build()?;
    println!("   ✓ Created proxy with custom CA certificate\n");

    println!("2. HTTPS proxy with skipped verification (for testing)...\n");
    let proxy_skip_verify = Proxy::all("https://proxy.example.com:443")
        .danger_accept_invalid_proxy_certs(true)
        .build()?;
    println!("   ✓ Created proxy with skipped certificate verification\n");

    println!("3. HTTPS proxy with mTLS (client certificate authentication)...\n");
    let proxy_mtls = Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_identity(
            "tests/file/cert.pem",
            "tests/file/key.pem",
            TlsFileType::PEM,
        )
        .build()?;
    println!("   ✓ Created proxy with client certificate (mTLS)\n");

    println!("4. HTTPS proxy with custom cipher suites...\n");
    let proxy_cipher = Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_cipher_list("ECDHE-RSA-AES256-GCM-SHA384")
        .build()?;
    println!("   ✓ Created proxy with custom cipher suites\n");

    println!("5. HTTPS proxy with protocol version restrictions...\n");
    let proxy_version = Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_min_proto_version(TlsVersion::TLS_1_2)
        .proxy_max_proto_version(TlsVersion::TLS_1_3)
        .build()?;
    println!("   ✓ Created proxy with TLS 1.2-1.3 version restriction\n");

    println!("6. HTTPS proxy with all TLS configurations combined...\n");
    let proxy_full = Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_identity(
            "tests/file/cert.pem",
            "tests/file/key.pem",
            TlsFileType::PEM,
        )
        .proxy_cipher_list("ECDHE-RSA-AES256-GCM-SHA384")
        .proxy_min_proto_version(TlsVersion::TLS_1_2)
        .proxy_max_proto_version(TlsVersion::TLS_1_3)
        .build()?;
    println!("   ✓ Created proxy with full TLS configuration\n");

    println!("=====================================");
    println!("All proxy configurations created successfully!");
    println!("\nNote: For HTTPS proxy to work, you need:");
    println!("  - A running HTTPS proxy server");
    println!("  - Correct CA certificate if using custom CA");
    println!("  - Or use danger_accept_invalid_proxy_certs(true) for testing");
    println!("  - Client certificates for mTLS authentication if required by proxy");

    Ok(())
}
