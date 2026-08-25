//! Build a Noop email service, send a stock verification envelope, assert the receipt.
//!
//! ```bash
//! cargo run -p lepton-smtp --example noop_send
//! ```

use lepton_smtp::{verification_email_envelope, EmailServiceBuilder, VerificationEmailFlow};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let email = EmailServiceBuilder::new().noop().build()?;

    let message = verification_email_envelope(
        "reader@example.test",
        "123456",
        VerificationEmailFlow::Signup,
    );
    let receipt = email.send(&message).await?;

    assert_eq!(receipt.provider, "noop");
    assert!(receipt.message_id.is_none());
    Ok(())
}
