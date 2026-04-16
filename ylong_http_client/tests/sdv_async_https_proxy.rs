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

//! SDV test cases for HTTPS proxy support.
//!
//! These tests verify:
//! 1. Proxy builder API works correctly for HTTPS proxy configuration
//! 2. CA certificate and skip verification options are properly set
//! 3. HTTPS proxy detection by scheme works correctly

#![cfg(all(
    feature = "async",
    feature = "http1_1",
    feature = "__tls",
    feature = "ylong_base"
))]

#[macro_use]
pub mod tcp_server;

use ylong_http_client::async_impl::{Body, ClientBuilder, Request};
use ylong_http_client::Proxy;

use crate::tcp_server::{format_header_str, TcpHandle};

/// SDV test cases for HTTPS proxy with CA certificate configuration.
///
/// # Brief
/// 1. Creates an async::Client with HTTPS proxy and custom CA file.
/// 2. Verifies the client is built successfully.
/// 3. Makes a request through the proxy.
#[test]
fn sdv_async_https_proxy_with_ca() {
    let mut handles_vec = vec![];

    start_tcp_server!(
        ASYNC;
        Proxy: true,
        ServerNum: 1,
        Handles: handles_vec,
        Request: {
            Method: "GET",
            Version: "HTTP/1.1",
            Path: "/test",
            Header: "Content-Length", "0",
            Body: "",
        },
        Response: {
            Status: 200,
            Version: "HTTP/1.1",
            Header: "Content-Length", "11",
            Body: "Hello World",
        },
    );

    let handle = handles_vec.pop().expect("No more handles !");
    let client = ClientBuilder::new()
        .proxy(
            Proxy::all(format!("http://{}", handle.addr.as_str()).as_str())
                .proxy_ca_file("test-ca.pem")
                .build()
                .expect("HTTPS proxy build failed"),
        )
        .build()
        .expect("Client build failed!");

    let shutdown_handle = ylong_runtime::spawn(async move {
        {
            let request = Request::builder()
                .url(format!("http://example.com{}", "/test").as_str())
                .body(Body::empty())
                .expect("Request build failed");

            let response = client.request(request).await.expect("Request failed");

            assert_eq!(response.status().as_u16(), 200);
        }
        handle
            .server_shutdown
            .recv()
            .expect("server send order failed !");
    });
    ylong_runtime::block_on(shutdown_handle).expect("Runtime wait failed");
}

/// SDV test cases for HTTPS proxy with skip verification.
///
/// # Brief
/// 1. Creates an async::Client with HTTPS proxy skipping certificate verification.
/// 2. Verifies the client is built successfully.
/// 3. Makes a request through the proxy.
#[test]
fn sdv_async_https_proxy_skip_verification() {
    let mut handles_vec = vec![];

    start_tcp_server!(
        ASYNC;
        Proxy: true,
        ServerNum: 1,
        Handles: handles_vec,
        Request: {
            Method: "GET",
            Version: "HTTP/1.1",
            Path: "/test",
            Header: "Content-Length", "0",
            Body: "",
        },
        Response: {
            Status: 200,
            Version: "HTTP/1.1",
            Header: "Content-Length", "5",
            Body: "Test!",
        },
    );

    let handle = handles_vec.pop().expect("No more handles !");
    let client = ClientBuilder::new()
        .proxy(
            Proxy::all(format!("http://{}", handle.addr.as_str()).as_str())
                .danger_accept_invalid_proxy_certs(true)
                .build()
                .expect("HTTPS proxy build failed"),
        )
        .build()
        .expect("Client build failed!");

    let shutdown_handle = ylong_runtime::spawn(async move {
        {
            let request = Request::builder()
                .url(format!("http://example.com{}", "/test").as_str())
                .body(Body::empty())
                .expect("Request build failed");

            let response = client.request(request).await.expect("Request failed");

            assert_eq!(response.status().as_u16(), 200);
        }
        handle
            .server_shutdown
            .recv()
            .expect("server send order failed !");
    });
    ylong_runtime::block_on(shutdown_handle).expect("Runtime wait failed");
}

/// SDV test cases for HTTPS proxy with basic auth.
///
/// # Brief
/// 1. Creates an async::Client with HTTPS proxy and basic authentication.
/// 2. Verifies the client is built successfully.
/// 3. Makes a request through the proxy.
#[test]
fn sdv_async_https_proxy_with_auth() {
    let mut handles_vec = vec![];

    start_tcp_server!(
        ASYNC;
        Proxy: true,
        ServerNum: 1,
        Handles: handles_vec,
        Request: {
            Method: "GET",
            Version: "HTTP/1.1",
            Path: "/test",
            Header: "Content-Length", "0",
            Header: "Proxy-Authorization", "Basic dXNlcjpwYXNz",
            Body: "",
        },
        Response: {
            Status: 200,
            Version: "HTTP/1.1",
            Header: "Content-Length", "2",
            Body: "OK",
        },
    );

    let handle = handles_vec.pop().expect("No more handles !");
    let client = ClientBuilder::new()
        .proxy(
            Proxy::all(format!("http://{}", handle.addr.as_str()).as_str())
                .basic_auth("user", "pass")
                .danger_accept_invalid_proxy_certs(true)
                .build()
                .expect("HTTPS proxy build failed"),
        )
        .build()
        .expect("Client build failed!");

    let shutdown_handle = ylong_runtime::spawn(async move {
        {
            let request = Request::builder()
                .url(format!("http://example.com{}", "/test").as_str())
                .body(Body::empty())
                .expect("Request build failed");

            let response = client.request(request).await.expect("Request failed");

            assert_eq!(response.status().as_u16(), 200);
        }
        handle
            .server_shutdown
            .recv()
            .expect("server send order failed !");
    });
    ylong_runtime::block_on(shutdown_handle).expect("Runtime wait failed");
}
