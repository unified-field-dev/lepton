//! Password policy checks, credential audit logging, and random token generation.
//!
//! # When to call
//!
//! | Task | API |
//! |------|-----|
//! | Password policy | [`crate::security::password_policy_error_message`], [`crate::security::password_policy_errors`] |
//! | Legal name policy | [`crate::security::legal_name_policy_error`] |
//! | Display name policy | [`crate::security::display_name_policy_error`] |
//! | Random token entropy (`ssr`) | `random_token_part` |
//! | Password re-check vs stored PHC (`ssr`) | `token_helpers::verify_token_secret` |
//!
//! # Examples
//!
//! ```rust
//! use lepton_auth::security::password_policy_error_message;
//!
//! assert!(password_policy_error_message("CorrectHorseBattery1!").is_none());
//! assert!(password_policy_error_message("short").is_some());
//! ```
//!
//! Password re-check before a sensitive mutation (after [`crate::require_auth_user`]):
//!
//! ```rust,ignore
//! use lepton_auth::token_helpers::verify_token_secret;
//!
//! verify_token_secret(&presented_password, &user.password_hash)?;
//! // … continue with the mutation …
//! ```
//!
//! Runnable: `examples/password_and_token`, `examples/step_up_totp`.

#[cfg(feature = "ssr")]
use rand_core::RngCore;

/// Minimum accepted password length in characters.
pub const PASSWORD_MIN_LENGTH: usize = 12;

/// Maximum accepted legal name length (matches Valence `MaxLength(255)`).
pub const LEGAL_NAME_MAX_LENGTH: usize = 255;

/// Maximum accepted display name length (matches Valence `MaxLength(255)`).
pub const DISPLAY_NAME_MAX_LENGTH: usize = 255;

/// Returns a stable user-facing error when `legal_name` fails policy, or `None` when valid.
///
/// Allowed: Unicode letters and combining marks, spaces, `'`, `’`, `-`, `.`.
/// Empty / control / other symbols (including `<` / digits) are rejected.
#[must_use]
pub fn legal_name_policy_error(legal_name: &str) -> Option<&'static str> {
    let trimmed = legal_name.trim();
    if trimmed.is_empty() {
        return Some("Legal name is required");
    }
    if trimmed.chars().count() > LEGAL_NAME_MAX_LENGTH {
        return Some("Legal name is too long");
    }

    let mut has_letter = false;
    for c in trimmed.chars() {
        if c.is_alphabetic() {
            has_letter = true;
            continue;
        }
        if is_unicode_mark(c) || matches!(c, ' ' | '\'' | '\u{2019}' | '-' | '.') {
            continue;
        }
        return Some("Legal name contains invalid characters");
    }
    if !has_letter {
        return Some("Legal name must include letters");
    }
    None
}

/// Returns a stable user-facing error when `display_name` fails policy, or `None` when valid.
///
/// Non-empty after trim, max [`DISPLAY_NAME_MAX_LENGTH`], no control characters or
/// HTML-meta characters (`<`, `>`, `"`, `` ` ``). Letters/digits/spaces/common punctuation allowed.
#[must_use]
pub fn display_name_policy_error(display_name: &str) -> Option<&'static str> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        return Some("Display name is required");
    }
    if trimmed.chars().count() > DISPLAY_NAME_MAX_LENGTH {
        return Some("Display name is too long");
    }
    for c in trimmed.chars() {
        if c.is_control() || matches!(c, '<' | '>' | '"' | '`') {
            return Some("Display name contains invalid characters");
        }
    }
    None
}

/// Combining marks commonly seen after letters in personal names (NFC leftovers).
const fn is_unicode_mark(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'..='\u{036F}' // Combining Diacritical Marks
            | '\u{1AB0}'..='\u{1AFF}' // Combining Diacritical Marks Extended
            | '\u{1DC0}'..='\u{1DFF}' // Combining Diacritical Marks Supplement
            | '\u{20D0}'..='\u{20FF}' // Combining Diacritical Marks for Symbols
            | '\u{FE20}'..='\u{FE2F}' // Combining Half Marks
    )
}

/// A single password requirement and whether a given password satisfies it.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordRequirementResult {
    /// Human-readable description of the requirement.
    pub label: &'static str,
    /// Whether the checked password satisfies this requirement.
    pub satisfied: bool,
}

/// Evaluate `password` against each individual policy requirement (length, case,
/// digit, special character).
pub fn password_requirement_results(password: &str) -> Vec<PasswordRequirementResult> {
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());
    let has_min_len = password.chars().count() >= PASSWORD_MIN_LENGTH;

    vec![
        PasswordRequirementResult {
            label: "At least 12 characters",
            satisfied: has_min_len,
        },
        PasswordRequirementResult {
            label: "At least one uppercase letter (A-Z)",
            satisfied: has_upper,
        },
        PasswordRequirementResult {
            label: "At least one lowercase letter (a-z)",
            satisfied: has_lower,
        },
        PasswordRequirementResult {
            label: "At least one number (0-9)",
            satisfied: has_digit,
        },
        PasswordRequirementResult {
            label: "At least one special character",
            satisfied: has_special,
        },
    ]
}

