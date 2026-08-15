//! SMTP relay [`EmailDeliveryService`] adapter.

use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use std::time::Instant;

use crate::driver::EmailDriver;
use crate::envelope::{DeliveryReceipt, EmailEnvelope};
use crate::error::EmailDeliveryError;
use crate::message::build_message;
use crate::service::EmailDeliveryService;
use crate::smtp::config::SmtpConfig;

/// [`EmailDeliveryService`] that relays mail through a configured SMTP server.
#[derive(Clone, Debug)]
pub struct SmtpAdapter {
    cfg: SmtpConfig,
}

impl SmtpAdapter {
    /// Construct from [`SmtpConfig`].
    #[must_use]
    pub const fn new(cfg: SmtpConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl EmailDeliveryService for SmtpAdapter {
    fn driver(&self) -> EmailDriver {
        EmailDriver::Smtp
    }

    async fn send(&self, envelope: &EmailEnvelope) -> Result<DeliveryReceipt, EmailDeliveryError> {
        let result = self.send_inner(envelope).await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("smtp", result.is_ok());
        result
    }
}

impl SmtpAdapter {
    async fn send_inner(
        &self,
        envelope: &EmailEnvelope,
    ) -> Result<DeliveryReceipt, EmailDeliveryError> {
        let started = Instant::now();
        tracing::info!(
            driver = "smtp",
            operation = "send",
            outcome = "start",
            host = %self.cfg.host,
            "email send"
        );
        let message = build_message(&self.cfg.from_name, &self.cfg.from_email, envelope)?;

        let mut transport_builder = if self.cfg.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.cfg.host).map_err(|e| {
                EmailDeliveryError::config(
                    "invalid_host",
                    format!("Invalid SMTP host for TLS relay: {e}"),
                )
            })?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.cfg.host)
        };
        transport_builder = transport_builder.port(self.cfg.port);

        if let (Some(username), Some(password)) =
            (self.cfg.username.as_ref(), self.cfg.password.as_ref())
        {
            transport_builder =
                transport_builder.credentials(Credentials::new(username.clone(), password.clone()));
        }

        let transport = transport_builder.build();
        transport.send(message).await.map_err(|e| {
            let elapsed_ms = started.elapsed().as_millis();
            tracing::warn!(
                driver = "smtp",
                operation = "send",
                outcome = "failure",
                elapsed_ms,
                reason_class = "transport_error",
                host = %self.cfg.host,
                "email send"
            );
            EmailDeliveryError::transport(
                "transport_error",
                format!(
                    "SMTP send failed via {}:{}: {e}",
                    self.cfg.host, self.cfg.port
                ),
            )
        })?;

        tracing::info!(
            driver = "smtp",
            operation = "send",
            outcome = "success",
            elapsed_ms = started.elapsed().as_millis(),
            host = %self.cfg.host,
            "email send"
        );

        Ok(DeliveryReceipt {
            provider: "smtp".to_string(),
            message_id: None,
        })
    }
}
