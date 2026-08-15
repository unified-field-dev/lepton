//! Twilio Verify Service configuration (custom-code SMS delivery).

use crate::twilio_config::{
    TwilioSmsAuth, TWILIO_ACCOUNT_SID_ENV, TWILIO_API_KEY_ENV, TWILIO_API_SECRET_ENV,
    TWILIO_AUTH_TOKEN_ENV,
};

/// Default Twilio Verify API origin (override for tests).
pub const TWILIO_VERIFY_API_BASE_URL: &str = "https://verify.twilio.com";

/// Env var for Twilio Verify Service SID (`VA…`).
pub const TWILIO_VERIFY_SERVICE_SID_ENV: &str = "UF_TWILIO_VERIFY_SERVICE_SID";

/// Plain Twilio Verify credentials for `TwilioVerifySmsAdapter` (Cargo feature `twilio`).
///
/// Requires Custom Verification Code enabled on the Verify Service. Auth reuses the
/// same Account SID + API key (or Auth Token) as Programmable Messaging.
#[derive(Clone)]
pub struct TwilioVerifyConfig {
    /// Verify Service SID (`VA…`).
    pub service_sid: String,
    /// Account SID (`AC…`; Auth Token username / fingerprint).
    pub account_sid: String,
    /// Basic-auth credentials (API key preferred).
    pub auth: TwilioSmsAuth,
    /// Verify API origin (default [`TWILIO_VERIFY_API_BASE_URL`]).
    pub api_base_url: String,
}

impl TwilioVerifyConfig {
    /// Username:password material for HTTP Basic auth.
    #[must_use]
    pub const fn basic_auth_pair(&self) -> (&str, &str) {
        match &self.auth {
            TwilioSmsAuth::ApiKey { key_sid, secret } => (key_sid.as_str(), secret.as_str()),
            TwilioSmsAuth::AuthToken(token) => (self.account_sid.as_str(), token.as_str()),
        }
    }

    /// Safe summary for operator diagnostics (prefixes only; no secrets).
    #[must_use]
    pub fn auth_fingerprint(&self) -> String {
        let mask = |s: &str| {
            let prefix: String = s.chars().take(4).collect();
            let suffix: String = s
                .chars()
                .rev()
                .take(4)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if s.len() <= 8 {
                format!("{prefix}…")
            } else {
                format!("{prefix}…{suffix}")
            }
        };
        match &self.auth {
            TwilioSmsAuth::ApiKey { key_sid, .. } => format!(
                "mode=verify_api_key service={} account={} key={}",
                mask(&self.service_sid),
                mask(&self.account_sid),
                mask(key_sid)
            ),
            TwilioSmsAuth::AuthToken(_) => format!(
                "mode=verify_auth_token service={} account={}",
                mask(&self.service_sid),
                mask(&self.account_sid)
            ),
        }
    }

    /// Start a validated builder.
    #[must_use]
    pub fn builder() -> TwilioVerifyConfigBuilder {
        TwilioVerifyConfigBuilder::default()
    }

    /// Load from `UF_TWILIO_VERIFY_SERVICE_SID` + shared `UF_TWILIO_*` auth env vars.
    ///
    /// # Errors
    ///
    /// Missing / empty required fields.
    pub fn from_env() -> Result<Self, crate::SmsDeliveryError> {
        let service_sid = std::env::var(TWILIO_VERIFY_SERVICE_SID_ENV).map_err(|_| {
            crate::SmsDeliveryError::config("missing_field", "Missing UF_TWILIO_VERIFY_SERVICE_SID")
        })?;
        let account_sid = std::env::var(TWILIO_ACCOUNT_SID_ENV).map_err(|_| {
            crate::SmsDeliveryError::config("missing_field", "Missing UF_TWILIO_ACCOUNT_SID")
        })?;

        let api_key = std::env::var(TWILIO_API_KEY_ENV).ok();
        let api_secret = std::env::var(TWILIO_API_SECRET_ENV).ok();
        let mut builder = Self::builder()
            .service_sid(service_sid)
            .account_sid(account_sid);

        match (api_key, api_secret) {
            (Some(key), Some(secret)) if !key.trim().is_empty() && !secret.trim().is_empty() => {
                builder = builder.api_key(key).api_secret(secret);
            }
            (Some(key), None) if !key.trim().is_empty() => {
                return Err(crate::SmsDeliveryError::config(
                    "missing_field",
                    "Missing UF_TWILIO_API_SECRET (UF_TWILIO_API_KEY is set)",
                ));
            }
            (None, Some(secret)) if !secret.trim().is_empty() => {
                return Err(crate::SmsDeliveryError::config(
                    "missing_field",
                    "Missing UF_TWILIO_API_KEY (UF_TWILIO_API_SECRET is set)",
                ));
            }
            _ => {
                let auth_token = std::env::var(TWILIO_AUTH_TOKEN_ENV).map_err(|_| {
                    crate::SmsDeliveryError::config(
                        "missing_field",
                        "Missing UF_TWILIO_API_KEY/UF_TWILIO_API_SECRET (or UF_TWILIO_AUTH_TOKEN)",
                    )
                })?;
                builder = builder.auth_token(auth_token);
            }
        }

        builder.build()
    }
}

