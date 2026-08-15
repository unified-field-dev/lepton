//! Direct-to-MX [`EmailDeliveryService`] adapter.

use async_trait::async_trait;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::time::Instant;
use tokio::time::{timeout, Duration};

use crate::direct_mx::config::DirectMxConfig;
use crate::direct_mx::dns::{extract_recipient_domain, resolve_mx_hosts};
use crate::driver::EmailDriver;
use crate::envelope::{DeliveryReceipt, EmailEnvelope};
use crate::error::EmailDeliveryError;
use crate::message::build_message;
use crate::service::EmailDeliveryService;

/// [`EmailDeliveryService`] that delivers directly to the recipient domain's MX hosts,
/// bypassing a relay.
#[derive(Clone, Debug)]
pub struct DirectMxAdapter {
    cfg: DirectMxConfig,
}

impl DirectMxAdapter {
    /// Construct from [`DirectMxConfig`].
    #[must_use]
    pub const fn new(cfg: DirectMxConfig) -> Self {
        Self { cfg }
    }

    async fn try_host(
        &self,
        mx_host: &str,
        message: Message,
        started: Instant,
    ) -> Result<DeliveryReceipt, String> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(mx_host)
            .port(self.cfg.port)
            .build();
        match timeout(
            Duration::from_secs(self.cfg.host_timeout_secs),
            transport.send(message),
        )
        .await
        {
            Err(_) => {
                tracing::warn!(
                    driver = "direct_mx",
                    operation = "send",
                    outcome = "failure",
                    elapsed_ms = started.elapsed().as_millis(),
                    reason_class = "host_timeout",
                    host = %mx_host,
                    "email send"
                );
                Err(format!(
                    "{mx_host}: timeout after {}s",
                    self.cfg.host_timeout_secs
                ))
            }
            Ok(Ok(_)) => {
                tracing::info!(
                    driver = "direct_mx",
                    operation = "send",
                    outcome = "success",
                    elapsed_ms = started.elapsed().as_millis(),
                    host = %mx_host,
                    "email send"
                );
                Ok(DeliveryReceipt {
                    provider: format!("direct_mx:{mx_host}"),
                    message_id: None,
                })
            }
            Ok(Err(error)) => {
                tracing::warn!(
                    driver = "direct_mx",
                    operation = "send",
                    outcome = "failure",
                    elapsed_ms = started.elapsed().as_millis(),
                    reason_class = "host_transport_error",
                    host = %mx_host,
                    "email send"
                );
                Err(format!("{mx_host}: {error}"))
            }
        }
    }
}

#[async_trait]
impl EmailDeliveryService for DirectMxAdapter {
    fn driver(&self) -> EmailDriver {
        EmailDriver::DirectMx
    }

    async fn send(&self, envelope: &EmailEnvelope) -> Result<DeliveryReceipt, EmailDeliveryError> {
        let result = self.send_inner(envelope).await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("direct_mx", result.is_ok());
        result
    }
}

impl DirectMxAdapter {
    async fn send_inner(
        &self,
        envelope: &EmailEnvelope,
    ) -> Result<DeliveryReceipt, EmailDeliveryError> {
        let started = Instant::now();
        let recipient_domain = extract_recipient_domain(&envelope.to)?;
        tracing::info!(
            driver = "direct_mx",
            operation = "send",
            outcome = "start",
            "email send"
        );
        let mx_hosts = timeout(
            Duration::from_secs(self.cfg.mx_lookup_timeout_secs),
            resolve_mx_hosts(&recipient_domain),
        )
        .await
        .map_err(|_| {
            EmailDeliveryError::transient(
                "mx_lookup_timeout",
                format!(
                    "MX lookup timed out after {}s",
                    self.cfg.mx_lookup_timeout_secs
                ),
            )
        })??;
        let capped_hosts: Vec<String> = mx_hosts.into_iter().take(self.cfg.max_hosts).collect();
        let mut errors = Vec::new();

        for mx_host in capped_hosts {
            tracing::info!(
                driver = "direct_mx",
                operation = "send",
                outcome = "attempt",
                host = %mx_host,
                "email send"
            );
            let message = build_message(&self.cfg.from_name, &self.cfg.from_email, envelope)?;
            match self.try_host(&mx_host, message, started).await {
                Ok(receipt) => return Ok(receipt),
                Err(err) => errors.push(err),
            }
        }

        tracing::warn!(
            driver = "direct_mx",
            operation = "send",
            outcome = "failure",
            elapsed_ms = started.elapsed().as_millis(),
            reason_class = "hosts_exhausted",
            "email send"
        );
        Err(EmailDeliveryError::transport(
            "hosts_exhausted",
            format!(
                "Direct MX delivery failed on port {}: {}",
                self.cfg.port,
                errors.join(" | ")
            ),
        ))
    }
}