/// Labels of the unmet requirements for `password`, if any.
pub fn password_policy_errors(password: &str) -> Vec<String> {
    password_requirement_results(password)
        .into_iter()
        .filter(|item| !item.satisfied)
        .map(|item| item.label.to_string())
        .collect()
}

/// A combined, user-facing error message listing unmet password requirements, or `None`
/// if `password` satisfies the policy.
pub fn password_policy_error_message(password: &str) -> Option<String> {
    let failures = password_policy_errors(password);
    if failures.is_empty() {
        return None;
    }

    Some(format!(
        "Password does not meet requirements: {}",
        failures.join("; ")
    ))
}

/// Mask an email for audit logs (`j***@example.com`). Never log full addresses.
#[must_use]
pub fn mask_email_for_audit(email: &str) -> String {
    let trimmed = email.trim();
    let Some((local, domain)) = trimmed.split_once('@') else {
        return "***".to_string();
    };
    let first = local.chars().next().unwrap_or('*');
    format!("{first}***@{domain}")
}

/// Emit a structured `[audit][credential]` log line for a credential-related event
/// (login, password change, etc.). Emails are masked — never full addresses.
#[cfg(feature = "ssr")]
pub fn log_credential_audit(event: &str, email: Option<&str>, outcome: &str, detail: Option<&str>) {
    let email_display = email.map_or_else(|| "<unknown>".to_string(), mask_email_for_audit);
    let detail_display = detail.unwrap_or("-");
    leptos::logging::log!(
        "[audit][credential] event={} email={} outcome={} detail={}",
        event,
        email_display,
        outcome,
        detail_display
    );
}

/// Generate a random hex string of `byte_len` bytes (`2 * byte_len` hex characters) using
/// the OS RNG, for use as a one-time token secret.
#[cfg(feature = "ssr")]
pub fn random_token_part(byte_len: usize) -> String {
    use std::fmt::Write;

    let mut bytes = vec![0u8; byte_len];
    rand_core::OsRng.fill_bytes(&mut bytes);
    let mut out = String::with_capacity(byte_len * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        display_name_policy_error, legal_name_policy_error, mask_email_for_audit,
        password_policy_error_message, password_requirement_results, PASSWORD_MIN_LENGTH,
    };

    #[test]
    fn legal_name_accepts_apostrophe_and_hyphen() {
        assert!(legal_name_policy_error("Alex Rivera").is_none());
        assert!(legal_name_policy_error("Mary-Jane").is_none());
        assert!(legal_name_policy_error("Jean-Luc Picard").is_none());
        assert!(legal_name_policy_error("O’Rourke").is_none()); // curly apostrophe
        assert!(legal_name_policy_error("José").is_none());
    }

    #[test]
    fn legal_name_rejects_empty_xss_and_digits() {
        assert!(legal_name_policy_error("").is_some());
        assert!(legal_name_policy_error("   ").is_some());
        assert!(legal_name_policy_error("Name<script>").is_some());
        assert!(legal_name_policy_error("Agent007").is_some());
        assert!(legal_name_policy_error("A".repeat(256).as_str()).is_some());
    }

    #[test]
    fn display_name_accepts_common_labels() {
        assert!(display_name_policy_error("Alex").is_none());
        assert!(display_name_policy_error("Alex42").is_none());
        assert!(display_name_policy_error("Ada Lovelace").is_none());
    }

    #[test]
    fn display_name_rejects_empty_and_html_meta() {
        assert!(display_name_policy_error("").is_some());
        assert!(display_name_policy_error("   ").is_some());
        assert!(display_name_policy_error("Name<script>").is_some());
        assert!(display_name_policy_error("A".repeat(256).as_str()).is_some());
    }

    #[test]
    fn mask_email_for_audit_hides_local_part() {
        assert_eq!(
            mask_email_for_audit("alice@example.com"),
            "a***@example.com"
        );
        assert_eq!(mask_email_for_audit("not-an-email"), "***");
    }

    #[test]
    fn password_requirement_results_empty_password_all_unmet() {
        let results = password_requirement_results("");
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|item| !item.satisfied));
    }

    #[test]
    fn password_requirement_results_strong_password_all_met() {
        let results = password_requirement_results("ValidPass123!");
        assert!(results.iter().all(|item| item.satisfied));
    }

    #[test]
    fn password_requirement_results_partial_satisfaction() {
        let results = password_requirement_results("short1!");
        let has_unmet_min_len = results
            .iter()
            .any(|item| item.label.contains(&PASSWORD_MIN_LENGTH.to_string()) && !item.satisfied);
        assert!(has_unmet_min_len);
    }

    #[test]
    fn password_policy_error_message_none_when_valid() {
        assert!(password_policy_error_message("ValidPass123!").is_none());
    }

    #[test]
    fn password_policy_error_message_lists_failures() {
        if let Some(message) = password_policy_error_message("short") {
            assert!(message.contains("Password does not meet requirements"));
        } else {
            panic!("expected policy error");
        }
    }
}
