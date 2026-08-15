//! Thin mount of [`lepton_test_support`] seed HTTP for Playwright.
//!
//! Domain logic lives in `lepton-test-support`. Do not mount this route on
//! production product hosts.

use std::sync::Arc;

use lepton_test_support::{SeedError, SeedValence};
use valence::Valence;

use crate::boot::{system_valence, AppState};

pub use lepton_test_support::{seed_data, SeedRequest, SeedResponse};

impl SeedValence for AppState {
    fn seed_valence(&self) -> Result<Valence, SeedError> {
        system_valence(Arc::clone(&self.valence_router), &self.default_backend_key).map_err(|_| {
            SeedError::Persistence {
                operation: "system_valence",
            }
        })
    }
}
