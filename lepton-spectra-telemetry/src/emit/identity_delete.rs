//! Guarded identity-delete counter.

use crate::helpers::LeptonIdentityDeleteRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Identity-delete operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityDeleteOperation {
    /// Erase whole account.
    EraseAccount,
    /// Delete user.
    DeleteUser,
    /// Delete membership.
    DeleteMembership,
    /// Delete email contact.
    DeleteEmail,
    /// Delete phone contact.
    DeletePhone,
}

impl IdentityDeleteOperation {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EraseAccount => "erase_account",
            Self::DeleteUser => "delete_user",
            Self::DeleteMembership => "delete_membership",
            Self::DeleteEmail => "delete_email",
            Self::DeletePhone => "delete_phone",
        }
    }
}

/// Best-effort bump of `lepton_identity_delete{operation,outcome,error_class}`.
pub fn record_identity_delete(
    operation: IdentityDeleteOperation,
    outcome: AuthOutcome,
    error_class: &'static str,
) {
    let error_class = bound_error_class(error_class);
    LeptonIdentityDeleteRecorder::record(
        1,
        serde_json::json!({
            "operation": operation.as_str(),
            "outcome": outcome.as_str(),
            "error_class": error_class,
        }),
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn record_identity_delete_maps_labels_happy() {
        assert_eq!(
            IdentityDeleteOperation::EraseAccount.as_str(),
            "erase_account"
        );
        record_identity_delete(
            IdentityDeleteOperation::EraseAccount,
            AuthOutcome::Success,
            "none",
        );
    }

    #[test]
    fn record_identity_delete_unknown_bounded_sad() {
        record_identity_delete(
            IdentityDeleteOperation::DeleteEmail,
            AuthOutcome::Failure,
            "user@x.test",
        );
    }

    #[test]
    fn record_identity_delete_without_spectra_soft_happy() {
        record_identity_delete(
            IdentityDeleteOperation::DeleteUser,
            AuthOutcome::Failure,
            "restrict_primary",
        );
    }
}
