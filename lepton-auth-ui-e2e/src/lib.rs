#![allow(missing_docs)]
#![recursion_limit = "256"]

pub mod app;
pub mod connected_accounts_section;
pub mod devices_section;
pub mod page_shell;
pub mod step_up_demo;
pub mod totp_section;
pub mod wipe_section;

#[cfg(feature = "ssr")]
pub mod boot;
#[cfg(feature = "ssr")]
pub mod seed;

pub use app::{shell, App};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::hydrate_body(App);
}
