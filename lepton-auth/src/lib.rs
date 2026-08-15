//! Auth library: server functions, factors, OAuth, password policy, and tokens.
//!
//! Compose with **higgs** / **higgs-host** for request context. Form UI lives in
//! [`lepton_auth_ui`](../lepton_auth_ui/index.html) (Orbital). Start with the task
//! table below, then the teaching binaries under `examples/`.
//!
//! # Organized by task
//!
//! | Task | Feature | Start here | See also |
//! |------|---------|------------|----------|
//! | **Boot delivery** — inject email/SMS + public URL | `ssr` (+ channels) | [`provide_auth_services`], [`services`] | [Boot delivery](#boot-delivery-email-only) |
//! | **Session Backend + middleware** | host | [`Backend`](../lepton_host_adapter/struct.Backend.html), [`session_snapshot_middleware`](../lepton_host_adapter/fn.session_snapshot_middleware.html) | [Host wiring](../lepton_host_adapter/index.html#host-wiring) |
//! | **Authenticated server fn** | `ssr` | [`require_auth_user`], [`user_valence`] | [Authenticated server fn](#authenticated-server-fn-current-user) |
//! | **Mount auth UI** | — | [`lepton_auth_ui`](../lepton_auth_ui/index.html), [`actions`], [`paths`] | [Mount `AuthDialog`](../lepton_auth_ui/index.html#mount-authdialog-shell) |
//! | **Verification / reset mail** | `email` | [`email_delivery`] | [`verification_email_envelope`](../lepton_smtp/fn.verification_email_envelope.html) |
//! | **OTP / TOTP verify** | `email` / `phone` / `totp` | [`factor`] | [`factor` Examples](factor/index.html#examples) |
//! | **Login MFA** | `ssr` (+ `totp`) | [`session_mfa`], [`actions::signin`] | Step-up (per-op): [UI](../lepton_auth_ui/index.html#step-up-critical-action) |
//! | **Step-up before sensitive op** | `totp` | [`StepUpDialog`](../lepton_auth_ui/fn.StepUpDialog.html), [`factor::FactorChallengeService`] | [Step-up (UI)](../lepton_auth_ui/index.html#step-up-critical-action) |
//! | **Signup** | `ssr` (+ `email`) | [`actions`] (UI), [`signup_api`], [`signup_policy`] | [`signup_api::ssr::create_pending_user`] |
//! | **OAuth / contacts / devices** | `ssr` (+ oauth / `webauthn`) | [`actions::oauth`], [`contacts`], [`devices`] | [`oauth`], [`actions::oauth_settings`] |
//! | **Confirm account** | `ssr` (+ `email` / `phone`) | [`actions::confirm_account`], [`trust::confirm_user`] | [`ConfirmAccountPrompt`](../lepton_auth_ui/fn.ConfirmAccountPrompt.html) |
//! | **TOTP enroll** | `totp` | [`actions::totp`] | Library path: [`totp`] |
//! | **Wipe / erase account** | `ssr` | [`actions::account::wipe_account`], [`identity_delete::erase_account`] | [`identity_delete`] |
//!
//! ## Also available
//!
//! One-time tokens ([`token_helpers`]), password policy / audit ([`security`]),
//! live verification status ([`verification`], [`events`]), durable Boson delivery
//! (`boson-delivery` → `delivery`), custom mail envelopes ([`EmailEnvelope`](../lepton_smtp/struct.EmailEnvelope.html)),
//! SMS adapters ([`lepton_sms`](../lepton_sms/index.html)), session bag keys
//! ([`session_binding`]), and referer helpers ([`routes`]). Module pages and the
//! sidebar list the full public surface.
//!
//! ## Typical verification flow (backend)
//!
//! 1. Inject SMTP/SMS via [`services::provide_auth_services`] at SSR boot.
//! 2. Issue challenges with [`factor::FactorChallengeService`] (feature-gated per channel).
//! 3. Consume tokens / codes; success marks the contact verified and publishes Photon.
//! 4. Refetch [`verification::verification_status`] with the challenge id.
//!
//! ## Features
//!
//! | Feature | Role |
//! |---------|------|
//! | `ssr` | Server functions, token/password helpers, Valence/higgs helpers |
//! | `email` | Verification / reset **mail** delivery (`lepton-smtp`); not email-as-login |
//! | `phone` | SMS OTP + `lepton-sms` adapters |
//! | `totp` / `two_factor` | TOTP enroll / verify (`totp-rs`) + QR SVG (`qrcode`) on SSR |
//! | `webauthn` | `WebAuthn` passkey ceremony for `AuthDevice` (`webauthn-rs`) |
//! | `oauth-google` | Live Google authorize URL + token/userinfo exchange; mock provider via `use_mock_provider` |
//! | `oauth-github` | Live GitHub authorize URL + token/user/emails exchange; mock provider via `use_mock_provider` |
//! | `full` | `email` + `phone` + `totp` + oauth flags + `webauthn` |
//! | `boson-delivery` | Durable email/SMS send + attempt log (`delivery` module, TTL 7d) |
//! | `spectra` | Emit auth funnel Spectra counters / `lepton_auth_failure` via `lepton-spectra-telemetry` (label tokens only; never emails, codes, or free-form errors) |
//! | `hydrate` | Client hydration helpers (`token_url`; hosts provide the session bridge) |
//! | `test-utils` | Photon publish capture for integration tests |
//!
//! Hosts that want today's mail behavior: `features = ["ssr", "email"]` (or `ssr,full`).
//! Account signup is available under `ssr` by default (UI in `lepton-auth-ui`). Private
//! hosts set `UF_LEPTON_SIGNUP_DISABLED=1` ([`signup_policy`]).
//!
//! ## Integration checklist
//!
//! 1. Call [`services::provide_auth_services`] at boot. Send paths return
//!    [`services::LeptonAuthServicesError::NotInContext`] when missing.
//! 2. Wire axum-login [`Backend`](../lepton_host_adapter/struct.Backend.html) + session
//!    middleware from `lepton-host-adapter`; provide higgs in the Leptos route context.
//! 3. Resolve secrets **outside** this crate (plain strings into builders). This crate
//!    does not load a secrets manager.
//! 4. Mount photon-leptos WS + Origin allowlist for live verification UI (host obligation).
//! 5. Validate SMTP with `infra/mailpit` when needed (`UF_MAILPIT=1`).
//! 6. For private deploys, set `UF_LEPTON_SIGNUP_DISABLED=1` and hide sign-up CTAs.
//! 7. Multi-instance hosts: persistent session store + shared OAuth CSRF (see kit `SECURITY.md`).
//!
//! Auth UI server functions in [`actions`] register through Leptos
//! `generate_route_list` / `leptos_routes_with_context`.
//!
//! ## Authenticated server fn (current user)
//!
//! Product server functions typically gate on the session user, then open a user-scoped
//! Valence. [`require_auth_user`] returns higgs context plus the signed-in
//! [`User`](../lepton_host_adapter/struct.User.html). [`user_valence`] builds Valence for
//! that actor. Use [`higgs_ctx`] when you need context without the auth gate.
//!
//! ```rust,ignore
//! use lepton_auth::{require_auth_user, user_valence};
//! use leptos::prelude::*;
//!
//! #[server]
//! async fn my_profile_id() -> Result<String, ServerFnError> {
//!     let (ctx, auth_user) = require_auth_user().await?;
//!     let _valence = user_valence(&ctx)?;
//!     Ok(auth_user.id.clone())
//! }
//! ```
//!
//! ## Boot delivery (email-only)
//!
//! Inject email once at SSR boot. Phone is optional via `.sms(...)` when the `phone`
//! feature is on.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use lepton_auth::services::{provide_auth_services, LeptonAuthServicesBuilder};
//! use lepton_smtp::{EmailServiceBuilder, SmtpConfig};
//!
//! // Cargo.toml: lepton-auth = { features = ["ssr", "email"] }
//!
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
//! provide_auth_services(
//!     Arc::new(
//!         LeptonAuthServicesBuilder::new()
//!             .email(email)
//!             .public_base_url("http://127.0.0.1:3000")
//!             .build()?,
//!     ),
//! );
//! ```
//!
//! ## Further reading
//!
//! - Crate [`README.md`](https://github.com/unified-field-dev/lepton/blob/main/lepton-auth/README.md)
//! - [`lepton_auth_ui`](../lepton_auth_ui/index.html) — dialogs and embeddable forms
//! - [`lepton_sms`](../lepton_sms/index.html) — SMS adapters
#![recursion_limit = "256"]

