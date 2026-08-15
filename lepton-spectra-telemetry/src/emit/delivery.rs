//! Bounded-label emit helpers for delivery counters.
//!
//! Prefer these over calling [`LeptonEmailSendRecorder`](crate::LeptonEmailSendRecorder) with
//! free-form label strings. Unknown drivers map to `unknown` (low cardinality).

use crate::helpers::{LeptonEmailSendRecorder, LeptonSmsSendRecorder};

/// Terminal email send outcome (not `start` / `attempt`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmailSendOutcome {
    /// Adapter returned `Ok`.
    Success,
    /// Adapter returned `Err`.
    Failure,
}

impl EmailSendOutcome {
    /// Label value for Spectra (`success` / `failure`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

/// Terminal SMS send outcome (not `start` / `attempt`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmsSendOutcome {
    /// Adapter returned `Ok`.
    Success,
    /// Adapter returned `Err`.
    Failure,
}

impl SmsSendOutcome {
    /// Label value for Spectra (`success` / `failure`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

const EMAIL_DRIVERS: &[&str] = &["smtp", "direct_mx", "noop", "twilio"];
const SMS_DRIVERS: &[&str] = &["noop", "test", "twilio", "twilio_verify"];

/// Map a driver string to an allowlisted label (or `unknown`).
#[must_use]
pub fn bound_email_driver(driver: &str) -> &'static str {
    let trimmed = driver.trim();
    EMAIL_DRIVERS
        .iter()
        .copied()
        .find(|&d| d.eq_ignore_ascii_case(trimmed))
        .unwrap_or("unknown")
}

/// Map a driver string to an allowlisted label (or `unknown`).
#[must_use]
pub fn bound_sms_driver(driver: &str) -> &'static str {
    let trimmed = driver.trim();
    SMS_DRIVERS
        .iter()
        .copied()
        .find(|&d| d.eq_ignore_ascii_case(trimmed))
        .unwrap_or("unknown")
}

/// Best-effort bump of `lepton_email_send{driver,outcome}`.
///
/// Infallible: when Spectra is not installed, the recorder soft-fails.
///
/// # Examples
///
/// ```rust,no_run
/// use lepton_spectra_telemetry::{record_email_send, EmailSendOutcome};
///
/// record_email_send("smtp", EmailSendOutcome::Success);
/// ```
pub fn record_email_send(driver: &str, outcome: EmailSendOutcome) {
    let driver = bound_email_driver(driver);
    LeptonEmailSendRecorder::record(
        1,
        serde_json::json!({
            "driver": driver,
            "outcome": outcome.as_str(),
        }),
    );
}

/// Best-effort bump of `lepton_sms_send{driver,outcome}`.
///
/// # Examples
///
/// ```rust,no_run
/// use lepton_spectra_telemetry::{record_sms_send, SmsSendOutcome};
///
/// record_sms_send("twilio", SmsSendOutcome::Failure);
/// ```
pub fn record_sms_send(driver: &str, outcome: SmsSendOutcome) {
    let driver = bound_sms_driver(driver);
    LeptonSmsSendRecorder::record(
        1,
        serde_json::json!({
            "driver": driver,
            "outcome": outcome.as_str(),
        }),
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn record_email_send_maps_success_labels_happy() {
        assert_eq!(bound_email_driver("smtp"), "smtp");
        assert_eq!(bound_email_driver("SMTP"), "smtp");
        assert_eq!(EmailSendOutcome::Success.as_str(), "success");
    }

    #[test]
    fn record_email_send_maps_failure_labels_happy() {
        assert_eq!(EmailSendOutcome::Failure.as_str(), "failure");
        assert_eq!(bound_email_driver("direct_mx"), "direct_mx");
        assert_eq!(bound_email_driver("noop"), "noop");
        assert_eq!(bound_email_driver("twilio"), "twilio");
    }

    #[test]
    fn record_sms_send_maps_labels_happy() {
        assert_eq!(bound_sms_driver("noop"), "noop");
        assert_eq!(bound_sms_driver("test"), "test");
        assert_eq!(bound_sms_driver("twilio_verify"), "twilio_verify");
        assert_eq!(SmsSendOutcome::Success.as_str(), "success");
        assert_eq!(SmsSendOutcome::Failure.as_str(), "failure");
    }

    #[test]
    fn record_email_send_unknown_driver_bounded_sad() {
        assert_eq!(bound_email_driver("user@example.com"), "unknown");
        assert_eq!(bound_email_driver("custom-relay"), "unknown");
        assert_eq!(bound_email_driver(""), "unknown");
    }

    #[test]
    fn record_sms_send_unknown_driver_bounded_sad() {
        assert_eq!(bound_sms_driver("+15551234567"), "unknown");
        assert_eq!(bound_sms_driver("other"), "unknown");
    }

    #[test]
    fn record_email_send_without_spectra_soft_happy() {
        // No Spectra::builder installed — must not panic.
        record_email_send("noop", EmailSendOutcome::Success);
        record_email_send("unknown-driver", EmailSendOutcome::Failure);
    }

    #[test]
    fn record_sms_send_without_spectra_soft_happy() {
        record_sms_send("noop", SmsSendOutcome::Success);
        record_sms_send("bogus", SmsSendOutcome::Failure);
    }
}
