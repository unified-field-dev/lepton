//! Browser `navigator.credentials` helpers for WebAuthn JSON ceremonies (`hydrate`).
//!
//! Converts server [`serde_json::Value`] creation / request options into
//! `credentials.create` / `get`, then returns credential `toJSON()` strings for
//! finish server fns. Uses the Level 3 JSON helpers when present
//! (`parseCreationOptionsFromJSON` / `parseRequestOptionsFromJSON` / `toJSON`).

use js_sys::{Function, Object, Reflect, JSON};
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::CredentialsContainer;

/// Errors from browser WebAuthn ceremony helpers.
#[derive(Clone, Debug, thiserror::Error)]
pub enum WebauthnBrowserError {
    /// Missing window / credentials API.
    #[error("reason_class=webauthn_browser: WebAuthn is not available in this browser")]
    Unavailable,
    /// JSON parse / options conversion failed.
    #[error("reason_class=webauthn_browser: invalid WebAuthn options")]
    InvalidOptions,
    /// User cancelled or authenticator failed.
    #[error("reason_class=webauthn_browser: authenticator cancelled or failed")]
    CeremonyFailed,
    /// Credential JSON serialization failed.
    #[error("reason_class=webauthn_browser: could not serialize credential")]
    SerializeFailed,
}

fn credentials() -> Result<CredentialsContainer, WebauthnBrowserError> {
    let window = web_sys::window().ok_or(WebauthnBrowserError::Unavailable)?;
    Ok(window.navigator().credentials())
}

fn public_key_ctor() -> Result<JsValue, WebauthnBrowserError> {
    let window = web_sys::window().ok_or(WebauthnBrowserError::Unavailable)?;
    Reflect::get(&window, &JsValue::from_str("PublicKeyCredential"))
        .map_err(|_| WebauthnBrowserError::Unavailable)
}

fn call_static(method: &str, arg: &JsValue) -> Result<JsValue, WebauthnBrowserError> {
    let ctor = public_key_ctor()?;
    let f = Reflect::get(&ctor, &JsValue::from_str(method))
        .map_err(|_| WebauthnBrowserError::Unavailable)?;
    let f: Function = f
        .dyn_into()
        .map_err(|_| WebauthnBrowserError::Unavailable)?;
    f.call1(&ctor, arg)
        .map_err(|_| WebauthnBrowserError::InvalidOptions)
}

fn value_to_js(value: &Value) -> Result<JsValue, WebauthnBrowserError> {
    let text = serde_json::to_string(value).map_err(|_| WebauthnBrowserError::InvalidOptions)?;
    JSON::parse(&text).map_err(|_| WebauthnBrowserError::InvalidOptions)
}

fn extract_public_key(options: &Value) -> Result<Value, WebauthnBrowserError> {
    if let Some(pk) = options.get("publicKey") {
        return Ok(pk.clone());
    }
    Ok(options.clone())
}

fn credential_to_json(credential: &JsValue) -> Result<String, WebauthnBrowserError> {
    let to_json = Reflect::get(credential, &JsValue::from_str("toJSON"))
        .map_err(|_| WebauthnBrowserError::SerializeFailed)?;
    let to_json: Function = to_json
        .dyn_into()
        .map_err(|_| WebauthnBrowserError::SerializeFailed)?;
    let json_obj = to_json
        .call0(credential)
        .map_err(|_| WebauthnBrowserError::SerializeFailed)?;
    let text = JSON::stringify(&json_obj).map_err(|_| WebauthnBrowserError::SerializeFailed)?;
    text.as_string()
        .ok_or(WebauthnBrowserError::SerializeFailed)
}

async fn invoke_credentials_method(
    method: &str,
    public_key: &JsValue,
) -> Result<JsValue, WebauthnBrowserError> {
    let creds = credentials()?;
    let opts = Object::new();
    Reflect::set(&opts, &JsValue::from_str("publicKey"), public_key)
        .map_err(|_| WebauthnBrowserError::InvalidOptions)?;
    let f = Reflect::get(creds.as_ref(), &JsValue::from_str(method))
        .map_err(|_| WebauthnBrowserError::Unavailable)?;
    let f: Function = f
        .dyn_into()
        .map_err(|_| WebauthnBrowserError::Unavailable)?;
    let promise = f
        .call1(creds.as_ref(), opts.as_ref())
        .map_err(|_| WebauthnBrowserError::CeremonyFailed)?;
    let promise: js_sys::Promise = promise
        .dyn_into()
        .map_err(|_| WebauthnBrowserError::CeremonyFailed)?;
    JsFuture::from(promise)
        .await
        .map_err(|_| WebauthnBrowserError::CeremonyFailed)
}

/// Run `navigator.credentials.create` for passkey registration.
///
/// `creation_options` is the server `CreationChallengeResponse` JSON (with or
/// without a top-level `publicKey` wrapper).
///
/// # Errors
///
/// [`WebauthnBrowserError`] when the API is missing, options are invalid, or the
/// authenticator fails / is cancelled.
pub async fn credentials_create_json(
    creation_options: &Value,
) -> Result<String, WebauthnBrowserError> {
    let public_key_json = extract_public_key(creation_options)?;
    let public_key_js = value_to_js(&public_key_json)?;
    let parsed = call_static("parseCreationOptionsFromJSON", &public_key_js)?;
    let credential = invoke_credentials_method("create", &parsed).await?;
    if credential.is_null() || credential.is_undefined() {
        return Err(WebauthnBrowserError::CeremonyFailed);
    }
    credential_to_json(&credential)
}

/// Run `navigator.credentials.get` for passkey assertion.
///
/// `request_options` is the server `RequestChallengeResponse` JSON (with or
/// without a top-level `publicKey` wrapper).
///
/// # Errors
///
/// [`WebauthnBrowserError`] when the API is missing, options are invalid, or the
/// authenticator fails / is cancelled.
pub async fn credentials_get_json(request_options: &Value) -> Result<String, WebauthnBrowserError> {
    let public_key_json = extract_public_key(request_options)?;
    let public_key_js = value_to_js(&public_key_json)?;
    let parsed = call_static("parseRequestOptionsFromJSON", &public_key_js)?;
    let credential = invoke_credentials_method("get", &parsed).await?;
    if credential.is_null() || credential.is_undefined() {
        return Err(WebauthnBrowserError::CeremonyFailed);
    }
    credential_to_json(&credential)
}
