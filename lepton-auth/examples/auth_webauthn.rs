//! Teaching example: `WebAuthn` `AuthDevice` register / assert / revoke.
//!
//! ```bash
//! cargo check -p lepton-auth --example auth_webauthn --features ssr,webauthn
//! ```
//!
//! OAuth is intentionally omitted — `WebAuthn` is device trust after sign-in.

#![allow(dead_code)]

use lepton_auth::devices::{
    begin_webauthn_assertion, begin_webauthn_registration, finish_webauthn_assertion,
    finish_webauthn_registration, list_auth_devices, revoke_auth_device, WebauthnRpConfig,
};
use serde_json::Value;

async fn register_and_assert_passkey(
    v: &valence::Valence,
    user: valence::RecordId,
    attestation_json: &Value,
    assertion_json: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let rp = WebauthnRpConfig {
        rp_id: "localhost".into(),
        rp_origin: "http://localhost:3000".into(),
        rp_name: "Lepton".into(),
    };

    let reg = begin_webauthn_registration(v, &rp, &user, "YubiKey").await?;
    let device =
        finish_webauthn_registration(v, &rp, &user, &reg.ceremony_id, attestation_json).await?;

    let assert_start = begin_webauthn_assertion(v, &rp, &user).await?;
    let _ =
        finish_webauthn_assertion(v, &rp, &user, &assert_start.ceremony_id, assertion_json).await?;

    let devices = list_auth_devices(v, &user).await?;
    assert!(devices.iter().any(|d| d.id == device.device_id));
    revoke_auth_device(v, &user, &device.device_id).await?;
    Ok(())
}

fn main() {
    let _ = register_and_assert_passkey;
}
