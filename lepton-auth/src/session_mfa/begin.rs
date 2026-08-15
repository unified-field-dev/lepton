//! Password / authenticated-user begin paths for login MFA.

use lepton_host_adapter::auth::Credentials;
use lepton_host_adapter::User;
use tower_sessions::Session;
use valence::Valence;

use super::helpers::{finalize_login, store_pending_mfa, user_has_enabled_totp, user_has_webauthn};
use super::{SessionMfaError, SignInOutcome};
use crate::devices::{verify_device_binding, DeviceBindingCookie};
use crate::session_binding::clear_pending_mfa;

/// Authenticate email/password; login immediately or enter pending MFA.
///
/// When `device_cookie` verifies for this user, MFA is skipped and the session is bound.
///
/// # Errors
///
/// [`SessionMfaError`] variants for credentials / store / session.
pub async fn begin_password_sign_in(
    auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    session: &Session,
    valence: &Valence,
    email: String,
    password: String,
    referer: Option<String>,
    device_cookie: Option<&DeviceBindingCookie>,
) -> Result<SignInOutcome, SessionMfaError> {
    let creds = Credentials { email, password };
    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            #[cfg(feature = "spectra")]
            crate::spectra_emit::signin(
                crate::spectra_emit::SigninStage::Password,
                crate::spectra_emit::AuthOutcome::Failure,
                "invalid_credentials",
                crate::spectra_emit::AuthFactor::None,
            );
            return Err(SessionMfaError::InvalidCredentials);
        }
        Err(_) => {
            #[cfg(feature = "spectra")]
            crate::spectra_emit::signin(
                crate::spectra_emit::SigninStage::Password,
                crate::spectra_emit::AuthOutcome::Failure,
                "auth",
                crate::spectra_emit::AuthFactor::None,
            );
            return Err(SessionMfaError::Auth);
        }
    };
    begin_session_for_authenticated_user(
        auth_session,
        session,
        valence,
        &user,
        referer,
        device_cookie,
    )
    .await
}

/// After OAuth (or password) has resolved a [`User`], apply MFA gate / skip / login.
///
/// # Errors
///
/// Store / session / login failures.
pub async fn begin_session_for_authenticated_user(
    auth_session: &mut axum_login::AuthSession<lepton_host_adapter::Backend>,
    session: &Session,
    valence: &Valence,
    user: &User,
    referer: Option<String>,
    device_cookie: Option<&DeviceBindingCookie>,
) -> Result<SignInOutcome, SessionMfaError> {
    clear_pending_mfa(session).await;

    let needs_mfa = user_has_enabled_totp(valence, &user.id).await?;
    if !needs_mfa {
        finalize_login(auth_session, session, user, None).await?;
        tracing::info!(
            operation = "session_mfa.begin",
            outcome = "completed",
            has_totp = false,
            skip_attempted = false,
            "session_mfa begin completed without mfa"
        );
        #[cfg(feature = "spectra")]
        {
            crate::spectra_emit::signin(
                crate::spectra_emit::SigninStage::Password,
                crate::spectra_emit::AuthOutcome::Success,
                "none",
                crate::spectra_emit::AuthFactor::None,
            );
            crate::spectra_emit::signin(
                crate::spectra_emit::SigninStage::Session,
                crate::spectra_emit::AuthOutcome::Success,
                "none",
                crate::spectra_emit::AuthFactor::None,
            );
        }
        return Ok(SignInOutcome::Completed {
            email_verified: user.email_verified,
        });
    }

    if let Some(cookie) = device_cookie {
        match verify_device_binding(valence, &user.id, cookie).await {
            Ok(device_id) => {
                let _ = crate::devices::touch_auth_device(valence, &user.id, &device_id).await;
                finalize_login(auth_session, session, user, Some(&device_id)).await?;
                tracing::info!(
                    operation = "session_mfa.begin",
                    outcome = "completed",
                    has_totp = true,
                    skip_attempted = true,
                    "session_mfa trusted_browser skip"
                );
                #[cfg(feature = "spectra")]
                {
                    crate::spectra_emit::signin(
                        crate::spectra_emit::SigninStage::Password,
                        crate::spectra_emit::AuthOutcome::Success,
                        "none",
                        crate::spectra_emit::AuthFactor::None,
                    );
                    crate::spectra_emit::signin(
                        crate::spectra_emit::SigninStage::Session,
                        crate::spectra_emit::AuthOutcome::Success,
                        "none",
                        crate::spectra_emit::AuthFactor::TrustedBrowser,
                    );
                }
                return Ok(SignInOutcome::Completed {
                    email_verified: user.email_verified,
                });
            }
            Err(_) => {
                tracing::info!(
                    operation = "session_mfa.skip",
                    kind = "trusted_browser",
                    outcome = "reject",
                    "trusted browser skip rejected"
                );
            }
        }
    }

    store_pending_mfa(session, user, referer).await?;
    let has_webauthn = user_has_webauthn(valence, &user.id).await?;
    tracing::info!(
        operation = "session_mfa.begin",
        outcome = "needs_mfa",
        has_totp = true,
        skip_attempted = device_cookie.is_some(),
        "session_mfa needs mfa"
    );
    #[cfg(feature = "spectra")]
    {
        crate::spectra_emit::signin(
            crate::spectra_emit::SigninStage::Password,
            crate::spectra_emit::AuthOutcome::NeedsMfa,
            "none",
            crate::spectra_emit::AuthFactor::None,
        );
        crate::spectra_emit::signin(
            crate::spectra_emit::SigninStage::MfaPending,
            crate::spectra_emit::AuthOutcome::NeedsMfa,
            "none",
            crate::spectra_emit::AuthFactor::Totp,
        );
    }
    Ok(SignInOutcome::NeedsMfa { has_webauthn })
}
