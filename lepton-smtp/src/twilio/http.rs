//! `SendGrid` Mail Send helpers (status mapping; no PII in errors).

use crate::error::EmailDeliveryError;

/// Map an HTTP status from `SendGrid` Mail Send into a typed delivery error.
#[must_use]
pub(super) fn map_http_status(status: u16) -> EmailDeliveryError {
    match status {
        401 => EmailDeliveryError::rejected(
            "auth_failed",
            format!("SendGrid rejected API key (HTTP {status})"),
        ),
        // Often an unverified From / missing sender auth, not a bad API key.
        403 => EmailDeliveryError::rejected(
            "sender_forbidden",
            format!(
                "SendGrid forbade send (HTTP {status}); verify UF_EMAIL_FROM as a Single Sender or complete Domain Authentication"
            ),
        ),
        429 => EmailDeliveryError::transient(
            "rate_limited",
            format!("SendGrid rate limited the request (HTTP {status})"),
        ),
        400 | 404 | 413 | 422 => EmailDeliveryError::rejected(
            "provider_rejected",
            format!("SendGrid rejected the message (HTTP {status})"),
        ),
        s if (500..600).contains(&s) => EmailDeliveryError::transient(
            "provider_unavailable",
            format!("SendGrid unavailable (HTTP {status})"),
        ),
        _ => EmailDeliveryError::transport(
            "transport_error",
            format!("SendGrid returned unexpected HTTP {status}"),
        ),
    }
}

/// Build the Mail Send URL.
#[must_use]
pub(super) fn mail_send_url(api_base_url: &str) -> String {
    format!("{}/v3/mail/send", api_base_url.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_http_status_auth_failed_sad() {
        let err = map_http_status(401);
        assert!(matches!(err, EmailDeliveryError::ProviderRejected(_)));
        assert!(err.to_string().contains("reason_class=auth_failed"));
    }

    #[test]
    fn map_http_status_rate_limited_sad() {
        let err = map_http_status(429);
        assert!(matches!(err, EmailDeliveryError::Transient(_)));
        assert!(err.to_string().contains("reason_class=rate_limited"));
    }
}
