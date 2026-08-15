//! Live Twilio `SendGrid` Mail Send adapter (`feature = "twilio"`).

mod adapter;
mod config;
mod http;

pub use adapter::TwilioEmailAdapter;
pub use config::{
    TwilioEmailConfig, TwilioEmailConfigBuilder, TWILIO_EMAIL_API_BASE_URL,
    TWILIO_EMAIL_API_KEY_ENV,
};
