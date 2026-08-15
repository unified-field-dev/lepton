//! Boson task handlers for durable email / SMS delivery.

// Macro-generated `*Params` / handle types lack field rustdoc.
#![allow(missing_docs)]

#[cfg(feature = "email")]
mod send_email;
#[cfg(feature = "phone")]
mod send_sms;

#[cfg(feature = "email")]
pub use send_email::{LeptonSendEmail, LeptonSendEmailParams};
#[cfg(feature = "phone")]
pub use send_sms::{LeptonSendSms, LeptonSendSmsParams};
