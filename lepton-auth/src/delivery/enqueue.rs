//! Enqueue durable delivery jobs with LWT idempotency keys.

use serde::Serialize;
use thiserror::Error;

#[cfg(feature = "phone")]
use lepton_sms::SmsEnvelope;
#[cfg(feature = "email")]
use lepton_smtp::EmailEnvelope;

/// Email delivery intent for Boson enqueue.
#[cfg(feature = "email")]
#[derive(Clone, Debug)]
pub struct EmailDeliveryIntent {
    /// Auth flow label (e.g. `signup_verify`).
    pub intent_kind: String,
    /// Soft correlation id (token id).
    pub intent_id: String,
    /// Message to send (includes recipient + body — stored in Boson params).
    pub envelope: EmailEnvelope,
}

/// SMS delivery intent for Boson enqueue.
#[cfg(feature = "phone")]
#[derive(Clone, Debug)]
pub struct SmsDeliveryIntent {
    /// Auth flow label (e.g. `sms_otp`).
    pub intent_kind: String,
    /// Soft correlation id (challenge / token id).
    pub intent_id: String,
    /// Message to send.
    pub envelope: SmsEnvelope,
}

/// Errors enqueueing durable delivery.
#[derive(Debug, Error)]
pub enum EnqueueDeliveryError {
    /// Boson runtime not configured.
    #[error("reason_class=delivery: boson not configured")]
    BosonNotConfigured,
    /// Enqueue / serialize failure (opaque).
    #[error("reason_class=delivery: enqueue failed")]
    Enqueue,
}

fn system_actor_json() -> serde_json::Value {
    serde_json::json!({"System": {"operation": "lepton_delivery"}})
}

async fn enqueue_named<P: Serialize>(
    task_name: &'static str,
    idempotency_key: String,
    params: P,
) -> Result<String, EnqueueDeliveryError> {
    let boson = boson_runtime::default().ok_or(EnqueueDeliveryError::BosonNotConfigured)?;
    let params_json = serde_json::to_value(params).map_err(|_| EnqueueDeliveryError::Enqueue)?;
    tracing::info!(
        channel = task_name,
        outcome = "queued",
        "lepton.delivery.enqueue"
    );
    boson
        .enqueue(
            task_name,
            system_actor_json(),
            params_json,
            Some(idempotency_key),
        )
        .await
        .map_err(|_| EnqueueDeliveryError::Enqueue)
}

/// Enqueue the `lepton_send_email` Boson task ([`LeptonSendEmailParams`](crate::delivery::tasks::LeptonSendEmailParams))
/// with LWT key `email:{intent_kind}:{intent_id}`.
///
/// # Errors
///
/// [`EnqueueDeliveryError`] when Boson is missing or enqueue fails.
#[cfg(feature = "email")]
pub async fn enqueue_email(intent: EmailDeliveryIntent) -> Result<String, EnqueueDeliveryError> {
    let key = format!("email:{}:{}", intent.intent_kind, intent.intent_id);
    let params = crate::delivery::tasks::LeptonSendEmailParams {
        intent_kind: intent.intent_kind,
        intent_id: intent.intent_id,
        to: intent.envelope.to,
        subject: intent.envelope.subject,
        text_body: intent.envelope.text_body,
        html_body: intent.envelope.html_body,
    };
    enqueue_named("lepton_send_email", key, params).await
}

/// Enqueue the `lepton_send_sms` Boson task ([`LeptonSendSmsParams`](crate::delivery::tasks::LeptonSendSmsParams))
/// with LWT key `sms:{intent_kind}:{intent_id}`.
///
/// # Errors
///
/// [`EnqueueDeliveryError`] when Boson is missing or enqueue fails.
#[cfg(feature = "phone")]
pub async fn enqueue_sms(intent: SmsDeliveryIntent) -> Result<String, EnqueueDeliveryError> {
    let key = format!("sms:{}:{}", intent.intent_kind, intent.intent_id);
    let params = crate::delivery::tasks::LeptonSendSmsParams {
        intent_kind: intent.intent_kind,
        intent_id: intent.intent_id,
        to_e164: intent.envelope.to_e164,
        body: intent.envelope.body,
        otp_code: intent.envelope.otp_code,
    };
    enqueue_named("lepton_send_sms", key, params).await
}
