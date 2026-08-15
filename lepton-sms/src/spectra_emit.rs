//! Best-effort Spectra delivery counters (`feature = "spectra"`).

use lepton_spectra_telemetry::{record_sms_send, SmsSendOutcome};

/// Record a terminal SMS send outcome (no PII labels).
pub fn record_terminal(driver: &str, ok: bool) {
    record_sms_send(
        driver,
        if ok {
            SmsSendOutcome::Success
        } else {
            SmsSendOutcome::Failure
        },
    );
}
