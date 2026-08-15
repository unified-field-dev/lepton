//! `redirect_uri` allowlist helpers for the mock IdP.

use url::Url;

/// True when `redirect_uri` is allowed for lab OAuth.
///
/// Accepts:
/// - loopback HTTP(S) (`127.0.0.1`, `localhost`, `::1`)
/// - relative paths starting with `/` (treated as same-app)
///
/// Rejects other absolute hosts (open-redirect guard).
#[must_use]
pub fn redirect_uri_allowed(redirect_uri: &str) -> bool {
    let trimmed = redirect_uri.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('/') && !trimmed.starts_with("//") {
        return true;
    }
    let Ok(url) = Url::parse(trimmed) else {
        return false;
    };
    match url.scheme() {
        "http" | "https" => {}
        _ => return false,
    }
    matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_loopback_and_path_happy() {
        assert!(redirect_uri_allowed(
            "http://127.0.0.1:3000/auth/oauth/callback"
        ));
        assert!(redirect_uri_allowed("http://localhost:3000/cb"));
        assert!(redirect_uri_allowed("/auth/oauth/callback"));
    }

    #[test]
    fn redirect_external_host_sad() {
        assert!(!redirect_uri_allowed("https://evil.example/phish"));
        assert!(!redirect_uri_allowed(""));
        assert!(!redirect_uri_allowed("//evil.example/x"));
    }
}
