//! No-op SMS adapter for local development and CI.

use async_trait::async_trait;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::service::SmsDeliveryService;

/// [`SmsDeliveryService`] that accepts SMS without sending it.
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
