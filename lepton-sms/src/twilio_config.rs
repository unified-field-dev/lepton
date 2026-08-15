//! Twilio SMS configuration and builder.

/// Default Twilio REST API origin (override via [`TwilioSmsConfigBuilder::api_base_url`] for tests).
pub const TWILIO_API_BASE_URL: &str = "https://api.twilio.com";

/// Env var for Twilio Account SID (host boot helper).
pub const TWILIO_ACCOUNT_SID_ENV: &str = "UF_TWILIO_ACCOUNT_SID";
/// Env var for Twilio Auth Token (legacy host boot helper).
pub const TWILIO_AUTH_TOKEN_ENV: &str = "UF_TWILIO_AUTH_TOKEN";
/// Env var for Twilio API Key SID (`SK…`; preferred host boot helper).
pub const TWILIO_API_KEY_ENV: &str = "UF_TWILIO_API_KEY";
/// Env var for Twilio API Key secret (preferred host boot helper).
pub const TWILIO_API_SECRET_ENV: &str = "UF_TWILIO_API_SECRET";
/// Env var for Twilio From number or Messaging Service SID (host boot helper).
pub const TWILIO_FROM_ENV: &str = "UF_TWILIO_FROM";

/// How Twilio Messages REST Basic auth is formed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TwilioSmsAuth {
    /// Preferred: API Key SID (`SK…`) + secret.
    ApiKey {
        /// API Key SID (`SK…`); Basic-auth username.
        key_sid: String,
        /// API Key secret; Basic-auth password (never log).
        secret: String,
    },
    /// Legacy: Account Auth Token (often unavailable on new Twilio consoles).
    AuthToken(String),
}

/// Plain Twilio credentials / sender identity for the live Twilio SMS adapter
/// (`TwilioSmsAdapter`, `feature = "twilio"`).
///
/// Prefer [`TwilioSmsAuth::ApiKey`]. Account SID is still required for the Messages
/// URL path. [`Debug`] redacts secrets.
#[derive(Clone)]
pub struct TwilioSmsConfig {
    /// Twilio Account SID (`AC…`; used in the REST path, not necessarily as Basic username).
    pub account_sid: String,
    /// Basic-auth credentials (API key preferred).
    pub auth: TwilioSmsAuth,
    /// Twilio `From` number (E.164) or Messaging Service SID (`MG…`).
    pub from: String,
    /// REST API origin (default [`TWILIO_API_BASE_URL`]; override for wiremock).
    pub api_base_url: String,
}

impl TwilioSmsConfig {
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
            TwilioSmsAuth::ApiKey { key_sid, .. } => {
                format!(
                    "mode=api_key account={} key={} from={}",
                    mask(&self.account_sid),
                    mask(key_sid),
                    mask(&self.from)
                )
            }
            TwilioSmsAuth::AuthToken(_) => {
                format!(
                    "mode=auth_token account={} from={}",
                    mask(&self.account_sid),
                    mask(&self.from)
                )
            }
        }
    }
}

impl std::fmt::Debug for TwilioSmsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth = match &self.auth {
            TwilioSmsAuth::ApiKey { key_sid, .. } => format!("ApiKey({key_sid}, [redacted])"),
            TwilioSmsAuth::AuthToken(_) => "AuthToken([redacted])".to_string(),
        };
        f.debug_struct("TwilioSmsConfig")
            .field("account_sid", &self.account_sid)
            .field("auth", &auth)
            .field("from", &self.from)
            .field("api_base_url", &self.api_base_url)
            .finish()
    }
}

impl TwilioSmsConfig {
    /// Start a validated builder.
    #[must_use]
    pub fn builder() -> TwilioSmsConfigBuilder {
        TwilioSmsConfigBuilder::default()
    }

