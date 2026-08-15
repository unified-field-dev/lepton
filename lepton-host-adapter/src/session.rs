//! Middleware helpers: mirror axum-login session into `higgs_identity::SessionSnapshot`.

use axum::{extract::Request, middleware::Next, response::Response};
use axum_login::{AuthSession, AuthUser};
use higgs_identity::SessionSnapshot;

use crate::auth::Backend;

/// Axum middleware that inserts `Extension<SessionSnapshot>` when the user is authenticated.
///
/// Run after axum-login session middleware so `AuthSession<Backend>` is available.
pub async fn session_snapshot_middleware(
    auth_session: AuthSession<Backend>,
    mut req: Request,
    next: Next,
) -> Response {
    if let Some(user) = auth_session.user.as_ref() {
        req.extensions_mut().insert(SessionSnapshot::new(
            AuthUser::id(user),
            AuthUser::session_auth_hash(user),
        ));
    }
    next.run(req).await
}
