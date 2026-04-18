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

//! Proxy intercept and matching logic.

use core::convert::TryFrom;
use std::net::IpAddr;

use ylong_http::headers::HeaderValue;
use ylong_http::request::uri::{Authority, Scheme, Uri};

use crate::error::HttpClientError;
use crate::util::base64::encode;
use crate::util::normalizer::UriFormatter;

#[cfg(feature = "__tls")]
use crate::util::config::{Certificate, ProxyTlsConfig};

/// Proxy intercept type.
#[derive(Clone)]
pub enum Intercept {
    /// Matches HTTP URIs only.
    Http(ProxyInfo),
    /// Matches HTTPS URIs only.
    Https(ProxyInfo),
    /// Matches all URIs (both HTTP and HTTPS).
    All(ProxyInfo),
}

impl Intercept {
    /// Returns a reference to the underlying ProxyInfo.
    pub fn proxy_info(&self) -> &ProxyInfo {
        match self {
            Self::All(info) => info,
            Self::Http(info) => info,
            Self::Https(info) => info,
        }
    }
}

/// Proxy information including scheme, authority, and authentication.
#[derive(Clone)]
pub struct ProxyInfo {
    pub(crate) scheme: Scheme,
    pub(crate) authority: Authority,
    pub(crate) basic_auth: Option<HeaderValue>,
}

impl ProxyInfo {
    /// Creates a new ProxyInfo from a proxy URI string.
    pub fn new(uri: &str) -> Result<Self, HttpClientError> {
        let mut uri = match Uri::try_from(uri) {
            Ok(u) => u,
            Err(e) => {
                return err_from_other!(Build, e);
            }
        };
        UriFormatter::new().format(&mut uri)?;
        let (scheme, authority, _, _) = uri.into_parts();
        Ok(Self {
            basic_auth: None,
            scheme: scheme.unwrap(),
            authority: authority.unwrap(),
        })
    }

    /// Returns the authority (host:port) of the proxy.
    pub fn authority(&self) -> &Authority {
        &self.authority
    }

    /// Returns the scheme of the proxy.
    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }
}

/// A configured proxy with intercept rules and optional TLS configuration.
#[derive(Clone)]
pub struct Proxy {
    pub(crate) intercept: Intercept,
    pub(crate) no_proxy: Option<NoProxy>,
    #[cfg(feature = "__tls")]
    pub(crate) tls_config: ProxyTlsConfig,
}

impl Proxy {
    /// Creates a new HTTP proxy.
    pub fn http(uri: &str) -> Result<Self, HttpClientError> {
        Ok(Proxy::new(Intercept::Http(ProxyInfo::new(uri)?)))
    }

    /// Creates a new HTTPS proxy.
    pub fn https(uri: &str) -> Result<Self, HttpClientError> {
        Ok(Proxy::new(Intercept::Https(ProxyInfo::new(uri)?)))
    }

    /// Creates a proxy that intercepts all protocols.
    pub fn all(uri: &str) -> Result<Self, HttpClientError> {
        Ok(Proxy::new(Intercept::All(ProxyInfo::new(uri)?)))
    }

    /// Sets basic authentication for this proxy.
    pub fn basic_auth(&mut self, username: &str, password: &str) {
        let auth = encode(format!("{username}:{password}").as_bytes());
        let mut auth = HeaderValue::from_bytes(auth.as_slice())
            .unwrap_or_else(|_| HeaderValue::from_bytes(b"").unwrap());
        auth.set_sensitive(true);

        match &mut self.intercept {
            Intercept::All(info) => info.basic_auth = Some(auth),
            Intercept::Http(info) => info.basic_auth = Some(auth),
            Intercept::Https(info) => info.basic_auth = Some(auth),
        }
    }

    /// Sets the no_proxy exclusion list.
    pub fn no_proxy(&mut self, no_proxy: &str) {
        self.no_proxy = NoProxy::from_str(no_proxy);
    }

    /// Sets the CA certificate for HTTPS proxy.
    #[cfg(feature = "__tls")]
    pub fn set_proxy_ca(&mut self, cert: Certificate) {
        self.tls_config.ca = Some(cert);
    }

    /// Sets whether to skip certificate verification for HTTPS proxy.
    #[cfg(feature = "__tls")]
    pub fn set_danger_accept_invalid_proxy(&mut self, skip: bool) {
        self.tls_config.skip_verify = skip;
    }

    /// Returns the TLS configuration for this proxy.
    #[cfg(feature = "__tls")]
    pub fn tls_config(&self) -> &ProxyTlsConfig {
        &self.tls_config
    }

    /// Returns whether this is an HTTPS proxy.
    pub fn is_https_proxy(&self) -> bool {
        matches!(self.intercept.proxy_info().scheme, Scheme::HTTPS)
    }

