//! Interactive live TOTP enroll + challenge with a real authenticator app.
//!
//! Creates a confirmed user (Noop email + Test SMS; no Twilio). Prints an `otpauth://`
//! URI for Google Authenticator, then prompts for enroll and challenge codes.
//!
//! Gate: `UF_LEPTON_LIVE_TOTP=1`.

use std::process::ExitCode;
use std::sync::Arc;

use lepton_e2e::{
    boot_lab, run_device_totp_challenge_flow, run_signup_verify_flow, LiveVerifyError,
    SignupVerifyOpts, StdinTotpCodeSource, TestCodeSource,
};
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => {
            println!(
                "lepton-live-totp: OK — test user confirmed, device trusted, TOTP enrolled + challenged"
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("lepton-live-totp: FAIL {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), LiveVerifyError> {
    if std::env::var("UF_LEPTON_LIVE_TOTP").ok().as_deref() != Some("1") {
        return Err(LiveVerifyError::config("UF_LEPTON_LIVE_TOTP must be 1"));
    }

    let legal_name = std::env::var("UF_LIVE_VERIFY_LEGAL_NAME")
        .map_err(|_| LiveVerifyError::config("missing UF_LIVE_VERIFY_LEGAL_NAME"))?;
    let email = std::env::var("UF_LIVE_VERIFY_EMAIL")
        .map_err(|_| LiveVerifyError::config("missing UF_LIVE_VERIFY_EMAIL"))?;
    let phone = std::env::var("UF_LIVE_VERIFY_PHONE")
        .map_err(|_| LiveVerifyError::config("missing UF_LIVE_VERIFY_PHONE"))?;
    let password = std::env::var("UF_LIVE_VERIFY_PASSWORD")
        .map_err(|_| LiveVerifyError::config("missing UF_LIVE_VERIFY_PASSWORD"))?;
    let device_label = std::env::var("UF_LIVE_VERIFY_DEVICE_LABEL")
        .unwrap_or_else(|_| "Live TOTP Browser".to_string());
    let issuer =
        std::env::var("UF_LIVE_VERIFY_TOTP_ISSUER").unwrap_or_else(|_| "Lepton Auth".to_string());

    let span = tracing::info_span!("lepton_e2e.live_totp");
    let _guard = span.enter();

    info!(phase = "signup", "live_totp");
    let lab = boot_lab("lepton_live_totp").await?;
    let signup_codes = TestCodeSource::new(Arc::clone(&lab.test_sms));

    println!(
        "Creating test account for {} / {} (no Twilio) …",
        email.trim(),
        phone.trim()
    );
    let signup = run_signup_verify_flow(
        &lab.valence,
        &lab.services,
        &signup_codes,
        legal_name.trim(),
        email.trim(),
        phone.trim(),
        password.trim(),
        SignupVerifyOpts::default(),
    )
    .await?;

    if !(signup.email_verified && signup.phone_verified && signup.confirmed) {
        return Err(LiveVerifyError::ConfirmBlocked);
    }

    info!(phase = "device", "live_totp");
    info!(phase = "totp", "live_totp");
    println!("Signup confirmed. Next: device trust (automatic) + authenticator enroll.");
    let outcome = run_device_totp_challenge_flow(
        &lab.valence,
        &lab.services,
        &signup.user_id,
        device_label.trim(),
        email.trim(),
        issuer.trim(),
        &StdinTotpCodeSource,
    )
    .await?;

    if !(outcome.device_trusted && outcome.totp_enabled && outcome.challenge_ok) {
        return Err(LiveVerifyError::totp("mismatch"));
    }

    info!(phase = "done", "live_totp");
    Ok(())
}
