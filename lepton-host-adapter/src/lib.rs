//! Token lifecycle schemas + axum-login backend bridging `lepton-identity` to higgs.
//!
//! All modules require the `ssr` feature: this crate has no client-side surface.
//!
//! # Host recipes
//!
//! | Recipe | Start here |
//! |--------|------------|
//! | Axum-login backend + session user | [`Backend`], [`Credentials`], [`User`] |
//! | Mirror session into higgs | [`session_snapshot_middleware`] |
//! | Photon WS auth bridge | [`photon_auth`], [`PhotonAuth`], [`extract_user_key`] |
//! | Profile photo upload / serve | [`files`] |
//! | Product row → `User` edge | [`lepton_identity`](../lepton_identity/index.html#product-composition) |
//!
//! Typical host wiring: register [`Backend`] with `axum-login`, layer
//! [`session_snapshot_middleware`], and read [`AuthSession`] /
//! `higgs_identity::SessionSnapshot` from request extensions. Runnable smoke:
//! `cargo run -p lepton-host-adapter --example axum_session_snapshot --features ssr`.
//!
//! Token / factor Valence models live in [`generated`] (schema inventory and sealed
//! fields on each type); see [`SECURITY.md`](https://github.com/unified-field-dev/lepton/blob/main/SECURITY.md).
//! Router logical-name constants: [`embedded_surreal`].
//!
//! # Host wiring
//!
//! ```rust,ignore
//! use std::sync::Arc;
//!
//! use axum::{middleware, Router};
//! use axum_login::AuthManagerLayerBuilder;
//! use higgs::HiggsConfig;
//! use lepton_host_adapter::{session_snapshot_middleware, Backend};
//! use tower_sessions::{MemoryStore, SessionManagerLayer};
//!
//! // Share the same valence factory Arc with HiggsConfig.
//! let higgs: Arc<HiggsConfig> = /* host boot */;
//! let backend = Backend::new(Arc::clone(higgs.valence_factory()));
//! // Hosts must set Secure / HttpOnly / SameSite on the session cookie (see SECURITY.md).
//! let session_layer = SessionManagerLayer::new(MemoryStore::default());
//! let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
//!
//! let app: Router = Router::new()
//!     // .route(...) — mount Leptos routes / auth UI here
//!     .layer(middleware::from_fn(session_snapshot_middleware))
//!     .layer(auth_layer);
//! // `Extension<higgs_identity::SessionSnapshot>` is available to higgs for
//! // authenticated requests; anonymous requests simply lack the extension.
//! ```

/// Token lifecycle Valence models, plus re-exports of `lepton-identity`'s core models
/// (see the [crate-level walkthrough](self)).
#[cfg(feature = "ssr")]
pub mod generated;

/// Logical embedded Surreal database name/storage constants for token schemas.
#[cfg(feature = "ssr")]
pub mod embedded_surreal;

#[cfg(feature = "ssr")]
mod schemas;

/// axum-login backend + session user/credential types bridging to `lepton-identity`.
#[cfg(feature = "ssr")]
pub mod auth;

/// Middleware mirroring the axum-login session into `higgs_identity::SessionSnapshot`.
#[cfg(feature = "ssr")]
pub mod session;

/// Photon WebSocket auth extractor built on [`auth::Backend`].
#[cfg(feature = "ssr")]
pub mod photon_auth;

/// Profile photo `/api/files/*` handlers and [`files::FileByteBackend`].
#[cfg(feature = "ssr")]
pub mod files;

#[cfg(feature = "ssr")]
pub use auth::{Backend, Credentials, User};

#[cfg(feature = "ssr")]
pub use axum_login::AuthSession;

#[cfg(feature = "ssr")]
pub use photon_auth::{extract_user_key, PhotonAuth};

#[cfg(feature = "ssr")]
pub use session::session_snapshot_middleware;
