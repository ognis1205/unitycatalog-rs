use std::time::Duration;
use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder, NoProxy, Proxy};
use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::error::{Error, Result};

static DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

fn map_client_error(e: reqwest::Error) -> Error {
    Error::Generic {
        source: Box::new(e),
    }
}

/// Configuration keys for [`ClientOptions`]
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ClientConfigKey {
    /// Allow non-TLS, i.e. non-HTTPS connections
    AllowHttp,
    /// Skip certificate validation on https connections.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort or for testing
    AllowInvalidCertificates,
    /// Timeout for only the connect phase of a Client
    ConnectTimeout,
    /// default CONTENT_TYPE for uploads
    DefaultContentType,
    /// Only use http1 connections
    Http1Only,
    /// Interval for HTTP2 Ping frames should be sent to keep a connection alive.
    Http2KeepAliveInterval,
    /// Timeout for receiving an acknowledgement of the keep-alive ping.
    Http2KeepAliveTimeout,
    /// Enable HTTP2 keep alive pings for idle connections
    Http2KeepAliveWhileIdle,
    /// Sets the maximum frame size to use for HTTP2.
    Http2MaxFrameSize,
    /// Only use http2 connections
    Http2Only,
    /// The pool max idle timeout
    ///
    /// This is the length of time an idle connection will be kept alive
    PoolIdleTimeout,
    /// maximum number of idle connections per host
    PoolMaxIdlePerHost,
    /// HTTP proxy to use for requests
    ProxyUrl,
    /// PEM-formatted CA certificate for proxy connections
    ProxyCaCertificate,
    /// List of hosts that bypass proxy
    ProxyExcludes,
    /// Request timeout
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished
    Timeout,
    /// User-Agent header to be used by this client
    UserAgent,
}

impl AsRef<str> for ClientConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::AllowHttp => "allow_http",
            Self::AllowInvalidCertificates => "allow_invalid_certificates",
            Self::ConnectTimeout => "connect_timeout",
            Self::DefaultContentType => "default_content_type",
            Self::Http1Only => "http1_only",
            Self::Http2Only => "http2_only",
            Self::Http2KeepAliveInterval => "http2_keep_alive_interval",
            Self::Http2KeepAliveTimeout => "http2_keep_alive_timeout",
            Self::Http2KeepAliveWhileIdle => "http2_keep_alive_while_idle",
            Self::Http2MaxFrameSize => "http2_max_frame_size",
            Self::PoolIdleTimeout => "pool_idle_timeout",
            Self::PoolMaxIdlePerHost => "pool_max_idle_per_host",
            Self::ProxyUrl => "proxy_url",
            Self::ProxyCaCertificate => "proxy_ca_certificate",
            Self::ProxyExcludes => "proxy_excludes",
            Self::Timeout => "timeout",
            Self::UserAgent => "user_agent",
        }
    }
}

impl FromStr for ClientConfigKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allow_http" => Ok(Self::AllowHttp),
            "allow_invalid_certificates" => Ok(Self::AllowInvalidCertificates),
            "connect_timeout" => Ok(Self::ConnectTimeout),
            "default_content_type" => Ok(Self::DefaultContentType),
            "http1_only" => Ok(Self::Http1Only),
            "http2_only" => Ok(Self::Http2Only),
            "http2_keep_alive_interval" => Ok(Self::Http2KeepAliveInterval),
            "http2_keep_alive_timeout" => Ok(Self::Http2KeepAliveTimeout),
            "http2_keep_alive_while_idle" => Ok(Self::Http2KeepAliveWhileIdle),
            "http2_max_frame_size" => Ok(Self::Http2MaxFrameSize),
            "pool_idle_timeout" => Ok(Self::PoolIdleTimeout),
            "pool_max_idle_per_host" => Ok(Self::PoolMaxIdlePerHost),
            "proxy_url" => Ok(Self::ProxyUrl),
            "proxy_ca_certificate" => Ok(Self::ProxyCaCertificate),
            "proxy_excludes" => Ok(Self::ProxyExcludes),
            "timeout" => Ok(Self::Timeout),
            "user_agent" => Ok(Self::UserAgent),
            _ => Err(Error::UnknownConfigurationKey { key: s.into() }),
        }
    }
}

