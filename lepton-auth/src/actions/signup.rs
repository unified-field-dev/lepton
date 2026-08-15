//! Server function for creating a new account.

use leptos::prelude::*;

/// Create a new account and log the user in.
///
/// Does not redirect: the signup wizard advances to verification steps in the
/// auth modal. Hosts may refuse signup with `UF_LEPTON_SIGNUP_DISABLED=1` (see
/// [`crate::signup_policy`] and the kit `SECURITY.md`).
#[server(Signup)]
pub async fn signup(
    /// Legal name (private profile field).
    legal_name: String,
    /// Display name (public profile field).
    display_name: String,
    /// New account email.
    email: String,
    /// New account password (must satisfy policy).
    password: String,
    /// Confirmation of `password`.
    confirm: String,
    /// Referer retained for form compatibility (client uses for post-wizard return).
    referer: Option<String>,
) -> Result<(), ServerFnError> {
    use leptos_axum::extract;

    let _ = referer;

    if !crate::signup_policy::signup_enabled() {
        return Err(ServerFnError::new("Signup is disabled on this host"));
    }

    let ctx = higgs::Higgs::from_request().await?;
    let mut auth_session: axum_login::AuthSession<lepton_host_adapter::Backend> = extract().await?;
    // Account create requires SYSTEM_ONLY policies on identity rows.
    let valence = ctx
        .unsafe_system_valence()
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let signup_result = crate::signup_api::ssr::execute(
        &valence,
        &mut auth_session,
        crate::signup_api::ssr::SignupRequest {
            legal_name,
            display_name,
            email,
            password,
            confirm,
        },
    )
    .await?;

    leptos::logging::log!("[auth] signup success for {}", signup_result.email);
    Ok(())
}
