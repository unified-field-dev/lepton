//! Credential smoke + real SMTP send to Mailpit (mailpit).
//!
//! Requires Mailpit on `127.0.0.1:1025` (see `infra/mailpit`). Prefer
//! `./infra/mailpit/smtp_smoke.sh` which runs the validating integ test; this
//! example is a teaching binary for builder → SMTP receipt.
//!
//! ## Command
//! ```bash
//! docker compose -f infra/mailpit/docker-compose.yml up -d
//! CARGO_BUILD_JOBS=1 \
//!   cargo run -p lepton-auth --example auth_flows_smtp_mailpit --features ssr,email
//! ```
//!
//! ## Success
//! Stdout prints `auth_flows_smtp_mailpit: OK` and Mailpit UI at
//! `http://127.0.0.1:8025` shows the messages.

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
    SmtpConfig, VerificationEmailFlow,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let password = "CorrectHorseBattery1!";
    anyhow::ensure!(password_policy_error_message(password).is_none());
    let password_hash = hash_password(password)?;
    verify_token_secret(password, &password_hash)
        .map_err(|e| anyhow::anyhow!("login verify failed: {}", e.message()))?;

    let marker = random_token_part(8);
    let email = format!("demo+{marker}@example.test");
    let verify_token = random_token_part(16);
    let reset_token = random_token_part(16);
    let base = "http://127.0.0.1:3000";
    let reset_link = build_public_token_url(base, RESET_PASSWORD_CONFIRM, &reset_token);

    let service = EmailServiceBuilder::new()
        .smtp(
            SmtpConfig::builder()
                .host("127.0.0.1")
                .port(1025)
                .use_tls(false)
                .from_email("noreply@example.test")
                .from_name("Lepton Auth")
                .build()?,
        )
        .build()?;
    anyhow::ensure!(service.driver() == EmailDriver::Smtp);

    let verify_receipt = service
        .send(&verification_email_envelope(
            &email,
            &verify_token,
            VerificationEmailFlow::Signup,
        ))
        .await?;
    anyhow::ensure!(verify_receipt.provider == "smtp");

    let reset_receipt = service
        .send(&password_reset_email_envelope(&email, &reset_link))
        .await?;
    anyhow::ensure!(reset_receipt.provider == "smtp");

    println!(
        "auth_flows_smtp_mailpit: OK — provider=smtp marker={marker}; inspect http://127.0.0.1:8025"
    );
    Ok(())
}
