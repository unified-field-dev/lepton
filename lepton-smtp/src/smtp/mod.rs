//! SMTP relay delivery: config, adapter, and Mailpit-friendly local wiring.
//!
//! Use this module when you send through Mailpit or a real SMTP host. Prefer
//! [`crate::EmailServiceBuilder::smtp`] at boot so the host injects an
//! [`crate::EmailDeliveryService`]; [`SmtpAdapter::new`] is available when you
//! construct the adapter directly.
//!
//! # Prerequisites (Mailpit)
//!
//! Local SMTP without TLS typically uses `host = "127.0.0.1"`, `port = 1025`,
//! `use_tls = false`. Start Mailpit from the lepton workspace (`infra/mailpit`),
//! then optionally run `UF_MAILPIT=1 cargo test -p lepton-smtp --test smtp_mailpit`.
//!
//! # Typical flow
//!
//! 1. Build [`SmtpConfig`] with [`SmtpConfig::builder`] (required: `host`, `port`, `from_email`).
//! 2. Pass it to [`crate::EmailServiceBuilder::smtp`] and [`crate::EmailServiceBuilder::build`].
//! 3. Build an [`crate::EmailEnvelope`] (stock helper or hand-written).
//! 4. Call [`crate::EmailDeliveryService::send`] and inspect [`crate::DeliveryReceipt`]
//!    (`provider = "smtp"` on success).
//!
//! Config validation failures are [`crate::EmailDeliveryError::ConfigError`]
//! (`reason_class=missing_field` or `incomplete_credentials`). Send failures are usually
//! [`crate::EmailDeliveryError::TransportError`] (`reason_class=transport_error`).
//!
//! # Examples
//!
//! ```no_run
//! use lepton_smtp::{
//!     verification_email_envelope, EmailDeliveryService, EmailServiceBuilder, SmtpConfig,
//!     VerificationEmailFlow,
//! };
//!
//! # async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
//! let email = EmailServiceBuilder::new()
//!     .smtp(
//!         SmtpConfig::builder()
//!             .host("127.0.0.1")
//!             .port(1025)
//!             .use_tls(false)
//!             .from_email("noreply@example.test")
//!             .build()?,
//!     )
//!     .build()?;
//!
//! let message = verification_email_envelope(
//!     "reader@example.test",
//!     "123456",
//!     VerificationEmailFlow::Signup,
//! );
//! let receipt = email.send(&message).await?;
//! assert_eq!(receipt.provider, "smtp");
//! # Ok(())
//! # }
//! ```
//!
//! # See also
//!
//! - [`SmtpConfig`] / [`SmtpConfigBuilder`] — validated relay settings
//! - [`SmtpAdapter`] — SMTP [`crate::EmailDeliveryService`] implementation
//! - [`crate::EmailDeliveryError`] — typed config and transport failures

mod adapter;
mod config;

pub use adapter::SmtpAdapter;
pub use config::{SmtpConfig, SmtpConfigBuilder};
