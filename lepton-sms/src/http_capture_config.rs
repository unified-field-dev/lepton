//! Configuration for the HTTP capture SMS adapter (lab sink).

use crate::error::SmsDeliveryError;
use std::time::Duration;

/// Config for [`crate::HttpCaptureSmsAdapter`].
///
/// Points at a local SMS HTTP capture sink (default lab URL
/// `http://127.0.0.1:8099`).
#[derive(Clone, Debug)]
pub struct HttpCaptureSmsConfig {
    /// Sink base URL (no trailing path); adapter POSTs to `{base}/v1/messages`.
    pub base_url: String,
    /// HTTP timeout for send. Default: 5 seconds.
    pub timeout: Duration,
}

impl HttpCaptureSmsConfig {
    /// Build config from a base URL with the default timeout.
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`] when `base_url` is empty after trim.
    pub fn new(base_url: impl Into<String>) -> Result<Self, SmsDeliveryError> {
        let base_url = base_url.into().trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Err(SmsDeliveryError::config(
                "missing_base_url",
                "HttpCaptureSmsConfig base_url must be non-empty",
            ));
        }
        Ok(Self {
            base_url,
            timeout: Duration::from_secs(5),
        })
    }

    /// Override the HTTP timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Absolute URL for `POST /v1/messages`.
    #[must_use]
    pub fn messages_url(&self) -> String {
        format!("{}/v1/messages", self.base_url)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn http_capture_config_trims_slash_happy() {
        let cfg = HttpCaptureSmsConfig::new("http://127.0.0.1:8099/").expect("cfg");
        assert_eq!(cfg.messages_url(), "http://127.0.0.1:8099/v1/messages");
    }

    #[test]
    fn http_capture_config_empty_sad() {
        let err = HttpCaptureSmsConfig::new("  ").expect_err("empty");
        assert_eq!(err.reason_class(), Some("missing_base_url"));
    }
}
