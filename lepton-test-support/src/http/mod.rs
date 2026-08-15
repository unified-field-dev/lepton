//! Seed HTTP request/response types and optional Axum handler.
//!
//! # Safety
//!
//! `SeedResponse` may include plaintext password, reset token, and TOTP secret.
//! Mount only on harness / e2e-enabled hosts — never on production product binaries.

#[cfg(feature = "axum")]
mod axum_handler;

#[cfg(feature = "axum")]
pub use axum_handler::{seed_data, seed_error_status, SeedValence};

use serde::{Deserialize, Serialize};

/// Seed request body for `POST /api/test/seed-data`.
#[derive(Debug, Clone, Deserialize)]
pub struct SeedRequest {
    /// Scenario id (see [`crate::scenario`] constants).
    pub scenario: String,
    /// Optional email override.
    #[serde(default)]
    pub email: Option<String>,
    /// Optional password override.
    #[serde(default)]
    pub password: Option<String>,
}

/// Seed response (harness-only; may carry secrets for Playwright).
#[derive(Debug, Clone, Serialize)]
pub struct SeedResponse {
    /// Echoed scenario id.
    pub scenario: String,
    /// Seeded email.
    pub email: String,
    /// Seeded plaintext password.
    pub password: String,
    /// Password-reset token when the scenario issues one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_token: Option<String>,
    /// Base32 TOTP secret for Playwright code generation (test-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_secret: Option<String>,
}
