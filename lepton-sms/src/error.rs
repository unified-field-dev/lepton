//! Typed errors for SMS configuration and delivery.

/// Errors from SMS configuration or delivery.
///
/// Display strings include a `reason_class=` token where useful. Messages never include
/// OTP bodies or full E.164 numbers beyond validation failure class.
#[derive(Debug, thiserror::Error)]
pub enum SmsDeliveryError {
    /// Missing or invalid configuration (E.164, builder fields, …).
    #[error("{0}")]
    ConfigError(String),
    /// Transport-level failure from an adapter.
    #[error("{0}")]
    TransportError(String),
    /// The provider rejected the message.
    #[error("{0}")]
    Rejected(String),
    /// A transient failure that may succeed on retry.
    #[error("{0}")]
    Transient(String),
}

impl SmsDeliveryError {
    /// Build a [`ConfigError`](Self::ConfigError) with an explicit `reason_class`.
    pub(crate) fn config(reason_class: &str, message: impl Into<String>) -> Self {
        Self::ConfigError(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// Build a [`TransportError`](Self::TransportError) with an explicit `reason_class`.
    pub(crate) fn transport(reason_class: &str, message: impl Into<String>) -> Self {
        Self::TransportError(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// Build a [`Rejected`](Self::Rejected) with an explicit `reason_class`.
    pub(crate) fn rejected(reason_class: &str, message: impl Into<String>) -> Self {
        Self::Rejected(format!("reason_class={reason_class}: {}", message.into()))
    }

    /// Build a [`Transient`](Self::Transient) with an explicit `reason_class`.
    pub(crate) fn transient(reason_class: &str, message: impl Into<String>) -> Self {
        Self::Transient(format!("reason_class={reason_class}: {}", message.into()))
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
            | Self::Rejected(s)
            | Self::Transient(s) => s.as_str(),
        }
    }
}

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
    fn sms_is_transient_happy_path() {
        let err = SmsDeliveryError::Transient("reason_class=rate_limited: retry later".into());
        assert!(err.is_transient());
        assert_eq!(err.reason_class(), Some("rate_limited"));
    }

    #[test]
    fn sms_is_transient_permanent_sad() {
        let err = SmsDeliveryError::Rejected("reason_class=bad_request: provider said no".into());
        assert!(!err.is_transient());
        assert_eq!(err.reason_class(), Some("bad_request"));
    }
}
