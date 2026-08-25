//! Twilio `SendGrid` email configuration and builder.
//!
//! Requires the `twilio` Cargo feature. Prefer the crate-root
//! [Twilio `SendGrid` guide](crate#twilio-sendgrid) for prerequisites, send → receipt, and
//! provider/transient errors. Credentials are `SendGrid` API keys, not SMS Account SID.

use crate::error::EmailDeliveryError;

/// Default `SendGrid` API origin (override via [`TwilioEmailConfigBuilder::api_base_url`] for tests).
pub const TWILIO_EMAIL_API_BASE_URL: &str = "https://api.sendgrid.com";

/// Env var for `SendGrid` API key (Twilio email; host boot helper).
pub const TWILIO_EMAIL_API_KEY_ENV: &str = "UF_TWILIO_EMAIL_API_KEY";

/// Plain `SendGrid` credentials / sender identity for [`super::TwilioEmailAdapter`].
///
/// Twilio transactional email uses `SendGrid` Mail Send. Credentials differ from SMS
/// Account SID / Auth Token. [`Debug`] redacts `api_key`. Teaching path: crate-root
/// [Twilio `SendGrid` guide](crate#twilio-sendgrid).
#[derive(Clone)]
pub struct TwilioEmailConfig {
    /// `SendGrid` API key (Bearer token; never log).
    pub api_key: String,
    /// `From:` address.
    pub from_email: String,
    /// `From:` display name.
    pub from_name: String,
    /// API origin (default [`TWILIO_EMAIL_API_BASE_URL`]; override for wiremock).
    pub api_base_url: String,
}

impl std::fmt::Debug for TwilioEmailConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwilioEmailConfig")
            .field("api_key", &"[redacted]")
            .field("from_email", &self.from_email)
            .field("from_name", &self.from_name)
            .field("api_base_url", &self.api_base_url)
            .finish()
    }
}

impl TwilioEmailConfig {
    /// Start a validated builder.
    #[must_use]
    pub fn builder() -> TwilioEmailConfigBuilder {
        TwilioEmailConfigBuilder::default()
    }

    /// Load from env (host helper; boot only).
    ///
    /// Requires `UF_TWILIO_EMAIL_API_KEY` and `UF_EMAIL_FROM`. Optional
    /// `UF_EMAIL_FROM_NAME` (default `Orbital`).
    ///
    /// # Errors
    ///
    /// Returns [`EmailDeliveryError::ConfigError`] when required vars are missing or empty.
    pub fn from_env() -> Result<Self, EmailDeliveryError> {
        let api_key = std::env::var(TWILIO_EMAIL_API_KEY_ENV).map_err(|_| {
            EmailDeliveryError::config("missing_field", "Missing UF_TWILIO_EMAIL_API_KEY")
        })?;
        let from_email = std::env::var("UF_EMAIL_FROM")
            .map_err(|_| EmailDeliveryError::config("missing_field", "Missing UF_EMAIL_FROM"))?;
        let from_name =
            std::env::var("UF_EMAIL_FROM_NAME").unwrap_or_else(|_| "Orbital".to_string());
        Self::builder()
            .api_key(api_key)
            .from_email(from_email)
            .from_name(from_name)
            .build()
    }
}

/// Fluent builder for [`TwilioEmailConfig`].
#[derive(Clone, Default)]
pub struct TwilioEmailConfigBuilder {
    api_key: Option<String>,
    from_email: Option<String>,
    from_name: Option<String>,
    api_base_url: Option<String>,
}

impl TwilioEmailConfigBuilder {
    /// `SendGrid` API key (required).
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// `From:` address (required).
    #[must_use]
    pub fn from_email(mut self, from_email: impl Into<String>) -> Self {
        self.from_email = Some(from_email.into());
        self
    }

    /// `From:` display name (default `Orbital`).
    #[must_use]
    pub fn from_name(mut self, from_name: impl Into<String>) -> Self {
        self.from_name = Some(from_name.into());
        self
    }

    /// Override API origin (tests / proxies).
    #[must_use]
    pub fn api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = Some(api_base_url.into());
        self
    }

    /// Validate and build [`TwilioEmailConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`EmailDeliveryError::ConfigError`] when required fields are missing or empty.
    pub fn build(self) -> Result<TwilioEmailConfig, EmailDeliveryError> {
        let api_key = self
            .api_key
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| EmailDeliveryError::config("missing_field", "api_key is required"))?;
        let from_email = self
            .from_email
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| EmailDeliveryError::config("missing_field", "from_email is required"))?;
        let from_name = self
            .from_name
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Orbital".to_string());
        let api_base_url = self
            .api_base_url
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| TWILIO_EMAIL_API_BASE_URL.to_string());

        Ok(TwilioEmailConfig {
            api_key,
            from_email,
            from_name,
            api_base_url,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn twilio_email_config_missing_sad() {
        let err = TwilioEmailConfig::builder()
            .from_email("noreply@example.test")
            .build()
            .expect_err("api_key required");
        assert!(err.to_string().contains("reason_class=missing_field"));
        assert!(err.to_string().contains("api_key"));
    }

    #[test]
    fn twilio_email_config_debug_redacts_key_happy_path() {
        let cfg = TwilioEmailConfig::builder()
            .api_key("SG.super-secret-key")
            .from_email("noreply@example.test")
            .build()
            .expect("cfg");
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("[redacted]"));
        assert!(!dbg.contains("SG.super-secret-key"));
    }
}
