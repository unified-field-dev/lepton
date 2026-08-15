//! Signup-time row ownership for identity models (`valence_data_ownership`).

use valence::owner_ref::{OwnerKind, OwnerRef};
use valence::ownership::{normalize_record_id_for_ownership, OwnershipService};
use valence::{RecordId, Result, Valence};

/// Bare record id segment for ownership lookups (strips `table:` when present).
pub fn bare_id_from_record(rid: &RecordId) -> String {
    valence::extract_id_from_record(rid)
        .unwrap_or_else(|_| normalize_record_id_for_ownership(rid.id()))
}

/// Assign founding-user ownership to identity rows created during anonymous signup.
///
/// `extra` holds additional `(valence_model, bare_record_id)` pairs (e.g. profile, membership).
pub async fn ensure_signup_identity_ownership(
    valence: &Valence,
    user_bare: &str,
    account_bare: &str,
    extra: &[(&str, &str)],
) -> Result<()> {
    let owner = OwnerRef {
        owner_id: user_bare.to_string(),
        owner_kind: OwnerKind::User,
    };

    for (table, bare) in [("account", account_bare), ("user", user_bare)]
        .into_iter()
        .chain(extra.iter().copied())
    {
        OwnershipService::ensure_active_ownership(table, bare, owner.clone(), valence).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use valence::RecordId;

    #[test]
    fn bare_id_from_record_strips_table_prefix_when_present() {
        let rid = RecordId::new("user", "alice");
        let bare = bare_id_from_record(&rid);
        assert!(!bare.is_empty());
        assert!(!bare.contains("user:"), "{bare}");
    }
}
