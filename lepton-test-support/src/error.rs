//! Typed errors for test seed / builder APIs.

use thiserror::Error;

/// Errors from [`crate::builder::TestUserBuilder`] and [`crate::scenario::run_seed`].
#[derive(Debug, Error)]
pub enum SeedError {
    /// Scenario id is not in the catalog.
    #[error("reason_class=unknown_scenario: unknown scenario `{scenario}`")]
    UnknownScenario {
        /// Requested scenario id (not user PII).
        scenario: String,
    },
    /// Builder or request failed validation before writes.
    #[error("reason_class=invalid_input: {reason}")]
    InvalidInput {
        /// Stable reason token (no secrets).
        reason: &'static str,
    },
    /// Valence create / upsert / commit failed.
    #[error("reason_class=persistence: {operation} failed")]
    Persistence {
        /// Operation name (e.g. `user_create`).
        operation: &'static str,
    },
    /// Password or token hashing failed.
    #[error("reason_class=crypto: {operation} failed")]
    Crypto {
        /// Operation name (e.g. `hash_password`).
        operation: &'static str,
    },
    /// [`lepton_auth::trust::confirm_user`] failed.
    #[error("reason_class=trust: confirm failed")]
    Trust(#[from] lepton_auth::trust::TrustError),
    /// Phone / contact helpers failed.
    #[error("reason_class=contact: contact seed failed")]
    Contact(#[from] lepton_auth::contacts::ContactError),
}

impl SeedError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::UnknownScenario { .. } => "unknown_scenario",
            Self::InvalidInput { .. } => "invalid_input",
            Self::Persistence { .. } => "persistence",
            Self::Crypto { .. } => "crypto",
            Self::Trust(_) => "trust",
            Self::Contact(_) => "contact",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_scenario_reason_class() {
        let err = SeedError::UnknownScenario {
            scenario: "nope".into(),
        };
        assert_eq!(err.reason_class(), "unknown_scenario");
        assert!(err.to_string().contains("unknown_scenario"));
    }

    #[test]
    fn invalid_input_reason_class() {
        let err = SeedError::InvalidInput {
            reason: "empty_email",
        };
        assert_eq!(err.reason_class(), "invalid_input");
    }
}
