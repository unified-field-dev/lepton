//! Photon WebSocket auth extractor bridging axum-login [`Backend`].
//!
//! # Examples
//!
//! Pass [`PhotonAuth`] to `photon_axum::apply_ws_routes` so synced Leptos handlers
//! scope subscriptions to the signed-in user:
//!
//! ```rust,ignore
//! use axum::Router;
//! use lepton_host_adapter::{photon_auth::PhotonAuth, Backend, session_snapshot_middleware};
//! use photon_axum::apply_ws_routes;
//!
//! let app: Router = Router::new()
//!     .merge(apply_ws_routes::<PhotonAuth>(/* ws router from photon_leptos */))
//!     .layer(session_snapshot_middleware)
//!     .layer(auth_layer);
//! ```
//!
//! Read the subscription key directly when wiring custom Photon handlers:
//!
//! ```rust,ignore
//! use lepton_host_adapter::photon_auth::extract_user_key;
//!
//! if let Some(user_id) = extract_user_key(&auth_session) {
//!     // scope topic to user_id
//! }
//! ```

use axum::extract::FromRequestParts;
use axum_login::AuthUser;
use http::request::Parts;

use crate::auth::Backend;

/// Extract the Photon subscription key from an auth session.
///
/// Used by `#[photon_leptos::synced(auth = "user")]` generated handlers to scope
/// WebSocket subscriptions to the authenticated user.
pub fn extract_user_key(auth: &axum_login::AuthSession<Backend>) -> Option<String> {
    auth.user.as_ref().map(AuthUser::id)
}

/// Newtype wrapping `AuthSession<Backend>` that implements
/// [`photon_axum::PhotonUserExtractor`] for use with
/// `photon_axum::apply_ws_routes` / `photon_leptos::server::ws_router`.
///
/// Lives here (with [`Backend`]) to satisfy the orphan rule.
pub struct PhotonAuth(pub axum_login::AuthSession<Backend>);

impl photon_axum::PhotonUserExtractor for PhotonAuth {
    fn user_key(&self) -> Option<String> {
        extract_user_key(&self.0)
    }
}

impl<S> FromRequestParts<S> for PhotonAuth
where
    S: Send + Sync,
    axum_login::AuthSession<Backend>: FromRequestParts<S>,
{
    type Rejection = <axum_login::AuthSession<Backend> as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        axum_login::AuthSession::<Backend>::from_request_parts(parts, state)
            .await
            .map(PhotonAuth)
    }
}
