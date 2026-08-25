//! SMS delivery trait and service builder.
//!
//! Hosts construct an [`SmsDeliveryService`] once with [`SmsServiceBuilder`], then call
//! [`SmsDeliveryService::send`] with an [`crate::SmsEnvelope`]. Prefer crate-root backend
//! guides for teaching paths; builder methods below are API reference.

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
///
/// # Contract
///
/// * `driver_name` reports a stable adapter label for tracing.
/// * `send` delivers `envelope` and returns a [`SmsDeliveryReceipt`] on success.
/// * Implementations must not log E.164, bodies, OTPs, or credentials in tracing fields.
#[async_trait]
pub trait SmsDeliveryService: Send + Sync {
    /// Stable adapter name for tracing (`noop`, `test`, `twilio`, …).
    fn driver_name(&self) -> &'static str;
    /// Send `envelope`, returning a [`SmsDeliveryReceipt`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError`] when configuration is invalid, the transport fails,
    /// the provider rejects the message, or a transient error occurs. Callers may use
    /// [`SmsDeliveryError::is_transient`] to decide on retry.
    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError>;
}

/// Builder for an [`SmsDeliveryService`].
///
/// Requires an explicit adapter mode (`noop`, `test`, `adapter`, `http_capture`, or `twilio`
/// when the `twilio` Cargo feature is enabled).
///
/// # Examples
///
/// Noop path (send and inspect the receipt):
///
/// ```no_run
/// use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder};
///
/// # async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
/// let sms = SmsServiceBuilder::new().noop().build()?;
/// let receipt = sms
///     .send(&SmsEnvelope {
///         to_e164: "+15551234567".into(),
///         body: "Your code is 123456".into(),
///         otp_code: Some("123456".into()),
///     })
///     .await?;
/// assert_eq!(receipt.provider, "noop");
/// # Ok(())
/// # }
/// ```
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

    /// Select the no-op adapter (local / CI). Does not contact a network.
    ///
    /// Still call [`build`](Self::build) after this. Teaching path: crate-root
    /// [Noop guide](crate#noop).
    #[must_use]
    pub fn noop(mut self) -> Self {
        self.clear_modes();
        self.use_noop = true;
        self
    }

    /// Select the in-memory test adapter (captures envelopes).
    ///
    /// Still call [`build`](Self::build) after this. Prefer [`adapter`](Self::adapter) with a
    /// shared [`TestSmsAdapter`] when tests need [`TestSmsAdapter::recorded`]. Teaching path:
    /// crate-root [Test guide](crate#test).
    #[must_use]
    pub fn test(mut self) -> Self {
        self.clear_modes();
        self.use_test = true;
        self
    }

    /// Inject a custom adapter (e.g. a shared [`TestSmsAdapter`] for asserts).
    ///
    /// Still call [`build`](Self::build) after this.
    #[must_use]
    pub fn adapter(mut self, adapter: Arc<dyn SmsDeliveryService>) -> Self {
        self.clear_modes();
        self.adapter = Some(adapter);
        self
    }

    /// Select the HTTP capture adapter (lab sink; default `:8099`).
    ///
    /// Still call [`build`](Self::build) after this. Teaching path: crate-root
    /// [HTTP capture guide](crate#http-capture).
    #[must_use]
    pub fn http_capture(mut self, cfg: HttpCaptureSmsConfig) -> Self {
        self.clear_modes();
        self.http_capture = Some(cfg);
        self
    }

    /// Select the live Twilio Messages REST adapter (`feature = "twilio"`).
    ///
    /// Still call [`build`](Self::build) after this. Teaching path: crate-root
    /// [Twilio Messages guide](crate#twilio-messages).
    #[cfg(feature = "twilio")]
    #[must_use]
    pub fn twilio(mut self, cfg: TwilioSmsConfig) -> Self {
        self.clear_modes();
        self.twilio = Some(cfg);
        self
    }

    /// Select the live Twilio Verify custom-code adapter (`feature = "twilio"`).
    ///
    /// Still call [`build`](Self::build) after this. Teaching path: crate-root
    /// [Twilio Verify guide](crate#twilio-verify).
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
