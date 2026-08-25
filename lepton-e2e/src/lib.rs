//! Lepton CI e2e + interactive live Twilio / TOTP / Google OAuth harness.
//!
//! # Features
//!
//! - **In-memory lab boot** — Builds Valence and test delivery services for CI without
//!   Docker. Start with [`boot::boot_lab`] on the [CI e2e](#ci-e2e) path.
//! - **Signup → confirm** — Runs [`flow::run_signup_verify_flow`] when you need a full
//!   signup through confirmation under Noop/Test delivery ([CI e2e](#ci-e2e)).
//! - **Device + TOTP** — Exercises [`flow::run_device_totp_challenge_flow`] after signup
//!   when covering trusted-device and authenticator enrollment ([CI e2e](#ci-e2e)).
//! - **OAuth flows** — Drives [`oauth_flow::run_oauth_signup_login_flow`] with
//!   [`oauth_flow::MockOAuthCodeSource`] for mock provider coverage on the same lab
//!   ([CI e2e](#ci-e2e)).
//! - **Live CLIs** — Provides interactive Twilio, authenticator TOTP, and Google/GitHub
//!   OAuth bins when validating real providers outside CI ([Live CLIs](#live-clis)).
//! - **Lab sidecars** — Includes [`sms_sink`] and [`mock_oidc`] for HTTP capture and mock
//!   OIDC during live or local harness runs ([Live CLIs](#live-clis)).
//!
//! # Getting started
//!
//! ## CI e2e
//!
//! CI e2e boots an in-memory lab and runs signup → confirm (and optional device/TOTP)
//! under Noop/Test delivery so coverage stays Docker-free. Use this path in automated
//! test binaries.
//!
//! Prerequisites: this crate on the test binary; Noop/Test delivery via [`boot::boot_lab`].
//!
//! 1. [`boot::boot_lab`] for Valence + services.
//! 2. [`flow::run_signup_verify_flow`] with a [`flow::TestCodeSource`].
//! 3. Assert `signup.confirmed`.
//! 4. Optional: [`flow::run_device_totp_challenge_flow`] and assert device/TOTP flags.
//!
//! Errors: lab boot / flow helpers return [`LiveVerifyError`]. Next: add OAuth mock
//! flows or move to [Live CLIs](#live-clis) for real providers.
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
//! Live CLIs drive real Twilio, authenticator apps, and OAuth providers interactively.
//! Use them for manual provider validation; never run these bins in CI.
//!
//! Prerequisites: matching env gates and Cargo features (`live-twilio`, `live-oauth`, …).
//! See the crate `README.md` for credential setup.
//!
//! 1. Export the gate env var for the CLI you want.
//! 2. `cargo run -p lepton-e2e --bin <name> --features …`.
//! 3. Follow stdin prompts; success prints flow completion lines.
//!
//! Errors: missing env fails closed before interactive prompts. Next: CI path above
//! for Docker-free coverage.
//!
//! ```rust,ignore
//! // Gates (set in the shell before cargo run):
//! //   UF_LEPTON_LIVE_TWILIO=1  → bin lepton-live-verify  (--features live-twilio)
//! //   UF_LEPTON_LIVE_TOTP=1    → bin lepton-live-totp
//! //   UF_LEPTON_LIVE_OAUTH=1   → bin lepton-live-oauth   (--features live-oauth)
//! assert!(std::env::var("UF_LEPTON_LIVE_TWILIO").is_ok());
//! // cargo run -p lepton-e2e --bin lepton-live-verify --features live-twilio
//! ```
//!
//! Sidecars: [`sms_sink`] / `lepton-sms-sink`, [`mock_oidc`] / `lepton-mock-oidc`.
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `boson-delivery` (default) | Durable delivery wiring in lab boot |
//! | `live-twilio` | Live Twilio boot helpers + CLI paths |
//! | `live-oauth` / `live-oauth-google` / `live-oauth-github` | Live OAuth CLI paths |

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