/// Server-side account settings overview + mutation logic.
pub mod account_api;
/// `#[server]` functions backing the auth UI.
pub mod actions;
/// Multi-email / multi-phone contacts and primary selection (`ssr`).
#[cfg(feature = "ssr")]
pub mod contacts;
/// Durable Boson delivery + [`DeliveryAttempt`](lepton_host_adapter::generated::DeliveryAttempt) log.
#[cfg(all(feature = "ssr", feature = "boson-delivery"))]
pub mod delivery;
/// Trusted browser / WebAuthn device APIs and DTOs.
pub mod devices;
/// Server-side email dispatch for verification/reset flows (`email` feature).
#[cfg(feature = "email")]
pub mod email_delivery;
/// Photon topics for verification completion (best-effort publish).
pub mod events;
/// Multi-factor challenge issue / verify (email OTP, SMS OTP, TOTP).
pub mod factor;
/// Guarded user/email/membership deletes and account erase (`ssr`).
#[cfg(feature = "ssr")]
pub mod identity_delete;
/// OAuth login / signup / link (`ssr`; mock provider + provider flags).
#[cfg(feature = "ssr")]
pub mod oauth;
/// Route path constants shared between server routes and client-side links.
pub mod paths;
/// Redirect/referer path parsing and sanitization for post-auth navigation.
pub mod routes;
/// Password policy checks, credential audit logging, and random tokens.
pub mod security;
/// Injected email/SMS/public-URL services for auth send paths.
#[cfg(feature = "ssr")]
pub mod services;
/// Session bag keys for AuthDevice binding (`ssr`).
#[cfg(feature = "ssr")]
pub mod session_binding;
/// Login MFA pending bag + complete / skip (`ssr`).
#[cfg(feature = "ssr")]
pub mod session_mfa;
/// Signup library API (`create_pending_user` + session `execute`).
#[cfg(feature = "ssr")]
pub mod signup_api;
/// Open-signup enablement (`UF_LEPTON_SIGNUP_DISABLED`).
pub mod signup_policy;
/// One-time token issuance and lifecycle checks (verification/reset).
pub mod token_helpers;
/// Client-side one-time token URL helpers (fragment-first; legacy query strip).
#[cfg(feature = "hydrate")]
pub mod token_url;
/// TOTP enroll / disable / recovery codes (`totp`).
#[cfg(all(feature = "ssr", feature = "totp"))]
pub mod totp;
/// Confirm / id-verify library APIs (`ssr`; no product UI).
#[cfg(feature = "ssr")]
pub mod trust;
/// Challenge-keyed verification status server fn (+ optional Photon synced refetch).
pub mod verification;
/// Browser `navigator.credentials` WebAuthn JSON helpers (`hydrate`).
#[cfg(feature = "hydrate")]
pub mod webauthn_browser;

#[cfg(all(feature = "ssr", feature = "spectra"))]
mod spectra_emit;

#[cfg(feature = "ssr")]
mod ssr_support;

#[cfg(feature = "ssr")]
pub use ssr_support::{extract_auth_user, higgs_ctx, require_auth_user, user_valence};

#[cfg(all(feature = "ssr", feature = "totp"))]
pub use factor::verify_totp_against_sealed;
#[cfg(feature = "ssr")]
pub use factor::{FactorChallengeError, FactorChallengeService};
#[cfg(feature = "ssr")]
pub use services::{
    auth_services, provide_auth_services, LeptonAuthServices, LeptonAuthServicesBuilder,
    LeptonAuthServicesError,
};
#[cfg(all(feature = "ssr", feature = "phone"))]
pub use token_helpers::{generate_phone_otp_code, IssuedPhoneChallenge, PHONE_OTP_DIGIT_LEN};

pub use events::VerificationKind;
pub use factor::FactorChallengeKind;
pub use verification::{verification_status, VerificationStatusSnapshot};
