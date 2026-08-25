//! SMTP relay [`EmailDeliveryService`] adapter.
//!
//! [`SmtpAdapter`] sends through a configured SMTP server. Prefer constructing it via
//! [`crate::EmailServiceBuilder::smtp`] at host boot; use [`SmtpAdapter::new`] when you need
//! the concrete type.

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
///
/// Use for Mailpit or a production SMTP host after building [`SmtpConfig`]. On success,
/// [`send`](EmailDeliveryService::send) returns a [`DeliveryReceipt`] with `provider = "smtp"`.
/// Tracing logs driver, operation, outcome, and host—never recipient, body, or password.
///
/// Local Mailpit: `host = "127.0.0.1"`, `port = 1025`, `use_tls = false`. Validate with
/// `UF_MAILPIT=1 cargo test -p lepton-smtp --test smtp_mailpit` when Docker is available.
///
/// # Examples
///
/// ```no_run
/// use lepton_smtp::{
///     verification_email_envelope, EmailDeliveryService, EmailServiceBuilder, SmtpConfig,
///     VerificationEmailFlow,
/// };
///
/// # async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
/// let email = EmailServiceBuilder::new()
///     .smtp(
///         SmtpConfig::builder()
///             .host("127.0.0.1")
///             .port(1025)
///             .use_tls(false)
///             .from_email("noreply@example.test")
///             .build()?,
///     )
///     .build()?;
///
/// let message = verification_email_envelope(
///     "reader@example.test",
///     "123456",
///     VerificationEmailFlow::Signup,
/// );
/// let receipt = email.send(&message).await?;
/// assert_eq!(receipt.provider, "smtp");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct SmtpAdapter {
    cfg: SmtpConfig,
}

impl SmtpAdapter {
    /// Construct from [`SmtpConfig`].
    ///
    /// Prefer [`crate::EmailServiceBuilder::smtp`] when injecting `Arc<dyn EmailDeliveryService>`.
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

    /// Deliver `envelope` through the configured SMTP relay.
    ///
    /// # Errors
    ///
    /// * [`EmailDeliveryError::ConfigError`] — invalid TLS relay host (`reason_class=invalid_host`)
    ///   or message construction failure.
    /// * [`EmailDeliveryError::TransportError`] — connection or protocol failure
    ///   (`reason_class=transport_error`). Retry only if your ops policy treats the failure as
    ///   transient; this path does not return [`EmailDeliveryError::Transient`].
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
