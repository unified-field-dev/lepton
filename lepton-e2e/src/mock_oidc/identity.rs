//! Mock subject / email contract (parity with `lepton_auth::oauth` in-process mock).

/// Resolve a mock authorization code into `(provider_subject, email_hint, name_hint)`.
///
/// Special codes:
/// - `no-email` / `noemail:*` — signup without an email hint.
///
/// # Errors
///
/// Returns `Err("empty_code")` when `code` is empty after trim.
pub fn identity_from_code(
    provider: &str,
    code: &str,
) -> Result<(String, Option<String>, Option<String>), &'static str> {
    let code = code.trim();
    if code.is_empty() {
        return Err("empty_code");
    }
    let subject = format!("mock:{provider}:{code}");
    let name_hint = Some("Mock User".to_string());
    if code == "no-email" || code.starts_with("noemail:") {
        return Ok((subject, None, name_hint));
    }
    let email_hint = Some(format!("{code}@oauth.mock.test"));
    Ok((subject, email_hint, name_hint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_subject_email_matches_exchange_mock_code_happy() {
        let cases = [
            ("google", "mock-code", true),
            ("github", "abc", true),
            ("google", "no-email", false),
            ("google", "noemail:x", false),
        ];
        for (provider, code, expect_email) in cases {
            let (subject, email, name) = identity_from_code(provider, code).expect("ok");
            assert_eq!(subject, format!("mock:{provider}:{code}"));
            assert_eq!(name.as_deref(), Some("Mock User"));
            if expect_email {
                assert_eq!(email, Some(format!("{code}@oauth.mock.test")));
            } else {
                assert!(email.is_none());
            }
        }
    }

    #[test]
    fn identity_empty_code_sad() {
        assert_eq!(identity_from_code("google", "  "), Err("empty_code"));
    }
}
