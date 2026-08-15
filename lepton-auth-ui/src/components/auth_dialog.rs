//! [`AuthDialog`] — the shared signin/signup/logout modal.

use leptos::prelude::*;
use orbital_base_components::OpenBind;

use super::auth_modal_shell::AuthModalShell;
use super::{LogoutContent, SigninContent, SignupContent};

/// Which content [`AuthDialog`] should render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthDialogKind {
    /// Show the sign-in form.
    Signin,
    /// Show the sign-up form.
    Signup,
    /// Show the log-out confirmation.
    Logout,
}

/// Optional callbacks for [`AuthDialog`] content events.
#[derive(Clone, Copy, Default)]
pub struct AuthDialogCallbacks {
    /// Invoked after a successful auth action.
    pub on_success: Option<Callback<()>>,
    /// Invoked when the dialog should close.
    pub on_close: Option<Callback<()>>,
    /// Invoked when the user asks to switch to the sign-in form.
    pub on_switch_signin: Option<Callback<()>>,
    /// Invoked when the user asks to switch to the sign-up form.
    pub on_switch_signup: Option<Callback<()>>,
}

/// Shared sign-in / sign-up / log-out modal for toolbar and dedicated auth routes.
///
/// Requires `open` ([`OpenBind`]), `kind`, and `referer` (post-auth path). Optional
/// [`AuthDialogCallbacks`] handle close and sign-in ↔ sign-up switches. Submits call
/// [`lepton_auth::actions`]. See the [crate root](crate) for a full mount example.
#[component]
pub fn AuthDialog(
    open: OpenBind,
    kind: Signal<AuthDialogKind>,
    referer: Signal<String>,
    #[prop(default = AuthDialogCallbacks::default())] callbacks: AuthDialogCallbacks,
) -> impl IntoView {
    let title = Signal::derive(move || match kind.get() {
        AuthDialogKind::Signin => "Sign in".to_string(),
        AuthDialogKind::Signup => "Sign up".to_string(),
        AuthDialogKind::Logout => "Log out".to_string(),
    });

    view! {
        <AuthModalShell open=open title=title>
            <div data-testid="auth-dialog-root">
                {move || match kind.get() {
                AuthDialogKind::Signin => view! {
                    <SigninContent
                        referer=referer
                        on_success=callbacks.on_success
                        on_switch_signup=callbacks.on_switch_signup
                    />
                }.into_any(),
                AuthDialogKind::Signup => view! {
                    <SignupContent
                        referer=referer
                        on_success=callbacks.on_success
                        on_switch_signin=callbacks.on_switch_signin
                    />
                }.into_any(),
                AuthDialogKind::Logout => view! {
                    <LogoutContent
                        referer=referer
                        on_success=callbacks.on_success
                        on_close=callbacks.on_close
                        on_switch_signin=callbacks.on_switch_signin
                    />
                }.into_any(),
            }}
            </div>
        </AuthModalShell>
    }
}
