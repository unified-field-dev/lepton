//! Teaching example B1: contacts + confirm (library API; `no_run` style).
//!
//! ```bash
//! cargo check -p lepton-auth --example auth_contacts_confirm --features ssr
//! ```

#![allow(dead_code)]

use lepton_auth::contacts::{
    add_account_email, add_account_phone, mark_account_email_verified, mark_account_phone_verified,
    set_account_primary_email, set_account_primary_phone, set_primary_email, set_primary_phone,
};
use lepton_auth::trust::{confirm_user, primary_email_verified, primary_phone_verified};
use valence::Valence;

async fn promote_backup_and_confirm(
    v: &Valence,
    user: valence::RecordId,
    account: valence::RecordId,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(primary_email_verified(v, &user).await?);

    let backup = add_account_email(v, &account, "backup@example.com").await?;
    mark_account_email_verified(v, &backup).await?;
    let backup_id = backup.id().cloned().ok_or("backup missing id")?;
    set_primary_email(v, &user, &backup_id).await?;
    set_account_primary_email(v, &account, &backup_id).await?;

    let phone = add_account_phone(v, &account, "+15555550100").await?;
    mark_account_phone_verified(v, &phone).await?;
    let phone_id = phone.id().cloned().ok_or("phone missing id")?;
    set_primary_phone(v, &user, &phone_id).await?;
    set_account_primary_phone(v, &account, &phone_id).await?;
    assert!(primary_phone_verified(v, &user).await?);

    confirm_user(v, &user).await?;
    Ok(())
}

fn main() {
    let _ = promote_backup_and_confirm;
}
