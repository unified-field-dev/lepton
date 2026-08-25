//! SSR identity and token model bindings for Unified Field hosts.
//!
//! Enable the `ssr` feature to import [`generated`] (`User`, token models, and related
//! Valence shapes) from the host adapter. Session boot, delivery, and step-up live in
//! `lepton-host-adapter` and `lepton-auth`. Ownership helpers without the adapter graph
//! live in [`lepton_identity`](../lepton_identity/index.html).
//!
//! # Features
//!
//! - **SSR model bindings** — Re-exports host-adapter identity and token types from
//!   [`generated`] so hosts import `User` and related shapes from one facade when the
//!   `ssr` feature is on ([Getting started](#getting-started)).
//!
//! # Getting started
//!
//! Import generated identity models when the host enables `ssr` and wants `User` (and
//! related token shapes) from the lepton facade instead of reaching into the adapter.
//!
//! Prerequisites: `lepton = { features = ["ssr"] }` on the host.
//!
//! 1. Import [`generated::User`] (or other generated models).
//! 2. Use the type with Valence / auth code that expects the host identity shape.
//! 3. Confirm the type is in scope (compile) and carry a real user id when you have a row.
//!
//! Errors: without `ssr`, `generated` is unavailable. Next: password hashing on
//! [`lepton_identity`](../lepton_identity/index.html#hash-a-password), or session
//! wiring on [`lepton_host_adapter`](../lepton_host_adapter/index.html#host-wiring).
//!
//! ```rust,ignore
//! // Cargo.toml: lepton = { features = ["ssr"] }
//! use lepton::generated::{User, UserStatus};
//!
//! fn is_active(user: &User) -> bool {
//!     matches!(user.status(), Some(UserStatus::Active))
//! }
//! // After load when the row should be active:
//! assert!(is_active(&user));
//! ```
//!
//! Prefer [`lepton_identity`](../lepton_identity/index.html#getting-started) for password
//! hashing and ownership, or
//! [`lepton_host_adapter`](../lepton_host_adapter/index.html#getting-started) when wiring
//! axum-login yourself.
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `ssr` | Exposes [`generated`] from `lepton-host-adapter` |
//! | *(default)* | Empty surface until `ssr` is enabled |
//!
//! # Further reading
//!
//! - [`generated`] — identity and token models
//! - [`lepton_identity`](../lepton_identity/index.html) — hash + ownership without adapter
//! - [`lepton_host_adapter`](../lepton_host_adapter/index.html) — session backend

#[cfg(feature = "ssr")]
/// Generated identity and token models from [`lepton_host_adapter`] (SSR).
pub mod generated {
    pub use lepton_host_adapter::generated::*;
}
