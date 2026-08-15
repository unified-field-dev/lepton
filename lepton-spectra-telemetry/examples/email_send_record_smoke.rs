//! Smoke: boot mem Spectra and record one email send counter.
//!
//! ```bash
//! cargo run -p lepton-spectra-telemetry --example email_send_record_smoke
//! ```

#![allow(clippy::print_stderr, clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use lepton_spectra_telemetry::{record_email_send, EmailSendOutcome};
use spectra::{MemEventsBackend, MemMetricsBackend, Spectra};
use spectra_core::{current_emit_ts, MetricsQueryRange};

#[tokio::main]
async fn main() {
    let spectra = Spectra::builder()
        .metrics_backend(Arc::new(MemMetricsBackend::new()))
        .events_backend(Arc::new(MemEventsBackend::new()))
        .embedded()
        .build()
        .expect("spectra boot");

    record_email_send("noop", EmailSendOutcome::Success);
    tokio::time::sleep(Duration::from_millis(80)).await;

    let now = current_emit_ts();
    let points = spectra
        .router()
        .query_metrics(MetricsQueryRange {
            metric_name: "lepton_email_send".into(),
            start: now - chrono::Duration::seconds(5),
            end: now + chrono::Duration::seconds(1),
            label_matchers: vec![],
        })
        .await
        .expect("query");

    eprintln!(
        "email_send_record_smoke OK: {} lepton_email_send point(s)",
        points.len()
    );
}
