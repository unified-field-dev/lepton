//! Step-up TOTP + password re-check before a sensitive operation (library composition).
//!
//! Hosts call [`require_auth_user`](lepton_auth::require_auth_user), then
//! [`FactorChallengeService::verify_totp_code`](lepton_auth::FactorChallengeService::verify_totp_code)
//! and/or [`verify_token_secret`](lepton_auth::token_helpers::verify_token_secret) before the mutation.
//! Client modal: [`StepUpDialog`](lepton_auth_ui::StepUpDialog). This binary is a compile/run
//! sketch for the password re-check path; TOTP enroll/verify against Valence is shown as
//! dead-code helpers — pair with the UI crate example for a full critical-action flow.
//!
//! ```bash
//! CARGO_BUILD_JOBS=1 cargo check -p lepton-auth --example step_up_totp --features "ssr,totp"
//! CARGO_BUILD_JOBS=1 cargo run -p lepton-auth --example step_up_totp --features "ssr,totp"
//! ```
//!
//! Success: stderr prints `step_up_totp: OK — password re-check + TOTP step-up sketch`.

#![allow(clippy::print_stderr, dead_code)]

use lepton_auth::factor::FactorChallengeService;
use lepton_auth::security::password_policy_error_message;
use lepton_auth::token_helpers::verify_token_secret;
use lepton_auth::totp::{begin_totp_enroll, confirm_totp_enroll};

/// Password re-check against a stored PHC hash (same primitive as login).
fn password_recheck(presented: &str, stored_phc: &str) -> Result<(), Box<dyn std::error::Error>> {
    verify_token_secret(presented, stored_phc)
        .map_err(|e| std::io::Error::other(e.message()).into())
}

/// After session gate + TOTP verify, run the sensitive mutation.
async fn sensitive_op_after_totp(
    v: &valence::Valence,
    user: valence::RecordId,
    svc: &FactorChallengeService,
    totp_code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    svc.verify_totp_code(v, &user, totp_code).await?;
    // Host: delete billing method, change email, etc.
    Ok(())
}

/// Enroll then step-up (host would load an already-enrolled factor in production).
async fn enroll_then_step_up(
    v: &valence::Valence,
    user: valence::RecordId,
    svc: &FactorChallengeService,
    code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pending = begin_totp_enroll(v, &user, "user", "Lepton").await?;
    confirm_totp_enroll(v, &user, &pending.factor_id, code).await?;
    sensitive_op_after_totp(v, user, svc, code).await
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = "CorrectHorseBattery1!";
    assert!(password_policy_error_message(password).is_none());
    let stored = lepton_host_adapter::auth::hash_password(password)?;
    password_recheck(password, &stored)?;

    let _ = enroll_then_step_up;
    let _ = sensitive_op_after_totp;

    eprintln!("step_up_totp: OK — password re-check + TOTP step-up sketch");
    Ok(())
}
