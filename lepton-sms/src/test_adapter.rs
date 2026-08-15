//! In-memory SMS adapter for test assertions (captures envelopes).

use async_trait::async_trait;
use std::sync::Mutex;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::service::SmsDeliveryService;

/// [`SmsDeliveryService`] that records envelopes in memory for test asserts.
#[derive(Debug, Default)]
pub struct TestSmsAdapter {
    messages: Mutex<Vec<SmsEnvelope>>,
}

impl TestSmsAdapter {
    /// Create an empty test sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of recorded envelopes (clone) for assertions.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn recorded(&self) -> Vec<SmsEnvelope> {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Clear recorded envelopes.
    pub fn clear(&self) {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

#[async_trait]
impl SmsDeliveryService for TestSmsAdapter {
    fn driver_name(&self) -> &'static str {
        "test"
    }

    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        let result = async {
            validate_e164(&envelope.to_e164)?;
            self.messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(envelope.clone());
            tracing::info!(
                driver = "test",
                operation = "send",
                outcome = "success",
                reason_class = "recorded",
                "sms send"
            );
            Ok(SmsDeliveryReceipt {
                provider: "test".to_string(),
                message_id: None,
            })
        }
        .await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("test", result.is_ok());
        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sms_records_message_happy_path() {
        let adapter = TestSmsAdapter::new();
        adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "test-marker-otp".into(),
                otp_code: Some("123456".into()),
            })
            .await
            .expect("test send");
        let recorded = adapter.recorded();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].to_e164, "+15551234567");
        assert!(recorded[0].body.contains("test-marker-otp"));
    }
}
