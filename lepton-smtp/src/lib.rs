//! Provider-agnostic email delivery: build a service once, send an [`EmailEnvelope`],
//! inspect a [`DeliveryReceipt`].
//!
//! Wire [`EmailServiceBuilder`] at process boot (Noop, SMTP relay, direct MX, or optional
//! Twilio `SendGrid`), inject [`EmailDeliveryService`], then send. Adapters share one trait so
//! hosts swap transports without changing call sites.
//!
//! # Features
//!
//! - **Builder-first SMTP** — Provides a single boot path: build once with
//!   [`EmailServiceBuilder`], inject [`EmailDeliveryService`], then send. Start with
//!   [Noop](#noop) for local and CI sends without a network.
//! - **Stock envelopes** — Offers ready verification and password-reset bodies so auth
//!   hosts do not hand-write subjects. See [Noop](#noop) for a full send with
//!   [`verification_email_envelope`].
//! - **Swappable backends** — Lets hosts pick Noop, SMTP relay, Direct MX, or Twilio
//!   `SendGrid` without changing call sites ([Choose a delivery backend](#choose-a-delivery-backend)).
//! - **Typed outcomes** — Returns [`DeliveryReceipt`] on success or [`EmailDeliveryError`]
//!   on failure so callers can branch and retry ([Handle outcomes](#handle-outcomes)).
//! - **Safe tracing** — Keeps recipient, body, and credentials out of adapter log fields
//!   when diagnosing delivery in production ([Noop](#noop)).
//! - **Optional Spectra** — Emits `lepton_email_send` counters when the `spectra` Cargo
//!   feature is on, for ops dashboards after send ([Noop](#noop)).
//!
//! # Getting started
//!
//! ## Noop
//!
//! Noop accepts mail without contacting a network. Use it in local runs and CI.
//!
//! Prerequisites: none beyond this crate.
//!
//! 1. [`EmailServiceBuilder::new`] → [`EmailServiceBuilder::noop`] → [`EmailServiceBuilder::build`].
//! 2. Build an [`EmailEnvelope`] (stock helper or hand-written).
//! 3. Call [`EmailDeliveryService::send`].
//! 4. Assert [`DeliveryReceipt::provider`] is `"noop"`. Noop does not return send errors.
//!
//! ```no_run
//! use lepton_smtp::{
//!     verification_email_envelope, EmailDeliveryService, EmailServiceBuilder,
//!     VerificationEmailFlow,
//! };
//!
//! # async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
//! let email = EmailServiceBuilder::new().noop().build()?;
//!
//! let message = verification_email_envelope(
//!     "reader@example.test",
//!     "123456",
//!     VerificationEmailFlow::Signup,
//! );
//! let receipt = email.send(&message).await?;
//! assert_eq!(receipt.provider, "noop");
//! # Ok(())
//! # }
//! ```
//!
//! Runnable: `cargo run -p lepton-smtp --example noop_send`
//!
//! ## Choose a delivery backend
//!
//! | Backend | When to use | Guide | API reference |
//! |---------|-------------|-------|---------------|
//! | **Noop** | Local / CI; no network | [Noop](#noop) | [`EmailServiceBuilder::noop`], [`NoopEmailAdapter`] |
//! | **SMTP relay** | Mailpit or a real SMTP host | [SMTP](#smtp-mailpit-or-relay) | [`EmailServiceBuilder::smtp`], [`SmtpConfig`], [`SmtpAdapter`] |
//! | **Direct MX** | Deliver to the recipient domain's MX (often needs outbound port 25) | [Direct MX](#direct-mx) | [`EmailServiceBuilder::direct_mx`], [`DirectMxConfig`], [`DirectMxAdapter`] |
//! | **Twilio `SendGrid`** | Live `SendGrid` Mail Send (`twilio` feature) | [Twilio `SendGrid`](#twilio-sendgrid) | `EmailServiceBuilder::twilio`, `TwilioEmailConfig`, `TwilioEmailAdapter` |
//!
//! Prefer builders with plain config values at boot. [`EmailServiceBuilder::from_env`] and
//! [`build_email_service_from_env`] remain for hosts that load credentials once from the
//! environment. Do not rebuild from env on every send.
//!
//! ## SMTP (Mailpit or relay)
//!
//! Use an SMTP relay when you already have Mailpit or a real SMTP host.
//!
//! Prerequisites: reachable SMTP host/port; for local Mailpit use `host = "127.0.0.1"`,
//! `port = 1025`, `use_tls = false`. Start Mailpit from `infra/mailpit` in the lepton
//! workspace. Optional gated test: `UF_MAILPIT=1 cargo test -p lepton-smtp --test smtp_mailpit`.
//!
//! 1. Build [`SmtpConfig`] with [`SmtpConfig::builder`] (`host`, `port`, `from_email`).
//! 2. [`EmailServiceBuilder::smtp`] → [`EmailServiceBuilder::build`].
//! 3. Build an [`EmailEnvelope`] and call [`EmailDeliveryService::send`].
//! 4. On success, [`DeliveryReceipt::provider`] is `"smtp"`.
//!
//! Config failures are [`EmailDeliveryError::ConfigError`]. Send failures are usually
//! [`EmailDeliveryError::TransportError`]. See [`EmailDeliveryError::is_transient`] for retry.
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
//! ## Direct MX
//!
//! Direct MX resolves the recipient domain's MX records and delivers to those hosts without a
//! relay. Use it when you must speak SMTP to the recipient domain directly.
//!
//! Prerequisites: outbound connectivity to MX hosts (often port 25), a valid `from_email`, and
//! DNS resolution for the recipient domain. Many cloud networks block port 25.
//!
//! 1. Build [`DirectMxConfig`] with [`DirectMxConfig::builder`] (required: `from_email`).
//! 2. [`EmailServiceBuilder::direct_mx`] → [`EmailServiceBuilder::build`].
//! 3. Build an [`EmailEnvelope`] and call [`EmailDeliveryService::send`].
//! 4. On success, [`DeliveryReceipt::provider`] looks like `direct_mx:<host>`.
//!
//! Failures include config errors, DNS/MX lookup failures, host timeouts, and transport errors
//! ([`EmailDeliveryError`]). Transient cases are marked via [`EmailDeliveryError::is_transient`].
//!
//! ```no_run
//! use lepton_smtp::{
//!     verification_email_envelope, DirectMxConfig, EmailDeliveryService, EmailServiceBuilder,
//!     VerificationEmailFlow,
//! };
//!
//! # async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
//! let email = EmailServiceBuilder::new()
//!     .direct_mx(
//!         DirectMxConfig::builder()
//!             .from_email("noreply@example.test")
//!             .port(25)
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
//! assert!(receipt.provider.starts_with("direct_mx:"));
//! # Ok(())
//! # }
//! ```
//!
//! ## Twilio `SendGrid`
//!
//! Live email through Twilio `SendGrid` Mail Send. Requires the `twilio` Cargo feature and a
//! `SendGrid` API key (`UF_TWILIO_EMAIL_API_KEY` when loading from env). This is separate from
//! Twilio SMS Account SID / Auth Token.
//!
//! Prerequisites: `lepton-smtp` with `features = ["twilio"]`, API key, and a verified from
//! address on the `SendGrid` side.
//!
//! 1. Build `TwilioEmailConfig` (`api_key`, `from_email`).
//! 2. `EmailServiceBuilder::twilio` → `EmailServiceBuilder::build`.
//! 3. Build an [`EmailEnvelope`] and call [`EmailDeliveryService::send`].
//! 4. On success, [`DeliveryReceipt::provider`] is `"twilio"` (message id set when returned).
//!
//! Expect config errors for missing credentials, provider rejection for hard API failures, and
//! transient classification for retryable HTTP/status cases.
//!
//! ```ignore
//! // Requires: lepton-smtp = { version = "…", features = ["twilio"] }
//! use lepton_smtp::{
//!     verification_email_envelope, EmailDeliveryService, EmailServiceBuilder, TwilioEmailConfig,
//!     VerificationEmailFlow,
//! };
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let email = EmailServiceBuilder::new()
//!     .twilio(
//!         TwilioEmailConfig::builder()
//!             .api_key(std::env::var("UF_TWILIO_EMAIL_API_KEY")?)
//!             .from_email("noreply@example.test")
//!             .from_name("App")
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
//! assert_eq!(receipt.provider, "twilio");
//! # Ok(())
//! # }
//! ```
//!
//! ## Build the envelope
//!
//! Stock helpers fill subject and body for common auth-shaped messages:
//! [`verification_email_envelope`], [`password_reset_email_envelope`]. For product copy,
//! construct [`EmailEnvelope`] yourself (or mutate a stock helper) and pass it to
//! [`EmailDeliveryService::send`].
//!
//! ## Handle outcomes
//!
//! Success returns [`DeliveryReceipt`] (`provider`, optional `message_id`). Failures are
//! [`EmailDeliveryError`]: config, transport, provider rejection, or transient (see
//! [`EmailDeliveryError::is_transient`]). Display strings may include `reason_class=…` for ops
//! triage; they omit passwords and message bodies.
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | *(none)* | SMTP / `DirectMX` / Noop |
//! | `twilio` | Live Twilio `SendGrid` Mail Send (`TwilioEmailAdapter`) |
//! | `spectra` | Emit `lepton_email_send{driver,outcome}` via `lepton-spectra-telemetry` |
//!
//! Twilio email uses **`SendGrid`** credentials (`UF_TWILIO_EMAIL_API_KEY`), not SMS Account SID.
//!
//! With `spectra`, boot Spectra in the host first. Counters are best-effort and never fail the
//! send. Labels are `driver` + `outcome` only.
//!
//! # Integration checklist
//!
//! 1. Call [`EmailServiceBuilder::build`] at boot; keep an `Arc<dyn EmailDeliveryService>`.
//! 2. Never log `to`, body, or password / API-key fields (tracing allowlist on adapters).
//! 3. Validate real SMTP with Mailpit when needed; keep CI unit tests Docker-free via Noop.
//!
//! Secrets (SMTP password / `SendGrid` API key) are plain strings from the host. This crate does
//! not load a secrets manager.
//!
//! Default driver is [`EmailDriver::Noop`] when `UF_EMAIL_DRIVER` is unset and `UF_SMTP_HOST`
//! is empty.
//!
//! # Optional integrations
//!
//! - Auth hosts that inject this service: `lepton-auth` (email feature).
//! - SMS delivery: [`lepton_sms`](../lepton_sms/index.html).
//! - Durable send/retry orchestration: `lepton-auth` `delivery` module (`boson-delivery` feature).
//!
//! # Further reading
//!
//! - [Noop](#noop) / [SMTP](#smtp-mailpit-or-relay) / [Direct MX](#direct-mx) / [Twilio `SendGrid`](#twilio-sendgrid) — backend guides
//! - [`EmailServiceBuilder`] — boot and driver selection (API reference)
//! - [`EmailDeliveryService`] — send contract
//! - [`verification_email_envelope`] — stock envelope helper (see also [`password_reset_email_envelope`])
//! - [`EmailEnvelope`] / [`DeliveryReceipt`] — message and outcome types
//! - [`EmailDeliveryError`] — typed failures

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
