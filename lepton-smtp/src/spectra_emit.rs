//! Best-effort Spectra delivery counters (`feature = "spectra"`).

use lepton_spectra_telemetry::{record_email_send, EmailSendOutcome};

/// Record a terminal email send outcome (no PII labels).
pub fn record_terminal(driver: &str, ok: bool) {
    record_email_send(
        driver,
        if ok {
            EmailSendOutcome::Success
        } else {
            EmailSendOutcome::Failure
        },
    );
}
