//! Axum routes for the SMS HTTP capture sink.

use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;

use super::store::{CapturedSms, MessageStore, MAX_BODY_BYTES};

#[derive(Clone)]
pub(super) struct AppState {
    pub store: Arc<MessageStore>,
}

#[derive(Debug, Deserialize)]
struct PostBody {
    to_e164: Option<String>,
    body: Option<String>,
    otp_code: Option<String>,
}

/// Build the sink router (testable without binding a fixed port).
pub fn router(store: Arc<MessageStore>) -> Router {
    let state = AppState { store };
    Router::new()
        .route("/v1/messages", post(post_message))
        .route("/v1/messages", get(list_messages))
        .route("/v1/messages", delete(clear_messages))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}

async fn post_message(
    State(state): State<AppState>,
    Json(payload): Json<PostBody>,
) -> impl IntoResponse {
    let Some(to_e164) = payload
        .to_e164
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "missing_to_e164"})),
        )
            .into_response();
    };
    let Some(body) = payload
        .body
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "missing_body"})),
        )
            .into_response();
    };

    let msg = CapturedSms {
        to_e164,
        body,
        otp_code: payload
            .otp_code
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    };

    match state.store.push(msg) {
        Ok(()) => {
            let count = state.store.len();
            tracing::info!(
                operation = "ingest",
                outcome = "success",
                message_count = count,
                "sms_sink"
            );
            (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response()
        }
        Err("store_full") => {
            tracing::warn!(
                operation = "ingest",
                outcome = "failure",
                reason_class = "store_full",
                "sms_sink"
            );
            (
                StatusCode::INSUFFICIENT_STORAGE,
                Json(serde_json::json!({"error": "store_full"})),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "internal"})),
        )
            .into_response(),
    }
}

async fn list_messages(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.store.list())
}

async fn clear_messages(State(state): State<AppState>) -> impl IntoResponse {
    state.store.clear();
    StatusCode::NO_CONTENT
}
