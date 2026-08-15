//! Step-up factor verify counter.

use crate::helpers::LeptonStepUpRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Step-up path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepUpPath {
    /// TOTP path.
    Totp,
    /// Bound-device path.
    BoundDevice,
    /// Rejected / no path.
    Reject,
}

impl StepUpPath {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Totp => "totp",
            Self::BoundDevice => "bound_device",
            Self::Reject => "reject",
        }
    }
}

/// Best-effort bump of `lepton_step_up{path,outcome,error_class}`.
pub fn record_step_up(path: StepUpPath, outcome: AuthOutcome, error_class: &'static str) {
    let error_class = bound_error_class(error_class);
    LeptonStepUpRecorder::record(
        1,
        serde_json::json!({
            "path": path.as_str(),
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
    fn record_step_up_maps_labels_happy() {
        assert_eq!(StepUpPath::BoundDevice.as_str(), "bound_device");
        record_step_up(StepUpPath::Totp, AuthOutcome::Success, "none");
    }

    #[test]
    fn record_step_up_unknown_bounded_sad() {
        record_step_up(StepUpPath::Reject, AuthOutcome::Failure, "otp=999999");
    }

    #[test]
    fn record_step_up_without_spectra_soft_happy() {
        record_step_up(
            StepUpPath::BoundDevice,
            AuthOutcome::Failure,
            "device_binding",
        );
    }
}