/// Represents a CA certificate provided by the user.
///
/// This is used to configure the client to trust a specific certificate. See
/// [Self::from_pem] for an example
#[derive(Debug, Clone)]
pub struct Certificate(reqwest::tls::Certificate);

impl Certificate {
    /// Create a `Certificate` from a PEM encoded certificate.
    ///
    /// # Example from a PEM file
    ///
    /// ```no_run
    /// # use cloud_client::Certificate;
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// let mut buf = Vec::new();
    /// File::open("my_cert.pem").unwrap()
    ///   .read_to_end(&mut buf).unwrap();
    /// let cert = Certificate::from_pem(&buf).unwrap();
    ///
    /// ```
    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        Ok(Self(
            reqwest::tls::Certificate::from_pem(pem).map_err(map_client_error)?,
        ))
    }

    /// Create a collection of `Certificate` from a PEM encoded certificate
    /// bundle.
    ///
    /// Files that contain such collections have extensions such as `.crt`,
    /// `.cer` and `.pem` files.
    pub fn from_pem_bundle(pem_bundle: &[u8]) -> Result<Vec<Self>> {
        Ok(reqwest::tls::Certificate::from_pem_bundle(pem_bundle)
            .map_err(map_client_error)?
            .into_iter()
            .map(Self)
            .collect())
    }

    /// Create a `Certificate` from a binary DER encoded certificate.
    pub fn from_der(der: &[u8]) -> Result<Self> {
        Ok(Self(
            reqwest::tls::Certificate::from_der(der).map_err(map_client_error)?,
        ))
    }
}

/// HTTP client configuration for remote object stores
#[derive(Debug, Clone)]
pub struct ClientOptions {
    user_agent: Option<ConfigValue<HeaderValue>>,
    root_certificates: Vec<Certificate>,
    content_type_map: HashMap<String, String>,
    default_content_type: Option<String>,
    default_headers: Option<HeaderMap>,
    proxy_url: Option<String>,
    proxy_ca_certificate: Option<String>,
    proxy_excludes: Option<String>,
    allow_http: ConfigValue<bool>,
    allow_insecure: ConfigValue<bool>,
    timeout: Option<ConfigValue<Duration>>,
    connect_timeout: Option<ConfigValue<Duration>>,
    pool_idle_timeout: Option<ConfigValue<Duration>>,
    pool_max_idle_per_host: Option<ConfigValue<usize>>,
    http2_keep_alive_interval: Option<ConfigValue<Duration>>,
    http2_keep_alive_timeout: Option<ConfigValue<Duration>>,
    http2_keep_alive_while_idle: ConfigValue<bool>,
    http2_max_frame_size: Option<ConfigValue<u32>>,
    http1_only: ConfigValue<bool>,
    http2_only: ConfigValue<bool>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        // Defaults based on
        // <https://docs.aws.amazon.com/sdkref/latest/guide/feature-smart-config-defaults.html>
        // <https://docs.aws.amazon.com/whitepapers/latest/s3-optimizing-performance-best-practices/timeouts-and-retries-for-latency-sensitive-applications.html>
        // Which recommend a connection timeout of 3.1s and a request timeout of 2s
        //
        // As object store requests may involve the transfer of non-trivial volumes of data
        // we opt for a slightly higher default timeout of 30 seconds
        Self {
            user_agent: None,
            root_certificates: Default::default(),
            content_type_map: Default::default(),
            default_content_type: None,
            default_headers: None,
            proxy_url: None,
            proxy_ca_certificate: None,
            proxy_excludes: None,
            allow_http: Default::default(),
            allow_insecure: Default::default(),
            timeout: Some(Duration::from_secs(30).into()),
            connect_timeout: Some(Duration::from_secs(5).into()),
            pool_idle_timeout: None,
            pool_max_idle_per_host: None,
            http2_keep_alive_interval: None,
            http2_keep_alive_timeout: None,
            http2_keep_alive_while_idle: Default::default(),
            http2_max_frame_size: None,
            // HTTP2 is known to be significantly slower than HTTP1, so we default
            // to HTTP1 for now.
            // https://github.com/apache/arrow-rs/issues/5194
            http1_only: true.into(),
            http2_only: Default::default(),
        }
    }
}

