//! `TrustedBrowser` MFA-skip after identity is known (used from begin path).

use lepton_host_adapter::User;
use tower_sessions::Session;
use valence::Valence;

use super::helpers::finalize_login;
use super::{SessionMfaError, SignInOutcome};
use crate::devices::{touch_auth_device, verify_device_binding, DeviceBindingCookie};

/// If `cookie` verifies for `user`, login and bind the device.
///
/// Returns [`None`] when the cookie is absent or invalid (caller continues MFA).
///
/// # Errors
///
/// Login / session failures after a **valid** cookie (invalid cookie → `Ok(None)`).
pub async fn try_mfa_skip_trusted_browser(
    auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    session: &Session,
    valence: &Valence,
    user: &User,
    cookie: &DeviceBindingCookie,
) -> Result<Option<SignInOutcome>, SessionMfaError> {
    let Ok(device_id) = verify_device_binding(valence, &user.id, cookie).await else {
        tracing::info!(
            operation = "session_mfa.skip",
            kind = "trusted_browser",
            outcome = "reject",
            "trusted browser skip rejected"
        );
        return Ok(None);
    };
    let _ = touch_auth_device(valence, &user.id, &device_id).await;
    finalize_login(auth_session, session, user, Some(&device_id)).await?;
    tracing::info!(
        operation = "session_mfa.skip",
        kind = "trusted_browser",
        outcome = "ok",
        "trusted browser mfa skip"
    );
    Ok(Some(SignInOutcome::Completed {
        email_verified: user.email_verified,
    }))
}
