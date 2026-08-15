//! Axum routes for the mock OIDC sidecar.

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::redirect::redirect_uri_allowed;
use super::store::CodeStore;

#[derive(Clone)]
pub(super) struct AppState {
    pub store: Arc<CodeStore>,
    pub issuer: String,
}

/// Build the mock IdP router.
///
/// `issuer` is the public base URL (e.g. `http://127.0.0.1:5556`) used in discovery.
pub fn router(store: Arc<CodeStore>, issuer: impl Into<String>) -> Router {
    let state = AppState {
        store,
        issuer: issuer.into().trim_end_matches('/').to_string(),
    };
    Router::new()
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/authorize", get(authorize))
        .route("/token", post(token))
        .route("/userinfo", get(userinfo))
        .with_state(state)
}

#[derive(Serialize)]
struct Discovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    response_types_supported: [&'static str; 1],
    subject_types_supported: [&'static str; 1],
    id_token_signing_alg_values_supported: [&'static str; 1],
}

async fn discovery(State(state): State<AppState>) -> Json<Discovery> {
    let issuer = state.issuer.clone();
    Json(Discovery {
        authorization_endpoint: format!("{issuer}/authorize"),
        token_endpoint: format!("{issuer}/token"),
        userinfo_endpoint: format!("{issuer}/userinfo"),
        issuer,
        response_types_supported: ["code"],
        subject_types_supported: ["public"],
        id_token_signing_alg_values_supported: ["none"],
    })
}

#[derive(Debug, Deserialize)]
struct AuthorizeQuery {
    state: Option<String>,
    redirect_uri: Option<String>,
    provider: Option<String>,
    /// Optional fixed code for tests (`code_hint`); otherwise server-generated.
    code_hint: Option<String>,
}

async fn authorize(State(state): State<AppState>, Query(q): Query<AuthorizeQuery>) -> Response {
    let Some(state_param) = q.state.filter(|s| !s.trim().is_empty()) else {
        return (StatusCode::BAD_REQUEST, "missing state").into_response();
    };
    let Some(redirect_uri) = q.redirect_uri.filter(|s| !s.trim().is_empty()) else {
        return (StatusCode::BAD_REQUEST, "missing redirect_uri").into_response();
    };
    if !redirect_uri_allowed(&redirect_uri) {
        return (StatusCode::BAD_REQUEST, "disallowed redirect_uri").into_response();
    }
    let provider = q
        .provider
        .unwrap_or_else(|| "google".to_string())
        .trim()
        .to_string();
    if provider.is_empty() {
        return (StatusCode::BAD_REQUEST, "missing provider").into_response();
    }

    let code = match state.store.issue_code(&provider, q.code_hint.as_deref()) {
        Ok(c) => c,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid code").into_response(),
    };

    let sep = if redirect_uri.contains('?') { '&' } else { '?' };
    let location = format!(
        "{redirect_uri}{sep}code={}&state={}",
        urlencoding::encode(&code),
        urlencoding::encode(&state_param),
    );
    tracing::info!(operation = "authorize", outcome = "success", provider = %provider, "mock_oidc");
    Redirect::temporary(&location).into_response()
}

#[derive(Debug, Deserialize)]
struct TokenForm {
    grant_type: Option<String>,
    code: Option<String>,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
}

async fn token(State(state): State<AppState>, Form(form): Form<TokenForm>) -> Response {
    let grant = form.grant_type.as_deref().unwrap_or("");
    if grant != "authorization_code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "unsupported_grant_type"})),
        )
            .into_response();
    }
    let Some(code) = form.code.filter(|s| !s.trim().is_empty()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid_request"})),
        )
            .into_response();
    };
    match state.store.exchange_code(code.trim()) {
        Ok(access_token) => {
            tracing::info!(operation = "token", outcome = "success", "mock_oidc");
            Json(TokenResponse {
                access_token,
                token_type: "Bearer",
                expires_in: 3600,
            })
            .into_response()
        }
        Err(_) => {
            tracing::info!(operation = "token", outcome = "failure", "mock_oidc");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid_grant"})),
            )
                .into_response()
        }
    }
}

async fn userinfo(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let token = bearer_token(&headers);
    let Some(token) = token else {
        return (StatusCode::UNAUTHORIZED, "missing bearer").into_response();
    };
    let Some(id) = state.store.identity_for_token(&token) else {
        return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
    };
    let mut body = HashMap::new();
    body.insert("sub", id.subject);
    if let Some(email) = id.email {
        body.insert("email", email);
    }
    if let Some(name) = id.name {
        body.insert("name", name);
    }
    tracing::debug!(
        operation = "userinfo",
        outcome = "success",
        has_email = body.contains_key("email"),
        "mock_oidc"
    );
    Json(body).into_response()
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let rest = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    let t = rest.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
