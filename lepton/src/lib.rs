//! Thin re-export of generated SSR identity / token types from
//! [`lepton_host_adapter`]. Prefer `lepton-host-adapter` and `lepton-auth` for
//! session boot, delivery, and step-up. For relating product Valence models to
//! `User`, see [`lepton_identity`](../lepton_identity/index.html#product-composition).
//!
//! ## Features
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `ssr` | Re-exports [`generated`] from `lepton-host-adapter` |
//! | `hydrate` / default | Empty surface (no SSR bindings) |
//!
//! ## Example
//!
//! ```rust,ignore
//! // Cargo.toml: lepton = { features = ["ssr"] }
//! use lepton::generated::User;
//! ```

#[cfg(feature = "ssr")]
/// Generated identity and token models re-exported from [`lepton_host_adapter`].
pub mod generated {
    pub use lepton_host_adapter::generated::*;
}
