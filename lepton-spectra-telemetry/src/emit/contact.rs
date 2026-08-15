//! Contact / primary counter.

use crate::helpers::LeptonContactRecorder;

use super::common::{bound_error_class, AuthOutcome};
use super::verify::VerifyChannel;

/// Contact operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactOperation {
    /// Add backup contact.
    Add,
    /// Promote to primary.
    SetPrimary,
    /// Mark verified.
    MarkVerified,
    /// Delete contact.
    Delete,
}

impl ContactOperation {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::SetPrimary => "set_primary",
            Self::MarkVerified => "mark_verified",
            Self::Delete => "delete",
        }
    }
}

/// Best-effort bump of `lepton_contact{channel,operation,outcome,error_class}`.
pub fn record_contact(
    channel: VerifyChannel,
    operation: ContactOperation,
    outcome: AuthOutcome,
    error_class: &'static str,
) {
    let error_class = bound_error_class(error_class);
    LeptonContactRecorder::record(
        1,
        serde_json::json!({
            "channel": channel.as_str(),
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
    fn record_contact_maps_labels_happy() {
        assert_eq!(ContactOperation::SetPrimary.as_str(), "set_primary");
        record_contact(
            VerifyChannel::Email,
            ContactOperation::Add,
            AuthOutcome::Success,
            "none",
        );
    }

    #[test]
    fn record_contact_unknown_bounded_sad() {
        record_contact(
            VerifyChannel::Phone,
            ContactOperation::Delete,
            AuthOutcome::Failure,
            "user@x.test",
        );
    }

    #[test]
    fn record_contact_without_spectra_soft_happy() {
        record_contact(
            VerifyChannel::Email,
            ContactOperation::MarkVerified,
            AuthOutcome::Failure,
            "address_taken",
        );
    }
}
