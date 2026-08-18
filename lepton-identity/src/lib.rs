//! Leptos-free identity models and password hashing for Unified Field apps.
//!
//! Valence schemas/codegen for core identity tables (`user`, `account`,
//! `user_profile`, memberships, contacts, devices) so worker binaries and other
//! headless crates can depend on them without pulling Leptos.
//!
//! # Host recipes
//!
//! | Recipe | Start here |
//! |--------|------------|
//! | Identity Valence models | [`generated`] |
//! | Relate a product row to `User` | [Product composition](#product-composition) |
//! | Hash passwords (Argon2) | [`auth::hash_password`] |
//! | Signup row ownership | [`ownership::ensure_signup_identity_ownership`] |
//! | Session / axum-login | [`lepton_host_adapter`](../lepton_host_adapter/index.html) |
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Valence models / codegen | [`generated`] |
//! | Router logical DB names | [`embedded_surreal`] |
//! | Password hashing | [`auth::hash_password`] |
//! | Signup ownership side effects | [`ownership`], [`side_effects`] |
//!
//! ## Storage features
//!
//! | Feature | Engine id on [`embedded_surreal::IDENTITY_DEFAULT_STORAGE`] |
//! |---------|--------------------------------------------------------------|
//! | `db-sqlite` (default) | `valence::SQLITE_ENGINE_ID` — local / embedded SQLite |
//! | `db-hybrid` | `valence::HYBRID_ENGINE_ID` — host router with mixed backends |
//!
//! Pick one per binary. Worker crates that only need models can disable default
//! features and omit both when they do not open Valence storage.
//!
//! ## Model overview
//!
//! [`generated`] holds the Valence models (from `schemas/` via `valence_codegen`):
//!
//! - `Account` — legal identity with a required founding `user` FK (set at signup/OAuth),
//!   optional primaries, and Cascade `emails` / `phones`. Product shape today is one
//!   founding User + Owner membership.
//! - `AccountEmail` / `AccountPhone` — contacts owned by `Account` (unique `address` /
//!   `e164`, per-row `verified_at`).
//! - `User.primary_email` / `primary_phone` — login FKs only (no email/phone collections
//!   on `User`).
//! - `AccountMembership`, `LinkedIdentity`, `AuthDevice` / `AuthDeviceCeremony` — membership,
//!   OAuth, and trusted devices.
//!
//! Password hashing: [`auth`]. Session types live in `lepton-host-adapter`. Signup ownership
//! and membership side effects: [`ownership`], [`side_effects`]. Router constants:
//! [`embedded_surreal`].
//!
//! ## Product composition
//!
//! Point a product Valence model at Lepton's `user` table with a `Record("user")` field
//! and a `HasOne` connection whose `model` is `lepton_identity::generated::User` (same
//! pattern host-adapter token schemas use). After codegen, hop with the instance method
//! `get_user(&valence)` (and `user_thing()` for the raw [`RecordId`](valence::RecordId)).
//!
//! ```text
//! // In your product schema (valence_schema!):
//! fields: [
//!     user: {
//!         r#type: FieldType::Record("user"),
//!         required: true,
//!     },
//! ],
//! connections: [
//!     user: {
//!         table: "user",
//!         cardinality: HasOne,
//!         required: true,
//!         on_delete: Cascade, // or Restrict — product choice
//!         model: "lepton_identity::generated::User",
//!     },
//! ]
//!
//! // After codegen, on a loaded row:
//! // let user = order.get_user(&valence).await?;
//! ```
//!
//! When the product row is owned by that user, apply `OWNER_BY_USER_FIELD` on the field
//! or row policies as appropriate. That is a privacy choice, separate from the hop API.
//!
//! ## Examples
//!
//! Hash a password for persistence (SSR boot or worker signup):
//!
//! ```rust
//! use lepton_identity::auth::hash_password;
//!
//! let phc = hash_password("ValidPass123!").expect("hash");
//! assert!(phc.starts_with("$argon2"));
//! ```
//!
//! Assign founding-user ownership after anonymous signup (library callers pass bare ids):
//!
//! ```rust,no_run
//! use lepton_identity::ownership::ensure_signup_identity_ownership;
//! use valence::Valence;
//!
//! async fn after_signup(valence: &Valence, user_bare: &str, account_bare: &str) -> valence::Result<()> {
//!     ensure_signup_identity_ownership(valence, user_bare, account_bare, &[]).await
//! }
//! ```

/// Argon2 password hashing shared by SSR and worker crates.
pub mod auth;
/// Logical database name/storage constants for identity schemas.
pub mod embedded_surreal;
/// Build-generated Valence models for identity tables (see the [crate-level
/// overview](self)).
pub mod generated;
/// Signup-time row ownership assignment for identity models.
pub mod ownership;
/// Valence mutation side effects for identity models.
pub mod side_effects;

mod schemas;
