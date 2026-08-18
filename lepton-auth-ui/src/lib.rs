//! Leptos auth UI: dialogs and embeddable forms composed with Orbital.
//!
//! Server functions and library APIs live in [`lepton_auth`]. This crate supplies
//! Leptos components and Orbital layout so hosts that need OAuth/factors alone do
//! not pull UI dependencies.
//!
//! Auth UI `#[server]` functions in [`lepton_auth::actions`] register through
//! Leptos `generate_route_list` / `leptos_routes_with_context`. You do not mount
//! them as separate axum handlers. Session chrome (`get_session` /
//! `init_auth_resource`) wires in from the host app shell.
//!
//! # Organized by task
//!
//! | Task | Start here | See also |
//! |------|------------|----------|
//! | Sign-in / sign-up / log-out modal | [`AuthDialog`] | [Mount `AuthDialog`](#mount-authdialog-shell); [`AuthDialogKind`], [`lepton_auth::paths`] |
//! | Dedicated `/auth/*` pages | [`lepton_auth::paths`] | [Routes `/auth/*`](#routes-auth) |
//! | Password reset | [`PasswordResetDialog`] | [`PasswordResetRequestContent`], [`PasswordResetConfirmContent`] |
//! | OAuth callback page | [`OAuthCallbackContent`] | [`lepton_auth::paths::OAUTH_CALLBACK`], [`lepton_auth::actions::oauth`] |
//! | Step-up before a sensitive op | [`provide_step_up_controller`], [`StepUpDialog`] | [Step-up](#step-up-critical-action); server verify in [`lepton_auth::factor`] |
//! | Login MFA (sign-in 2FA) | [`SigninContent`], [`lepton_auth::session_mfa`] | Per-op re-auth: [Step-up](#step-up-critical-action) |
//! | Confirm account prompt | [`ConfirmAccountPrompt`], [`ConfirmAccountPage`] | [`lepton_auth::actions::confirm_account`]; Continue carries `?referer=` |
//! | Paged sign-up wizard | [`SignupContent`] | email → details → email/phone/TOTP (skippable); soft confirm banner until email+phone verified |
//!
//! Embed forms without the modal: [`SigninContent`], [`SignupContent`], [`LogoutContent`].
//! Custom chrome / brand marks ([`AuthModalShell`], [`GoogleMark`], [`GitHubMark`],
//! [`StepUpContent`], [`OAuthProviderButtons`]) are for hosts that compose their own shell.
//!
//! # Features
//!
//! | Feature | Role |
//! |---------|------|
//! | `ssr` | Server-side rendering of auth UI |
//! | `hydrate` | Client hydration |
//! | `oauth-google` | Continue with Google (and the “or” divider) in [`OAuthProviderButtons`] |
//! | `oauth-github` | Continue with GitHub (and the “or” divider) in [`OAuthProviderButtons`] |
//!
//! Account creation UI ([`SignupContent`]) is always included. OAuth buttons
//! render only when the matching `oauth-*` features are enabled on this crate
//! (forward [`lepton_auth`] `oauth-google` / `oauth-github`). Hosts must enable
//! the same features on both SSR and hydrate graphs.
//!
//! # Mount `AuthDialog` (shell)
//!
//! Provide `open`, `kind`, and `referer`. Form submit calls [`lepton_auth::actions`].
//!
//! ```rust,ignore
//! use lepton_auth_ui::{
//!     AuthDialog, AuthDialogCallbacks, AuthDialogKind, provide_step_up_controller, StepUpDialog,
//! };
//! use leptos::prelude::*;
//!
//! let open = RwSignal::new(false);
//! let kind = RwSignal::new(AuthDialogKind::Signin);
//! let referer = Signal::derive(|| "/welcome".to_string());
//! let _step_up = provide_step_up_controller();
//!
//! view! {
//!     <button type="button" on:click=move |_| open.set(true)>"Sign in"</button>
//!     <AuthDialog
//!         open=open.into()
//!         kind=kind.into()
//!         referer=referer
//!         callbacks=AuthDialogCallbacks {
//!             on_close: Some(Callback::new(move |()| open.set(false))),
//!             on_success: Some(Callback::new(move |()| open.set(false))),
//!             on_switch_signin: Some(Callback::new(move |()| kind.set(AuthDialogKind::Signin))),
//!             on_switch_signup: Some(Callback::new(move |()| kind.set(AuthDialogKind::Signup))),
//!             ..Default::default()
//!         }
//!     />
//!     <StepUpDialog/>
//! }
//! ```
//!
//! Enable OAuth UI on the host (SSR + hydrate), then set boot env for the
//! provider exchange:
//!
//! ```rust,ignore
//! // app/Cargo.toml hydrate + ssr:
//! //   "lepton-auth-app/oauth-google",
//! //   "lepton-auth-app/oauth-github",
//! // Boot: UF_PUBLIC_BASE_URL + UF_OAUTH_* or UF_OAUTH_USE_MOCK=1
//! use lepton_auth_ui::OAuthProviderButtons;
//! use leptos::prelude::*;
//!
//! view! { <OAuthProviderButtons referer=Signal::derive(|| "/welcome".into()) /> }
//! // Neither oauth-* feature: empty. Both features: divider + both buttons.
//! ```
//!
//! # Routes (`/auth/*`)
//!
//! The host owns routes. Use [`lepton_auth::paths`] constants and mount pages that
//! open [`AuthDialog`] (or embed content components). Typical paths:
//! [`SIGNIN`](lepton_auth::paths::SIGNIN), [`SIGNUP`](lepton_auth::paths::SIGNUP),
//! [`LOGOUT`](lepton_auth::paths::LOGOUT),
//! [`RESET_PASSWORD_REQUEST`](lepton_auth::paths::RESET_PASSWORD_REQUEST),
//! [`RESET_PASSWORD_CONFIRM`](lepton_auth::paths::RESET_PASSWORD_CONFIRM),
//! [`OAUTH_CALLBACK`](lepton_auth::paths::OAUTH_CALLBACK).
//!
//! ```rust,ignore
//! use leptos_router::components::{Route, Routes};
//! use leptos_router::path;
//!
//! // Inside your <Router>:
//! view! {
//!     <Routes fallback=|| view! { <p>"Not found"</p> }>
//!         <Route path=path!("/auth/signin") view=SigninPage/>
//!         <Route path=path!("/auth/signup") view=SignupPage/>
//!         <Route path=path!("/auth/oauth/callback") view=OAuthCallbackPage/>
//!     </Routes>
//! }
//! ```
//!
//! # Step-up (critical action)
//!
//! Mount [`StepUpDialog`] once after [`provide_step_up_controller`]. On a sensitive
//! action, call [`StepUpController::request`], then verify with
//! [`lepton_auth::factor::FactorChallengeService`] (or password re-check) and call
//! [`StepUpController::complete_success`] / [`StepUpController::report_error`].
//!
//! The product shell (`lepton-shell` in lepton-uf-app) mounts the dialog for **future**
//! host apps (for example Gluon control-plane ops). Account Settings does not drive
//! step-up today; wipe uses its own password (+ TOTP) ladder on the wipe server fn.
//!
//! ```rust,ignore
//! use lepton_auth_ui::{
//!     provide_step_up_controller, StepUpDialog, StepUpPolicy, StepUpRequest,
//! };
//! use leptos::prelude::*;
//!
//! let step_up = provide_step_up_controller();
//! view! { <StepUpDialog/> };
//!
//! step_up.request(
//!     StepUpRequest {
//!         title: "Confirm it's you".into(),
//!         description: Some("Enter your authenticator code.".into()),
//!         policy: StepUpPolicy::Totp,
//!     },
//!     Callback::new(move |factors| {
//!         let step_up = step_up;
//!         leptos::task::spawn_local(async move {
//!             match grant_permission(target, factors.totp_code).await {
//!                 Ok(()) => step_up.complete_success(),
//!                 Err(e) => step_up.report_error(e.to_string()),
//!             }
//!         });
//!     }),
//! );
//! ```
//!
//! For password + TOTP, use [`StepUpPolicy::PasswordAndTotp`] and pass both factors
//! into your server fn. Server-side verify: [`lepton_auth::factor`].

#![recursion_limit = "256"]

/// Leptos UI components for the auth flows (dialogs and embeddable form content).
pub mod components;

pub use components::{
    confirm_account_status_resource, provide_step_up_controller, use_step_up_controller,
    AuthDialog, AuthDialogCallbacks, AuthDialogKind, AuthModalShell, ConfirmAccountPage,
    ConfirmAccountPrompt, ConfirmAccountPromptVariant, GitHubMark, GoogleMark, LogoutContent,
    OAuthCallbackContent, OAuthProviderButtons, PasswordResetConfirmContent, PasswordResetDialog,
    PasswordResetDialogKind, PasswordResetRequestContent, SigninContent, SignupContent,
    StepUpContent, StepUpController, StepUpDialog, StepUpFactors, StepUpPolicy, StepUpRequest,
};
