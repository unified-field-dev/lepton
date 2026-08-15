//! Email delivery trait and service builder.

use async_trait::async_trait;
use std::sync::Arc;

use crate::direct_mx::{DirectMxAdapter, DirectMxConfig};
use crate::driver::EmailDriver;
use crate::envelope::{DeliveryReceipt, EmailEnvelope};
use crate::error::EmailDeliveryError;
use crate::noop::NoopEmailAdapter;
use crate::smtp::{SmtpAdapter, SmtpConfig};

#[cfg(feature = "twilio")]
use crate::twilio::{TwilioEmailAdapter, TwilioEmailConfig};

/// Sends [`EmailEnvelope`]s via a specific transport ([`SmtpAdapter`], [`DirectMxAdapter`],
/// or [`NoopEmailAdapter`]).
#[async_trait]
pub trait EmailDeliveryService: Send + Sync {
    /// Which [`EmailDriver`] this implementation represents.
    fn driver(&self) -> EmailDriver;
    /// Send `envelope`, returning a [`DeliveryReceipt`] on success.
    async fn send(&self, envelope: &EmailEnvelope) -> Result<DeliveryReceipt, EmailDeliveryError>;
}

/// Builder-first construction of an [`EmailDeliveryService`].
///
/// Prefer this at host boot. [`EmailServiceBuilder::from_env`] and
/// [`crate::build_email_service_from_env`] remain as optional host helpers.
///
/// # Examples
///
/// ```no_run
/// use lepton_smtp::{EmailServiceBuilder, SmtpConfig};
///
/// # fn main() -> Result<(), lepton_smtp::EmailDeliveryError> {
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
/// let _ = email.driver();
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default)]
pub struct EmailServiceBuilder {
    driver: Option<EmailDriver>,
    smtp: Option<SmtpConfig>,
    direct_mx: Option<DirectMxConfig>,
    #[cfg(feature = "twilio")]
    twilio: Option<TwilioEmailConfig>,
    force_noop: bool,
}

impl EmailServiceBuilder {
    /// Empty builder (caller must select a driver/config before [`build`](Self::build)).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the SMTP relay adapter.
    #[must_use]
    pub fn smtp(mut self, cfg: SmtpConfig) -> Self {
        self.smtp = Some(cfg);
        self.force_noop = false;
        self
    }

    /// Configure the direct-MX adapter.
    #[must_use]
    pub fn direct_mx(mut self, cfg: DirectMxConfig) -> Self {
        self.direct_mx = Some(cfg);
        self.force_noop = false;
        self
    }

    /// Configure the Twilio `SendGrid` adapter (`feature = "twilio"`).
    #[cfg(feature = "twilio")]
    #[must_use]
    pub fn twilio(mut self, cfg: TwilioEmailConfig) -> Self {
        self.twilio = Some(cfg);
        self.force_noop = false;
        self
    }

    /// Use the no-op adapter (local / CI).
    #[must_use]
    pub const fn noop(mut self) -> Self {
        self.force_noop = true;
        self.driver = Some(EmailDriver::Noop);
        self
    }

    /// Optionally pin the driver (overrides inference from configured configs).
    #[must_use]
    pub const fn driver(mut self, driver: EmailDriver) -> Self {
        self.driver = Some(driver);
        self
    }

    /// Load driver + matching config from process environment (host helper).
    ///
    /// When `UF_EMAIL_DRIVER` is unset and `UF_SMTP_HOST` is empty, selects noop.
    pub fn from_env() -> Result<Self, EmailDeliveryError> {
        let driver = EmailDriver::from_env()?;
        let mut builder = Self::new().driver(driver);
        match driver {
            EmailDriver::Smtp => {
                builder = builder.smtp(SmtpConfig::from_env()?);
            }
            EmailDriver::DirectMx => {
                builder = builder.direct_mx(DirectMxConfig::from_env()?);
            }
            EmailDriver::Noop => {
                builder = builder.noop();
            }
            #[cfg(feature = "twilio")]
            EmailDriver::Twilio => {
                builder = builder.twilio(TwilioEmailConfig::from_env()?);
            }
        }
        Ok(builder)
    }

