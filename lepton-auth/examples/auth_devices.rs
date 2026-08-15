//! Teaching example B3: `TrustedBrowser` device register / confirm / revoke.
//!
//! ```bash
//! cargo check -p lepton-auth --example auth_devices --features ssr
//! ```

#![allow(dead_code)]

use lepton_auth::devices::{
    confirm_auth_device, list_auth_devices, register_auth_device, revoke_auth_device,
    AuthDeviceKind,
};

async fn trust_this_browser(
    v: &valence::Valence,
    user: valence::RecordId,
) -> Result<(), Box<dyn std::error::Error>> {
    let pending = register_auth_device(v, &user, AuthDeviceKind::TrustedBrowser, "Laptop").await?;
    confirm_auth_device(v, &user, &pending.device_id, &pending.confirm_code).await?;
    let devices = list_auth_devices(v, &user).await?;
    assert!(devices.iter().any(|d| d.trusted_at.is_some()));
    revoke_auth_device(v, &user, &pending.device_id).await?;
    Ok(())
}

fn main() {
    let _ = trust_this_browser;
}