    /// Load from `UF_TWILIO_*` env vars (host helper; boot only).
    ///
    /// Prefers `UF_TWILIO_API_KEY` + `UF_TWILIO_API_SECRET`. Falls back to
    /// `UF_TWILIO_AUTH_TOKEN` when the API key pair is unset.
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`](crate::SmsDeliveryError::ConfigError) when
    /// required vars are missing or empty.
    pub fn from_env() -> Result<Self, crate::SmsDeliveryError> {
        let account_sid = std::env::var(TWILIO_ACCOUNT_SID_ENV).map_err(|_| {
            crate::SmsDeliveryError::config("missing_field", "Missing UF_TWILIO_ACCOUNT_SID")
        })?;
        let from = std::env::var(TWILIO_FROM_ENV).map_err(|_| {
            crate::SmsDeliveryError::config("missing_field", "Missing UF_TWILIO_FROM")
        })?;

        let api_key = std::env::var(TWILIO_API_KEY_ENV).ok();
        let api_secret = std::env::var(TWILIO_API_SECRET_ENV).ok();
        let mut builder = Self::builder().account_sid(account_sid).from(from);

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

/// Fluent builder for [`TwilioSmsConfig`].
#[derive(Clone, Default)]
pub struct TwilioSmsConfigBuilder {
    account_sid: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    auth_token: Option<String>,
    from: Option<String>,
    api_base_url: Option<String>,
}

impl TwilioSmsConfigBuilder {
    /// Twilio Account SID (required; `AC…`).
    #[must_use]
    pub fn account_sid(mut self, account_sid: impl Into<String>) -> Self {
        self.account_sid = Some(account_sid.into());
        self
    }

    /// Twilio API Key SID (`SK…`; preferred with [`Self::api_secret`]).
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Twilio API Key secret (preferred with [`Self::api_key`]).
    #[must_use]
    pub fn api_secret(mut self, api_secret: impl Into<String>) -> Self {
        self.api_secret = Some(api_secret.into());
        self
    }

    /// Legacy Account Auth Token (fallback when API key pair is not set).
    #[must_use]
    pub fn auth_token(mut self, auth_token: impl Into<String>) -> Self {
        self.auth_token = Some(auth_token.into());
        self
    }

    /// From number (E.164) or Messaging Service SID (required).
    #[must_use]
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Override REST API origin (tests / proxies).
    #[must_use]
    pub fn api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = Some(api_base_url.into());
        self
    }

    /// Validate and build [`TwilioSmsConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`](crate::SmsDeliveryError::ConfigError) when
    /// required fields are missing or empty.
    pub fn build(self) -> Result<TwilioSmsConfig, crate::SmsDeliveryError> {
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
                "UF_TWILIO_ACCOUNT_SID must start with AC (not SK/SG)",
            ));
        }
        let from = self
            .from
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| crate::SmsDeliveryError::config("missing_field", "from is required"))?;

        // From may be E.164 (`+…`) or a Messaging Service SID (`MG…`).
        if !from.starts_with('+') && !from.starts_with("MG") {
            return Err(crate::SmsDeliveryError::config(
                "invalid_from",
                "from must be E.164 (+…) or a Messaging Service SID (MG…)",
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
                        "UF_TWILIO_API_KEY must be an API Key SID starting with SK (not SG SendGrid or AC Account SID)",
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
            .unwrap_or_else(|| TWILIO_API_BASE_URL.to_string());

        Ok(TwilioSmsConfig {
            account_sid,
            auth,
            from,
            api_base_url,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn twilio_sms_config_missing_sad() {
        let err = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .from("+15551234567")
            .build()
            .expect_err("auth required");
        assert!(err.to_string().contains("reason_class=missing_field"));
        assert!(err.to_string().contains("api_key"));
    }

    #[test]
    fn twilio_sms_config_api_key_partial_sad() {
        let err = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_key("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .from("+15551234567")
            .build()
            .expect_err("secret required");
        assert!(err.to_string().contains("api_secret"));
    }

    #[test]
    fn twilio_sms_config_invalid_from_sad() {
        let err = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .auth_token("secret-token")
            .from("15551234567")
            .build()
            .expect_err("invalid from");
        assert!(err.to_string().contains("reason_class=invalid_from"));
    }

    #[test]
    fn twilio_sms_config_debug_redacts_token_happy_path() {
        let cfg = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .auth_token("super-secret-token")
            .from("+15551234567")
            .build()
            .expect("cfg");
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("[redacted]"));
        assert!(!dbg.contains("super-secret-token"));
        assert!(dbg.contains("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"));
    }

    #[test]
    fn twilio_sms_config_api_key_happy_path() {
        let cfg = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_key("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_secret("super-secret-key")
            .from("MGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_base_url("http://127.0.0.1:9999/")
            .build()
            .expect("cfg");
        assert_eq!(cfg.api_base_url, "http://127.0.0.1:9999");
        assert_eq!(cfg.from, "MGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_eq!(
            cfg.basic_auth_pair(),
            ("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "super-secret-key")
        );
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("SKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"));
        assert!(!dbg.contains("super-secret-key"));
        assert!(cfg.auth_fingerprint().contains("mode=api_key"));
    }

    #[test]
    fn twilio_sms_config_rejects_sendgrid_key_as_sms_key() {
        let err = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_key("SGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .api_secret("not-an-sms-secret")
            .from("+15551234567")
            .build()
            .expect_err("SG key");
        assert!(err.to_string().contains("reason_class=invalid_api_key"));
    }

    #[test]
    fn twilio_sms_config_builder_auth_token_happy_path() {
        let cfg = TwilioSmsConfig::builder()
            .account_sid("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .auth_token("token")
            .from("MGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
            .build()
            .expect("cfg");
        assert_eq!(
            cfg.basic_auth_pair(),
            ("ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "token")
        );
    }
}