impl ClientOptions {
    /// Create a new [`ClientOptions`] with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Set an option by key
    pub fn with_config(mut self, key: ClientConfigKey, value: impl Into<String>) -> Self {
        match key {
            ClientConfigKey::AllowHttp => self.allow_http.parse(value),
            ClientConfigKey::AllowInvalidCertificates => self.allow_insecure.parse(value),
            ClientConfigKey::ConnectTimeout => {
                self.connect_timeout = Some(ConfigValue::Deferred(value.into()))
            }
            ClientConfigKey::DefaultContentType => self.default_content_type = Some(value.into()),
            ClientConfigKey::Http1Only => self.http1_only.parse(value),
            ClientConfigKey::Http2Only => self.http2_only.parse(value),
            ClientConfigKey::Http2KeepAliveInterval => {
                self.http2_keep_alive_interval = Some(ConfigValue::Deferred(value.into()))
            }
            ClientConfigKey::Http2KeepAliveTimeout => {
                self.http2_keep_alive_timeout = Some(ConfigValue::Deferred(value.into()))
            }
            ClientConfigKey::Http2KeepAliveWhileIdle => {
                self.http2_keep_alive_while_idle.parse(value)
            }
            ClientConfigKey::Http2MaxFrameSize => {
                self.http2_max_frame_size = Some(ConfigValue::Deferred(value.into()))
            }
            ClientConfigKey::PoolIdleTimeout => {
                self.pool_idle_timeout = Some(ConfigValue::Deferred(value.into()))
            }
            ClientConfigKey::PoolMaxIdlePerHost => {
                self.pool_max_idle_per_host = Some(ConfigValue::Deferred(value.into()))
            }
            ClientConfigKey::ProxyUrl => self.proxy_url = Some(value.into()),
            ClientConfigKey::ProxyCaCertificate => self.proxy_ca_certificate = Some(value.into()),
            ClientConfigKey::ProxyExcludes => self.proxy_excludes = Some(value.into()),
            ClientConfigKey::Timeout => self.timeout = Some(ConfigValue::Deferred(value.into())),
            ClientConfigKey::UserAgent => {
                self.user_agent = Some(ConfigValue::Deferred(value.into()))
            }
        }
        self
    }

    /// Get an option by key
    pub fn get_config_value(&self, key: &ClientConfigKey) -> Option<String> {
        match key {
            ClientConfigKey::AllowHttp => Some(self.allow_http.to_string()),
            ClientConfigKey::AllowInvalidCertificates => Some(self.allow_insecure.to_string()),
            ClientConfigKey::ConnectTimeout => self.connect_timeout.as_ref().map(fmt_duration),
            ClientConfigKey::DefaultContentType => self.default_content_type.clone(),
            ClientConfigKey::Http1Only => Some(self.http1_only.to_string()),
            ClientConfigKey::Http2KeepAliveInterval => {
                self.http2_keep_alive_interval.as_ref().map(fmt_duration)
            }
            ClientConfigKey::Http2KeepAliveTimeout => {
                self.http2_keep_alive_timeout.as_ref().map(fmt_duration)
            }
            ClientConfigKey::Http2KeepAliveWhileIdle => {
                Some(self.http2_keep_alive_while_idle.to_string())
            }
            ClientConfigKey::Http2MaxFrameSize => {
                self.http2_max_frame_size.as_ref().map(|v| v.to_string())
            }
            ClientConfigKey::Http2Only => Some(self.http2_only.to_string()),
            ClientConfigKey::PoolIdleTimeout => self.pool_idle_timeout.as_ref().map(fmt_duration),
            ClientConfigKey::PoolMaxIdlePerHost => {
                self.pool_max_idle_per_host.as_ref().map(|v| v.to_string())
            }
            ClientConfigKey::ProxyUrl => self.proxy_url.clone(),
            ClientConfigKey::ProxyCaCertificate => self.proxy_ca_certificate.clone(),
            ClientConfigKey::ProxyExcludes => self.proxy_excludes.clone(),
            ClientConfigKey::Timeout => self.timeout.as_ref().map(fmt_duration),
            ClientConfigKey::UserAgent => self
                .user_agent
                .as_ref()
                .and_then(|v| v.get().ok())
                .and_then(|v| v.to_str().ok().map(|s| s.to_string())),
        }
    }

