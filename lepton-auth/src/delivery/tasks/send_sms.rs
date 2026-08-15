//! `lepton_send_sms` Boson task.

use anyhow::Result;
use boson_core::ExecutionContext;
use boson_valence_identity::valence_from_context;
use lepton_host_adapter::generated::{DeliveryAttemptChannel, DeliveryAttemptOutcome};
use lepton_sms::SmsEnvelope;

use crate::delivery::attempt::{record_delivery_attempt, DeliveryAttemptInput};
use crate::delivery::runtime::DeliveryRuntime;

/// Durable SMS send: retry Transient; terminal-Ok permanent failures after logging.
#[boson_macros::task(
    name = "lepton_send_sms",
    priority = 50,
    pool = "global",
    idempotency_mode = "lwt",
    max_attempts = 5,
    base_delay_ms = 1000,
    backoff_multiplier = 2.0,
    max_delay_ms = 60_000,
    max_in_flight = 100,
    max_enqueue_per_second = 50
)]
pub async fn lepton_send_sms(
    ctx: Box<dyn ExecutionContext>,
    intent_kind: String,
    intent_id: String,
    to_e164: String,
    body: String,
    otp_code: Option<String>,
) -> Result<()> {
    let valence = valence_from_context(ctx.as_ref()).map_err(|e| anyhow::anyhow!("{e}"))?;
    let runtime = DeliveryRuntime::get().map_err(|e| anyhow::anyhow!("{e}"))?;
    let sms = runtime.sms().map_err(|e| anyhow::anyhow!("{e}"))?;
    let envelope = SmsEnvelope {
        to_e164,
        body,
        otp_code,
    };

    match sms.send(&envelope).await {
        Ok(receipt) => {
            tracing::info!(
                channel = "sms",
                intent_kind = %intent_kind,
                outcome = "success",
                provider = %receipt.provider,
                has_message_id = receipt.message_id.is_some(),
                "lepton.delivery.attempt"
            );
            record_delivery_attempt(
                &valence,
                DeliveryAttemptInput {
                    channel: DeliveryAttemptChannel::Sms,
                    intent_kind,
                    intent_id,
                    provider: Some(receipt.provider),
                    message_id: receipt.message_id,
                    outcome: DeliveryAttemptOutcome::Success,
                    reason_class: None,
                    boson_job_id: None,
                },
            )
            .await?;
            Ok(())
        }
        Err(err) if err.is_transient() => {
            let reason = err.reason_class().unwrap_or("transient").to_string();
            tracing::warn!(
                channel = "sms",
                intent_kind = %intent_kind,
                outcome = "transient",
                reason_class = %reason,
                "lepton.delivery.attempt"
            );
            let _ = record_delivery_attempt(
                &valence,
                DeliveryAttemptInput {
                    channel: DeliveryAttemptChannel::Sms,
                    intent_kind,
                    intent_id,
                    provider: None,
                    message_id: None,
                    outcome: DeliveryAttemptOutcome::Transient,
                    reason_class: Some(reason.clone()),
                    boson_job_id: None,
                },
            )
            .await;
            Err(anyhow::anyhow!("transient delivery: {reason}"))
        }
        Err(err) => {
            let reason = err.reason_class().unwrap_or("permanent").to_string();
            tracing::warn!(
                channel = "sms",
                intent_kind = %intent_kind,
                outcome = "permanent",
                reason_class = %reason,
                "lepton.delivery.attempt"
            );
            let _ = record_delivery_attempt(
                &valence,
                DeliveryAttemptInput {
                    channel: DeliveryAttemptChannel::Sms,
                    intent_kind,
                    intent_id,
                    provider: None,
                    message_id: None,
                    outcome: DeliveryAttemptOutcome::Permanent,
                    reason_class: Some(reason),
                    boson_job_id: None,
                },
            )
            .await;
            Ok(())
        }
    }
}
