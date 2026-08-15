//! Emit helpers must not panic when Spectra is not booted.

#![allow(clippy::unwrap_used)]

use lepton_spectra_telemetry::{
    log_auth_failure, record_account, record_email_send, record_signin, record_signup,
    record_sms_send, AccountOperation, AuthFactor, AuthFailureFlow, AuthOutcome, EmailSendOutcome,
    SigninStage, SmsSendOutcome,
};

#[test]
fn record_without_spectra_soft_happy() {
    record_email_send("noop", EmailSendOutcome::Success);
    record_email_send("smtp", EmailSendOutcome::Failure);
    record_sms_send("noop", SmsSendOutcome::Success);
    record_sms_send("test", SmsSendOutcome::Failure);
    record_signup(AuthOutcome::Success, "none");
    record_signin(
        SigninStage::Password,
        AuthOutcome::Failure,
        "invalid_credentials",
        AuthFactor::None,
    );
    record_account(
        AccountOperation::Wipe,
        AuthOutcome::Failure,
        "confirm_phrase",
    );
    log_auth_failure(
        AuthFailureFlow::Account,
        "wipe",
        "confirm_phrase",
        None,
        None,
    );
}
