//! Client helpers for one-time token URLs (fragment-first, legacy query support).

use crate::routes::parse_token_from_url_parts;

/// Read a one-time token from the current URL fragment (preferred) or query (legacy).
#[cfg(feature = "hydrate")]
pub fn read_token_from_window_location() -> String {
    let Some(window) = leptos::web_sys::window() else {
        return String::new();
    };
    let search = window.location().search().unwrap_or_default();
    let hash = window.location().hash().unwrap_or_default();
    parse_token_from_url_parts(&search, &hash).unwrap_or_default()
}

/// Remove a legacy `?token=` query param from the address bar after the client reads it.
#[cfg(feature = "hydrate")]
pub fn strip_legacy_token_query_from_address_bar() {
    let Some(window) = leptos::web_sys::window() else {
        return;
    };
    let Ok(search) = window.location().search() else {
        return;
    };
    if !search.contains("token=") {
        return;
    }
    let Ok(pathname) = window.location().pathname() else {
        return;
    };
    let hash = window.location().hash().unwrap_or_default();
    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(
            &wasm_bindgen::JsValue::NULL,
            "",
            Some(&format!("{pathname}{hash}")),
        );
    }
}
