//! Trusted browser / `WebAuthn` device registration.
//!
//! Manages TrustedBrowser confirm-code lifecycle, binding cookie mint/verify for MFA-skip,
//! `WebAuthn` registration / assertion ceremony (feature `webauthn`), list / revoke,
//! and serializable DTOs for server fns. Google / GitHub OAuth ([`crate::oauth`] /
//! `LinkedIdentity`) and passwordless login are separate; host enroll UI uses Security
//! devices + [`crate::actions::devices`]. Session MFA orchestration lives in [`crate::session_mfa`].
//!
//! # Examples
//!
//! Trusted browser:
//!
//! ```rust,ignore
//! use lepton_auth::devices::{
//!     confirm_auth_device, issue_device_binding, list_auth_devices, register_auth_device,
//!     revoke_auth_device, AuthDeviceKind,
//! };
//!
//! async fn trust_this_browser(v: &valence::Valence, user: valence::RecordId) -> Result<(), Box<dyn std::error::Error>> {
//!     let pending = register_auth_device(v, &user, AuthDeviceKind::TrustedBrowser, "Laptop").await?;
//!     confirm_auth_device(v, &user, &pending.device_id, &pending.confirm_code).await?;
//!     let cookie = issue_device_binding(v, &user, &pending.device_id).await?;
//!     let devices = list_auth_devices(v, &user).await?;
//!     assert!(devices.iter().any(|d| d.trusted_at.is_some()));
//!     let _ = cookie;
//!     revoke_auth_device(v, &user, &pending.device_id).await?;
//!     Ok(())
//! }
//! ```
//!
//! `WebAuthn` (requires feature `webauthn`):
//!
//! ```rust,ignore
//! use lepton_auth::devices::{
//!     begin_webauthn_assertion, begin_webauthn_registration, finish_webauthn_assertion,
//!     finish_webauthn_registration, list_auth_devices, revoke_auth_device, WebauthnRpConfig,
//! };
//!
//! async fn register_passkey(v: &valence::Valence, user: valence::RecordId) -> Result<(), Box<dyn std::error::Error>> {
//!     let rp = WebauthnRpConfig {
//!         rp_id: "localhost".into(),
//!         rp_origin: "http://localhost:3000".into(),
//!         rp_name: "Lepton".into(),
//!     };
//!     let reg = begin_webauthn_registration(v, &rp, &user, "YubiKey").await?;
//!     // Host: navigator.credentials.create(reg.creation_options) → attestation_json
//!     let device = finish_webauthn_registration(v, &rp, &user, &reg.ceremony_id, &attestation_json).await?;
//!     let assert_start = begin_webauthn_assertion(v, &rp, &user).await?;
//!     let _ = finish_webauthn_assertion(v, &rp, &user, &assert_start.ceremony_id, &assertion_json).await?;
//!     let _ = list_auth_devices(v, &user).await?;
//!     revoke_auth_device(v, &user, &device.device_id).await?;
//!     Ok(())
//! }
//! ```

mod error;
mod types;

#[cfg(feature = "ssr")]
mod api;
#[cfg(feature = "ssr")]
mod binding;

#[cfg(all(feature = "ssr", feature = "webauthn"))]
mod ceremony;
#[cfg(all(feature = "ssr", feature = "webauthn"))]
mod rp;
#[cfg(all(feature = "ssr", feature = "webauthn"))]
mod webauthn;

pub use error::DeviceError;
pub use types::{
    AuthDeviceKind, AuthDeviceView, PendingAuthDevice, PendingWebauthnAssertion,
    PendingWebauthnRegistration, RegisteredWebauthnDevice,
};

#[cfg(feature = "ssr")]
pub use api::{
    confirm_auth_device, list_auth_devices, register_auth_device, revoke_auth_device,
    touch_auth_device,
};
#[cfg(feature = "ssr")]
pub use binding::{
    issue_device_binding, register_and_bind_trusted_browser, verify_device_binding,
    DeviceBindingCookie, DEVICE_BINDING_COOKIE,
};

#[cfg(all(feature = "ssr", feature = "webauthn"))]
pub use rp::WebauthnRpConfig;
#[cfg(all(feature = "ssr", feature = "webauthn"))]
pub use webauthn::{
    begin_webauthn_assertion, begin_webauthn_registration, finish_webauthn_assertion,
    finish_webauthn_registration,
};
