//! SSR helpers for lepton-auth server functions.

use lepton_host_adapter::{Backend, User};
use leptos::prelude::*;

/// Extract [`higgs::Higgs`] from the current request.
pub async fn higgs_ctx() -> Result<higgs::Higgs, ServerFnError> {
    higgs::Higgs::from_request().await
}

/// Map [`higgs::HiggsError`] into a server function error.
pub fn map_higgs_err(err: &higgs::HiggsError) -> ServerFnError {
    ServerFnError::ServerError(err.to_string())
}

/// Build a user-scoped Valence instance for the current request.
pub fn user_valence(ctx: &higgs::Higgs) -> Result<valence::Valence, ServerFnError> {
    ctx.valence().map_err(|e| map_higgs_err(&e))
}

/// Require an authenticated axum-login user alongside Higgs context.
pub async fn require_auth_user() -> Result<(higgs::Higgs, User), ServerFnError> {
    let ctx = higgs_ctx().await?;
    let user = extract_auth_user().await?;
    Ok((ctx, user))
}

/// Extract the authenticated user from axum-login, if present.
pub async fn extract_auth_user() -> Result<User, ServerFnError> {
    use leptos_axum::extract;

    let auth_session: axum_login::AuthSession<Backend> = extract().await?;
    auth_session
        .user
        .ok_or_else(|| ServerFnError::Args("You must be signed in".into()))
}
