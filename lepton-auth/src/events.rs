//! Photon topics for auth verification progress (live UI refetch).
//!
//! # When to call
//!
//! After Valence persistence succeeds (token consume / factor confirm), call
//! `publish_verification_completed` (`ssr`). Publish is best-effort — failures must not
//! roll back verification. Hosts mount photon-leptos `ws_router` + Origin allowlist
//! separately.
//!
//! Payload is challenge-keyed (`challenge_id` + `kind` only). Never include email, phone,
//! OTP codes, or TOTP secrets.
//!
//! # Examples
//!
//! ```rust,ignore
//! use lepton_auth::events::{publish_verification_completed, VerificationKind};
//!
//! // … Valence consume / confirm succeeded …
//! publish_verification_completed(challenge_id, VerificationKind::Email).await;
//! ```
//!
//! Clients refetch [`crate::verification::verification_status`] keyed by the same
//! high-entropy `challenge_id`.

/// Logical verification channel completed for a challenge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationKind {
    /// Email verification token consumed.
    Email,
    /// Phone / SMS OTP token consumed.
    Phone,
    /// TOTP factor confirmed / enabled.
    Totp,
}

impl VerificationKind {
    /// Wire form used in [`VerificationCompleted::kind`].
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Totp => "totp",
        }
    }
}

/// Published after a verification challenge succeeds (keyed by `challenge_id`).
#[cfg(feature = "ssr")]
#[photon::topic(name = "auth.verification.completed", keyed_by = "challenge_id")]
pub struct VerificationCompleted {
    /// Token / challenge id that was verified.
    pub challenge_id: String,
    /// `"email"`, `"phone"`, or `"totp"`.
    pub kind: String,
}

/// Short opaque fingerprint for logs (never log full challenge ids / OTP capability keys).
#[cfg(feature = "ssr")]
#[must_use]
pub fn challenge_id_fingerprint(challenge_id: &str) -> String {
    use std::fmt::Write;
    let bytes = challenge_id.as_bytes();
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        acc ^= u64::from(*b);
        acc = acc.wrapping_mul(0x0100_0000_01b3);
    }
    let mut out = String::with_capacity(16);
    let _ = write!(out, "{acc:016x}");
    out
}

/// Publish [`VerificationCompleted`].
///
/// Publish failures are logged at warn (fingerprint only) and **do not** fail the caller —
/// verification persistence is the source of truth; Photon is best-effort for live UI.
#[cfg(feature = "ssr")]
pub async fn publish_verification_completed(challenge_id: String, kind: VerificationKind) {
    #[cfg(any(test, feature = "test-utils"))]
    test_support::record_publish(challenge_id.clone(), kind);

    let event = VerificationCompleted {
        challenge_id: challenge_id.clone(),
        kind: kind.as_str().to_string(),
    };
    if let Err(e) = event.publish().await {
        tracing::warn!(
            challenge_fp = %challenge_id_fingerprint(&challenge_id),
            kind = kind.as_str(),
            error = %e,
            "auth.verification.completed publish failed (ignored)"
        );
    }
}

/// Publish capture for unit / integration tests (`test-utils` feature or `cfg(test)`).
#[cfg(all(feature = "ssr", any(test, feature = "test-utils")))]
pub mod test_support {
    use super::VerificationKind;
    use std::sync::{Mutex, OnceLock};

    fn log() -> &'static Mutex<Vec<(String, VerificationKind)>> {
        static LOG: OnceLock<Mutex<Vec<(String, VerificationKind)>>> = OnceLock::new();
        LOG.get_or_init(|| Mutex::new(Vec::new()))
    }

    pub(super) fn record_publish(challenge_id: String, kind: VerificationKind) {
        if let Ok(mut guard) = log().lock() {
            guard.push((challenge_id, kind));
        }
    }

    /// Drain recorded publishes.
    #[must_use]
    pub fn take_published() -> Vec<(String, VerificationKind)> {
        log()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect()
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn photon_publish_records_on_success_path_happy_path() {
        let _ = test_support::take_published();
        publish_verification_completed("chal-abc".into(), VerificationKind::Email).await;
        let got = test_support::take_published();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, "chal-abc");
        assert_eq!(got[0].1, VerificationKind::Email);
    }

    #[test]
    fn challenge_id_fingerprint_stable_and_not_plaintext() {
        let fp = challenge_id_fingerprint("super-secret-challenge");
        assert_eq!(fp.len(), 16);
        assert!(!fp.contains("secret"));
        assert_eq!(fp, challenge_id_fingerprint("super-secret-challenge"));
    }
}
