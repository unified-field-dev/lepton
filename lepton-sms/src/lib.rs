//! Provider-agnostic SMS delivery: build a service once, send an [`SmsEnvelope`],
//! inspect an [`SmsDeliveryReceipt`].
//!
//! Wire [`SmsServiceBuilder`] at process boot (Noop, Test, HTTP capture, or optional
//! Twilio Messages / Verify), inject [`SmsDeliveryService`], then send. Adapters share one
//! trait so hosts swap transports without changing call sites.
//!
//! # Features
//!
//! - **Builder-first SMS** — Provides a single boot path: build once with
//!   [`SmsServiceBuilder`], inject [`SmsDeliveryService`], then send. Start with
//!   [Noop](#noop) for local and CI sends without a network.
//! - **E.164 validation** — Validates destination numbers on send so bad addresses fail
//!   closed before the adapter runs ([Noop](#noop)).
//! - **Swappable adapters** — Lets hosts pick Noop, Test, HTTP capture, or Twilio
//!   Messages / Verify without changing call sites
//!   ([Choose a delivery backend](#choose-a-delivery-backend)).
//! - **Typed outcomes** — Returns [`SmsDeliveryReceipt`] on success or [`SmsDeliveryError`]
//!   on failure so callers can branch and retry ([Handle outcomes](#handle-outcomes)).
//! - **Safe tracing** — Keeps E.164, body, OTP, and credentials out of adapter log fields
//!   when diagnosing delivery ([Noop](#noop)).
//! - **Optional Spectra** — Emits `lepton_sms_send` counters when the `spectra` Cargo
//!   feature is on, for ops dashboards after send ([Noop](#noop)).
//!
//! # Getting started
//!
//! ## Noop
//!
//! Noop validates E.164 and accepts the message without contacting a network. Use it in
//! local runs and CI.
//!
//! Prerequisites: none beyond this crate. Destination must be E.164 (`+[1-9]…`).
//!
//! 1. [`SmsServiceBuilder::new`] → [`SmsServiceBuilder::noop`] → [`SmsServiceBuilder::build`].
//! 2. Build an [`SmsEnvelope`] (`to_e164`, `body`; optional `otp_code`).
//! 3. Call [`SmsDeliveryService::send`].
//! 4. Assert [`SmsDeliveryReceipt::provider`] is `"noop"`. Invalid E.164 returns
//!    [`SmsDeliveryError::ConfigError`] (`reason_class=invalid_e164`).
//!
//! ```no_run
//! use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder};
//!
//! # async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
//! let sms = SmsServiceBuilder::new().noop().build()?;
//! let receipt = sms
//!     .send(&SmsEnvelope {
//!         to_e164: "+15551234567".into(),
//!         body: "Your code is 123456".into(),
//!         otp_code: Some("123456".into()),
//!     })
//!     .await?;
//! assert_eq!(receipt.provider, "noop");
//! # Ok(())
//! # }
//! ```
//!
//! Runnable: `cargo run -p lepton-sms --example noop_send`
//!
//! ## Choose a delivery backend
//!
//! | Backend | When to use | Guide | API reference |
//! |---------|-------------|-------|---------------|
//! | **Noop** | Local / CI; no network | [Noop](#noop) | [`SmsServiceBuilder::noop`], [`NoopSmsAdapter`] |
//! | **Test** | Unit tests; assert recorded envelopes | [Test](#test) | [`SmsServiceBuilder::test`], [`TestSmsAdapter`] |
//! | **HTTP capture** | Lab sink (`:8099`) | [HTTP capture](#http-capture) | [`SmsServiceBuilder::http_capture`], [`HttpCaptureSmsConfig`], [`HttpCaptureSmsAdapter`] |
//! | **Twilio Messages** | Live SMS via Messages REST (`twilio` feature) | [Twilio Messages](#twilio-messages) | `SmsServiceBuilder::twilio`, `TwilioSmsConfig`, `TwilioSmsAdapter` |
//! | **Twilio Verify** | Live OTP via Verify `CustomCode` (`twilio` feature) | [Twilio Verify](#twilio-verify) | `SmsServiceBuilder::twilio_verify`, `TwilioVerifyConfig`, `TwilioVerifySmsAdapter` |
//!
//! Prefer builders with plain config values at boot. Do not rebuild credentials on every send.
//!
//! ## Test
//!
//! Provides an in-memory SMS adapter for unit tests so callers can assert on
//! `recorded()` envelopes without a network. Prefer [`SmsServiceBuilder::adapter`] with a
//! shared [`TestSmsAdapter`] when tests need the same instance.
//!
//! Prerequisites: none. Destination must be E.164.
//!
//! 1. [`SmsServiceBuilder::test`] → [`SmsServiceBuilder::build`], or inject via
//!    [`SmsServiceBuilder::adapter`].
//! 2. Send an [`SmsEnvelope`].
//! 3. On success, [`SmsDeliveryReceipt::provider`] is `"test"`; call
//!    [`TestSmsAdapter::recorded`] on the shared adapter.
//!
//! Invalid E.164 returns [`SmsDeliveryError::ConfigError`] (`reason_class=invalid_e164`),
//! same as Noop. The adapter does not contact a network, so there is no transport failure path.
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
//!
//! ## HTTP capture
//!
//! POSTs JSON envelopes to a lab HTTP sink (default `http://127.0.0.1:8099`). Use when you
//! want a process-boundary capture without Twilio.
//!
//! Prerequisites: sink listening (for example `cargo run -p lepton-e2e --bin lepton-sms-sink`).
//!
//! 1. [`HttpCaptureSmsConfig::new`] → [`SmsServiceBuilder::http_capture`] →
//!    [`SmsServiceBuilder::build`].
//! 2. Send an [`SmsEnvelope`].
//! 3. On success, [`SmsDeliveryReceipt::provider`] is `"http_capture"`.
//!
//! Sink/HTTP failures map to [`SmsDeliveryError`] transport / transient / rejected classes.
//!
//! ```no_run
//! use lepton_sms::{
//!     HttpCaptureSmsConfig, SmsDeliveryService, SmsEnvelope, SmsServiceBuilder,
//! };
//!
//! # async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
//! let sms = SmsServiceBuilder::new()
//!     .http_capture(HttpCaptureSmsConfig::new("http://127.0.0.1:8099")?)
//!     .build()?;
//! let receipt = sms
//!     .send(&SmsEnvelope {
//!         to_e164: "+15551234567".into(),
//!         body: "lab capture".into(),
//!         otp_code: Some("123456".into()),
//!     })
//!     .await?;
//! assert_eq!(receipt.provider, "http_capture");
//! # Ok(())
//! # }
//! ```
//!
//! ## Twilio Messages
//!
//! Provides live SMS through Twilio Messages REST when the product must reach real
//! handsets. Requires the `twilio` Cargo feature.
//!
//! Prerequisites: Account SID, From number, and either API key SID+secret or Auth Token.
//! Prefer API key (`SK…`) + secret.
//!
//! 1. Build `TwilioSmsConfig`.
//! 2. `SmsServiceBuilder::twilio` → `SmsServiceBuilder::build`.
//! 3. Send an [`SmsEnvelope`] (Messages may ignore `otp_code`).
//! 4. On success, provider is `"twilio"` (message id when Twilio returns a SID).
//!
//! Expect config errors for missing credentials, provider rejection for hard API failures,
//! and transient classification for retryable HTTP/status cases.
//!
//! ```ignore
//! // Requires: lepton-sms = { version = "…", features = ["twilio"] }
//! use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder, TwilioSmsConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let sms = SmsServiceBuilder::new()
//!     .twilio(
//!         TwilioSmsConfig::builder()
//!             .account_sid(std::env::var("UF_TWILIO_ACCOUNT_SID")?)
//!             .api_key(std::env::var("UF_TWILIO_API_KEY")?)
//!             .api_secret(std::env::var("UF_TWILIO_API_SECRET")?)
//!             .from(std::env::var("UF_TWILIO_FROM")?)
//!             .build()?,
//!     )
//!     .build()?;
//! let receipt = sms
//!     .send(&SmsEnvelope {
//!         to_e164: "+15551234567".into(),
//!         body: "Your code is 123456".into(),
//!         otp_code: None,
//!     })
//!     .await?;
//! assert_eq!(receipt.provider, "twilio");
//! # Ok(())
//! # }
//! ```
//!
//! ## Twilio Verify
//!
//! Provides live OTP delivery through Twilio Verify with `CustomCode` when Valence (or
//! the host) still verifies the code. Requires the `twilio` Cargo feature and Custom
//! Verification Code enabled on the Verify Service.
//!
//! Prerequisites: Verify Service SID plus Twilio credentials; [`SmsEnvelope::otp_code`] must
//! be 4..=10 characters.
//!
//! 1. Build `TwilioVerifyConfig`.
//! 2. `SmsServiceBuilder::twilio_verify` → `SmsServiceBuilder::build`.
//! 3. Send an [`SmsEnvelope`] with `otp_code`.
//! 4. On success, provider is `"twilio_verify"`.
//!
//! Missing/invalid `otp_code` is [`SmsDeliveryError::ConfigError`]. Provider/transient
//! failures follow the same pattern as Messages.
//!
//! ```ignore
//! // Requires: lepton-sms = { version = "…", features = ["twilio"] }
//! use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder, TwilioVerifyConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let sms = SmsServiceBuilder::new()
//!     .twilio_verify(TwilioVerifyConfig::from_env()?)
//!     .build()?;
//! let receipt = sms
//!     .send(&SmsEnvelope {
//!         to_e164: "+15551234567".into(),
//!         body: "ignored for Verify body channel".into(),
//!         otp_code: Some("123456".into()),
//!     })
//!     .await?;
//! assert_eq!(receipt.provider, "twilio_verify");
//! # Ok(())
//! # }
//! ```
//!
//! ## Build the envelope
//!
//! Set [`SmsEnvelope::to_e164`], [`SmsEnvelope::body`], and optionally
//! [`SmsEnvelope::otp_code`] (required for Twilio Verify). Adapters call
//! [`validate_e164`] before send.
//!
//! ## Handle outcomes
//!
//! Success returns [`SmsDeliveryReceipt`] (`provider`, optional `message_id`). Failures are
//! [`SmsDeliveryError`]: config, transport, provider rejection, or transient (see
//! [`SmsDeliveryError::is_transient`]). Display strings may include `reason_class=…`; they
//! omit full E.164 numbers and message bodies.
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | *(none)* | Noop / Test / HTTP capture / custom adapter; Twilio config types for host wiring |
//! | `twilio` | Live Messages + Verify adapters; `SmsServiceBuilder::twilio` / `twilio_verify` |
//! | `spectra` | Emit `lepton_sms_send{driver,outcome}` via `lepton-spectra-telemetry` |
//!
//! With `spectra`, boot Spectra in the host first. Counters are best-effort and never fail
//! the send. Labels are `driver` + `outcome` only.
//!
//! # Integration checklist
//!
//! 1. Call [`SmsServiceBuilder::build`] at boot; keep an `Arc<dyn SmsDeliveryService>`.
//! 2. Never log `to_e164`, body, OTP, or auth tokens (tracing allowlist on adapters).
//! 3. Enable `twilio` for production SMS; use Noop / Test in CI.
//!
//! Secrets (Twilio auth token / API secret) are plain strings from the host. This crate does
//! not load a secrets manager.
//!
//! # Optional integrations
//!
//! - Auth hosts that inject this service: `lepton-auth` (`phone` feature).
//! - Email delivery: [`lepton_smtp`](../lepton_smtp/index.html).
//!
//! # Further reading
//!
//! - [Noop](#noop) / [Test](#test) / [HTTP capture](#http-capture) / [Twilio Messages](#twilio-messages) / [Twilio Verify](#twilio-verify) — backend guides
//! - [`SmsServiceBuilder`] — boot and adapter selection (API reference)
//! - [`SmsDeliveryService`] — send contract
//! - [`SmsEnvelope`] / [`SmsDeliveryReceipt`] — message and outcome types
//! - [`SmsDeliveryError`] — typed failures

