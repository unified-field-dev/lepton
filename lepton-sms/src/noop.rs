//! No-op SMS adapter for local development and CI.
//!
//! [`NoopSmsAdapter`] validates E.164 and returns a [`crate::SmsDeliveryReceipt`] with
//! `provider = "noop"`. Build it with [`crate::SmsServiceBuilder::noop`], then
//! [`crate::SmsServiceBuilder::build`].
//!
//! Teaching path: crate-root [Noop guide](crate#noop). Runnable example:
//! `cargo run -p lepton-sms --example noop_send`.
//!
//! # Examples
//!
//! ```no_run
//! use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder};
//!
//! # async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
//! let sms = SmsServiceBuilder::new().noop().build()?;
//! let receipt = sms
//!     .send(&SmsEnvelope {
//!         to_e164: "+15551234567".into(),
//!         body: "Your code is 123456".into(),
//!         otp_code: Some("123456".into()),
//!     })
//!     .await?;
//! assert_eq!(receipt.provider, "noop");
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::service::SmsDeliveryService;

/// [`SmsDeliveryService`] that accepts SMS without sending it (local / CI).
///
/// On success, [`SmsDeliveryReceipt::provider`] is `"noop"`. Invalid E.164 returns
/// [`SmsDeliveryError::ConfigError`]. See the crate-root [Noop guide](crate#noop).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopSmsAdapter;

#[async_trait]
impl SmsDeliveryService for NoopSmsAdapter {
    fn driver_name(&self) -> &'static str {
        "noop"
    }

    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        let result = async {
            validate_e164(&envelope.to_e164)?;
            tracing::info!(
                driver = "noop",
                operation = "send",
                outcome = "success",
                reason_class = "noop",
                "sms send"
            );
            Ok(SmsDeliveryReceipt {
                provider: "noop".to_string(),
                message_id: None,
            })
        }
        .await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("noop", result.is_ok());
        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_sms_send_happy_path() {
        let adapter = NoopSmsAdapter;
        let receipt = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "code 123456".into(),
                otp_code: Some("123456".into()),
            })
            .await
            .expect("noop send");
        assert_eq!(receipt.provider, "noop");
        assert_eq!(adapter.driver_name(), "noop");
    }

    #[tokio::test]
    async fn sms_invalid_e164_sad() {
        let adapter = NoopSmsAdapter;
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: String::new(),
                body: "x".into(),
                otp_code: None,
            })
            .await
            .expect_err("empty e164");
        assert!(matches!(err, SmsDeliveryError::ConfigError(_)));
        assert!(err.to_string().contains("reason_class=invalid_e164"));

        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "15551234567".into(),
                body: "x".into(),
                otp_code: None,
            })
            .await
            .expect_err("missing plus");
        assert!(err.to_string().contains("reason_class=invalid_e164"));
    }
}
