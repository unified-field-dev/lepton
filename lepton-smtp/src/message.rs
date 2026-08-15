//! Shared lettre [`Message`] assembly for SMTP and direct-MX adapters.

use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::Message;

use crate::envelope::EmailEnvelope;
use crate::error::EmailDeliveryError;

/// Build a multipart alternative message from envelope fields.
pub fn build_message(
    from_name: &str,
    from_email: &str,
    envelope: &EmailEnvelope,
) -> Result<Message, EmailDeliveryError> {
    let from_mailbox = Mailbox::new(
        Some(from_name.to_string()),
        from_email.parse().map_err(|e| {
            EmailDeliveryError::config("invalid_mailbox", format!("Invalid from address: {e}"))
        })?,
    );
    let to_mailbox: Mailbox = envelope.to.parse().map_err(|e| {
        EmailDeliveryError::config("invalid_mailbox", format!("Invalid recipient email: {e}"))
    })?;

    Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(envelope.subject.as_str())
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(envelope.text_body.clone()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(envelope.html_body.clone()),
                ),
        )
        .map_err(|e| {
            EmailDeliveryError::config(
                "message_build_failed",
                format!("Failed to build email message: {e}"),
            )
        })
}