mod envelope;
mod error;
mod http_capture;
mod http_capture_config;
mod noop;
mod service;
mod test_adapter;
mod twilio_config;
mod twilio_verify_config;

#[cfg(feature = "spectra")]
mod spectra_emit;

#[cfg(feature = "twilio")]
mod twilio;

pub use envelope::{SmsDeliveryReceipt, SmsEnvelope};
pub use error::SmsDeliveryError;
pub use http_capture::HttpCaptureSmsAdapter;
pub use http_capture_config::HttpCaptureSmsConfig;
pub use noop::NoopSmsAdapter;
pub use service::{SmsDeliveryService, SmsServiceBuilder};
pub use test_adapter::TestSmsAdapter;
pub use twilio_config::{
    TwilioSmsAuth, TwilioSmsConfig, TwilioSmsConfigBuilder, TWILIO_ACCOUNT_SID_ENV,
    TWILIO_API_BASE_URL, TWILIO_API_KEY_ENV, TWILIO_API_SECRET_ENV, TWILIO_AUTH_TOKEN_ENV,
    TWILIO_FROM_ENV,
};
pub use twilio_verify_config::{
    TwilioVerifyConfig, TwilioVerifyConfigBuilder, TWILIO_VERIFY_API_BASE_URL,
    TWILIO_VERIFY_SERVICE_SID_ENV,
};

#[cfg(feature = "twilio")]
pub use twilio::{TwilioSmsAdapter, TwilioVerifySmsAdapter};

/// Validate E.164 phone numbers (used by adapters).
pub use envelope::validate_e164;
