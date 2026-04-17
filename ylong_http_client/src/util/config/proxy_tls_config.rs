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

//! Proxy TLS configuration for HTTPS proxies.

#[cfg(feature = "__tls")]
use crate::util::TlsVersion;

#[cfg(feature = "__tls")]
use super::Certificate;

#[cfg(feature = "__tls")]
use crate::util::TlsFileType;

#[cfg(feature = "__tls")]
use crate::util::c_openssl::adapter::TlsConfigBuilder;

#[cfg(feature = "__tls")]
#[derive(Clone, Default)]
pub(crate) struct ProxyTlsConfig {
    pub ca: Option<Certificate>,
    pub skip_verify: bool,
    pub identity: Option<Identity>,
    pub cipher_list: Option<String>,
    pub min_version: Option<TlsVersion>,
    pub max_version: Option<TlsVersion>,
}

#[cfg(feature = "__tls")]
impl ProxyTlsConfig {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn apply_to_builder(
        &self,
        builder: TlsConfigBuilder,
    ) -> Result<TlsConfigBuilder, crate::HttpClientError> {
        let mut builder = builder;

        if self.skip_verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(ref ca) = self.ca {
            builder = ca.clone().apply_to_builder(builder);
        }

        if let Some(ref cipher_list) = self.cipher_list {
            builder = builder.cipher_list(cipher_list);
        }

        if let Some(ref min_version) = self.min_version {
            builder = builder.min_proto_version(*min_version);
        }

        if let Some(ref max_version) = self.max_version {
            builder = builder.max_proto_version(*max_version);
        }

        if let Some(ref identity) = self.identity {
            builder = identity.apply_to_builder(builder);
        }

        Ok(builder)
    }
}

#[cfg(feature = "__tls")]
#[derive(Clone)]
pub(crate) struct PrivateKey {
    path: String,
    file_type: TlsFileType,
}

#[cfg(feature = "__tls")]
impl Default for PrivateKey {
    fn default() -> Self {
        Self {
            path: String::new(),
            file_type: TlsFileType::default(),
        }
    }
}

#[cfg(feature = "__tls")]
impl PrivateKey {
    pub(crate) fn from_path(path: &str, file_type: TlsFileType) -> Self {
        Self {
            path: path.to_string(),
            file_type,
        }
    }

    pub(crate) fn apply_to_builder(&self, builder: TlsConfigBuilder) -> TlsConfigBuilder {
        builder.private_key_file(&self.path, self.file_type.clone())
    }
}

#[cfg(feature = "__tls")]
#[derive(Clone, Default)]
pub(crate) struct Identity {
    cert: Certificate,
    key: PrivateKey,
}

#[cfg(feature = "__tls")]
impl Identity {
    pub(crate) fn new(cert: Certificate, key: PrivateKey) -> Self {
        Self { cert, key }
    }

    pub(crate) fn apply_to_builder(&self, builder: TlsConfigBuilder) -> TlsConfigBuilder {
        let builder = self.cert.clone().apply_to_builder(builder);
        self.key.apply_to_builder(builder)
    }
}

#[cfg(test)]
mod ut_proxy_tls_config {
    #[cfg(feature = "__tls")]
    use crate::util::config::ProxyTlsConfig;

    #[cfg(feature = "__tls")]
    #[test]
    fn ut_proxy_tls_config_default() {
        let config = ProxyTlsConfig::new();
        assert!(config.ca.is_none());
        assert!(!config.skip_verify);
        assert!(config.identity.is_none());
        assert!(config.cipher_list.is_none());
        assert!(config.min_version.is_none());
        assert!(config.max_version.is_none());
    }
}
