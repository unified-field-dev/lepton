//! QR SVG + manual-entry helpers for otpauth URIs (SSR).

use super::TotpEnrollError;

/// Format a base32 secret for manual authenticator entry (groups of 4).
#[must_use]
pub fn format_manual_secret(secret: &str) -> String {
    let compact: String = secret
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let mut out = String::with_capacity(compact.len() + compact.len() / 4);
    for (i, ch) in compact.chars().enumerate() {
        if i > 0 && i % 4 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

/// Extract the base32 `secret=` value from an `otpauth://` URI.
#[must_use]
pub fn manual_secret_from_otpauth_uri(uri: &str) -> Option<String> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return None;
    }
    let url = url::Url::parse(trimmed).ok()?;
    url.query_pairs()
        .find(|(key, _)| key == "secret")
        .map(|(_, value)| value.into_owned())
        .filter(|s| !s.is_empty())
}

/// Render an `otpauth://` URI as a minimal SVG QR code string.
///
/// # Errors
///
/// [`TotpEnrollError::Store`] when the URI is empty or cannot be encoded as a QR.
pub fn qr_svg_for_otpauth(uri: &str) -> Result<String, TotpEnrollError> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return Err(TotpEnrollError::Store);
    }
    let code = qrcode::QrCode::new(trimmed.as_bytes()).map_err(|_| TotpEnrollError::Store)?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .dark_color(qrcode::render::svg::Color("#000000"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    const SAMPLE_URI: &str =
        "otpauth://totp/Acme%20Site:you%40example.com?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Acme%20Site&algorithm=SHA1&digits=6&period=30";

    #[test]
    fn qr_svg_nonempty_happy() {
        let svg = qr_svg_for_otpauth(SAMPLE_URI).expect("svg");
        assert!(svg.contains("<svg"));
        assert!(svg.len() > 100);
    }

    #[test]
    fn qr_svg_empty_uri_sad() {
        assert!(matches!(
            qr_svg_for_otpauth("   "),
            Err(TotpEnrollError::Store)
        ));
    }

    #[test]
    fn manual_secret_from_otpauth_happy() {
        let secret = manual_secret_from_otpauth_uri(SAMPLE_URI).expect("secret");
        assert_eq!(secret, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
    }

    #[test]
    fn manual_secret_missing_sad() {
        assert!(manual_secret_from_otpauth_uri("otpauth://totp/x").is_none());
        assert!(manual_secret_from_otpauth_uri("").is_none());
    }

    #[test]
    fn format_manual_secret_groups_happy() {
        assert_eq!(
            format_manual_secret("GEZDGNBVGY3TQOJQ"),
            "GEZD GNBV GY3T QOJQ"
        );
    }
}
