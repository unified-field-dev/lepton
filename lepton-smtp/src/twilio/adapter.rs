//! Live Twilio `SendGrid` Mail Send [`EmailDeliveryService`] adapter.
//!
//! Requires the `twilio` Cargo feature. Prefer the crate-root
//! [Twilio `SendGrid` guide](crate#twilio-sendgrid) for the full credential → send → receipt path.

use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;

use crate::driver::EmailDriver;
use crate::envelope::{DeliveryReceipt, EmailEnvelope};
use crate::error::EmailDeliveryError;
use crate::service::EmailDeliveryService;

use super::config::TwilioEmailConfig;
use super::http::{mail_send_url, map_http_status};

/// [`EmailDeliveryService`] that sends via Twilio `SendGrid` Mail Send v3.
///
/// # Errors
///
/// Config errors when credentials/client setup fail; provider rejection and transient
/// classification for HTTP/status failures ([`EmailDeliveryError::is_transient`]).
///
/// # Examples
///
/// ```no_run
/// use lepton_smtp::{
///     verification_email_envelope, EmailDeliveryService, EmailServiceBuilder, TwilioEmailConfig,
///     VerificationEmailFlow,
/// };
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let email = EmailServiceBuilder::new()
///     .twilio(
///         TwilioEmailConfig::builder()
///             .api_key(std::env::var("UF_TWILIO_EMAIL_API_KEY")?)
///             .from_email("noreply@example.test")
///             .from_name("App")
///             .build()?,
///     )
///     .build()?;
/// let message = verification_email_envelope(
///     "reader@example.test",
///     "123456",
///     VerificationEmailFlow::Signup,
/// );
/// let receipt = email.send(&message).await?;
/// assert_eq!(receipt.provider, "twilio");
/// # Ok(())
/// # }
/// ```
pub struct TwilioEmailAdapter {
    cfg: TwilioEmailConfig,
    client: reqwest::Client,
}

impl TwilioEmailAdapter {
    /// Construct from validated [`TwilioEmailConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`EmailDeliveryError::ConfigError`] when the HTTP client cannot be built.
    pub fn new(cfg: TwilioEmailConfig) -> Result<Self, EmailDeliveryError> {
        let client = reqwest::Client::builder().build().map_err(|e| {
            EmailDeliveryError::config("http_client", format!("failed to build HTTP client: {e}"))
        })?;
        tracing::info!(
            driver = "twilio",
            operation = "init",
            outcome = "success",
            "email service"
        );
        Ok(Self { cfg, client })
    }
}

#[async_trait]
impl EmailDeliveryService for TwilioEmailAdapter {
    fn driver(&self) -> EmailDriver {
        EmailDriver::Twilio
    }

    async fn send(&self, envelope: &EmailEnvelope) -> Result<DeliveryReceipt, EmailDeliveryError> {
        let result = self.send_inner(envelope).await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("twilio", result.is_ok());
        result
    }
}

impl TwilioEmailAdapter {
    async fn send_inner(
        &self,
        envelope: &EmailEnvelope,
    ) -> Result<DeliveryReceipt, EmailDeliveryError> {
        let started = Instant::now();
        tracing::info!(
            driver = "twilio",
            operation = "send",
            outcome = "start",
            "email send"
        );

        let url = mail_send_url(&self.cfg.api_base_url);
        let payload = json!({
            "personalizations": [{
                "to": [{ "email": envelope.to }]
            }],
            "from": {
                "email": self.cfg.from_email,
                "name": self.cfg.from_name,
            },
            "subject": envelope.subject,
            "content": [
                { "type": "text/plain", "value": envelope.text_body },
                { "type": "text/html", "value": envelope.html_body },
            ],
        });

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.cfg.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                let elapsed_ms = started.elapsed().as_millis();
                tracing::warn!(
                    driver = "twilio",
                    operation = "send",
                    outcome = "failure",
                    elapsed_ms,
                    reason_class = "transport_error",
                    "email send"
                );
                EmailDeliveryError::transport(
                    "transport_error",
                    format!("SendGrid HTTP request failed: {e}"),
                )
            })?;

        let status = response.status().as_u16();
        // SendGrid returns 202 Accepted on success; some sandboxes may return 200.
        if status != 202 && status != 200 {
            let elapsed_ms = started.elapsed().as_millis();
            let err = map_http_status(status);
            let reason_class = match &err {
                EmailDeliveryError::ProviderRejected(s)
                | EmailDeliveryError::Transient(s)
                | EmailDeliveryError::TransportError(s)
                | EmailDeliveryError::ConfigError(s) => s
                    .strip_prefix("reason_class=")
                    .and_then(|rest| rest.split(':').next())
                    .unwrap_or("transport_error"),
            };
            tracing::warn!(
                driver = "twilio",
                operation = "send",
                outcome = "failure",
                elapsed_ms,
                reason_class,
                http_status = status,
                "email send"
            );
            let _ = response.bytes().await;
            return Err(err);
        }

        let message_id = response
            .headers()
            .get("X-Message-Id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);

        // Drain body without logging.
        let _ = response.bytes().await;

        tracing::info!(
            driver = "twilio",
            operation = "send",
            outcome = "success",
            elapsed_ms = started.elapsed().as_millis(),
            "email send"
        );

        Ok(DeliveryReceipt {
            provider: "twilio".to_string(),
            message_id,
        })
    }
}

#[cfg(all(test, feature = "twilio"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_cfg(base: &str) -> TwilioEmailConfig {
        TwilioEmailConfig::builder()
            .api_key("SG.test-api-key")
            .from_email("noreply@example.test")
            .from_name("Test")
            .api_base_url(base)
            .build()
            .expect("cfg")
    }

    #[tokio::test]
    async fn twilio_email_send_happy_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mail/send"))
            .respond_with(ResponseTemplate::new(202).insert_header("X-Message-Id", "msg-abc-123"))
            .mount(&server)
            .await;

        let adapter = TwilioEmailAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let receipt = adapter
            .send(&EmailEnvelope {
                to: "user@example.test".into(),
                subject: "Verify".into(),
                text_body: "click link".into(),
                html_body: "<p>click</p>".into(),
            })
            .await
            .expect("send");
        assert_eq!(receipt.provider, "twilio");
        assert_eq!(receipt.message_id.as_deref(), Some("msg-abc-123"));
        assert_eq!(adapter.driver(), EmailDriver::Twilio);
    }

    #[tokio::test]
    async fn twilio_email_auth_failed_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mail/send"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let adapter = TwilioEmailAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let err = adapter
            .send(&EmailEnvelope {
                to: "user@example.test".into(),
                subject: "Verify".into(),
                text_body: "x".into(),
                html_body: "x".into(),
            })
            .await
            .expect_err("401");
        assert!(matches!(err, EmailDeliveryError::ProviderRejected(_)));
        assert!(err.to_string().contains("reason_class=auth_failed"));
        assert!(!err.to_string().contains("SG.test-api-key"));
    }

    #[tokio::test]
    async fn twilio_email_rate_limited_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mail/send"))
            .respond_with(ResponseTemplate::new(429).set_body_string("slow down"))
            .mount(&server)
            .await;

        let adapter = TwilioEmailAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let err = adapter
            .send(&EmailEnvelope {
                to: "user@example.test".into(),
                subject: "Verify".into(),
                text_body: "x".into(),
                html_body: "x".into(),
            })
            .await
            .expect_err("429");
        assert!(matches!(err, EmailDeliveryError::Transient(_)));
        assert!(err.to_string().contains("reason_class=rate_limited"));
    }
}
