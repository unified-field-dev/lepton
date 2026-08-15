//! Relying-party configuration for `WebAuthn` ceremonies.

use url::Url;
use webauthn_rs::prelude::{Webauthn, WebauthnBuilder};

use super::DeviceError;

/// Host-supplied `WebAuthn` relying party (no secrets).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WebauthnRpConfig {
    /// RP ID (usually eTLD+1, e.g. `example.com` or `localhost`).
    pub rp_id: String,
    /// Expected origin URL (e.g. `https://app.example.com`).
    pub rp_origin: String,
    /// Human-readable relying party name.
    pub rp_name: String,
}

impl WebauthnRpConfig {
    /// Build a [`Webauthn`] instance from this config.
    ///
    /// # Errors
    ///
    /// [`DeviceError::Config`] when `rp_id` / `rp_origin` cannot form a valid RP.
    pub(super) fn build_webauthn(&self) -> Result<Webauthn, DeviceError> {
        let rp_id = self.rp_id.trim();
        let rp_name = self.rp_name.trim();
        if rp_id.is_empty() || rp_name.is_empty() {
            return Err(DeviceError::Config);
        }
        let origin = Url::parse(self.rp_origin.trim()).map_err(|_| DeviceError::Config)?;
        let builder = WebauthnBuilder::new(rp_id, &origin).map_err(|_| DeviceError::Config)?;
        builder
            .rp_name(rp_name)
            .build()
            .map_err(|_| DeviceError::Config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webauthn_rp_config_invalid_origin_sad() {
        let rp = WebauthnRpConfig {
            rp_id: "localhost".into(),
            rp_origin: "not-a-url".into(),
            rp_name: "Lepton".into(),
        };
        assert!(matches!(rp.build_webauthn(), Err(DeviceError::Config)));
        assert_eq!(DeviceError::Config.reason_class(), "config");
    }

    #[test]
    fn webauthn_rp_config_empty_rp_id_sad() {
        let rp = WebauthnRpConfig {
            rp_id: "  ".into(),
            rp_origin: "http://localhost:3000".into(),
            rp_name: "Lepton".into(),
        };
        assert!(matches!(rp.build_webauthn(), Err(DeviceError::Config)));
    }
}
