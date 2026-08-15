//! Validating SMS counter emit against an in-memory Spectra backend.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use lepton_spectra_telemetry::{record_sms_send, SmsSendOutcome};
use spectra::{MemEventsBackend, MemMetricsBackend, SchemaRegistry, Spectra};
use spectra_core::{current_emit_ts, MetricsQueryRange};

static SPECTRA: OnceLock<Spectra> = OnceLock::new();

fn spectra() -> &'static Spectra {
    SPECTRA.get_or_init(|| {
        Spectra::builder()
            .metrics_backend(Arc::new(MemMetricsBackend::new()))
            .events_backend(Arc::new(MemEventsBackend::new()))
            .embedded()
            .build()
            .expect("spectra boot")
    })
}

async fn metric_points(name: &str) -> usize {
    tokio::time::sleep(Duration::from_millis(80)).await;
    let now = current_emit_ts();
    spectra()
        .router()
        .query_metrics(MetricsQueryRange {
            metric_name: name.into(),
            start: now - chrono::Duration::seconds(30),
            end: now + chrono::Duration::seconds(5),
            label_matchers: vec![],
        })
        .await
        .expect("query")
        .len()
}

#[tokio::test]
async fn sms_send_success_with_mem_spectra_happy() {
    let _ = spectra();
    let before = metric_points("lepton_sms_send").await;
    record_sms_send("test", SmsSendOutcome::Success);
    let after = metric_points("lepton_sms_send").await;
    assert!(
        after > before,
        "expected lepton_sms_send points to increase (before={before}, after={after})"
    );
}

#[tokio::test]
async fn sms_send_failure_with_mem_spectra_happy() {
    let _ = spectra();
    let before = metric_points("lepton_sms_send").await;
    record_sms_send("twilio", SmsSendOutcome::Failure);
    let after = metric_points("lepton_sms_send").await;
    assert!(
        after > before,
        "expected failure emit to persist (before={before}, after={after})"
    );
}

#[test]
fn schemas_register_lepton_sms_send_happy() {
    let registry = SchemaRegistry::auto_discover();
    assert!(
        registry.has_schema("lepton_sms_send"),
        "lepton_sms_send must register via inventory; have {:?}",
        registry.list_schemas()
    );
}
