//! Test-only Lepton identity builders, seed scenarios, and harness seed HTTP.
//!
//! `publish = false`. Shortcut seeds for Playwright / integ — not the production
//! signup pipeline (`lepton_e2e` / `signup_api`).
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Fluent user in a known state | [`builder::TestUserBuilder`] |
//! | Named Playwright scenarios | [`scenario::run_seed`] |
//! | HTTP types | [`http::SeedRequest`] / [`http::SeedResponse`] |
//! | Axum mount (`feature = "axum"`) | `http::seed_data`, `http::SeedValence` (see [`http`]) |
//! | Fixed TOTP secret | [`builder::HARNESS_TOTP_SECRET`] |
//!
//! # Must not mount in production
//!
//! The Axum seed route is unauthenticated and returns plaintext passwords /
//! reset tokens / TOTP secrets. Use only on harness binaries or hosts gated by
//! an e2e Cargo feature.

#![deny(missing_docs)]

pub mod builder;
pub mod error;
pub mod http;
pub mod scenario;

pub use builder::{
    unique_e164, SeededUser, TestUserBuilder, DEFAULT_PASSWORD, HARNESS_TOTP_SECRET,
};
pub use error::SeedError;
pub use http::{SeedRequest, SeedResponse};
pub use scenario::run_seed;

#[cfg(feature = "axum")]
pub use http::{seed_data, seed_error_status, SeedValence};
