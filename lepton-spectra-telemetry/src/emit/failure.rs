//! Auth failure event emit (ops-id fields only).

use crate::helpers::LeptonAuthFailureLogger;

use super::common::{
    bound_error_class, bound_optional_channel, bound_optional_provider, AuthFailureFlow,
};

/// Best-effort `lepton_auth_failure` event.
///
/// Fields are allowlisted enums / `reason_class` tokens only — never passwords, PII,
/// tokens, or error Display text.
pub fn log_auth_failure(
    flow: AuthFailureFlow,
    operation: &'static str,
    error_class: &'static str,
    provider: Option<&'static str>,
    channel: Option<&'static str>,
) {
    let error_class = bound_error_class(error_class);
    let operation = bound_operation(operation);
    let provider = bound_optional_provider(provider).to_string();
    let channel = bound_optional_channel(channel).to_string();
    LeptonAuthFailureLogger::log(
        flow.as_str().to_string(),
        operation.to_string(),
        error_class.to_string(),
        provider,
        channel,
    );
}

/// Bound operation tokens used on failure events (closed set + flow-specific ops).
fn bound_operation(raw: &str) -> &'static str {
    const OPS: &[&str] = &[
        "signup",
        "signin",
        "begin",
        "complete",
        "issue",
        "consume",
        "request",
        "confirm",
        "begin_enroll",
        "confirm_enroll",
        "disable",
        "regenerate_recovery",
        "verify",
        "register",
        "revoke",
        "assert_begin",
        "assert_finish",
        "list",
        "add",
        "set_primary",
        "mark_verified",
        "delete",
        "change_password",
        "change_email_request",
        "resend_verification",
        "wipe",
        "logout",
        "erase_account",
        "delete_user",
        "delete_membership",
        "delete_email",
        "delete_phone",
        "step_up",
        "password",
        "mfa_pending",
        "mfa_complete",
        "session",
        "none",
        "unknown",
    ];
    let trimmed = raw.trim();
    OPS.iter()
        .copied()
        .find(|&o| o.eq_ignore_ascii_case(trimmed))
        .unwrap_or("unknown")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn log_auth_failure_maps_fields_happy() {
        assert_eq!(bound_operation("change_password"), "change_password");
        log_auth_failure(AuthFailureFlow::Signup, "signup", "validation", None, None);
    }

    #[test]
    fn log_auth_failure_pii_shapes_bounded_sad() {
        assert_eq!(bound_operation("user@x.test"), "unknown");
        assert_eq!(bound_optional_provider(Some("user@x.test")), "unknown");
        assert_eq!(bound_optional_channel(Some("+15551234567")), "unknown");
        log_auth_failure(
            AuthFailureFlow::Account,
            "user@x.test",
            "password=secret",
            Some("user@x.test"),
            Some("+15551234567"),
        );
    }

    #[test]
    fn log_auth_failure_without_spectra_soft_happy() {
        log_auth_failure(
            AuthFailureFlow::Oauth,
            "complete",
            "oauth_state",
            Some("google"),
            None,
        );
    }

    #[test]
    fn spectra_labels_forbid_pii_shapes_sad() {
        let error = bound_error_class("alice@example.com");
        let provider = bound_optional_provider(Some("password=hunter2"));
        let channel = bound_optional_channel(Some("+15551234567"));
        let op = bound_operation("123456");
        let payload = serde_json::json!({
            "flow": "signup",
            "operation": op,
            "error_class": error,
            "provider": provider,
            "channel": channel,
        });
        let s = payload.to_string();
        assert!(
            !s.contains('@'),
            "payload must not retain email shapes: {s}"
        );
        assert!(
            !s.contains("+1"),
            "payload must not retain phone shapes: {s}"
        );
        assert!(
            !s.contains("password="),
            "payload must not retain password= shapes: {s}"
        );
        assert_eq!(error, "unknown");
        assert_eq!(provider, "unknown");
        assert_eq!(channel, "unknown");
        assert_eq!(op, "unknown");
    }
}
