//! Lepton CI e2e + interactive live Twilio / TOTP / Google OAuth harness.
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Boot in-memory Valence | [`boot::boot_valence`] |
//! | Valence + services (+ Boson) | [`boot::boot_lab`] |
//! | Noop email + Test SMS (sync only) | [`boot::boot_services_test`] |
//! | Live Twilio services | `boot::boot_services_twilio` (`live-twilio`) |
//! | Full signup → confirm flow | [`flow::run_signup_verify_flow`] |
//! | Device + TOTP challenge | [`flow::run_device_totp_challenge_flow`] |
//! | OAuth signup → login | [`oauth_flow::run_oauth_signup_login_flow`] |
//! | Mock OAuth codes | [`oauth_flow::MockOAuthCodeSource`] |
//! | Live Google callback | [`oauth_callback::LocalhostOAuthCodeSource`] |
//! | Test TOTP codes | [`flow::TestTotpCodeSource`] |
//! | Live authenticator codes | [`flow::StdinTotpCodeSource`] |
//! | Parse email token paste | [`parse::email_token_from_input`] |
//! | Parse otpauth secret | [`parse::totp_secret_from_otpauth_uri`] |
//! | Interactive Twilio CLI | `lepton-live-verify` bin (`UF_LEPTON_LIVE_TWILIO=1`) |
//! | Interactive TOTP CLI | `lepton-live-totp` bin (`UF_LEPTON_LIVE_TOTP=1`) |
//! | Interactive Google OAuth CLI | `lepton-live-oauth` bin (`UF_LEPTON_LIVE_OAUTH=1`) |
//! | SMS HTTP capture sink | [`sms_sink`] / `lepton-sms-sink` bin (`:8099`) |
//! | Mock OIDC sidecar | [`mock_oidc`] / `lepton-mock-oidc` bin (`:5556`) |
//!
//! ## CI e2e
//!
//! ```rust,ignore
//! use lepton_e2e::flow::{
//!     run_device_totp_challenge_flow, run_signup_verify_flow, TestCodeSource, TestTotpCodeSource,
//!     SignupVerifyOpts,
//! };
//! use lepton_e2e::boot::boot_lab;
//!
//! # async fn demo() -> Result<(), lepton_e2e::LiveVerifyError> {
//! let lab = boot_lab("demo").await?;
//! let codes = TestCodeSource::new(lab.test_sms.clone());
//! let signup = run_signup_verify_flow(
//!     &lab.valence,
//!     &lab.services,
//!     &codes,
//!     "Alex Rivera",
//!     "e2e@example.test",
//!     "+15555550100",
//!     "CorrectHorseBattery1!",
//!     lepton_e2e::SignupVerifyOpts::default(),
//! ).await?;
//! assert!(signup.confirmed);
//! let outcome = run_device_totp_challenge_flow(
//!     &lab.valence,
//!     &lab.services,
//!     &signup.user_id,
//!     "Test Browser",
//!     "e2e@example.test",
//!     "Acme Site",
//!     &TestTotpCodeSource,
//! ).await?;
//! assert!(outcome.device_trusted && outcome.totp_enabled && outcome.challenge_ok);
//! # Ok(())
//! # }
//! ```
//!
//! ## Live CLIs
//!
//! - Twilio email/SMS: `lepton-live-verify` (`UF_LEPTON_LIVE_TWILIO=1`) — see crate `README.md`.
//! - Google Authenticator TOTP: `lepton-live-totp` (`UF_LEPTON_LIVE_TOTP=1`) — test user setup,
//!   prints `otpauth://` URI, stdin enroll + challenge codes. Never run in CI.
//! - Google / GitHub OAuth signup/login: `lepton-live-oauth` (`UF_LEPTON_LIVE_OAUTH=1`,
//!   `UF_OAUTH_PROVIDER=google|github`, feature `live-oauth`) — loopback callback + live
//!   token exchange. Never run in CI.

// `#[async_trait]` marks its generated boxed-future returns `#[must_use]`, which trips
// `double_must_use` on trait methods returning `Result`. This crate does not inherit the
// workspace clippy config, so silence the macro false positive here.
#![allow(clippy::double_must_use)]

pub mod boot;
pub mod error;
pub mod flow;
pub mod mock_oidc;
pub mod oauth_callback;
pub mod oauth_flow;
pub mod parse;
pub mod sms_sink;

pub use boot::{boot_lab, boot_services_test, boot_valence, Lab, TestServices};
pub use error::LiveVerifyError;
pub use flow::{
    issue_sms_challenge, run_device_totp_challenge_flow, run_signup_verify_flow,
    verify_email_token, CodeSource, DeviceTotpOutcome, SignupVerifyOpts, SignupVerifyOutcome,
    StdinCodeSource, StdinTotpCodeSource, TestCodeSource, TestTotpCodeSource, TotpCodeSource,
};
pub use oauth_callback::LocalhostOAuthCodeSource;
pub use oauth_flow::{
    run_oauth_signup_login_flow, MockOAuthCodeSource, OAuthCodeSource, OAuthPhase,
    OAuthSignupLoginOpts, OAuthSignupLoginOutcome,
};
pub use parse::{
    email_token_from_input, totp_manual_entry_from_otpauth_uri, totp_secret_from_otpauth_uri,
    TotpManualEntry,
};

#[cfg(feature = "live-twilio")]
pub use boot::{boot_lab_twilio, boot_services_twilio, TwilioLab};
