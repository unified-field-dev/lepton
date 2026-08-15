//! Sign-up / login / password-reset primitives with a local Noop SMTP sink.
//!
//! Full Leptos SSR hosts call server functions that persist users via Valence and
//! `axum-login`. This example teaches the credential + email path without a DB:
//! password policy, PHC hash verify (login), reset/verification envelopes, and
//! builder-constructed noop delivery (no process-env hot path).
//!
//! ## When to use
//! Local auth flow smoke before wiring host product SSR.
//!
//! ## Command
//! ```bash
//! CARGO_BUILD_JOBS=1 \
//!   cargo run -p lepton-auth --example auth_flows_noop_smtp --features ssr,email
//! ```
//!
//! ## Success
//! Stdout prints `auth_flows_noop_smtp: OK — signup/login/reset + noop SMTP`.
//!
//! ## Look next
//! `lepton-smtp` `SmtpAdapter` via [`EmailServiceBuilder`]; host auth pages; Higgs SSR host.

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "ssr")]

use lepton_auth::{
    paths::RESET_PASSWORD_CONFIRM,
    routes::build_public_token_url,
    security::{password_policy_error_message, random_token_part},
    token_helpers::verify_token_secret,
};
use lepton_host_adapter::auth::hash_password;
use lepton_smtp::{
    password_reset_email_envelope, verification_email_envelope, EmailDriver, EmailServiceBuilder,
    VerificationEmailFlow,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Sign-up: policy + store PHC hash (hosts persist via Valence) ---
    let password = "CorrectHorseBattery1!";
    anyhow::ensure!(password_policy_error_message(password).is_none());
    let password_hash = hash_password(password)?;

    // --- Login: verify candidate against stored hash ---
    verify_token_secret(password, &password_hash)
        .map_err(|e| anyhow::anyhow!("login verify failed: {}", e.message()))?;
    let bad = verify_token_secret("wrong-password", &password_hash);
    anyhow::ensure!(bad.is_err(), "bad password must fail");

    // --- Verification + reset mail via local Noop sink (builder-first) ---
    let email = "demo@example.test";
    let verify_token = random_token_part(16);
    let reset_token = random_token_part(16);
    let base = "http://127.0.0.1:3000";
    let reset_link = build_public_token_url(base, RESET_PASSWORD_CONFIRM, &reset_token);

    let service = EmailServiceBuilder::new().noop().build()?;
    anyhow::ensure!(service.driver() == EmailDriver::Noop);

    let verify_receipt = service
        .send(&verification_email_envelope(
            email,
            &verify_token,
            VerificationEmailFlow::Signup,
        ))
        .await?;
    anyhow::ensure!(verify_receipt.provider == "noop");

    let reset_receipt = service
        .send(&password_reset_email_envelope(email, &reset_link))
        .await?;
    anyhow::ensure!(reset_receipt.provider == "noop");

    println!("auth_flows_noop_smtp: OK — signup/login/reset + noop SMTP");
    Ok(())
}
