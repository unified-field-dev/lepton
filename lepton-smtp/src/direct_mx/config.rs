//! Direct-MX delivery configuration and builder.

use crate::error::EmailDeliveryError;

/// Configuration for [`crate::DirectMxAdapter`].
///
/// Prefer [`DirectMxConfig::builder`]. [`DirectMxConfig::from_env`] remains as a host helper.
#[derive(Clone, Debug)]
pub struct DirectMxConfig {
    /// Port to connect to on each resolved MX host (default 25).
    pub port: u16,
    /// `From:` address.
    pub from_email: String,
    /// `From:` display name (default `Orbital`).
    pub from_name: String,
    /// Timeout in seconds for the MX DNS lookup.
    pub mx_lookup_timeout_secs: u64,
    /// Timeout in seconds per MX host delivery attempt.
    pub host_timeout_secs: u64,
    /// Maximum number of resolved MX hosts to try.
    pub max_hosts: usize,
}

impl DirectMxConfig {
    /// Start a validated builder.
    #[must_use]
    pub fn builder() -> DirectMxConfigBuilder {
        DirectMxConfigBuilder::default()
    }

    /// Load from `UF_DIRECT_MX_*` / `UF_EMAIL_*` env vars (host helper).
    pub fn from_env() -> Result<Self, EmailDeliveryError> {
        let port = std::env::var("UF_DIRECT_MX_PORT")
            .ok()
            .map(|value| {
                value.parse::<u16>().map_err(|_| {
                    EmailDeliveryError::config(
                        "invalid_port",
                        "Invalid UF_DIRECT_MX_PORT: expected integer",
                    )
                })
            })
            .transpose()?
            .unwrap_or(25);
        let from_email = std::env::var("UF_EMAIL_FROM")
            .map_err(|_| EmailDeliveryError::config("missing_field", "Missing UF_EMAIL_FROM"))?;
        let from_name =
            std::env::var("UF_EMAIL_FROM_NAME").unwrap_or_else(|_| "Orbital".to_string());
        let mx_lookup_timeout_secs =
            parse_optional_u64_env("UF_DIRECT_MX_MX_LOOKUP_TIMEOUT_SECS", 5)?;
        let host_timeout_secs = parse_optional_u64_env("UF_DIRECT_MX_HOST_TIMEOUT_SECS", 8)?;
        let max_hosts = parse_optional_usize_env("UF_DIRECT_MX_MAX_HOSTS", 2)?;

        Self::builder()
            .port(port)
            .from_email(from_email)
            .from_name(from_name)
            .mx_lookup_timeout_secs(mx_lookup_timeout_secs)
            .host_timeout_secs(host_timeout_secs)
            .max_hosts(max_hosts)
            .build()
    }
}

/// Fluent builder for [`DirectMxConfig`].
///
/// Required: `from_email`. Defaults: port `25`, MX lookup timeout `5s`, host timeout `8s`,
/// max hosts `2`, from name `Orbital`.
#[derive(Clone, Debug, Default)]
pub struct DirectMxConfigBuilder {
    port: Option<u16>,
    from_email: Option<String>,
    from_name: Option<String>,
    mx_lookup_timeout_secs: Option<u64>,
    host_timeout_secs: Option<u64>,
    max_hosts: Option<usize>,
}

impl DirectMxConfigBuilder {
    /// MX SMTP port (default 25).
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
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

    /// MX DNS lookup timeout in seconds (default 5).
    #[must_use]
    pub const fn mx_lookup_timeout_secs(mut self, secs: u64) -> Self {
        self.mx_lookup_timeout_secs = Some(secs);
        self
    }

    /// Per-host send timeout in seconds (default 8).
    #[must_use]
    pub const fn host_timeout_secs(mut self, secs: u64) -> Self {
        self.host_timeout_secs = Some(secs);
        self
    }

    /// Max MX hosts to attempt (default 2).
    #[must_use]
    pub const fn max_hosts(mut self, max_hosts: usize) -> Self {
        self.max_hosts = Some(max_hosts);
        self
    }

    /// Validate and build [`DirectMxConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`EmailDeliveryError::ConfigError`] when `from_email` is missing or timeouts
    /// are zero.
    pub fn build(self) -> Result<DirectMxConfig, EmailDeliveryError> {
        let from_email = self
            .from_email
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
            .ok_or_else(|| EmailDeliveryError::config("missing_field", "from_email is required"))?;
        let mx_lookup_timeout_secs = self.mx_lookup_timeout_secs.unwrap_or(5);
        let host_timeout_secs = self.host_timeout_secs.unwrap_or(8);
        let max_hosts = self.max_hosts.unwrap_or(2);
        if mx_lookup_timeout_secs == 0 || host_timeout_secs == 0 || max_hosts == 0 {
            return Err(EmailDeliveryError::config(
                "invalid_timeout",
                "timeouts and max_hosts must be > 0",
            ));
        }
        let from_name = self
            .from_name
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Orbital".to_string());

        Ok(DirectMxConfig {
            port: self.port.unwrap_or(25),
            from_email,
            from_name,
            mx_lookup_timeout_secs,
            host_timeout_secs,
            max_hosts,
        })
    }
}

fn parse_optional_u64_env(key: &str, default: u64) -> Result<u64, EmailDeliveryError> {
    match std::env::var(key) {
        Ok(raw) => {
            let parsed = raw.parse::<u64>().map_err(|_| {
                EmailDeliveryError::config(
                    "invalid_timeout",
                    format!("Invalid {key}: expected integer"),
                )
            })?;
            if parsed == 0 {
                return Err(EmailDeliveryError::config(
                    "invalid_timeout",
                    format!("Invalid {key}: expected value > 0"),
                ));
            }
            Ok(parsed)
        }
        Err(_) => Ok(default),
    }
}

fn parse_optional_usize_env(key: &str, default: usize) -> Result<usize, EmailDeliveryError> {
    match std::env::var(key) {
        Ok(raw) => {
            let parsed = raw.parse::<usize>().map_err(|_| {
                EmailDeliveryError::config(
                    "invalid_timeout",
                    format!("Invalid {key}: expected integer"),
                )
            })?;
            if parsed == 0 {
                return Err(EmailDeliveryError::config(
                    "invalid_timeout",
                    format!("Invalid {key}: expected value > 0"),
                ));
            }
            Ok(parsed)
        }
        Err(_) => Ok(default),
    }
}
