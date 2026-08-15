//! Direct-to-MX delivery driver: config, DNS helpers, and adapter.

mod adapter;
mod config;
mod dns;

pub use adapter::DirectMxAdapter;
pub use config::{DirectMxConfig, DirectMxConfigBuilder};
