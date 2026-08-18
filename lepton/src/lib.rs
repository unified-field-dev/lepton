//! SSR identity and token model bindings for Unified Field hosts.
//!
//! With the `ssr` feature, this crate exposes [`generated`] (`User`, token models, and related
//! Valence shapes) for server-side auth. Host session boot, delivery, and step-up live in
//! `lepton-host-adapter` and `lepton-auth`. For relating product Valence models to `User`, see
//! [`lepton_identity`](../lepton_identity/index.html#product-composition).
//!
//! ## Features
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `ssr` | Exposes [`generated`] from `lepton-host-adapter` |
//! | (default) | Empty surface until `ssr` is enabled |
//!
//! ## Example
//!
//! ```rust,ignore
//! // Cargo.toml: lepton = { features = ["ssr"] }
//! use lepton::generated::User;
//! ```
//!
//! Prefer `lepton-identity` when you need ownership helpers without the host adapter graph, or
//! `lepton-host-adapter` when you wire axum-login session and token schemas yourself.

#[cfg(feature = "ssr")]
/// Generated identity and token models from [`lepton_host_adapter`] (SSR).
pub mod generated {
    pub use lepton_host_adapter::generated::*;
}
