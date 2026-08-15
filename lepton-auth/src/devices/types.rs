//! Serializable device DTOs shared by library APIs and `#[server]` fns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Public device kind (mirrors Valence enum).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthDeviceKind {
    /// Cookie / browser trust after confirm.
    TrustedBrowser,
    /// `WebAuthn` / passkey device (use ceremony APIs; confirm-code register unsupported).
    WebAuthn,
}

/// Pending registration: device id + one-time confirm code (never logged).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingAuthDevice {
    /// Valence `auth_device` id.
    pub device_id: String,
    /// One-time confirm code returned once to the caller.
    pub confirm_code: String,
}

/// Safe list view (no secret hashes / passkey material).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthDeviceView {
    /// Device id.
    pub id: String,
    /// Kind label.
    pub kind: AuthDeviceKind,
    /// Operator-facing label.
    pub label: String,
    /// `WebAuthn` credential id when present (safe handle).
    pub credential_id: Option<String>,
    /// `WebAuthn` signature counter when present.
    pub sign_count: Option<i64>,
    /// When confirmed trusted.
    pub trusted_at: Option<DateTime<Utc>>,
    /// Last seen.
    pub last_seen_at: Option<DateTime<Utc>>,
    /// When revoked.
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Pending registration: ceremony id + creation options for the browser.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingWebauthnRegistration {
    /// Server-side ceremony id (paired with stored registration state).
    pub ceremony_id: String,
    /// `PublicKeyCredentialCreationOptions` JSON for `navigator.credentials.create`.
    pub creation_options: Value,
}

/// Pending assertion: ceremony id + request options for the browser.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingWebauthnAssertion {
    /// Server-side ceremony id (paired with stored authentication state).
    pub ceremony_id: String,
    /// `PublicKeyCredentialRequestOptions` JSON for `navigator.credentials.get`.
    pub request_options: Value,
}

/// Result of a finished `WebAuthn` registration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisteredWebauthnDevice {
    /// Valence `auth_device` id.
    pub device_id: String,
    /// Credential id (base64url) for operators.
    pub credential_id: String,
}
