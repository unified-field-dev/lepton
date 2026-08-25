//! SMS envelope and delivery receipt types.
//!
//! Build an [`SmsEnvelope`], pass it to [`crate::SmsDeliveryService::send`], then inspect
//! the [`SmsDeliveryReceipt`]. Destination numbers must pass [`validate_e164`]. Teaching
//! paths start at the crate-root [Noop guide](crate#noop).

use crate::error::SmsDeliveryError;

/// A single SMS to send via an [`crate::SmsDeliveryService`].
///
/// Set `to_e164` (E.164), `body`, and optionally `otp_code` (required for Twilio Verify
/// `CustomCode`; Messages / Noop / Test may ignore it).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmsEnvelope {
    /// Destination phone number in E.164 form (`+[country][subscriber]`).
    pub to_e164: String,
    /// Message body (e.g. OTP text). Never log this field.
    pub body: String,
    /// OTP for channels that need a discrete code (Twilio Verify `CustomCode`).
    ///
    /// Test/Noop/Messages adapters may ignore this; Verify requires 4..=10 characters.
    pub otp_code: Option<String>,
}

/// Successful SMS delivery outcome.
///
/// Returned after a successful [`crate::SmsDeliveryService::send`]. `provider` names the
/// path that accepted the message (for example `noop`, `test`, `http_capture`, `twilio`,
/// or `twilio_verify`). `message_id` is set when the provider assigns one.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SmsDeliveryReceipt {
    /// Adapter / provider identifier (e.g. `noop`, `test`).
    pub provider: String,
    /// Provider-assigned message id, if any.
    pub message_id: Option<String>,
}

/// True when `value` matches Valence `Validator::Phone` E.164
/// (`+[1-9]` then 1..=14 more digits).
fn is_e164(value: &str) -> bool {
    let Some(digits) = value.strip_prefix('+') else {
        return false;
    };
    let bytes = digits.as_bytes();
    // `[1-9]\d{1,14}` → 2..=15 digit characters after `+`.
    if !(2..=15).contains(&bytes.len()) {
        return false;
    }
    if !(b'1'..=b'9').contains(&bytes[0]) {
        return false;
    }
    bytes[1..].iter().all(u8::is_ascii_digit)
}

/// Validate that `to_e164` is E.164 (`+[country digit 1-9][subscriber]`, max 15 digits).
///
/// Same rule as Valence `Validator::Phone` / `validate_phone`. Kept local so
/// `lepton-sms` stays free of a Valence dependency.
///
/// # Errors
///
/// Returns [`SmsDeliveryError::ConfigError`] with `reason_class=invalid_e164` when
/// invalid. The message never includes the number.
pub fn validate_e164(to_e164: &str) -> Result<(), SmsDeliveryError> {
    let trimmed = to_e164.trim();
    if is_e164(trimmed) {
        Ok(())
    } else {
        Err(SmsDeliveryError::config(
            "invalid_e164",
            "to_e164 must be E.164 (+[1-9] and 1..=14 more digits)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_e164_accepts_valid() {
        assert!(validate_e164("+15551234567").is_ok());
        assert!(validate_e164("+442071234567").is_ok());
        assert!(validate_e164("  +15555550100  ").is_ok());
    }

    #[test]
    fn validate_e164_rejects_invalid() {
        for bad in [
            "",
            "+",
            "+0",
            "+0123",
            "15551234567",
            "+1234567890123456",
            "555",
        ] {
            let err = validate_e164(bad).expect_err(bad);
            let msg = err.to_string();
            assert!(msg.contains("reason_class=invalid_e164"), "{bad}: {msg}");
            // Message describes the rule; it must not paste the caller value.
            assert!(
                !msg.contains("15551234567")
                    && !msg.contains("+0123")
                    && !msg.contains("+1234567890123456"),
                "must not echo sample inputs: {msg}"
            );
        }
    }
}
