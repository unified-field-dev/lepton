//! Complete pending MFA with TOTP or `WebAuthn`.

#[cfg(any(feature = "totp", feature = "webauthn"))]
use axum_login::AuthnBackend;
use tower_sessions::Session;
use valence::Valence;

#[cfg(any(feature = "totp", feature = "webauthn"))]
use super::helpers::finalize_login;
use super::helpers::load_pending_mfa;
use super::{SessionMfaError, SignInOutcome};
#[cfg(feature = "totp")]
use crate::devices::register_and_bind_trusted_browser;
use crate::devices::DeviceBindingCookie;
#[cfg(feature = "totp")]
use crate::factor::{FactorChallengeError, FactorChallengeService};
#[cfg(feature = "totp")]
use crate::services::LeptonAuthServices;
#[cfg(any(feature = "totp", feature = "webauthn"))]
use crate::session_binding::clear_pending_mfa;

/// Whether to mint a `TrustedBrowser` binding cookie after TOTP success.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RememberDevice {
    /// Do not issue a binding cookie.
    No,
    /// Register + confirm `TrustedBrowser` and return binding cookie.
    Yes {
        /// Device label shown in Account Settings.
        label: &'static str,
    },
}

impl RememberDevice {
    /// Convenience for UI checkbox.
    #[must_use]
    pub const fn from_bool(remember: bool) -> Self {
        if remember {
            Self::Yes {
                label: "Trusted browser",
            }
        } else {
            Self::No
        }
    }
}

/// Result of completing MFA (may include a Set-Cookie binding).
#[derive(Clone, Debug)]
pub struct CompleteMfaResult {
    /// Sign-in outcome (always [`SignInOutcome::Completed`] on success).
    pub outcome: SignInOutcome,
    /// Optional `TrustedBrowser` binding cookie to set on the response.
    pub binding_cookie: Option<DeviceBindingCookie>,
    /// Optional referer stored when pending MFA began.
    pub referer: Option<String>,
}

/// Read pending MFA user id (for `WebAuthn` begin before finish).
///
/// # Errors
///
/// Missing / expired pending bag.
pub async fn pending_mfa_user_id(session: &Session) -> Result<String, SessionMfaError> {
    Ok(load_pending_mfa(session).await?.user_id)
}

/// True when `code` is shaped like a six-digit authenticator code (not a recovery token).
#[cfg(feature = "totp")]
fn is_totp_digit_code(code: &str) -> bool {
    let trimmed = code.trim();
    trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit())
}

/// Complete pending MFA with a TOTP or recovery code; optionally remember this browser.
///
/// Six ASCII digits → TOTP verify; otherwise → one-time recovery consume. Failures use the
/// same [`SessionMfaError::TotpInvalid`] path (no factor oracle).
///
/// # Errors
///
/// Pending / TOTP / login failures.
#[cfg(feature = "totp")]
#[allow(clippy::too_many_lines)] // pending load + TOTP/recovery verify + optional remember device
pub async fn complete_sign_in_totp(
    auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    session: &Session,
    valence: &Valence,
    services: std::sync::Arc<LeptonAuthServices>,
    code: &str,
    remember: RememberDevice,
) -> Result<CompleteMfaResult, SessionMfaError> {
    let pending = load_pending_mfa(session).await?;
    let backend = auth_session.backend.clone();
    let user = backend
        .get_user(&pending.user_id)
        .await
        .map_err(|_| SessionMfaError::Auth)?
        .ok_or(SessionMfaError::PendingStale)?;
    if user.session_stamp != pending.auth_hash {
        clear_pending_mfa(session).await;
        return Err(SessionMfaError::PendingStale);
    }

    let factors = FactorChallengeService::new(services);
    let user_rid = user.id.clone();
    let mfa_result = if is_totp_digit_code(code) {
        factors.verify_totp_code(valence, &user_rid, code).await
    } else {
        factors
            .consume_totp_recovery_code(valence, &user_rid, code)
            .await
    };
    match mfa_result {
        Ok(()) => {}
        Err(FactorChallengeError::TotpInvalid) => {
            #[cfg(feature = "spectra")]
            crate::spectra_emit::signin(
                crate::spectra_emit::SigninStage::MfaComplete,
                crate::spectra_emit::AuthOutcome::Failure,
                "mismatch",
                crate::spectra_emit::AuthFactor::Totp,
            );
            return Err(SessionMfaError::TotpInvalid);
        }
        Err(FactorChallengeError::TotpUnavailable) => {
            #[cfg(feature = "spectra")]
            crate::spectra_emit::signin(
                crate::spectra_emit::SigninStage::MfaComplete,
                crate::spectra_emit::AuthOutcome::Failure,
                "totp_unavailable",
                crate::spectra_emit::AuthFactor::Totp,
            );
            return Err(SessionMfaError::TotpUnavailable);
        }
        Err(FactorChallengeError::TotpSecret) => {
            tracing::warn!(
                operation = "session_mfa.complete_totp",
                outcome = "totp_secret",
                "totp secret unusable"
            );
            return Err(SessionMfaError::TotpUnavailable);
        }
        Err(_) => return Err(SessionMfaError::Store),
    }

    let mut binding_cookie = None;
    let mut device_id = None;
    if matches!(remember, RememberDevice::Yes { .. }) {
        let label = match remember {
            RememberDevice::Yes { label } => label,
            RememberDevice::No => "Trusted browser",
        };
        match register_and_bind_trusted_browser(valence, &user_rid, label).await {
            Ok(cookie) => {
                device_id = Some(cookie.device_id.clone());
                binding_cookie = Some(cookie);
            }
            Err(_) => {
                tracing::warn!(
                    operation = "session_mfa.complete_totp",
                    outcome = "remember_failed",
                    "remember device failed; continuing login"
                );
            }
        }
    }

    let referer = pending.referer.clone();
    let email_verified = user.email_verified;
    finalize_login(auth_session, session, &user, device_id.as_deref()).await?;
    tracing::info!(
        operation = "session_mfa.complete_totp",
        outcome = "ok",
        "totp mfa completed"
    );
    #[cfg(feature = "spectra")]
    {
        crate::spectra_emit::signin(
            crate::spectra_emit::SigninStage::MfaComplete,
            crate::spectra_emit::AuthOutcome::Success,
            "none",
            crate::spectra_emit::AuthFactor::Totp,
        );
        crate::spectra_emit::signin(
            crate::spectra_emit::SigninStage::Session,
            crate::spectra_emit::AuthOutcome::Success,
            "none",
            crate::spectra_emit::AuthFactor::Totp,
        );
    }
    Ok(CompleteMfaResult {
        outcome: SignInOutcome::Completed { email_verified },
        binding_cookie,
        referer,
    })
}

