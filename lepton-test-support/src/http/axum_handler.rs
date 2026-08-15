//! Axum handler for harness seed HTTP (`feature = "axum"`).

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use valence::Valence;

use crate::error::SeedError;
use crate::http::{SeedRequest, SeedResponse};
use crate::scenario::run_seed;

/// Host state that can mint a system [`Valence`] for seed writes.
pub trait SeedValence: Clone + Send + Sync + 'static {
    /// Build a system-actor Valence for test seed operations.
    ///
    /// # Errors
    ///
    /// Returns [`SeedError::Persistence`] when the host cannot build Valence.
    fn seed_valence(&self) -> Result<Valence, SeedError>;
}

/// Map [`SeedError`] to an HTTP status + opaque message (no secrets).
#[must_use]
pub fn seed_error_status(err: &SeedError) -> (StatusCode, String) {
    let status = match err {
        SeedError::UnknownScenario { .. } | SeedError::InvalidInput { .. } => {
            StatusCode::BAD_REQUEST
        }
        SeedError::Persistence { .. }
        | SeedError::Crypto { .. }
        | SeedError::Trust(_)
        | SeedError::Contact(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, err.to_string())
}

/// `POST /api/test/seed-data` — harness / e2e hosts only.
///
/// # Errors
///
/// Returns `(StatusCode, String)` for unknown scenarios, invalid input, or
/// persistence failures. Response body may include passwords / tokens on success.
pub async fn seed_data<S>(
    State(state): State<S>,
    Json(body): Json<SeedRequest>,
) -> Result<Json<SeedResponse>, (StatusCode, String)>
where
    S: SeedValence,
{
    let valence = state.seed_valence().map_err(|e| seed_error_status(&e))?;
    let response = run_seed(&valence, body)
        .await
        .map_err(|e| seed_error_status(&e))?;
    Ok(Json(response))
}
