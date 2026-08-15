use leptos::prelude::*;
use orbital_base_components::OpenBind;

use super::auth_modal_shell::AuthModalShell;
use super::{PasswordResetConfirmContent, PasswordResetRequestContent};

/// Which step [`PasswordResetDialog`] should render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordResetDialogKind {
    /// Show the "enter your email" request form.
    Request,
    /// Show the "enter your token and new password" confirm form.
    Confirm,
}

/// Shared password-reset modal for dedicated reset routes.
#[component]
pub fn PasswordResetDialog(
    open: OpenBind,
    kind: Signal<PasswordResetDialogKind>,
    #[prop(into)] token_from_query: Signal<String>,
) -> impl IntoView {
    let title = Signal::derive(move || match kind.get() {
        PasswordResetDialogKind::Request => "Reset password".to_string(),
        PasswordResetDialogKind::Confirm => "Set a new password".to_string(),
    });

    view! {
        <AuthModalShell open=open title=title>
            {move || match kind.get() {
                PasswordResetDialogKind::Request => view! {
                    <PasswordResetRequestContent />
                }.into_any(),
                PasswordResetDialogKind::Confirm => view! {
                    <PasswordResetConfirmContent token_from_query=token_from_query />
                }.into_any(),
            }}
        </AuthModalShell>
    }
}
