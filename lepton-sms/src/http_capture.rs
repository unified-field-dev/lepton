//! HTTP capture SMS adapter (POSTs envelopes to a lab sink).
//!
//! Prefer the crate-root [HTTP capture guide](crate#http-capture) for prerequisites
//! (sink on `:8099`), send → receipt (`provider = "http_capture"`), and sink errors.

use async_trait::async_trait;
use serde::Serialize;
use std::time::Instant;

use crate::envelope::{validate_e164, SmsDeliveryReceipt, SmsEnvelope};
use crate::error::SmsDeliveryError;
use crate::http_capture_config::HttpCaptureSmsConfig;
use crate::service::SmsDeliveryService;

#[derive(Debug, Serialize)]
struct CaptureBody<'a> {
    to_e164: &'a str,
    body: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    otp_code: Option<&'a str>,
}

/// [`SmsDeliveryService`] that POSTs JSON envelopes to an HTTP capture sink.
///
/// # Errors
///
/// Config / E.164 validation failures, plus transport / rejected / transient mapping for
/// sink HTTP responses ([`SmsDeliveryError::is_transient`]).
///
/// # Examples
///
/// ```no_run
/// use lepton_sms::{
///     HttpCaptureSmsConfig, SmsDeliveryService, SmsEnvelope, SmsServiceBuilder,
/// };
///
/// # async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
/// let sms = SmsServiceBuilder::new()
///     .http_capture(HttpCaptureSmsConfig::new("http://127.0.0.1:8099")?)
///     .build()?;
/// let receipt = sms
///     .send(&SmsEnvelope {
///         to_e164: "+15551234567".into(),
///         body: "lab capture".into(),
///         otp_code: Some("123456".into()),
///     })
///     .await?;
/// assert_eq!(receipt.provider, "http_capture");
/// # Ok(())
/// # }
/// ```
pub struct HttpCaptureSmsAdapter {
    cfg: HttpCaptureSmsConfig,
    client: reqwest::Client,
}

impl HttpCaptureSmsAdapter {
    /// Construct from validated [`HttpCaptureSmsConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`SmsDeliveryError::ConfigError`] when the HTTP client cannot be built.
    pub fn new(cfg: HttpCaptureSmsConfig) -> Result<Self, SmsDeliveryError> {
        let client = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .map_err(|e| {
                SmsDeliveryError::config("http_client", format!("failed to build HTTP client: {e}"))
            })?;
        tracing::info!(
            driver = "http_capture",
            operation = "init",
            outcome = "success",
            "sms service"
        );
        Ok(Self { cfg, client })
    }
}

#[async_trait]
impl SmsDeliveryService for HttpCaptureSmsAdapter {
    fn driver_name(&self) -> &'static str {
        "http_capture"
    }

    async fn send(&self, envelope: &SmsEnvelope) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        let result = self.send_inner(envelope).await;
        #[cfg(feature = "spectra")]
        crate::spectra_emit::record_terminal("http_capture", result.is_ok());
        result
    }
}

impl HttpCaptureSmsAdapter {
    async fn send_inner(
        &self,
        envelope: &SmsEnvelope,
    ) -> Result<SmsDeliveryReceipt, SmsDeliveryError> {
        validate_e164(&envelope.to_e164)?;
        let started = Instant::now();
        tracing::info!(
            driver = "http_capture",
            operation = "send",
            outcome = "start",
            "sms send"
        );

        let url = self.cfg.messages_url();
        let payload = CaptureBody {
            to_e164: &envelope.to_e164,
            body: &envelope.body,
            otp_code: envelope.otp_code.as_deref(),
        };

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                let elapsed_ms = started.elapsed().as_millis();
                tracing::warn!(
                    driver = "http_capture",
                    operation = "send",
                    outcome = "failure",
                    elapsed_ms,
                    reason_class = "transport_error",
                    "sms send"
                );
                SmsDeliveryError::transport(
                    "transport_error",
                    format!("http capture sink unreachable: {e}"),
                )
            })?;

        let status = response.status();
        let elapsed_ms = started.elapsed().as_millis();
        if status.is_success() {
            tracing::info!(
                driver = "http_capture",
                operation = "send",
                outcome = "success",
                elapsed_ms,
                "sms send"
            );
            return Ok(SmsDeliveryReceipt {
                provider: "http_capture".to_string(),
                message_id: None,
            });
        }

        let code = status.as_u16();
        if (500..600).contains(&code) {
            tracing::warn!(
                driver = "http_capture",
                operation = "send",
                outcome = "failure",
                elapsed_ms,
                reason_class = "provider_unavailable",
                "sms send"
            );
            return Err(SmsDeliveryError::transient(
                "provider_unavailable",
                format!("http capture sink returned HTTP {code}"),
            ));
        }

        tracing::warn!(
            driver = "http_capture",
            operation = "send",
            outcome = "failure",
            elapsed_ms,
            reason_class = "provider_rejected",
            "sms send"
        );
        Err(SmsDeliveryError::rejected(
            "provider_rejected",
            format!("http capture sink returned HTTP {code}"),
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn http_capture_send_happy() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;

        let adapter =
            HttpCaptureSmsAdapter::new(HttpCaptureSmsConfig::new(server.uri()).expect("cfg"))
                .expect("adapter");
        let receipt = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "Your verification code is: 123456".into(),
                otp_code: Some("123456".into()),
            })
            .await
            .expect("send");
        assert_eq!(receipt.provider, "http_capture");
    }

    #[tokio::test]
    async fn http_capture_send_500_transient_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let adapter =
            HttpCaptureSmsAdapter::new(HttpCaptureSmsConfig::new(server.uri()).expect("cfg"))
                .expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "x".into(),
                otp_code: None,
            })
            .await
            .expect_err("500");
        assert!(err.is_transient());
        assert_eq!(err.reason_class(), Some("provider_unavailable"));
    }

    #[tokio::test]
    async fn http_capture_send_400_permanent_sad() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;

        let adapter =
            HttpCaptureSmsAdapter::new(HttpCaptureSmsConfig::new(server.uri()).expect("cfg"))
                .expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "+15551234567".into(),
                body: "x".into(),
                otp_code: None,
            })
            .await
            .expect_err("400");
        assert!(!err.is_transient());
        assert_eq!(err.reason_class(), Some("provider_rejected"));
    }

    #[tokio::test]
    async fn http_capture_invalid_e164_sad() {
        let server = MockServer::start().await;
        let adapter =
            HttpCaptureSmsAdapter::new(HttpCaptureSmsConfig::new(server.uri()).expect("cfg"))
                .expect("adapter");
        let err = adapter
            .send(&SmsEnvelope {
                to_e164: "not-a-phone".into(),
                body: "x".into(),
                otp_code: None,
            })
            .await
            .expect_err("e164");
        assert_eq!(err.reason_class(), Some("invalid_e164"));
    }
}