impl std::fmt::Debug for TwilioVerifyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth = match &self.auth {
            TwilioSmsAuth::ApiKey { key_sid, .. } => format!("ApiKey({key_sid}, [redacted])"),
            TwilioSmsAuth::AuthToken(_) => "AuthToken([redacted])".to_string(),
        };
        f.debug_struct("TwilioVerifyConfig")
            .field("service_sid", &self.service_sid)
            .field("account_sid", &self.account_sid)
            .field("auth", &auth)
            .field("api_base_url", &self.api_base_url)
            .finish()
    }
}

/// Fluent builder for [`TwilioVerifyConfig`].
#[derive(Clone, Default)]
pub struct TwilioVerifyConfigBuilder {
    service_sid: Option<String>,
    account_sid: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    auth_token: Option<String>,
    api_base_url: Option<String>,
}

impl TwilioVerifyConfigBuilder {
    /// Verify Service SID (`VA…`).
    #[must_use]
    pub fn service_sid(mut self, service_sid: impl Into<String>) -> Self {
        self.service_sid = Some(service_sid.into());
        self
    }

    /// Account SID (`AC…`).
    #[must_use]
    pub fn account_sid(mut self, account_sid: impl Into<String>) -> Self {
        self.account_sid = Some(account_sid.into());
        self
    }

    /// API Key SID (`SK…`).
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// API Key secret.
    #[must_use]
    pub fn api_secret(mut self, api_secret: impl Into<String>) -> Self {
        self.api_secret = Some(api_secret.into());
        self
    }

    /// Legacy Auth Token.
    #[must_use]
    pub fn auth_token(mut self, auth_token: impl Into<String>) -> Self {
        self.auth_token = Some(auth_token.into());
        self
    }

    /// Override Verify API origin (tests).
    #[must_use]
    pub fn api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = Some(api_base_url.into());
        self
    }

    /// Validate and build.
    ///
    /// # Errors
    ///
    /// Missing fields or invalid SID prefixes.
    pub fn build(self) -> Result<TwilioVerifyConfig, crate::SmsDeliveryError> {
        let service_sid = self
            .service_sid
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                crate::SmsDeliveryError::config("missing_field", "service_sid is required")
            })?;
        if !service_sid.starts_with("VA") {
            return Err(crate::SmsDeliveryError::config(
                "invalid_service_sid",
                "UF_TWILIO_VERIFY_SERVICE_SID must start with VA",
            ));
        }
        let account_sid = self
            .account_sid
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                crate::SmsDeliveryError::config("missing_field", "account_sid is required")
            })?;
        if !account_sid.starts_with("AC") {
            return Err(crate::SmsDeliveryError::config(
                "invalid_account_sid",
                "UF_TWILIO_ACCOUNT_SID must start with AC",
            ));
        }

        let api_key = self
            .api_key
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let api_secret = self
            .api_secret
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let auth_token = self
            .auth_token
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let auth = match (api_key, api_secret, auth_token) {
            (Some(key_sid), Some(secret), _) => {
                if !key_sid.starts_with("SK") {
                    return Err(crate::SmsDeliveryError::config(
                        "invalid_api_key",
                        "UF_TWILIO_API_KEY must start with SK",
                    ));
                }
                TwilioSmsAuth::ApiKey { key_sid, secret }
            }
            (Some(_), None, _) => {
                return Err(crate::SmsDeliveryError::config(
                    "missing_field",
                    "api_secret is required when api_key is set",
                ));
            }
            (None, Some(_), _) => {
                return Err(crate::SmsDeliveryError::config(
                    "missing_field",
                    "api_key is required when api_secret is set",
                ));
            }
            (None, None, Some(token)) => TwilioSmsAuth::AuthToken(token),
            (None, None, None) => {
                return Err(crate::SmsDeliveryError::config(
                    "missing_field",
                    "api_key/api_secret (preferred) or auth_token is required",
                ));
            }
        };

        let api_base_url = self
            .api_base_url
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| TWILIO_VERIFY_API_BASE_URL.to_string());

        Ok(TwilioVerifyConfig {
            service_sid,
            account_sid,
            auth,
            api_base_url,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn twilio_verify_config_api_key_happy() {
        let cfg = TwilioVerifyConfig::builder()
            .service_sid("VAxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_key("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_secret("secret")
            .build()
            .expect("cfg");
        assert!(cfg.auth_fingerprint().contains("mode=verify_api_key"));
        assert_eq!(
            cfg.basic_auth_pair(),
            ("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "secret")
        );
    }

    #[test]
    fn twilio_verify_config_rejects_non_va_sid() {
        let err = TwilioVerifyConfig::builder()
            .service_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_key("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_secret("secret")
            .build()
            .expect_err("VA");
        assert!(err.to_string().contains("invalid_service_sid"));
    }
}
