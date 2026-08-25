//! In-memory SMS adapter for test assertions (captures envelopes).
//!
//! Prefer a shared [`TestSmsAdapter`] via [`crate::SmsServiceBuilder::adapter`] when tests
//! need [`TestSmsAdapter::recorded`]. Teaching path: crate-root [Test guide](crate#test).
//!
//! # Examples
//!
//! ```no_run
//! use std::sync::Arc;
//! use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder, TestSmsAdapter};
//!
//! # async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
//! let sink = Arc::new(TestSmsAdapter::new());
//! let sms = SmsServiceBuilder::new().adapter(sink.clone()).build()?;
//! let receipt = sms
//!     .send(&SmsEnvelope {
//!         to_e164: "+15551234567".into(),
//!         body: "hello".into(),
//!         otp_code: None,
//!     })
//!     .await?;
//! assert_eq!(receipt.provider, "test");
//! assert_eq!(sink.recorded().len(), 1);
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use std::sync::Mutex;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::service::SmsDeliveryService;

/// [`SmsDeliveryService`] that records envelopes in memory for test asserts.
///
/// On success, [`SmsDeliveryReceipt::provider`] is `"test"`. See the crate-root
/// [Test guide](crate#test).
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
    async fn test_adapter_records_envelopes() {
        let adapter = TestSmsAdapter::new();
        adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "a".into(),
                otp_code: None,
            })
            .await
            .expect("send");
        assert_eq!(adapter.recorded().len(), 1);
        adapter.clear();
        assert!(adapter.recorded().is_empty());
    }
}
