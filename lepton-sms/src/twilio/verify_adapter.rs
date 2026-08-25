//! Live Twilio Verify custom-code SMS adapter.
//!
//! Requires the `twilio` Cargo feature and Custom Verification Code on the Verify Service.
//! Prefer the crate-root [Twilio Verify guide](crate#twilio-verify). Valence (or the host)
//! still verifies the OTP; this adapter does not call `VerificationCheck`.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Deserialize;
use std::time::Instant;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::service::SmsDeliveryService;
use crate::twilio_verify_config::TwilioVerifyConfig;

use super::http::{map_twilio_http_error, parse_twilio_err_body, verifications_url};

#[derive(Debug, Deserialize)]
struct TwilioVerificationResponse {
    sid: Option<String>,
}

/// [`SmsDeliveryService`] that creates a Twilio Verify verification with `CustomCode`.
///
/// Does **not** call `VerificationCheck` — Valence still consumes the OTP. The Verify
/// Service must have Custom Verification Code enabled. [`SmsEnvelope::otp_code`] must be
/// 4..=10 characters.
///
/// # Errors
///
/// Config errors for missing/invalid `otp_code` or credentials; provider rejection and
/// transient classification for HTTP/status failures.
///
/// # Examples
///
/// ```no_run
/// use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder, TwilioVerifyConfig};
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let sms = SmsServiceBuilder::new()
///     .twilio_verify(TwilioVerifyConfig::from_env()?)
///     .build()?;
/// let receipt = sms
///     .send(&SmsEnvelope {
///         to_e164: "+15551234567".into(),
///         body: "ignored for Verify body channel".into(),
///         otp_code: Some("123456".into()),
///     })
///     .await?;
/// assert_eq!(receipt.provider, "twilio_verify");
/// # Ok(())
/// # }
/// ```
pub struct TwilioVerifySmsAdapter {
    cfg: TwilioVerifyConfig,
    client: reqwest::Client,
}

impl TwilioVerifySmsAdapter {
    /// Construct from validated [`TwilioVerifyConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`] when the HTTP client cannot be built.
    pub fn new(cfg: TwilioVerifyConfig) -> Result<Self, SmsDeliveryError> {
        let client = reqwest::Client::builder().build().map_err(|e| {
            SmsDeliveryError::config("http_client", format!("failed to build HTTP client: {e}"))
        })?;
        tracing::info!(
            driver = "twilio_verify",
            operation = "init",
            outcome = "success",
            "sms service"
        );
        Ok(Self { cfg, client })
    }
}

fn validate_custom_code(otp: Option<&str>) -> Result<&str, SmsDeliveryError> {
    let Some(code) = otp.map(str::trim).filter(|s| !s.is_empty()) else {
        return Err(SmsDeliveryError::config(
            "invalid_otp_code",
            "otp_code is required for Twilio Verify",
        ));
    };
    let len = code.chars().count();
    if !(4..=10).contains(&len) {
        return Err(SmsDeliveryError::config(
            "invalid_otp_code",
            "otp_code must be 4..=10 characters for Twilio Verify CustomCode",
        ));
    }
    Ok(code)
}

#[async_trait]
impl SmsDeliveryService for TwilioVerifySmsAdapter {
    fn driver_name(&self) -> &'static str {
        "twilio_verify"
    }

    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        let result = self.send_inner(envelope).await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("twilio_verify", result.is_ok());
        result
    }
}

impl TwilioVerifySmsAdapter {
    async fn send_inner(
        &self,
        envelope: &SmsEnvelope,
    ) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        validate_e164(&envelope.to_e164)?;
        let custom_code = validate_custom_code(envelope.otp_code.as_deref())?;
        let started = Instant::now();
        tracing::info!(
            driver = "twilio_verify",
            operation = "send",
            outcome = "start",
            "sms send"
        );

