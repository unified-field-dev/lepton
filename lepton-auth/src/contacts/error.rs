//! Typed errors for contact APIs.

use thiserror::Error;

/// Errors from [`super`] contact helpers.
#[derive(Debug, Error)]
pub enum ContactError {
    /// Contact is not verified; primary selection blocked.
    #[error("reason_class=unverified_contact: contact is not verified")]
    Unverified,
    /// Address / e164 already registered.
    #[error("reason_class=address_taken: contact address already registered")]
    Conflict,
    /// User row missing.
    #[error("reason_class=user: user not found")]
    UserMissing,
    /// Account row missing.
    #[error("reason_class=account: account not found")]
    AccountMissing,
    /// Contact's email is not on the target account (or user has no membership).
    #[error("reason_class=not_member: email is not on the target account")]
    NotMember,
    /// Contact row missing or not owned by user.
    #[error("reason_class=contact: contact not found")]
    ContactMissing,
    /// Persistence / Valence failure (opaque).
    #[error("reason_class=store: contact store operation failed")]
    Store,
}

impl ContactError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::Unverified => "unverified_contact",
            Self::Conflict => "address_taken",
            Self::UserMissing => "user",
            Self::AccountMissing => "account",
            Self::NotMember => "not_member",
            Self::ContactMissing => "contact",
            Self::Store => "store",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_error_display_omits_secrets_happy_path() {
        let err = ContactError::Conflict;
        let msg = err.to_string();
        assert!(msg.contains("reason_class=address_taken"));
        assert!(!msg.contains('@'));
        assert_eq!(err.reason_class(), "address_taken");
    }

    #[test]
    fn account_missing_and_not_member_reason_classes() {
        assert_eq!(ContactError::AccountMissing.reason_class(), "account");
        assert_eq!(ContactError::NotMember.reason_class(), "not_member");
        assert!(!ContactError::AccountMissing.to_string().contains('@'));
        assert!(!ContactError::NotMember.to_string().contains('@'));
    }
}
