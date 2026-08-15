//! Teaching example: confirm + id-verify library APIs (`no_run` style).
//!
//! ```bash
//! cargo check -p lepton-auth --example auth_trust_confirm --features ssr
//! ```

#![allow(dead_code)]

use lepton_auth::trust::{
    confirm_user, is_confirmed, is_id_verified, mark_user_id_verified, primary_email_verified,
    primary_phone_verified,
};
use valence::Valence;

async fn confirm_then_id_verify(
    v: &Valence,
    user: valence::RecordId,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(primary_email_verified(v, &user).await?);
    assert!(primary_phone_verified(v, &user).await?);

    confirm_user(v, &user).await?;
    assert!(is_confirmed(v, &user).await?);

    // No ID vendor yet — host/admin call after out-of-band checks.
    mark_user_id_verified(v, &user).await?;
    assert!(is_id_verified(v, &user).await?);
    Ok(())
}

fn main() {
    let _ = confirm_then_id_verify;
}
