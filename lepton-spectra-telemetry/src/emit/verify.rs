//! Email / phone / TOTP verification counter.

use crate::helpers::LeptonVerifyRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Verification channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyChannel {
    /// Email OTP / token.
    Email,
    /// Phone OTP.
    Phone,
    /// TOTP code.
    Totp,
}

impl VerifyChannel {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Totp => "totp",
        }
    }
}

/// Verification stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyStage {
    /// Challenge issued.
    Issue,
    /// Challenge consumed / verified.
    Consume,
}

impl VerifyStage {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Issue => "issue",
            Self::Consume => "consume",
        }
    }
}

const VERIFY_CHANNELS: &[&str] = &["email", "phone", "totp"];

/// Map a channel string to an allowlisted label (or `unknown`).
#[must_use]
pub fn bound_verify_channel(raw: &str) -> &'static str {
    let trimmed = raw.trim();
    VERIFY_CHANNELS
        .iter()
        .copied()
        .find(|&c| c.eq_ignore_ascii_case(trimmed))
        .unwrap_or("unknown")
}

/// Best-effort bump of `lepton_verify{channel,stage,outcome,error_class}`.
pub fn record_verify(
    channel: VerifyChannel,
    stage: VerifyStage,
    outcome: AuthOutcome,
    error_class: &'static str,
) {
    let error_class = bound_error_class(error_class);
    LeptonVerifyRecorder::record(
        1,
        serde_json::json!({
            "channel": channel.as_str(),
            "stage": stage.as_str(),
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
    fn record_verify_maps_labels_happy() {
        assert_eq!(bound_verify_channel("email"), "email");
        record_verify(
            VerifyChannel::Email,
            VerifyStage::Issue,
            AuthOutcome::Success,
            "none",
        );
    }

    #[test]
    fn record_verify_unknown_channel_bounded_sad() {
        assert_eq!(bound_verify_channel("+15551234567"), "unknown");
        assert_eq!(bound_verify_channel("user@x.test"), "unknown");
        record_verify(
            VerifyChannel::Phone,
            VerifyStage::Consume,
            AuthOutcome::Failure,
            "mismatch",
        );
    }

    #[test]
    fn record_verify_without_spectra_soft_happy() {
        record_verify(
            VerifyChannel::Totp,
            VerifyStage::Consume,
            AuthOutcome::Failure,
            "expired",
        );
    }
}
