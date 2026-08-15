//! SMS delivery trait and service builder.

use async_trait::async_trait;
use std::sync::Arc;

use crate::envelope::{SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::http_capture::HttpCaptureSmsAdapter;
use crate::http_capture_config::HttpCaptureSmsConfig;
use crate::noop::NoopSmsAdapter;
use crate::test_adapter::TestSmsAdapter;

#[cfg(feature = "twilio")]
use crate::twilio::{TwilioSmsAdapter, TwilioVerifySmsAdapter};
#[cfg(feature = "twilio")]
use crate::twilio_config::TwilioSmsConfig;
#[cfg(feature = "twilio")]
use crate::twilio_verify_config::TwilioVerifyConfig;

/// Sends [`SmsEnvelope`]s via a specific adapter ([`NoopSmsAdapter`], [`TestSmsAdapter`], …).
#[async_trait]
pub trait SmsDeliveryService: Send + Sync {
    /// Stable adapter name for tracing (`noop`, `test`, `twilio`, …).
    fn driver_name(&self) -> &'static str;
    /// Send `envelope`, returning a [`SmsDeliveryReceipt`] on success.
    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError>;
}

/// Builder for an [`SmsDeliveryService`].
///
/// Requires an explicit adapter mode (`noop`, `test`, `adapter`, or `twilio` when the
/// `twilio` Cargo feature is enabled).
#[derive(Clone, Default)]
pub struct SmsServiceBuilder {
    adapter: Option<Arc<dyn SmsDeliveryService>>,
    use_test: bool,
    use_noop: bool,
    http_capture: Option<HttpCaptureSmsConfig>,
    #[cfg(feature = "twilio")]
    twilio: Option<TwilioSmsConfig>,
    #[cfg(feature = "twilio")]
    twilio_verify: Option<TwilioVerifyConfig>,
}

impl SmsServiceBuilder {
    /// Empty builder (must select an adapter mode before [`Self::build`]).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn clear_modes(&mut self) {
        self.adapter = None;
        self.use_test = false;
        self.use_noop = false;
        self.http_capture = None;
        #[cfg(feature = "twilio")]
        {
            self.twilio = None;
            self.twilio_verify = None;
        }
    }

    /// Use the no-op adapter.
    #[must_use]
    pub fn noop(mut self) -> Self {
        self.clear_modes();
        self.use_noop = true;
        self
    }

    /// Use the in-memory test adapter (captures envelopes).
    #[must_use]
    pub fn test(mut self) -> Self {
        self.clear_modes();
        self.use_test = true;
        self
    }

    /// Inject a custom adapter (e.g. a shared [`TestSmsAdapter`] for asserts).
    #[must_use]
    pub fn adapter(mut self, adapter: Arc<dyn SmsDeliveryService>) -> Self {
        self.clear_modes();
        self.adapter = Some(adapter);
        self
    }

    /// POST envelopes to an HTTP capture sink (lab; default `:8099`).
    #[must_use]
    pub fn http_capture(mut self, cfg: HttpCaptureSmsConfig) -> Self {
        self.clear_modes();
        self.http_capture = Some(cfg);
        self
    }

    /// Use the live Twilio Messages REST adapter (`feature = "twilio"`).
    #[cfg(feature = "twilio")]
    #[must_use]
    pub fn twilio(mut self, cfg: TwilioSmsConfig) -> Self {
        self.clear_modes();
        self.twilio = Some(cfg);
        self
    }

    /// Use the live Twilio Verify custom-code adapter (`feature = "twilio"`).
    #[cfg(feature = "twilio")]
    #[must_use]
    pub fn twilio_verify(mut self, cfg: TwilioVerifyConfig) -> Self {
        self.clear_modes();
        self.twilio_verify = Some(cfg);
        self
    }

    /// Build an [`Arc`] SMS service.
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`] when no adapter mode was selected
    /// (call [`Self::noop`], [`Self::test`], [`Self::adapter`], [`Self::http_capture`],
    /// or — with Cargo feature `twilio` — `Self::twilio` / `Self::twilio_verify` first).
    pub fn build(self) -> Result<Arc<dyn SmsDeliveryService>, SmsDeliveryError> {
        if let Some(adapter) = self.adapter {
            return Ok(adapter);
        }
        if let Some(cfg) = self.http_capture {
            return Ok(Arc::new(HttpCaptureSmsAdapter::new(cfg)?));
        }
        #[cfg(feature = "twilio")]
        if let Some(cfg) = self.twilio_verify {
            return Ok(Arc::new(TwilioVerifySmsAdapter::new(cfg)?));
        }
        #[cfg(feature = "twilio")]
        if let Some(cfg) = self.twilio {
            return Ok(Arc::new(TwilioSmsAdapter::new(cfg)?));
        }
        if self.use_test {
            return Ok(Arc::new(TestSmsAdapter::new()));
        }
        if self.use_noop {
            return Ok(Arc::new(NoopSmsAdapter));
        }
        Err(SmsDeliveryError::config(
            "missing_sms_adapter",
            #[cfg(feature = "twilio")]
            "SmsServiceBuilder requires noop(), test(), adapter(), http_capture(), twilio(), or twilio_verify()",
            #[cfg(not(feature = "twilio"))]
            "SmsServiceBuilder requires noop(), test(), adapter(), or http_capture()",
        ))
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    clippy::manual_let_else,
    clippy::match_wild_err_arm,
    clippy::option_if_let_else
)]
mod tests {
    use super::*;

    #[test]
    fn sms_builder_requires_explicit_adapter_sad() {
        let Err(err) = SmsServiceBuilder::new().build() else {
            panic!("must fail without adapter mode");
        };
        assert!(matches!(err, SmsDeliveryError::ConfigError(_)));
        assert!(err.to_string().contains("missing_sms_adapter"));
    }

    #[test]
    fn sms_builder_noop_happy_path() {
        let svc = SmsServiceBuilder::new().noop().build().expect("noop build");
        assert_eq!(svc.driver_name(), "noop");
    }

    #[test]
    fn sms_builder_test_happy_path() {
        let svc = SmsServiceBuilder::new().test().build().expect("test build");
        assert_eq!(svc.driver_name(), "test");
    }

    #[test]
    fn sms_builder_http_capture_happy_path() {
        let cfg = HttpCaptureSmsConfig::new("http://127.0.0.1:8099").expect("cfg");
        let svc = SmsServiceBuilder::new()
            .http_capture(cfg)
            .build()
            .expect("http_capture build");
        assert_eq!(svc.driver_name(), "http_capture");
    }

    #[cfg(feature = "twilio")]
    #[test]
    fn sms_builder_twilio_happy_path() {
        let cfg = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .auth_token("token")
            .from("+15551234567")
            .build()
            .expect("cfg");
        let svc = SmsServiceBuilder::new()
            .twilio(cfg)
            .build()
            .expect("twilio build");
        assert_eq!(svc.driver_name(), "twilio");
    }

    #[cfg(feature = "twilio")]
    #[test]
    fn sms_builder_twilio_verify_happy_path() {
        let cfg = TwilioVerifyConfig::builder()
            .service_sid("VAxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_key("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_secret("secret")
            .build()
            .expect("cfg");
        let svc = SmsServiceBuilder::new()
            .twilio_verify(cfg)
            .build()
            .expect("verify build");
        assert_eq!(svc.driver_name(), "twilio_verify");
    }
}
