//! Validating email counter emit against an in-memory Spectra backend.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use lepton_spectra_telemetry::{record_email_send, EmailSendOutcome};
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
async fn email_send_success_with_mem_spectra_happy() {
    let _ = spectra();
    let before = metric_points("lepton_email_send").await;
    record_email_send("noop", EmailSendOutcome::Success);
    let after = metric_points("lepton_email_send").await;
    assert!(
        after > before,
        "expected lepton_email_send points to increase (before={before}, after={after})"
    );
}

#[tokio::test]
async fn email_send_failure_with_mem_spectra_happy() {
    let _ = spectra();
    let before = metric_points("lepton_email_send").await;
    record_email_send("smtp", EmailSendOutcome::Failure);
    let after = metric_points("lepton_email_send").await;
    assert!(
        after > before,
        "expected failure emit to persist (before={before}, after={after})"
    );
}

#[test]
fn schemas_register_lepton_email_send_happy() {
    let registry = SchemaRegistry::auto_discover();
    assert!(
        registry.has_schema("lepton_email_send"),
        "lepton_email_send must register via inventory; have {:?}",
        registry.list_schemas()
    );
}
