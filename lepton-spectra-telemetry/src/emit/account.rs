//! Account lifecycle counter.

use crate::helpers::LeptonAccountRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Account operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountOperation {
    /// Change password.
    ChangePassword,
    /// Request email change.
    ChangeEmailRequest,
    /// Resend verification.
    ResendVerification,
    /// Confirm account (`confirmed_at`).
    Confirm,
    /// Wipe account.
    Wipe,
    /// Logout.
    Logout,
}

impl AccountOperation {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChangePassword => "change_password",
            Self::ChangeEmailRequest => "change_email_request",
            Self::ResendVerification => "resend_verification",
            Self::Confirm => "confirm",
            Self::Wipe => "wipe",
            Self::Logout => "logout",
        }
    }
}

/// Best-effort bump of `lepton_account{operation,outcome,error_class}`.
pub fn record_account(
    operation: AccountOperation,
    outcome: AuthOutcome,
    error_class: &'static str,
) {
    let error_class = bound_error_class(error_class);
    LeptonAccountRecorder::record(
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
    fn record_account_maps_labels_happy() {
        assert_eq!(AccountOperation::ChangePassword.as_str(), "change_password");
        assert_eq!(AccountOperation::Confirm.as_str(), "confirm");
        record_account(AccountOperation::Logout, AuthOutcome::Success, "none");
        record_account(AccountOperation::Confirm, AuthOutcome::Success, "none");
    }

    #[test]
    fn record_account_unknown_bounded_sad() {
        record_account(
            AccountOperation::ChangePassword,
            AuthOutcome::Failure,
            "password=secret",
        );
    }

    #[test]
    fn record_account_without_spectra_soft_happy() {
        record_account(
            AccountOperation::Wipe,
            AuthOutcome::Failure,
            "confirm_phrase",
        );
    }
}