        let url = verifications_url(&self.cfg.api_base_url, &self.cfg.service_sid);
        let (user, pass) = self.cfg.basic_auth_pair();
        let auth = BASE64.encode(format!("{user}:{pass}"));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {auth}"))
            .form(&[
                ("To", envelope.to_e164.as_str()),
                ("Channel", "sms"),
                ("CustomCode", custom_code),
            ])
            .send()
            .await
            .map_err(|e| {
                let elapsed_ms = started.elapsed().as_millis();
                tracing::warn!(
                    driver = "twilio_verify",
                    operation = "send",
                    outcome = "failure",
                    elapsed_ms,
                    reason_class = "transport_error",
                    "sms send"
                );
                SmsDeliveryError::transport(
                    "transport_error",
                    format!("Twilio Verify HTTP request failed: {e}"),
                )
            })?;

        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            let elapsed_ms = started.elapsed().as_millis();
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
                driver = "twilio_verify",
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
                format!("failed to read Twilio Verify response: {e}"),
            )
        })?;
        let parsed: TwilioVerificationResponse = serde_json::from_slice(&body).map_err(|_| {
            SmsDeliveryError::transport(
                "invalid_response",
                "Twilio Verify response was not valid JSON with a sid",
            )
        })?;

        tracing::info!(
            driver = "twilio_verify",
            operation = "send",
            outcome = "success",
            elapsed_ms = started.elapsed().as_millis(),
            "sms send"
        );

        Ok(SmsDeliveryReceipt {
            provider: "twilio_verify".to_string(),
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

    fn test_cfg(base: &str) -> TwilioVerifyConfig {
        TwilioVerifyConfig::builder()
            .service_sid("VAtestsid000000000000000000000000")
            .account_sid("ACtestsid000000000000000000000000")
            .api_key("SKtestkey000000000000000000000000")
            .api_secret("api-secret")
            .api_base_url(base)
            .build()
            .expect("cfg")
    }

    #[tokio::test]
    async fn twilio_verify_custom_code_happy_path() {
        use wiremock::matchers::body_string_contains;

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(
                r"^/v2/Services/VAtestsid000000000000000000000000/Verifications$",
            ))
            .and(body_string_contains("CustomCode=123456"))
            .and(body_string_contains("To=%2B15551234567"))
            .and(body_string_contains("Channel=sms"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({ "sid": "VExxxxxxxx" })),
            )
            .mount(&server)
            .await;

        let adapter = TwilioVerifySmsAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let receipt = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "ignored by verify".into(),
                otp_code: Some("123456".into()),
            })
            .await
            .expect("send");
        assert_eq!(receipt.provider, "twilio_verify");
        assert_eq!(receipt.message_id.as_deref(), Some("VExxxxxxxx"));
    }

    #[tokio::test]
    async fn twilio_verify_custom_code_disabled_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"^/v2/Services/.+/Verifications$"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "code": 60204,
                "message": "Custom code not allowed",
            })))
            .mount(&server)
            .await;

        let adapter = TwilioVerifySmsAdapter::new(test_cfg(&server.uri())).expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "x".into(),
                otp_code: Some("123456".into()),
            })
            .await
            .expect_err("60204");
        assert!(err.to_string().contains("reason_class=feature_not_enabled"));
        assert!(!err.to_string().contains("rejected credentials"));
    }

    #[tokio::test]
    async fn twilio_verify_missing_otp_sad() {
        let adapter = TwilioVerifySmsAdapter::new(test_cfg("http://127.0.0.1:9")).expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "x".into(),
                otp_code: None,
            })
            .await
            .expect_err("otp required");
        assert!(err.to_string().contains("reason_class=invalid_otp_code"));
    }

    #[tokio::test]
    async fn twilio_verify_otp_too_long_sad() {
        let adapter = TwilioVerifySmsAdapter::new(test_cfg("http://127.0.0.1:9")).expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "x".into(),
                otp_code: Some("12345678901".into()),
            })
            .await
            .expect_err("too long");
        assert!(err.to_string().contains("reason_class=invalid_otp_code"));
    }
}
