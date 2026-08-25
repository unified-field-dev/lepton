//! Live Twilio Messages REST [`SmsDeliveryService`] adapter.
//!
//! Requires the `twilio` Cargo feature. Prefer the crate-root
//! [Twilio Messages guide](crate#twilio-messages) for credentials, send → receipt, and errors.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Deserialize;
use std::time::Instant;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::service::SmsDeliveryService;
use crate::twilio_config::TwilioSmsConfig;

use super::http::{map_twilio_http_error, messages_url, parse_twilio_err_body};

#[derive(Debug, Deserialize)]
struct TwilioMessageResponse {
    sid: Option<String>,
}

/// [`SmsDeliveryService`] that sends via Twilio Messages REST.
///
/// # Errors
///
/// Config / E.164 failures, provider rejection, and transient HTTP/status cases
/// ([`SmsDeliveryError::is_transient`]).
///
/// # Examples
///
/// ```no_run
/// use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder, TwilioSmsConfig};
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let sms = SmsServiceBuilder::new()
///     .twilio(
///         TwilioSmsConfig::builder()
///             .account_sid(std::env::var("UF_TWILIO_ACCOUNT_SID")?)
///             .api_key(std::env::var("UF_TWILIO_API_KEY")?)
///             .api_secret(std::env::var("UF_TWILIO_API_SECRET")?)
///             .from(std::env::var("UF_TWILIO_FROM")?)
///             .build()?,
///     )
///     .build()?;
/// let receipt = sms
///     .send(&SmsEnvelope {
///         to_e164: "+15551234567".into(),
///         body: "Your code is 123456".into(),
///         otp_code: None,
///     })
///     .await?;
/// assert_eq!(receipt.provider, "twilio");
/// # Ok(())
/// # }
/// ```
pub struct TwilioSmsAdapter {
    cfg: TwilioSmsConfig,
    client: reqwest::Client,
}

impl TwilioSmsAdapter {
    /// Construct from validated [`TwilioSmsConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`] when the HTTP client cannot be built.
    pub fn new(cfg: TwilioSmsConfig) -> Result<Self, SmsDeliveryError> {
        let client = reqwest::Client::builder().build().map_err(|e| {
            SmsDeliveryError::config("http_client", format!("failed to build HTTP client: {e}"))
        })?;
        tracing::info!(
            driver = "twilio",
            operation = "init",
            outcome = "success",
            "sms service"
        );
        Ok(Self { cfg, client })
    }
}

#[async_trait]
impl SmsDeliveryService for TwilioSmsAdapter {
    fn driver_name(&self) -> &'static str {
        "twilio"
    }

    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        let result = self.send_inner(envelope).await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("twilio", result.is_ok());
        result
    }
}

impl TwilioSmsAdapter {
    async fn send_inner(
        &self,
        envelope: &SmsEnvelope,
    ) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        validate_e164(&envelope.to_e164)?;
        let started = Instant::now();
        tracing::info!(
            driver = "twilio",
            operation = "send",
            outcome = "start",
            "sms send"
        );

        let url = messages_url(&self.cfg.api_base_url, &self.cfg.account_sid);
        let (user, pass) = self.cfg.basic_auth_pair();
        let auth = BASE64.encode(format!("{user}:{pass}"));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {auth}"))
            .form(&[
                ("To", envelope.to_e164.as_str()),
                ("From", self.cfg.from.as_str()),
                ("Body", envelope.body.as_str()),
            ])
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
                    "sms send"
                );
                SmsDeliveryError::transport(
                    "transport_error",
                    format!("Twilio HTTP request failed: {e}"),
                )
            })?;

        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            let elapsed_ms = started.elapsed().as_millis();
            // Body may echo recipient — classify from code/message; never log raw body.
            let body = response.bytes().await.unwrap_or_default();
            let (code, message) = parse_twilio_err_body(&body);
            let err = map_twilio_http_error(status, code, message.as_deref());
            let reason_class = match &err {
                SmsDeliveryError::Rejected(s)
                | SmsDeliveryError::Transient(s)
                | SmsDeliveryError::TransportError(s)
                | SmsDeliveryError::ConfigError(s) => s
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
                twilio_code = code,
                "sms send"
            );
            return Err(err);
        }

        let body = response.bytes().await.map_err(|e| {
            SmsDeliveryError::transport(
                "invalid_response",
                format!("failed to read Twilio response: {e}"),
            )
        })?;
        let parsed: TwilioMessageResponse = serde_json::from_slice(&body).map_err(|_| {
            SmsDeliveryError::transport(
                "invalid_response",
                "Twilio response was not valid JSON with a message sid",
            )
        })?;

        tracing::info!(
            driver = "twilio",
            operation = "send",
            outcome = "success",
            elapsed_ms = started.elapsed().as_millis(),
            "sms send"
        );

        Ok(SmsDeliveryReceipt {
            provider: "twilio".to_string(),
            message_id: parsed.sid,
        })
    }
}