    /// Rewrites a target URI to use the proxy.
    pub fn via_proxy(&self, uri: &Uri) -> Uri {
        let info = self.intercept.proxy_info();
        let mut builder = Uri::builder();
        builder = builder
            .scheme(info.scheme().clone())
            .authority(info.authority().clone());

        if let Some(path) = uri.path() {
            builder = builder.path(path.clone());
        }
        if let Some(query) = uri.query() {
            builder = builder.query(query.clone());
        }
        builder.build().unwrap()
    }

    /// Checks if this proxy should intercept the given URI.
    pub fn is_intercepted(&self, uri: &Uri) -> bool {
        let no_proxy = self
            .no_proxy
            .as_ref()
            .map(|no_proxy| no_proxy.contain(uri.host().unwrap().as_str()))
            .unwrap_or(false);

        match self.intercept {
            Intercept::All(_) => !no_proxy,
            Intercept::Http(_) => !no_proxy && *uri.scheme().unwrap() == Scheme::HTTP,
            Intercept::Https(_) => !no_proxy && *uri.scheme().unwrap() == Scheme::HTTPS,
        }
    }

    /// Returns the basic auth credentials if set.
    pub(crate) fn auth_header(&self) -> Option<&HeaderValue> {
        self.intercept.proxy_info().basic_auth.as_ref()
    }

    fn new(intercept: Intercept) -> Self {
        Self {
            intercept,
            no_proxy: None,
            #[cfg(feature = "__tls")]
            tls_config: ProxyTlsConfig::new(),
        }
    }
}

/// NoProxy matcher for excluding hosts from proxying.
#[derive(Clone, Default)]
pub struct NoProxy {
    ips: Vec<IpAddr>,
    domains: Vec<String>,
}

impl NoProxy {
    /// Creates a NoProxy matcher from a comma-separated string.
    pub fn from_str(no_proxy: &str) -> Option<Self> {
        if no_proxy.is_empty() {
            return None;
        }

        let no_proxy_vec = no_proxy.split(',').map(|c| c.trim()).collect::<Vec<&str>>();
        let mut ip_list = Vec::new();
        let mut domains_list = Vec::new();

        for host in no_proxy_vec {
            let address = match Uri::from_bytes(host.as_bytes()) {
                Ok(uri) => uri,
                Err(_) => continue,
            };
            match address.host().unwrap().as_str().parse::<IpAddr>() {
                Ok(ip) => ip_list.push(ip),
                Err(_) => domains_list.push(host.to_string()),
            }
        }
        Some(NoProxy {
            ips: ip_list,
            domains: domains_list,
        })
    }

    /// Checks if a host should bypass the proxy.
    pub fn contain(&self, proxy_host: &str) -> bool {
        match proxy_host.parse::<IpAddr>() {
            Ok(ip) => self.contains_ip(ip),
            Err(_) => self.contains_domain(proxy_host),
        }
    }

    fn contains_ip(&self, ip: IpAddr) -> bool {
        self.ips.contains(&ip)
    }

