//! Route path constants shared between server routes and client-side links.
//!
//! Mount matching Leptos `<Route>` views in the host (this crate does not register
//! routes for you). Pair with [`lepton_auth_ui`](../lepton_auth_ui/index.html)
//! [`AuthDialog`](../lepton_auth_ui/fn.AuthDialog.html) or content components.
//!

/// Signup page route.
pub const SIGNUP: &str = "/auth/signup";
/// Sign-in page route.
pub const SIGNIN: &str = "/auth/signin";
/// Logout action route.
pub const LOGOUT: &str = "/auth/logout";
/// Account settings page route.
pub const USER_ACCOUNT_SETTINGS: &str = "/user/account-settings";
/// Guided account confirm funnel (email → phone → confirm).
pub const USER_CONFIRM_ACCOUNT: &str = "/user/confirm-account";
/// User profile page route.
pub const USER_PROFILE: &str = "/user/profile";
/// Password reset request page route.
pub const RESET_PASSWORD_REQUEST: &str = "/auth/reset/request";
/// Password reset confirmation page route.
pub const RESET_PASSWORD_CONFIRM: &str = "/auth/reset/confirm";
/// OAuth provider callback route (authorization code + state).
pub const OAUTH_CALLBACK: &str = "/auth/oauth/callback";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_paths_are_rooted_under_auth() {
        for path in [
            SIGNUP,
            SIGNIN,
            LOGOUT,
            RESET_PASSWORD_REQUEST,
            RESET_PASSWORD_CONFIRM,
            OAUTH_CALLBACK,
        ] {
            assert!(path.starts_with("/auth/"), "{path}");
        }
    }

    #[test]
    fn user_paths_are_rooted_under_user() {
        assert!(USER_ACCOUNT_SETTINGS.starts_with("/user/"));
        assert!(USER_CONFIRM_ACCOUNT.starts_with("/user/"));
        assert!(USER_PROFILE.starts_with("/user/"));
    }
}
