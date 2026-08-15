//! `TrustedBrowser` / `WebAuthn` device counter.

use crate::helpers::LeptonDeviceRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Device kind label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    /// Trusted browser cookie device.
    TrustedBrowser,
    /// `WebAuthn` passkey.
    Webauthn,
}

impl DeviceKind {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrustedBrowser => "trusted_browser",
            Self::Webauthn => "webauthn",
        }
    }
}

/// Device operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceOperation {
    /// Register / begin enroll.
    Register,
    /// Confirm enroll.
    Confirm,
    /// Revoke device.
    Revoke,
    /// Begin assertion.
    AssertBegin,
    /// Finish assertion.
    AssertFinish,
    /// List devices.
    List,
}

impl DeviceOperation {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Confirm => "confirm",
            Self::Revoke => "revoke",
            Self::AssertBegin => "assert_begin",
            Self::AssertFinish => "assert_finish",
            Self::List => "list",
        }
    }
}

/// Best-effort bump of `lepton_device{device_kind,operation,outcome,error_class}`.
pub fn record_device(
    device_kind: DeviceKind,
    operation: DeviceOperation,
    outcome: AuthOutcome,
    error_class: &'static str,
) {
    let error_class = bound_error_class(error_class);
    LeptonDeviceRecorder::record(
        1,
        serde_json::json!({
            "device_kind": device_kind.as_str(),
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
    fn record_device_maps_labels_happy() {
        assert_eq!(DeviceKind::TrustedBrowser.as_str(), "trusted_browser");
        record_device(
            DeviceKind::Webauthn,
            DeviceOperation::Register,
            AuthOutcome::Success,
            "none",
        );
    }

    #[test]
    fn record_device_unknown_bounded_sad() {
        record_device(
            DeviceKind::TrustedBrowser,
            DeviceOperation::Confirm,
            AuthOutcome::Failure,
            "device_id=xyz",
        );
    }

    #[test]
    fn record_device_without_spectra_soft_happy() {
        record_device(
            DeviceKind::Webauthn,
            DeviceOperation::AssertFinish,
            AuthOutcome::Failure,
            "ceremony_invalid",
        );
    }
}
