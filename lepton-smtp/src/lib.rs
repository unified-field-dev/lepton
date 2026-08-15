//! Transactional email delivery for Lepton auth flows.
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | **Build email service** | [`EmailServiceBuilder`], [`SmtpConfig::builder`] |
//! | **Send email** | [`EmailDeliveryService`], [`EmailEnvelope`], [`DeliveryReceipt`] |
//! | **Auth envelopes (stock)** | [`verification_email_envelope`], [`password_reset_email_envelope`] |
//! | **Custom subject/body** | hand-built [`EmailEnvelope`] (see Examples) |
//! | **Twilio / `SendGrid`** | `TwilioEmailAdapter` (Cargo feature `twilio`) |
//! | **Errors** | [`EmailDeliveryError`] |
//!
//! Optional boot-from-env: [`EmailServiceBuilder::from_env`], [`build_email_service_from_env`].
//!
//! SMS delivery lives in **`lepton-sms`** (Noop / Test / optional Twilio SMS).
//!
//! ## Typical flow
//!
//! 1. At host boot, build plain [`SmtpConfig`] (or Twilio email config) via builders.
//! 2. Inject `Arc<dyn EmailDeliveryService>` into auth (`lepton-auth` services).
//! 3. Send paths use the injected adapter. Do not rebuild from env per message.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | *(none)* | SMTP / `DirectMX` / Noop |
//! | `twilio` | Live Twilio `SendGrid` Mail Send (`TwilioEmailAdapter`) |
//! | `spectra` | Emit `lepton_email_send{driver,outcome}` via `lepton-spectra-telemetry` |
//!
//! Twilio email uses **`SendGrid`** credentials (`UF_TWILIO_EMAIL_API_KEY`), not SMS Account SID.
//!
//! ## Builder-first
//!
//! Hosts supply plain config values at boot. Prefer builders over process env inside
//! library send paths. [`from_env`](EmailServiceBuilder::from_env) helpers stay available
//! for hosts that still wire credentials from environment once at startup.
//!
//! Default driver is [`EmailDriver::Noop`] when `UF_EMAIL_DRIVER` is unset and
//! `UF_SMTP_HOST` is empty.
//!
//! Secrets (SMTP password / `SendGrid` API key) are plain strings from the host.
//! This crate does not load a secrets manager.
//!
//! ## Integration checklist
//!
//! 1. Call [`EmailServiceBuilder::build`] at boot.
//! 2. Never log `to`, body, or password/API-key fields (tracing allowlist on adapters).
//! 3. Validate real SMTP with `infra/mailpit` (`UF_MAILPIT=1`) when needed.
//!
//! Examples use Noop or Mailpit SMTP. Typed errors carry `reason_class`; Display omits secrets.
//!
//! ## Examples
//!
//! Stock verification envelope, then a hand-built custom subject/body:
//!
//! ```rust
//! use lepton_smtp::{
//!     verification_email_envelope, EmailEnvelope, VerificationEmailFlow,
//! };
//!
//! let stock = verification_email_envelope(
//!     "user@example.test",
//!     "tok123",
//!     VerificationEmailFlow::Signup,
//! );
//! assert_eq!(stock.subject, "Your verification code");
//!
//! let custom = EmailEnvelope {
//!     to: "user@example.test".into(),
//!     subject: "Confirm your Unified Field account".into(),
//!     text_body: "Your code is tok123".into(),
//!     html_body: "<p>Your code is <code>tok123</code></p>".into(),
//! };
//! assert!(custom.subject.contains("Unified Field"));
//! ```
//!
//! Boot an SMTP (or noop) service:
//!
//! ```no_run
//! use lepton_smtp::{EmailServiceBuilder, SmtpConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//! let _ = email;
//! # Ok(())
//! # }
//! ```
//!
//! ## Further reading
//!
//! - [`README.md`](https://github.com/unified-field-dev/lepton/blob/main/lepton-smtp/README.md)
//! - [`lepton_sms`](../lepton_sms/index.html)

mod direct_mx;
mod driver;
mod envelope;
mod error;
mod message;
mod noop;
mod service;
mod smtp;

#[cfg(feature = "spectra")]
mod spectra_emit;

#[cfg(feature = "twilio")]
mod twilio;

pub use direct_mx::{DirectMxAdapter, DirectMxConfig, DirectMxConfigBuilder};
#[cfg(feature = "twilio")]
pub use driver::EMAIL_DRIVER_TWILIO;
pub use driver::{
    EmailDriver, EMAIL_DRIVER_DIRECT_MX, EMAIL_DRIVER_ENV, EMAIL_DRIVER_NOOP, EMAIL_DRIVER_SMTP,
};
pub use envelope::{
    greeting_name_from_email, password_reset_email_envelope, verification_email_envelope,
    verification_email_envelope_named, DeliveryReceipt, EmailEnvelope, VerificationEmailFlow,
};
pub use error::EmailDeliveryError;
pub use noop::NoopEmailAdapter;
pub use service::{build_email_service_from_env, EmailDeliveryService, EmailServiceBuilder};
pub use smtp::{SmtpAdapter, SmtpConfig, SmtpConfigBuilder};

#[cfg(feature = "twilio")]
pub use twilio::{
    TwilioEmailAdapter, TwilioEmailConfig, TwilioEmailConfigBuilder, TWILIO_EMAIL_API_BASE_URL,
    TWILIO_EMAIL_API_KEY_ENV,
};