    /// Build an [`Arc`] service for injection into auth / host context.
    ///
    /// # Errors
    ///
    /// Returns [`EmailDeliveryError::ConfigError`] when the selected driver is missing
    /// its required config.
    pub fn build(self) -> Result<Arc<dyn EmailDeliveryService>, EmailDeliveryError> {
        let driver = self.resolve_driver()?;
        tracing::info!(
            driver = driver.as_str(),
            operation = "init",
            outcome = "success",
            "email service"
        );
        match driver {
            EmailDriver::Smtp => {
                let cfg = self.smtp.ok_or_else(|| {
                    EmailDeliveryError::config(
                        "missing_config",
                        "SmtpConfig required for smtp driver",
                    )
                })?;
                Ok(Arc::new(SmtpAdapter::new(cfg)))
            }
            EmailDriver::DirectMx => {
                let cfg = self.direct_mx.ok_or_else(|| {
                    EmailDeliveryError::config(
                        "missing_config",
                        "DirectMxConfig required for direct_mx driver",
                    )
                })?;
                Ok(Arc::new(DirectMxAdapter::new(cfg)))
            }
            EmailDriver::Noop => Ok(Arc::new(NoopEmailAdapter)),
            #[cfg(feature = "twilio")]
            EmailDriver::Twilio => {
                let cfg = self.twilio.ok_or_else(|| {
                    EmailDeliveryError::config(
                        "missing_config",
                        "TwilioEmailConfig required for twilio driver",
                    )
                })?;
                Ok(Arc::new(TwilioEmailAdapter::new(cfg)?))
            }
        }
    }

    fn resolve_driver(&self) -> Result<EmailDriver, EmailDeliveryError> {
        if let Some(driver) = self.driver {
            return Ok(driver);
        }
        if self.force_noop {
            return Ok(EmailDriver::Noop);
        }
        if self.smtp.is_some() {
            return Ok(EmailDriver::Smtp);
        }
        if self.direct_mx.is_some() {
            return Ok(EmailDriver::DirectMx);
        }
        #[cfg(feature = "twilio")]
        if self.twilio.is_some() {
            return Ok(EmailDriver::Twilio);
        }
        Err(EmailDeliveryError::config(
            "missing_driver",
            #[cfg(feature = "twilio")]
            "EmailServiceBuilder needs smtp, direct_mx, twilio, or noop before build",
            #[cfg(not(feature = "twilio"))]
            "EmailServiceBuilder needs smtp, direct_mx, or noop before build",
        ))
    }
}

/// Thin wrapper so [`crate::build_email_service_from_env`] can keep returning `Box<dyn …>`.
struct BoxedArcEmailService(Arc<dyn EmailDeliveryService>);

#[async_trait]
impl EmailDeliveryService for BoxedArcEmailService {
    fn driver(&self) -> EmailDriver {
        self.0.driver()
    }

    async fn send(&self, envelope: &EmailEnvelope) -> Result<DeliveryReceipt, EmailDeliveryError> {
        self.0.send(envelope).await
    }
}

/// Build the [`EmailDeliveryService`] selected by [`EmailDriver::from_env`], loading that
/// driver's config from the environment (host helper; prefer [`EmailServiceBuilder`] at boot).
///
/// Returns `Box` for compatibility with existing call sites. New code should prefer
/// [`EmailServiceBuilder::build`] (`Arc`).
pub fn build_email_service_from_env() -> Result<Box<dyn EmailDeliveryService>, EmailDeliveryError> {
    let service = EmailServiceBuilder::from_env()?.build()?;
    Ok(Box::new(BoxedArcEmailService(service)))
}

#[cfg(all(test, feature = "twilio"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn email_builder_twilio_happy_path() {
        let cfg = TwilioEmailConfig::builder()
            .api_key("SG.test")
            .from_email("noreply@example.test")
            .build()
            .expect("cfg");
        let svc = EmailServiceBuilder::new()
            .twilio(cfg)
            .build()
            .expect("build");
        assert_eq!(svc.driver(), EmailDriver::Twilio);
    }
}
