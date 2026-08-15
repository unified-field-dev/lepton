//! SMTP relay driver: config + adapter.

mod adapter;
mod config;

pub use adapter::SmtpAdapter;
pub use config::{SmtpConfig, SmtpConfigBuilder};
