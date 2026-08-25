//! Message envelopes, stock helpers, and delivery receipts.
//!
//! Build an [`EmailEnvelope`] (hand-written or via a stock helper), pass it to
//! [`crate::EmailDeliveryService::send`], then inspect the [`DeliveryReceipt`].
//!
//! Stock helpers ([`verification_email_envelope`], [`password_reset_email_envelope`]) are the
//! usual next step after a backend guide ([Noop](crate#noop), [SMTP](crate#smtp-mailpit-or-relay)).
//! Runnable Noop path: `cargo run -p lepton-smtp --example noop_send`.

/// A single email to send via an [`crate::EmailDeliveryService`].
///
/// Set `to`, `subject`, `text_body`, and `html_body`. Stock helpers
/// ([`verification_email_envelope`], [`password_reset_email_envelope`]) fill subject and body
/// for common flows; customize fields when product copy differs.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailEnvelope {
    /// Recipient email address.
    pub to: String,
    /// Email subject line.
    pub subject: String,
    /// Plain-text body.
    pub text_body: String,
    /// HTML body.
    pub html_body: String,
}

/// Successful delivery outcome from an [`crate::EmailDeliveryService`].
///
/// Returned after a successful [`crate::EmailDeliveryService::send`]. `provider` names the
/// path that accepted the message (for example `noop`, `smtp`, or `direct_mx:<host>`).
/// `message_id` is set when the provider assigns one.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeliveryReceipt {
    /// Identifier of the provider/path that delivered the message (e.g. `smtp`, `noop`,
    /// or `direct_mx:<host>`).
    pub provider: String,
    /// Provider-assigned message id, if any.
    pub message_id: Option<String>,
}

/// Which auth-shaped flow triggered a verification email (selects the subject line).
#[derive(Clone, Copy, Debug)]
pub enum VerificationEmailFlow {
    /// Initial signup verification.
    Signup,
    /// User requested the verification code be resent.
    Resend,
    /// User is confirming a changed email address.
    ChangeEmail,
}

impl VerificationEmailFlow {
    const fn subject(self) -> &'static str {
        match self {
            Self::Signup | Self::Resend => "Your verification code",
            Self::ChangeEmail => "Confirm your new email address",
        }
    }
}

/// Local-part of `email` (before `@`), or the full string when `@` is missing.
#[must_use]
pub fn greeting_name_from_email(email: &str) -> &str {
    email
        .split_once('@')
        .map(|(local, _)| local)
        .filter(|local| !local.is_empty())
        .unwrap_or(email)
}

/// Build an [`EmailEnvelope`] for a verification-code email for the given `flow`.
///
/// `recipient_name` is the greeting name. When `None`, the local-part of
/// `recipient_email` is used. The body is code-first; clickable verify URLs are
/// deferred until a host route exists.
///
/// # Examples
///
/// ```
/// use lepton_smtp::{verification_email_envelope, VerificationEmailFlow};
///
/// let envelope = verification_email_envelope(
///     "user@example.test",
///     "tok123",
///     VerificationEmailFlow::Signup,
/// );
/// assert_eq!(envelope.subject, "Your verification code");
/// assert!(envelope.text_body.contains("tok123"));
/// ```
#[must_use]
pub fn verification_email_envelope(
    recipient_email: &str,
    verification_code: &str,
    flow: VerificationEmailFlow,
) -> EmailEnvelope {
    verification_email_envelope_named(recipient_email, None, verification_code, flow)
}

/// Like [`verification_email_envelope`], with an explicit greeting name.
#[must_use]
pub fn verification_email_envelope_named(
    recipient_email: &str,
    recipient_name: Option<&str>,
    verification_code: &str,
    flow: VerificationEmailFlow,
) -> EmailEnvelope {
    let name = recipient_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| greeting_name_from_email(recipient_email));
    let code = verification_code.trim();

    EmailEnvelope {
        to: recipient_email.to_string(),
        subject: flow.subject().to_string(),
        text_body: format!(
            "Hello {name},\n\nYour verification code is:\n\n{code}\n\nEnter this code to verify your email. If you did not ask for this, you can ignore the message.\n"
        ),
        html_body: format!(
            "<p>Hello {name},</p><p>Your verification code is:</p><p style=\"font-size:1.25rem;font-weight:600;letter-spacing:0.05em\"><code>{code}</code></p><p>Enter this code to verify your email. If you did not ask for this, you can ignore the message.</p>"
        ),
    }
}

/// Build an [`EmailEnvelope`] for a password-reset-link email.
#[must_use]
pub fn password_reset_email_envelope(recipient_email: &str, reset_link: &str) -> EmailEnvelope {
    let name = greeting_name_from_email(recipient_email);
    EmailEnvelope {
        to: recipient_email.to_string(),
        subject: "Reset your password".to_string(),
        text_body: format!(
            "Hello {name},\n\nA password reset was requested for your account.\n\nOpen this link to continue:\n\n{reset_link}\n\nIf you did not request a password reset, you can ignore this message.\n"
        ),
        html_body: format!(
            "<p>Hello {name},</p><p>A password reset was requested for your account.</p><p><a href=\"{reset_link}\">Reset password</a></p><p>If you did not request a password reset, you can ignore this message.</p>"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_verification_email_code_message() {
        let envelope = verification_email_envelope(
            "user@example.com",
            "deadbeefcafe",
            VerificationEmailFlow::Signup,
        );
        assert_eq!(envelope.to, "user@example.com");
        assert_eq!(envelope.subject, "Your verification code");
        assert!(envelope.text_body.contains("Hello user,"));
        assert!(envelope.text_body.contains("deadbeefcafe"));
        assert!(!envelope.text_body.contains("http"));
        assert!(envelope.html_body.contains("<code>deadbeefcafe</code>"));
        assert!(!envelope.html_body.contains("href="));
    }

    #[test]
    fn verification_email_uses_explicit_name() {
        let envelope = verification_email_envelope_named(
            "user@example.com",
            Some("Sam"),
            "abc123",
            VerificationEmailFlow::Resend,
        );
        assert!(envelope.text_body.contains("Hello Sam,"));
        assert_eq!(envelope.subject, "Your verification code");
    }

    #[test]
    fn builds_password_reset_email_message() {
        let envelope = password_reset_email_envelope(
            "user@example.com",
            "http://127.0.0.1:3000/auth/reset/confirm?token=abc",
        );
        assert_eq!(envelope.subject, "Reset your password");
        assert!(envelope.text_body.contains("Hello user,"));
        assert!(envelope.text_body.contains("password reset"));
        assert!(envelope.html_body.contains("Reset password"));
    }

    #[test]
    fn greeting_name_from_email_happy() {
        assert_eq!(greeting_name_from_email("sam@example.com"), "sam");
        assert_eq!(greeting_name_from_email("no-at"), "no-at");
    }
}