    /// Sets the User-Agent header to be used by this client
    ///
    /// Default is based on the version of this crate
    pub fn with_user_agent(mut self, agent: HeaderValue) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    /// Add a custom root certificate.
    ///
    /// This can be used to connect to a server that has a self-signed
    /// certificate for example.
    pub fn with_root_certificate(mut self, certificate: Certificate) -> Self {
        self.root_certificates.push(certificate);
        self
    }

    /// Set the default CONTENT_TYPE for uploads
    pub fn with_default_content_type(mut self, mime: impl Into<String>) -> Self {
        self.default_content_type = Some(mime.into());
        self
    }

    /// Set the CONTENT_TYPE for a given file extension
    pub fn with_content_type_for_suffix(
        mut self,
        extension: impl Into<String>,
        mime: impl Into<String>,
    ) -> Self {
        self.content_type_map.insert(extension.into(), mime.into());
        self
    }

    /// Sets the default headers for every request
    pub fn with_default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = Some(headers);
        self
    }

    /// Sets what protocol is allowed. If `allow_http` is :
    /// * false (default):  Only HTTPS are allowed
    /// * true:  HTTP and HTTPS are allowed
    pub fn with_allow_http(mut self, allow_http: bool) -> Self {
        self.allow_http = allow_http.into();
        self
    }
    /// Allows connections to invalid SSL certificates
    /// * false (default):  Only valid HTTPS certificates are allowed
    /// * true:  All HTTPS certificates are allowed
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort or for testing
    pub fn with_allow_invalid_certificates(mut self, allow_insecure: bool) -> Self {
        self.allow_insecure = allow_insecure.into();
        self
    }

    /// Only use http1 connections
    ///
    /// This is on by default, since http2 is known to be significantly slower than http1.
    pub fn with_http1_only(mut self) -> Self {
        self.http2_only = false.into();
        self.http1_only = true.into();
        self
    }

    /// Only use http2 connections
    pub fn with_http2_only(mut self) -> Self {
        self.http1_only = false.into();
        self.http2_only = true.into();
        self
    }

    /// Use http2 if supported, otherwise use http1.
    pub fn with_allow_http2(mut self) -> Self {
        self.http1_only = false.into();
        self.http2_only = false.into();
        self
    }

    /// Set a proxy URL to use for requests
    pub fn with_proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Set a trusted proxy CA certificate
    pub fn with_proxy_ca_certificate(mut self, proxy_ca_certificate: impl Into<String>) -> Self {
        self.proxy_ca_certificate = Some(proxy_ca_certificate.into());
        self
    }

    /// Set a list of hosts to exclude from proxy connections
    pub fn with_proxy_excludes(mut self, proxy_excludes: impl Into<String>) -> Self {
        self.proxy_excludes = Some(proxy_excludes.into());
        self
    }

    /// Set a request timeout
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished
    ///
    /// Default is 30 seconds
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(ConfigValue::Parsed(timeout));
        self
    }

    /// Disables the request timeout
    ///
    /// See [`Self::with_timeout`]
    pub fn with_timeout_disabled(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Set a timeout for only the connect phase of a Client
    ///
    /// Default is 5 seconds
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(ConfigValue::Parsed(timeout));
        self
    }

    /// Disables the connection timeout
    ///
    /// See [`Self::with_connect_timeout`]
    pub fn with_connect_timeout_disabled(mut self) -> Self {
        self.connect_timeout = None;
        self
    }

    /// Set the pool max idle timeout
    ///
    /// This is the length of time an idle connection will be kept alive
    ///
    /// Default is 90 seconds enforced by reqwest
    pub fn with_pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = Some(ConfigValue::Parsed(timeout));
        self
    }

    /// Set the maximum number of idle connections per host
    ///
    /// Default is no limit enforced by reqwest
    pub fn with_pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = Some(max.into());
        self
    }

    /// Sets an interval for HTTP2 Ping frames should be sent to keep a connection alive.
    ///
    /// Default is disabled enforced by reqwest
    pub fn with_http2_keep_alive_interval(mut self, interval: Duration) -> Self {
        self.http2_keep_alive_interval = Some(ConfigValue::Parsed(interval));
        self
    }

    /// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will be closed.
    /// Does nothing if http2_keep_alive_interval is disabled.
    ///
    /// Default is disabled enforced by reqwest
    pub fn with_http2_keep_alive_timeout(mut self, interval: Duration) -> Self {
        self.http2_keep_alive_timeout = Some(ConfigValue::Parsed(interval));
        self
    }

    /// Enable HTTP2 keep alive pings for idle connections
    ///
    /// If disabled, keep-alive pings are only sent while there are open request/response
    /// streams. If enabled, pings are also sent when no streams are active
    ///
    /// Default is disabled enforced by reqwest
    pub fn with_http2_keep_alive_while_idle(mut self) -> Self {
        self.http2_keep_alive_while_idle = true.into();
        self
    }

    /// Sets the maximum frame size to use for HTTP2.
    ///
    /// Default is currently 16,384 but may change internally to optimize for common uses.
    pub fn with_http2_max_frame_size(mut self, sz: u32) -> Self {
        self.http2_max_frame_size = Some(ConfigValue::Parsed(sz));
        self
    }

    /// Create a [`Client`] with overrides optimised for metadata endpoint access
    ///
    /// In particular:
    /// * Allows HTTP as metadata endpoints do not use TLS
    /// * Configures a low connection timeout to provide quick feedback if not present
    pub(crate) fn metadata_client(&self) -> Result<Client> {
        self.clone()
            .with_allow_http(true)
            .with_connect_timeout(Duration::from_secs(1))
            .client()
    }

    pub(crate) fn client(&self) -> Result<Client> {
        let mut builder = ClientBuilder::new();

        match &self.user_agent {
            Some(user_agent) => builder = builder.user_agent(user_agent.get()?),
            None => builder = builder.user_agent(DEFAULT_USER_AGENT),
        }

        if let Some(headers) = &self.default_headers {
            builder = builder.default_headers(headers.clone())
        }

        if let Some(proxy) = &self.proxy_url {
            let mut proxy = Proxy::all(proxy).map_err(map_client_error)?;

            if let Some(certificate) = &self.proxy_ca_certificate {
                let certificate = reqwest::tls::Certificate::from_pem(certificate.as_bytes())
                    .map_err(map_client_error)?;

                builder = builder.add_root_certificate(certificate);
            }

            if let Some(proxy_excludes) = &self.proxy_excludes {
                let no_proxy = NoProxy::from_string(proxy_excludes);

                proxy = proxy.no_proxy(no_proxy);
            }

            builder = builder.proxy(proxy);
        }

        for certificate in &self.root_certificates {
            builder = builder.add_root_certificate(certificate.0.clone());
        }

        if let Some(timeout) = &self.timeout {
            builder = builder.timeout(timeout.get()?)
        }

        if let Some(timeout) = &self.connect_timeout {
            builder = builder.connect_timeout(timeout.get()?)
        }

        if let Some(timeout) = &self.pool_idle_timeout {
            builder = builder.pool_idle_timeout(timeout.get()?)
        }

        if let Some(max) = &self.pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(max.get()?)
        }

        if let Some(interval) = &self.http2_keep_alive_interval {
            builder = builder.http2_keep_alive_interval(interval.get()?)
        }

        if let Some(interval) = &self.http2_keep_alive_timeout {
            builder = builder.http2_keep_alive_timeout(interval.get()?)
        }

        if self.http2_keep_alive_while_idle.get()? {
            builder = builder.http2_keep_alive_while_idle(true)
        }

        if let Some(sz) = &self.http2_max_frame_size {
            builder = builder.http2_max_frame_size(Some(sz.get()?))
        }

        if self.http1_only.get()? {
            builder = builder.http1_only()
        }

        if self.http2_only.get()? {
            builder = builder.http2_prior_knowledge()
        }

        if self.allow_insecure.get()? {
            builder = builder.danger_accept_invalid_certs(true)
        }

        // Reqwest will remove the `Content-Length` header if it is configured to
        // transparently decompress the body via the non-default `gzip` feature.
        builder = builder.no_gzip();

        builder
            .https_only(!self.allow_http.get()?)
            .build()
            .map_err(map_client_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn client_test_config_from_map() {
        let allow_http = "true".to_string();
        let allow_invalid_certificates = "false".to_string();
        let connect_timeout = "90 seconds".to_string();
        let default_content_type = "object_store:fake_default_content_type".to_string();
        let http1_only = "true".to_string();
        let http2_only = "false".to_string();
        let http2_keep_alive_interval = "90 seconds".to_string();
        let http2_keep_alive_timeout = "91 seconds".to_string();
        let http2_keep_alive_while_idle = "92 seconds".to_string();
        let http2_max_frame_size = "1337".to_string();
        let pool_idle_timeout = "93 seconds".to_string();
        let pool_max_idle_per_host = "94".to_string();
        let proxy_url = "https://fake_proxy_url".to_string();
        let timeout = "95 seconds".to_string();
        let user_agent = "object_store:fake_user_agent".to_string();

        let options = HashMap::from([
            ("allow_http", allow_http.clone()),
            (
                "allow_invalid_certificates",
                allow_invalid_certificates.clone(),
            ),
            ("connect_timeout", connect_timeout.clone()),
            ("default_content_type", default_content_type.clone()),
            ("http1_only", http1_only.clone()),
            ("http2_only", http2_only.clone()),
            (
                "http2_keep_alive_interval",
                http2_keep_alive_interval.clone(),
            ),
            ("http2_keep_alive_timeout", http2_keep_alive_timeout.clone()),
            (
                "http2_keep_alive_while_idle",
                http2_keep_alive_while_idle.clone(),
            ),
            ("http2_max_frame_size", http2_max_frame_size.clone()),
            ("pool_idle_timeout", pool_idle_timeout.clone()),
            ("pool_max_idle_per_host", pool_max_idle_per_host.clone()),
            ("proxy_url", proxy_url.clone()),
            ("timeout", timeout.clone()),
            ("user_agent", user_agent.clone()),
        ]);

        let builder = options
            .into_iter()
            .fold(ClientOptions::new(), |builder, (key, value)| {
                builder.with_config(key.parse().unwrap(), value)
            });

        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::AllowHttp)
                .unwrap(),
            allow_http
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::AllowInvalidCertificates)
                .unwrap(),
            allow_invalid_certificates
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::ConnectTimeout)
                .unwrap(),
            connect_timeout
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::DefaultContentType)
                .unwrap(),
            default_content_type
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::Http1Only)
                .unwrap(),
            http1_only
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::Http2Only)
                .unwrap(),
            http2_only
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::Http2KeepAliveInterval)
                .unwrap(),
            http2_keep_alive_interval
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::Http2KeepAliveTimeout)
                .unwrap(),
            http2_keep_alive_timeout
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::Http2KeepAliveWhileIdle)
                .unwrap(),
            http2_keep_alive_while_idle
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::Http2MaxFrameSize)
                .unwrap(),
            http2_max_frame_size
        );

        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::PoolIdleTimeout)
                .unwrap(),
            pool_idle_timeout
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::PoolMaxIdlePerHost)
                .unwrap(),
            pool_max_idle_per_host
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::ProxyUrl)
                .unwrap(),
            proxy_url
        );
        assert_eq!(
            builder.get_config_value(&ClientConfigKey::Timeout).unwrap(),
            timeout
        );
        assert_eq!(
            builder
                .get_config_value(&ClientConfigKey::UserAgent)
                .unwrap(),
            user_agent
        );
    }
}
