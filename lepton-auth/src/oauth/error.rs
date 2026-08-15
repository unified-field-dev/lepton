//! Typed errors for OAuth APIs.

use thiserror::Error;

/// Errors from OAuth begin / complete / link.
#[derive(Debug, Error)]
pub enum OAuthError {
    /// CSRF / state mismatch or expired.
    #[error("reason_class=oauth_state: oauth state invalid")]
    State,
    /// Provider subject already linked to another user.
    #[error("reason_class=oauth_account_taken: oauth identity already linked")]
    AccountTaken,
    /// Missing client config / feature.
    #[error("reason_class=oauth_config: oauth client misconfigured")]
    Config,
    /// User missing for link intent.
    #[error("reason_class=user: user not found")]
    UserMissing,
    /// Linked identity missing.
    #[error("reason_class=link: linked identity not found")]
    LinkMissing,
    /// Provider HTTP / exchange failure (opaque).
    #[error("reason_class=oauth_provider: oauth provider error")]
    Provider,
    /// Persistence failure.
    #[error("reason_class=store: oauth store operation failed")]
    Store,
}

impl OAuthError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::State => "oauth_state",
            Self::AccountTaken => "oauth_account_taken",
            Self::Config => "oauth_config",
            Self::UserMissing => "user",
            Self::LinkMissing => "link",
            Self::Provider => "oauth_provider",
            Self::Store => "store",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_state_mismatch_sad() {
        let err = OAuthError::State;
        assert_eq!(err.reason_class(), "oauth_state");
        assert!(!err.to_string().contains("client_secret"));
    }

    #[test]
    fn oauth_account_taken_sad() {
        assert_eq!(
            OAuthError::AccountTaken.reason_class(),
            "oauth_account_taken"
        );
    }
}