    fn contains_domain(&self, domain: &str) -> bool {
        for block_domain in &self.domains {
            if block_domain == "*" {
                return true;
            }

            if block_domain.starts_with('*') {
                let suffix = block_domain.trim_start_matches('*');
                if suffix.is_empty() {
                    return false;
                }
                if let Some(suffix_pos) = domain.find(suffix) {
                    if suffix_pos > 0 && domain.as_bytes()[suffix_pos] == b'.' {
                        return true;
                    }
                }
            } else if block_domain == domain {
                return true;
            } else if domain.ends_with(block_domain) {
                let prefix_len = domain.len() - block_domain.len();
                if prefix_len == 0 || domain.as_bytes().get(prefix_len) == Some(&b'.') {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_http() {
        let proxy = Proxy::http("http://proxy.example.com:8080").unwrap();
        assert!(!proxy.is_https_proxy());
    }

    #[test]
    fn test_proxy_https() {
        let proxy = Proxy::https("https://proxy.example.com:8443").unwrap();
        assert!(proxy.is_https_proxy());
    }

    #[test]
    fn test_proxy_all() {
        let proxy = Proxy::all("http://proxy.example.com:8080").unwrap();
        assert!(!proxy.is_https_proxy());
    }

    #[test]
    fn test_no_proxy_domain() {
        let mut proxy = Proxy::http("http://proxy.example.com").unwrap();
        proxy.no_proxy(".example.com");
        assert!(proxy.no_proxy.is_some());
    }

    #[test]
    fn test_intercept_http() {
        let proxy = Proxy::http("http://proxy.example.com").unwrap();
        let uri = Uri::from_bytes(b"http://www.example.com").unwrap();
        assert!(proxy.is_intercepted(&uri));
    }

    #[test]
    fn test_intercept_https_rejected() {
        let proxy = Proxy::http("http://proxy.example.com").unwrap();
        let uri = Uri::from_bytes(b"https://www.example.com").unwrap();
        assert!(!proxy.is_intercepted(&uri));
    }

    #[test]
    fn test_intercept_https() {
        let proxy = Proxy::https("https://proxy.example.com").unwrap();
        let uri = Uri::from_bytes(b"https://www.example.com").unwrap();
        assert!(proxy.is_intercepted(&uri));
    }

    #[test]
    fn test_intercept_all() {
        let proxy = Proxy::all("http://proxy.example.com").unwrap();
        let http_uri = Uri::from_bytes(b"http://www.example.com").unwrap();
        let https_uri = Uri::from_bytes(b"https://www.example.com").unwrap();
        assert!(proxy.is_intercepted(&http_uri));
        assert!(proxy.is_intercepted(&https_uri));
    }

    #[test]
    fn test_via_proxy() {
        let proxy = Proxy::http("http://proxy.example.com:8080").unwrap();
        let uri = Uri::from_bytes(b"http://www.example.com/path?query=1").unwrap();
        let proxied = proxy.via_proxy(&uri);
        assert_eq!(
            proxied.to_string(),
            "http://proxy.example.com:8080/path?query=1"
        );
    }

    #[test]
    fn test_basic_auth() {
        let mut proxy = Proxy::http("http://proxy.example.com").unwrap();
        proxy.basic_auth("user", "pass");
        assert!(proxy.auth_header().is_some());
    }

    #[test]
    fn test_no_proxy_empty() {
        let no_proxy = NoProxy::from_str("");
        assert!(no_proxy.is_none());
    }

    #[test]
    fn test_no_proxy_wildcard() {
        let result = NoProxy::from_str("*.example.com");
        assert!(
            result.is_some(),
            "from_str should succeed for *.example.com"
        );
        let no_proxy = result.unwrap();
        assert!(no_proxy.contain("api.example.com"));
        assert!(no_proxy.contain("www.example.com"));
        assert!(!no_proxy.contain("example.com"));
    }

    #[test]
    fn test_no_proxy_ip() {
        let no_proxy = NoProxy::from_str("127.0.0.1").unwrap();
        assert!(no_proxy.contain("127.0.0.1"));
        assert!(!no_proxy.contain("127.0.0.2"));
    }

    #[test]
    fn test_no_proxy_domain_suffix() {
        let no_proxy = NoProxy::from_str(".local").unwrap();
        assert!(no_proxy.contain("host.local"));
    }

    #[test]
    fn test_proxy_clone() {
        let proxy1 = Proxy::http("http://proxy.example.com").unwrap();
        let proxy2 = proxy1.clone();
        assert_eq!(
            proxy1.intercept.proxy_info().scheme,
            proxy2.intercept.proxy_info().scheme
        );
    }

    #[test]
    fn test_intercept_enum_proxy_info() {
        let proxy = Proxy::http("http://proxy.example.com:8080").unwrap();
        let info = proxy.intercept.proxy_info();
        assert_eq!(info.authority().to_string(), "proxy.example.com:8080");
    }

    #[test]
    fn test_proxy_info_new_valid() {
        let info = ProxyInfo::new("http://proxy.example.com:8080").unwrap();
        assert_eq!(info.authority().host().as_str(), "proxy.example.com");
    }

    #[test]
    fn test_proxy_info_new_invalid() {
        let result = ProxyInfo::new("not a valid uri");
        assert!(result.is_err());
    }

    #[test]
    fn test_via_proxy_preserves_path_and_query() {
        let proxy = Proxy::https("https://proxy.example.com").unwrap();
        let uri = Uri::from_bytes(b"https://api.example.com/v1/users?id=123").unwrap();
        let proxied = proxy.via_proxy(&uri);
        let proxied_str = proxied.to_string();
        assert!(proxied_str.contains("/v1/users"));
        assert!(proxied_str.contains("id=123"));
    }

    #[test]
    fn test_no_proxy_with_spaces() {
        let no_proxy = NoProxy::from_str("example.com, example2.com").unwrap();
        assert!(no_proxy.contain("example.com"));
        assert!(no_proxy.contain("example2.com"));
    }

    #[test]
    fn test_no_proxy_case_sensitive() {
        let no_proxy = NoProxy::from_str("Example.com").unwrap();
        assert!(no_proxy.contain("Example.com"));
        assert!(!no_proxy.contain("example.com"));
    }
}
