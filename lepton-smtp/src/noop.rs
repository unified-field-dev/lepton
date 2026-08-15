//! No-op email adapter for local development and CI.

use async_trait::async_trait;

use crate::driver::EmailDriver;
use crate::envelope::{DeliveryReceipt, EmailEnvelope};
use crate::error::EmailDeliveryError;
use crate::service::EmailDeliveryService;

/// [`EmailDeliveryService`] that accepts mail without sending it (local dev / CI).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopEmailAdapter;

#[async_trait]
impl EmailDeliveryService for NoopEmailAdapter {
    fn driver(&self) -> EmailDriver {
        EmailDriver::Noop
    }

    async fn send(&self, _envelope: &EmailEnvelope) -> Result<DeliveryReceipt, EmailDeliveryError> {
        tracing::info!(
            driver = "noop",
            operation = "send",
            outcome = "success",
            reason_class = "noop",
            "email send"
        );
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("noop", true);
        Ok(DeliveryReceipt {
            provider: "noop".to_string(),
            message_id: None,
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::use_self)]
mod tests {
    use super::*;
    use crate::envelope::EmailEnvelope;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct Buf(Arc<Mutex<Vec<u8>>>);

    impl Write for Buf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?
                .extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Buf {
        type Writer = Self;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[tokio::test]
    async fn tracing_send_field_allowlist_happy_path() {
        let buf = Buf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buf.clone())
            .with_ansi(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        let envelope = EmailEnvelope {
            to: "secret.user@example.test".to_string(),
            subject: "hi".into(),
            text_body: "body-secret".into(),
            html_body: "<p>body-secret</p>".into(),
        };
        NoopEmailAdapter.send(&envelope).await.expect("noop send");

        let bytes = buf.0.lock().expect("lock").clone();
        let log = String::from_utf8_lossy(&bytes);
        assert!(
            !log.contains("secret.user@example.test"),
            "recipient must not appear in tracing: {log}"
        );
        assert!(
            !log.contains("body-secret"),
            "body must not appear in tracing: {log}"
        );
        assert!(log.contains("noop") || log.contains("email send"));
    }
}
