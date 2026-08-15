//! Verified phone seed helpers.

use lepton_auth::contacts::{
    add_account_phone, mark_account_phone_verified, set_account_primary_phone, set_primary_phone,
};
use valence::{RecordId, Valence};

use crate::error::SeedError;

pub(super) async fn seed_verified_phone(
    valence: &Valence,
    user_id: &RecordId,
    account_id: &RecordId,
    e164: &str,
) -> Result<(), SeedError> {
    let phone = add_account_phone(valence, account_id, e164).await?;
    mark_account_phone_verified(valence, &phone).await?;
    let phone_id = phone.id().cloned().ok_or(SeedError::Persistence {
        operation: "phone_id",
    })?;
    set_account_primary_phone(valence, account_id, &phone_id).await?;
    set_primary_phone(valence, user_id, &phone_id).await?;
    Ok(())
}