#[cfg(all(test, feature = "twilio"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_cfg(base: &str) -> TwilioSmsConfig {
        TwilioSmsConfig::builder()
            .account_sid("ACtestsid000000000000000000000000")
            .auth_token("test-auth-token")
            .from("+15550001111")
            .api_base_url(base)
            .build()
            .expect("cfg")
    }

    #[tokio::test]
    async fn twilio_sms_send_happy_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"^/2010-04-01/Accounts/.+/Messages\.json$"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({ "sid": "SMxxxxxxxx" })),
            )
            .mount(&server)
            .await;

        let adapter = TwilioSmsAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let receipt = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "Your verification code is: 123456".into(),
                otp_code: None,
            })
            .await
            .expect("send");
        assert_eq!(receipt.provider, "twilio");
        assert_eq!(receipt.message_id.as_deref(), Some("SMxxxxxxxx"));
    }

    #[tokio::test]
    async fn twilio_sms_api_key_basic_auth_happy_path() {
        use wiremock::matchers::header;

        let server = MockServer::start().await;
        let expected = BASE64.encode("SKtestkey000000000000000000000000:api-secret");
        Mock::given(method("POST"))
            .and(path_regex(
                r"^/2010-04-01/Accounts/ACtestsid000000000000000000000000/Messages\.json$",
            ))
            .and(header("Authorization", format!("Basic {expected}")))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({ "sid": "SMxxxxxxxx" })),
            )
            .mount(&server)
            .await;

        let cfg = TwilioSmsConfig::builder()
            .account_sid("ACtestsid000000000000000000000000")
            .api_key("SKtestkey000000000000000000000000")
            .api_secret("api-secret")
            .from("+15550001111")
            .api_base_url(server.uri())
            .build()
            .expect("cfg");
        let adapter = TwilioSmsAdapter::new(cfg).expect("adapter");
        let receipt = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "otp".into(),
                otp_code: None,
            })
            .await
            .expect("send");
        assert_eq!(receipt.message_id.as_deref(), Some("SMxxxxxxxx"));
    }

    #[tokio::test]
    async fn twilio_sms_auth_failed_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"^/2010-04-01/Accounts/.+/Messages\.json$"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let adapter = TwilioSmsAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "otp".into(),
                otp_code: None,
            })
            .await
            .expect_err("401");
        assert!(matches!(err, SmsDeliveryError::Rejected(_)));
        assert!(err.to_string().contains("reason_class=auth_failed"));
        assert!(!err.to_string().contains("test-auth-token"));
    }

    #[tokio::test]
    async fn twilio_sms_rate_limited_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"^/2010-04-01/Accounts/.+/Messages\.json$"))
            .respond_with(ResponseTemplate::new(429).set_body_string("slow down"))
            .mount(&server)
            .await;

        let adapter = TwilioSmsAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "otp".into(),
                otp_code: None,
            })
            .await
            .expect_err("429");
        assert!(matches!(err, SmsDeliveryError::Transient(_)));
        assert!(err.to_string().contains("reason_class=rate_limited"));
    }

    #[tokio::test]
    async fn twilio_sms_invalid_e164_sad() {
        let adapter = TwilioSmsAdapter::new(test_cfg("http://127.0.0.1:9")).expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "15551234567".into(),
                body: "otp".into(),
                otp_code: None,
            })
            .await
            .expect_err("bad e164");
        assert!(err.to_string().contains("reason_class=invalid_e164"));
    }
}