/// Stub when the `totp` feature is disabled — always returns [`SessionMfaError::TotpUnavailable`].
#[cfg(not(feature = "totp"))]
pub async fn complete_sign_in_totp(
    _auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    _session: &Session,
    _valence: &Valence,
    _services: std::sync::Arc<crate::services::LeptonAuthServices>,
    _code: &str,
    _remember: RememberDevice,
) -> Result<CompleteMfaResult, SessionMfaError> {
    Err(SessionMfaError::TotpUnavailable)
}

/// Complete pending MFA with a `WebAuthn` assertion; binds the asserted device.
///
/// # Errors
///
/// Pending / `WebAuthn` / login failures.
#[cfg(feature = "webauthn")]
pub async fn complete_sign_in_webauthn(
    auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    session: &Session,
    valence: &Valence,
    rp: &crate::devices::WebauthnRpConfig,
    ceremony_id: &str,
    assertion_json: &serde_json::Value,
) -> Result<CompleteMfaResult, SessionMfaError> {
    use crate::devices::finish_webauthn_assertion;

    let pending = load_pending_mfa(session).await?;
    let backend = auth_session.backend.clone();
    let user = backend
        .get_user(&pending.user_id)
        .await
        .map_err(|_| SessionMfaError::Auth)?
        .ok_or(SessionMfaError::PendingStale)?;
    if user.session_stamp != pending.auth_hash {
        clear_pending_mfa(session).await;
        return Err(SessionMfaError::PendingStale);
    }

    let user_rid = user.id.clone();
    let view = finish_webauthn_assertion(valence, rp, &user_rid, ceremony_id, assertion_json)
        .await
        .map_err(|_| SessionMfaError::Webauthn)?;

    let referer = pending.referer.clone();
    let email_verified = user.email_verified;
    finalize_login(auth_session, session, &user, Some(&view.id)).await?;
    tracing::info!(
        operation = "session_mfa.complete_webauthn",
        outcome = "ok",
        "webauthn mfa completed"
    );
    #[cfg(feature = "spectra")]
    {
        crate::spectra_emit::signin(
            crate::spectra_emit::SigninStage::MfaComplete,
            crate::spectra_emit::AuthOutcome::Success,
            "none",
            crate::spectra_emit::AuthFactor::Webauthn,
        );
        crate::spectra_emit::signin(
            crate::spectra_emit::SigninStage::Session,
            crate::spectra_emit::AuthOutcome::Success,
            "none",
            crate::spectra_emit::AuthFactor::Webauthn,
        );
    }
    Ok(CompleteMfaResult {
        outcome: SignInOutcome::Completed { email_verified },
        binding_cookie: None,
        referer,
    })
}

/// Stub when `webauthn` feature is off.
#[cfg(not(feature = "webauthn"))]
pub async fn complete_sign_in_webauthn(
    _auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    _session: &Session,
    _valence: &Valence,
    _ceremony_id: &str,
    _assertion_json: &serde_json::Value,
) -> Result<CompleteMfaResult, SessionMfaError> {
    Err(SessionMfaError::Webauthn)
}
