//! HTTP mock OIDC exchange against a lab issuer (`lepton-mock-oidc`).

use serde::Deserialize;

use super::api::OAuthProvider;
use super::error::OAuthError;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
}

/// Exchange an authorization `code` via issuer token + userinfo endpoints.
///
/// # Errors
///
/// [`OAuthError::Provider`] on HTTP/parse failures (no token/email in messages).
pub(super) async fn exchange_mock_http(
    issuer: &str,
    _provider: OAuthProvider,
    code: &str,
) -> Result<(String, Option<String>, Option<String>), OAuthError> {
    let issuer = issuer.trim().trim_end_matches('/');
    if issuer.is_empty() || code.trim().is_empty() {
        return Err(OAuthError::Provider);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|_| OAuthError::Provider)?;

    tracing::info!(
        operation = "exchange",
        driver = "mock_oidc",
        outcome = "start",
        "oauth"
    );

    let token_url = format!("{issuer}/token");
    let token_res = client
        .post(&token_url)
        .form(&[("grant_type", "authorization_code"), ("code", code.trim())])
        .send()
        .await
        .map_err(|_| {
            tracing::warn!(
                operation = "exchange",
                driver = "mock_oidc",
                outcome = "failure",
                reason_class = "token_transport",
                "oauth"
            );
            OAuthError::Provider
        })?;

    if !token_res.status().is_success() {
        tracing::warn!(
            operation = "exchange",
            driver = "mock_oidc",
            outcome = "failure",
            reason_class = "token_http",
            "oauth"
        );
        return Err(OAuthError::Provider);
    }

    let token_body: TokenResponse = token_res.json().await.map_err(|_| OAuthError::Provider)?;
    if token_body.access_token.trim().is_empty() {
        return Err(OAuthError::Provider);
    }

    let userinfo_url = format!("{issuer}/userinfo");
    let user_res = client
        .get(&userinfo_url)
        .bearer_auth(&token_body.access_token)
        .send()
        .await
        .map_err(|_| OAuthError::Provider)?;

    if !user_res.status().is_success() {
        tracing::warn!(
            operation = "exchange",
            driver = "mock_oidc",
            outcome = "failure",
            reason_class = "userinfo_http",
            "oauth"
        );
        return Err(OAuthError::Provider);
    }

    let info: UserInfo = user_res.json().await.map_err(|_| OAuthError::Provider)?;
    if info.sub.trim().is_empty() {
        return Err(OAuthError::Provider);
    }

    tracing::info!(
        operation = "exchange",
        driver = "mock_oidc",
        outcome = "success",
        "oauth"
    );

    Ok((info.sub, info.email, info.name))
}
