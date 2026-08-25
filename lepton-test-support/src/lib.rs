//! Test-only Lepton identity builders, seed scenarios, and harness seed HTTP.
//!
//! `publish = false`. Shortcut seeds for Playwright / integ — not the production
//! signup pipeline (`lepton_e2e` / `signup_api`).
//!
//! # Features
//!
//! - **Fluent user builder** — Seeds known harness identity states with
//!   [`builder::TestUserBuilder`] so Playwright and integ tests skip the signup UI
//!   ([Seed a user with TOTP](#seed-a-user-with-totp)).
//! - **Named seed scenarios** — Runs [`scenario::run_seed`] for the same JSON scenarios
//!   Playwright expects ([Seed a user with TOTP](#seed-a-user-with-totp)).
//! - **Seed HTTP types** — Exposes [`http::SeedRequest`] / [`http::SeedResponse`] for
//!   harness HTTP bodies ([Seed a user with TOTP](#seed-a-user-with-totp)).
//! - **Axum seed route** — Mounts `http::seed_data` when the `axum` feature is on for
//!   harness-only hosts ([`http`] module docs). Never enable on production.
//!
//! # Getting started
//!
//! ## Seed a user with TOTP
//!
//! Seeds a harness user in a known state (including TOTP) so Playwright and integration
//! tests skip the signup UI. Use this only on test binaries—not the production signup
//! pipeline.
//!
//! Prerequisites: a harness Valence and this crate on the test binary.
//!
//! 1. Build with [`TestUserBuilder`].
//! 2. Enable TOTP via [`TestUserBuilder::with_totp`] (fixed secret:
//!    [`HARNESS_TOTP_SECRET`]).
//! 3. Assert the seeded user carries a TOTP secret.
//!
//! Errors: builder/seed helpers return [`SeedError`]. Next: [`run_seed`] for named
//! Playwright scenarios, or the Axum seed route under `feature = "axum"`.
//!
//! ```rust,ignore
//! use lepton_test_support::{TestUserBuilder, HARNESS_TOTP_SECRET};
//!
//! # async fn demo(v: &valence::Valence) -> Result<(), lepton_test_support::SeedError> {
//! let user = TestUserBuilder::new()
//!     .email("seed@example.test")
//!     .with_totp()
//!     .build(v)
//!     .await?;
//! assert!(user.totp_secret.is_some());
//! assert_eq!(user.totp_secret.as_deref(), Some(HARNESS_TOTP_SECRET));
//! # Ok(())
//! # }
//! ```
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `axum` | Axum seed route helpers (`seed_data`, `SeedValence`) |
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
