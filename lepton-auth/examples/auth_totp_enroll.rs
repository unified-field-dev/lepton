//! Teaching example B2: TOTP enroll (`totp` feature).
//!
//! ```bash
//! cargo check -p lepton-auth --example auth_totp_enroll --features "ssr,totp"
//! ```

#![allow(dead_code)]

use lepton_auth::factor::FactorChallengeService;
use lepton_auth::totp::{
    begin_totp_enroll, confirm_totp_enroll, consume_totp_recovery_code, disable_totp,
    regenerate_totp_recovery_codes,
};

async fn enroll_then_disable_totp(
    v: &valence::Valence,
    user: valence::RecordId,
    svc: &FactorChallengeService,
    code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pending = begin_totp_enroll(v, &user, "user", "Unified Field").await?;
    // Host shows `pending.otpauth_uri` as a QR (product: `actions::totp` + Account Settings).
    confirm_totp_enroll(v, &user, &pending.factor_id, code).await?;
    let codes = regenerate_totp_recovery_codes(v, &user).await?;
    // MFA sign-in can use a recovery code once (`complete_sign_in_totp` dual-path).
    consume_totp_recovery_code(v, &user, &codes[0]).await?;
    svc.verify_totp_code(v, &user, code).await?;
    disable_totp(v, &user).await?;
    Ok(())
}

fn main() {
    let _ = enroll_then_disable_totp;
}
