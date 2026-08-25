//! Build a Noop SMS service, send an envelope, assert the receipt.
//!
//! ```bash
//! cargo run -p lepton-sms --example noop_send
//! ```

use lepton_sms::{SmsEnvelope, SmsServiceBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sms = SmsServiceBuilder::new().noop().build()?;

    let receipt = sms
        .send(&SmsEnvelope {
            to_e164: "+15551234567".into(),
            body: "Your code is 123456".into(),
            otp_code: Some("123456".into()),
        })
        .await?;

    assert_eq!(receipt.provider, "noop");
    assert!(receipt.message_id.is_none());
    Ok(())
}
