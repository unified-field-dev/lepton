//! Normalize human-entered phone numbers to E.164 for storage and SMS.

use thiserror::Error;

/// Failed to turn user input into a valid E.164 phone number.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PhoneNormalizeError {
    /// Empty or whitespace-only input.
    #[error("reason_class=invalid_phone: enter a phone number")]
    Empty,
    /// Could not parse into E.164 (`+[country][subscriber]`).
    #[error("reason_class=invalid_phone: enter a valid phone number")]
    Invalid,
}

impl PhoneNormalizeError {
    /// Stable reason class for ops / UI.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        "invalid_phone"
    }
}

/// Convert common phone spellings into E.164 (`+[1-9]` + up to 14 digits).
///
/// Accepts already-valid E.164, `+` with punctuation, and US/Canada 10-digit
/// national numbers (and `1` + 10 digits) which become `+1…`.
///
/// # Errors
///
/// [`PhoneNormalizeError`] when the value cannot be normalized.
pub fn normalize_phone_to_e164(input: &str) -> Result<String, PhoneNormalizeError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PhoneNormalizeError::Empty);
    }

    let has_plus = trimmed.starts_with('+');
    let digits: String = trimmed.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return Err(PhoneNormalizeError::Invalid);
    }

    let candidate = if has_plus {
        format!("+{digits}")
    } else if digits.len() == 10 {
        // US/Canada national without country code.
        format!("+1{digits}")
    } else if digits.len() == 11 && digits.starts_with('1') {
        format!("+{digits}")
    } else if digits.len() >= 8 && digits.len() <= 15 && !digits.starts_with('0') {
        // International without leading + (already includes country code).
        format!("+{digits}")
    } else {
        return Err(PhoneNormalizeError::Invalid);
    };

    if is_valid_e164(&candidate) {
        Ok(candidate)
    } else {
        Err(PhoneNormalizeError::Invalid)
    }
}

fn is_valid_e164(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 3 || bytes.len() > 16 || bytes[0] != b'+' {
        return false;
    }
    let rest = &bytes[1..];
    if !(b'1'..=b'9').contains(&rest[0]) {
        return false;
    }
    rest.iter().all(u8::is_ascii_digit) && rest.len() <= 15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_e164() {
        assert_eq!(
            normalize_phone_to_e164("+15551234567").unwrap(),
            "+15551234567"
        );
    }

    #[test]
    fn strips_punctuation_with_plus() {
        assert_eq!(
            normalize_phone_to_e164("+1 (555) 123-4567").unwrap(),
            "+15551234567"
        );
    }

    #[test]
    fn us_ten_digit_gets_plus_one() {
        assert_eq!(
            normalize_phone_to_e164("5551234567").unwrap(),
            "+15551234567"
        );
        assert_eq!(
            normalize_phone_to_e164("(555) 123-4567").unwrap(),
            "+15551234567"
        );
    }

    #[test]
    fn eleven_digit_leading_one() {
        assert_eq!(
            normalize_phone_to_e164("15551234567").unwrap(),
            "+15551234567"
        );
    }

    #[test]
    fn rejects_empty_and_garbage() {
        assert_eq!(
            normalize_phone_to_e164("").unwrap_err(),
            PhoneNormalizeError::Empty
        );
        assert_eq!(
            normalize_phone_to_e164("   ").unwrap_err(),
            PhoneNormalizeError::Empty
        );
        assert_eq!(
            normalize_phone_to_e164("not-a-phone").unwrap_err(),
            PhoneNormalizeError::Invalid
        );
        assert_eq!(
            normalize_phone_to_e164("123").unwrap_err(),
            PhoneNormalizeError::Invalid
        );
    }
}
