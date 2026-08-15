//! Typed errors for email configuration and delivery.

/// Errors from email configuration or delivery.
///
/// Display strings include a `reason_class=` token where useful for ops triage.
/// Messages never include passwords, message bodies, or full recipient mailboxes
/// beyond what the underlying transport already echoed (sanitize at the host if needed).
#[derive(Debug, thiserror::Error)]
pub enum EmailDeliveryError {
    /// Missing or invalid configuration (env vars, addresses, builder fields, …).
    #[error("{0}")]
    ConfigError(String),
    /// Transport-level failure (connection, timeout, protocol error).
    #[error("{0}")]
    TransportError(String),
    /// The receiving provider rejected the message.
    #[error("{0}")]
    ProviderRejected(String),
    /// A transient failure that may succeed on retry.
    #[error("{0}")]
    Transient(String),
}

impl EmailDeliveryError {
    /// Build a [`ConfigError`](Self::ConfigError) with an explicit `reason_class`.
    pub(crate) fn config(reason_class: &str, message: impl Into<String>) -> Self {
        Self::ConfigError(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// Build a [`TransportError`](Self::TransportError) with an explicit `reason_class`.
    pub(crate) fn transport(reason_class: &str, message: impl Into<String>) -> Self {
        Self::TransportError(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// Build a [`Transient`](Self::Transient) with an explicit `reason_class`.
    pub(crate) fn transient(reason_class: &str, message: impl Into<String>) -> Self {
        Self::Transient(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// Build a [`ProviderRejected`](Self::ProviderRejected) with an explicit `reason_class`.
    #[cfg(feature = "twilio")]
    pub(crate) fn rejected(reason_class: &str, message: impl Into<String>) -> Self {
        Self::ProviderRejected(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// True when this failure is retryable (transient).
    #[must_use]
    pub const fn is_transient(&self) -> bool {
        matches!(self, Self::Transient(_))
    }

    /// Parse the `reason_class=` token from the display string, when present.
    #[must_use]
    pub fn reason_class(&self) -> Option<&str> {
        parse_reason_class(self.as_message())
    }

    const fn as_message(&self) -> &str {
        match self {
            Self::ConfigError(s)
            | Self::TransportError(s)
            | Self::ProviderRejected(s)
            | Self::Transient(s) => s.as_str(),
        }
    }
}

/// Extract `reason_class` from a `reason_class=…:` message fragment.
fn parse_reason_class(message: &str) -> Option<&str> {
    let rest = message.strip_prefix("reason_class=")?;
    let end = rest.find(':').unwrap_or(rest.len());
    let class = rest.get(..end)?.trim();
    if class.is_empty() {
        None
    } else {
        Some(class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_error_display_omits_secrets_happy_path() {
        let err = EmailDeliveryError::config(
            "missing_password",
            "username set without password (value omitted)",
        );
        let s = err.to_string();
        assert!(s.contains("reason_class=missing_password"));
        assert!(!s.contains("hunter2"));
        assert!(!s.to_lowercase().contains("body"));
    }

    #[test]
    fn email_is_transient_happy_path() {
        let err = EmailDeliveryError::transient("rate_limited", "retry later");
        assert!(err.is_transient());
        assert_eq!(err.reason_class(), Some("rate_limited"));
    }

    #[test]
    fn email_is_transient_permanent_sad() {
        let err = EmailDeliveryError::ProviderRejected(
            "reason_class=bad_request: provider said no".into(),
        );
        assert!(!err.is_transient());
        assert_eq!(err.reason_class(), Some("bad_request"));
    }
}
