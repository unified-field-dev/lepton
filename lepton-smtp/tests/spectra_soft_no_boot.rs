//! `spectra` feature must not break send when Spectra is not installed.

#![cfg(feature = "spectra")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lepton_smtp::{EmailDeliveryService, EmailEnvelope, NoopEmailAdapter};

#[tokio::test]
async fn noop_send_without_spectra_boot_still_ok_happy() {
    let receipt = NoopEmailAdapter
        .send(&EmailEnvelope {
            to: "ops@example.test".into(),
            subject: "t".into(),
            text_body: "b".into(),
            html_body: "<p>b</p>".into(),
        })
        .await
        .expect("noop send");
    assert_eq!(receipt.provider, "noop");
}
