//! Logical database name/storage constants for Lepton identity schemas.
//!
//! Engine id is selected by Cargo feature: `db-sqlite` (default) or `db-hybrid`.

use valence::{Database, DatabaseFromEngine};

/// Identity logical database name on the host Valence router.
pub const IDENTITY_LOGICAL_NAME: &str = "default";

#[cfg(feature = "db-hybrid")]
const ENGINE_ID: &str = valence::HYBRID_ENGINE_ID;

#[cfg(not(feature = "db-hybrid"))]
const ENGINE_ID: &str = valence::SQLITE_ENGINE_ID;

/// Default Valence storage evaluator for identity schemas.
pub const IDENTITY_DEFAULT_STORAGE: DatabaseFromEngine =
    Database::from_engine(IDENTITY_LOGICAL_NAME, ENGINE_ID);

/// Logical names registered with the Valence router for identity persistence.
pub const EMBEDDED_SURREAL_LOGICAL_NAMES: &[&str] = &[IDENTITY_LOGICAL_NAME];

/// Alias for [`EMBEDDED_SURREAL_LOGICAL_NAMES`] (engine-neutral name).
pub const EMBEDDED_LOGICAL_NAMES: &[&str] = EMBEDDED_SURREAL_LOGICAL_NAMES;
