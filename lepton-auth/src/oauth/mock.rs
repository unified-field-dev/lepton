//! In-process OAuth mock (no live Google/GitHub HTTP).

use super::api::OAuthProvider;
use super::error::OAuthError;

/// Resolve a mock authorization code into `(provider_subject, email_hint, name_hint)`.
///
/// Special codes:
/// - `no-email` / `noemail:*` — signup without an email hint (primaries stay unset).
pub(super) fn exchange_mock_code(
    provider: OAuthProvider,
    code: &str,
) -> Result<(String, Option<String>, Option<String>), OAuthError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(OAuthError::Provider);
    }
    let subject = format!("mock:{}:{code}", provider.as_str());
    let name_hint = Some("Mock User".to_string());
    if code == "no-email" || code.starts_with("noemail:") {
        return Ok((subject, None, name_hint));
    }
    let email_hint = Some(format!("{code}@oauth.mock.test"));
    Ok((subject, email_hint, name_hint))
}
