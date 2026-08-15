//! SMS delivery for Lepton auth flows (Noop / Test; optional live Twilio).
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | **Build SMS service** | [`SmsServiceBuilder`] |
//! | **Send SMS** | [`SmsDeliveryService`], [`SmsEnvelope`], [`SmsDeliveryReceipt`] |
//! | **Live Twilio Messages** | `TwilioSmsAdapter` (Cargo feature `twilio`), [`TwilioSmsConfig`] |
//! | **Live Twilio Verify** | `TwilioVerifySmsAdapter` (Cargo feature `twilio`) |
//! | **Errors** | [`SmsDeliveryError`] |
//!
//! Test / CI adapters: [`NoopSmsAdapter`], [`TestSmsAdapter`]. Lab HTTP capture sink:
//! [`HttpCaptureSmsAdapter`] (see module docs).
//!
//! ## Typical flow
//!
//! 1. At host boot, build an SMS adapter via [`SmsServiceBuilder`] (Twilio when the
//!    `twilio` feature is on, or a test/noop adapter for CI).
//! 2. Inject `Arc<dyn SmsDeliveryService>` into auth (`lepton-auth` services, `phone` feature).
//! 3. Send paths use the injected adapter. Do not rebuild credentials per message.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | *(none)* | Noop / Test / HTTP capture / custom adapter; Twilio config types for host wiring |
//! | `twilio` | Live Messages + Verify adapters; `SmsServiceBuilder::twilio` / `twilio_verify` |
//! | `spectra` | Emit `lepton_sms_send{driver,outcome}` via `lepton-spectra-telemetry` |
//!
//! ## Builder-first
//!
//! Hosts supply plain config values at boot. Secrets (Twilio auth token) are plain strings
//! from the host. This crate does not load a secrets manager.
//!
//! ## Integration checklist
//!
//! 1. Call [`SmsServiceBuilder::build`] at boot (requires explicit adapter mode).
//! 2. Never log `to_e164`, body, or auth tokens (tracing allowlist on adapters).
//! 3. Enable `twilio` for production SMS; use [`TestSmsAdapter`] / [`NoopSmsAdapter`] in CI.
//!
//! ## Examples
//!
//! Noop adapter for CI / local smoke:
//!
//! ```no_run
//! use std::sync::Arc;
//! use lepton_sms::{NoopSmsAdapter, SmsServiceBuilder};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let sms = SmsServiceBuilder::new()
//!     .adapter(Arc::new(NoopSmsAdapter))
//!     .build()?;
//! let _ = sms;
//! # Ok(())
//! # }
//! ```
//!
//! ## Further reading
//!
//! - [`README.md`](https://github.com/unified-field-dev/lepton/blob/main/lepton-sms/README.md)

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
