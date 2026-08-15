//! Spectra ops telemetry for Lepton delivery and auth product funnels.
//!
//! Declares the Lepton Spectra family (`store: "lepton"`) with delivery counters,
//! auth funnel counters, and a bounded `lepton_auth_failure` event. There is no
//! process-wide install switch: hosts boot Spectra themselves, then either call
//! the `record_*` / [`log_auth_failure`] helpers or enable the `spectra` feature on
//! `lepton-smtp` / `lepton-sms` / `lepton-auth`.
//!
//! Labels and event fields are **ops-id only**: closed enums / `reason_class`
//! tokens. Recipient, body, emails, phones, user ids, passwords, OTPs, tokens,
//! challenge ids, and free-form error text are never recorded.
//!
//! ## Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Delivery | [`record_email_send`], [`record_sms_send`] |
//! | Auth funnel | [`record_signup`], [`record_signin`], [`record_oauth`], [`record_verify`], [`record_password_reset`], [`record_totp`], [`record_device`], [`record_contact`], [`record_account`], [`record_identity_delete`], [`record_step_up`] |
//! | Failures | [`log_auth_failure`] |
//!
//! Counter and event field names live on the schema types (and in [`topics`] for
//! transport DTOs). Label allowlists / sanitizers (`bound_*`) are in [`emit`] for
//! hosts that map raw strings before emit.
//!
//! ## Examples
//!
//! ```rust,ignore
//! use lepton_spectra_telemetry::{
//!     record_signin, AuthFactor, AuthOutcome, SigninStage,
//! };
//!
//! record_signin(SigninStage::Password, AuthOutcome::NeedsMfa, "none", AuthFactor::None);
//! record_signin(SigninStage::MfaComplete, AuthOutcome::Success, "none", AuthFactor::Totp);
//! ```
//!
//! OAuth:
//!
//! ```rust,ignore
//! use lepton_spectra_telemetry::{
//!     record_oauth, AuthOutcome, OAuthIntentLabel, OAuthProviderLabel, OAuthStage,
//! };
//!
//! record_oauth(
//!     OAuthProviderLabel::Google,
//!     OAuthIntentLabel::Signup,
//!     OAuthStage::Complete,
//!     AuthOutcome::Success,
//!     "none",
//! );
//! ```
//!
//! Runnable smoke (host must boot Spectra first):
//! `cargo run -p lepton-spectra-telemetry --example email_send_record_smoke`.

#![allow(clippy::too_long_first_doc_paragraph)]

/// Bounded emit helpers (delivery + auth funnels + failure events).
pub mod emit;
/// Typed emit helpers from Lepton Spectra schemas.
pub mod helpers;
// macro-generated Spectra schema types; documented via each schema's `description`
#[allow(missing_docs)]
mod schemas;
/// Transport `*Payload` / `*_TOPIC` DTOs from Lepton Spectra schemas.
pub mod topics;

pub use emit::{
    bound_email_driver, bound_error_class, bound_oauth_provider, bound_optional_channel,
    bound_optional_provider, bound_sms_driver, bound_verify_channel, log_auth_failure,
    record_account, record_contact, record_device, record_email_send, record_identity_delete,
    record_oauth, record_password_reset, record_signin, record_signup, record_sms_send,
    record_step_up, record_totp, record_verify, AccountOperation, AuthFactor, AuthFailureFlow,
    AuthOutcome, ContactOperation, DeviceKind, DeviceOperation, EmailSendOutcome,
    IdentityDeleteOperation, OAuthIntentLabel, OAuthProviderLabel, OAuthStage, PasswordResetStage,
    SigninStage, SmsSendOutcome, StepUpPath, TotpOperation, VerifyChannel, VerifyStage,
};
pub use helpers::*;
pub use topics::*;
