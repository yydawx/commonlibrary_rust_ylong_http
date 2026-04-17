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

//! End-to-end test for HTTPS proxy TLS configuration API.
//!
//! This example tests the HTTPS proxy TLS configuration API by verifying
//! that all configuration options can be created successfully.
//!
//! Run with:
//! ```
//! OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include RUSTFLAGS="-L /usr/lib/x86_64-linux-gnu -l ssl -l crypto" \
//!     cargo run --example async_https_proxy_e2e --features "async,http1_1,ylong_base,tls_default"
//! ```

use ylong_http_client::{Proxy, TlsFileType, TlsVersion};

fn main() -> std::io::Result<()> {
    println!("===========================================");
    println!("HTTPS Proxy TLS Configuration API Test");
    println!("===========================================");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: proxy_ca_file
    println!("\n[Test 1] proxy_ca_file...");
    match Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 2: danger_accept_invalid_proxy_certs
    println!("\n[Test 2] danger_accept_invalid_proxy_certs...");
    match Proxy::all("https://proxy.example.com:443")
        .danger_accept_invalid_proxy_certs(true)
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 3: proxy_identity (mTLS)
    println!("\n[Test 3] proxy_identity (mTLS)...");
    match Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_identity(
            "tests/file/cert.pem",
            "tests/file/key.pem",
            TlsFileType::PEM,
        )
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 4: proxy_cipher_list
    println!("\n[Test 4] proxy_cipher_list...");
    match Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_cipher_list("ECDHE-RSA-AES256-GCM-SHA384")
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 5: proxy_min_proto_version
    println!("\n[Test 5] proxy_min_proto_version...");
    match Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_min_proto_version(TlsVersion::TLS_1_2)
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 6: proxy_max_proto_version
    println!("\n[Test 6] proxy_max_proto_version...");
    match Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_max_proto_version(TlsVersion::TLS_1_3)
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 7: Full TLS configuration
    println!("\n[Test 7] Full TLS configuration (all options)...");
    match Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .proxy_identity(
            "tests/file/cert.pem",
            "tests/file/key.pem",
            TlsFileType::PEM,
        )
        .proxy_cipher_list("ECDHE-RSA-AES256-GCM-SHA384")
        .proxy_min_proto_version(TlsVersion::TLS_1_2)
        .proxy_max_proto_version(TlsVersion::TLS_1_3)
        .build()
    {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    // Test 8: Chained configuration
    println!("\n[Test 8] Chained configuration...");
    let proxy = Proxy::all("https://proxy.example.com:443")
        .proxy_ca_file("tests/file/root-ca.pem")
        .danger_accept_invalid_proxy_certs(false)
        .build();
    match proxy {
        Ok(_) => {
            println!("    ✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("    ✗ FAILED: {}", e);
            failed += 1;
        }
    }

    println!("\n===========================================");
    println!("Test Results: {} passed, {} failed", passed, failed);
    println!("===========================================");

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
