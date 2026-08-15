//! Signup funnel counter.

use crate::helpers::LeptonSignupRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Best-effort bump of `lepton_signup{outcome,error_class}`.
pub fn record_signup(outcome: AuthOutcome, error_class: &'static str) {
    let error_class = bound_error_class(error_class);
    LeptonSignupRecorder::record(
        1,
        serde_json::json!({
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
    fn record_signup_maps_labels_happy() {
        assert_eq!(AuthOutcome::Success.as_str(), "success");
        assert_eq!(bound_error_class("email_exists"), "email_exists");
        record_signup(AuthOutcome::Success, "none");
    }

    #[test]
    fn record_signup_unknown_error_class_bounded_sad() {
        assert_eq!(bound_error_class("user@x.test"), "unknown");
        record_signup(AuthOutcome::Failure, "user@x.test");
    }

    #[test]
    fn record_signup_without_spectra_soft_happy() {
        record_signup(AuthOutcome::Failure, "validation");
    }
}
