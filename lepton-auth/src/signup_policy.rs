//! Host-controlled open signup gate.
//!
//! By default signup is available under `ssr`. Private hosts set
//! `UF_LEPTON_SIGNUP_DISABLED` (`1` or `true`) so the `Signup` server function
//! fails closed. Hide sign-up CTAs in the host UI when disabled.

/// Process env: `1` / `true` refuses new account creation via `Signup`.
pub const SIGNUP_DISABLED_ENV: &str = "UF_LEPTON_SIGNUP_DISABLED";

/// Whether the `Signup` server function may create accounts (default: yes).
#[must_use]
pub fn signup_enabled() -> bool {
    signup_enabled_from_raw(std::env::var(SIGNUP_DISABLED_ENV).ok().as_deref())
}

/// Parse a raw env value the same way [`signup_enabled`] does (for tests).
#[must_use]
pub fn signup_enabled_from_raw(raw: Option<&str>) -> bool {
    raw.is_none_or(|raw| {
        let v = raw.trim();
        !(v == "1" || v.eq_ignore_ascii_case("true"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signup_enabled_default_happy_path() {
        assert!(signup_enabled_from_raw(None));
        assert!(signup_enabled_from_raw(Some("0")));
        assert!(signup_enabled_from_raw(Some("false")));
    }

    #[test]
    fn signup_disabled_env_values_sad_path() {
        assert!(!signup_enabled_from_raw(Some("1")));
        assert!(!signup_enabled_from_raw(Some("true")));
        assert!(!signup_enabled_from_raw(Some("TRUE")));
        assert!(!signup_enabled_from_raw(Some(" True ")));
    }
}
