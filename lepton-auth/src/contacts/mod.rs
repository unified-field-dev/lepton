//! Account email / phone contacts and primary selection.
//!
//! Emails and phones are owned by the user's `Account`. `User.primary_email` /
//! `primary_phone` are login pointers; `Account.primary_email` / `primary_phone` are
//! the legal primaries (Restrict blocks deleting that row while the account still
//! points at it). For account wipe, use [`crate::identity_delete::erase_account`].
//!
//! Human-entered phone numbers can be normalized with [`crate::contacts::normalize_phone_to_e164`]
//! before storage or SMS issue.
//!
//! # Examples
//!
//! ```rust,ignore
//! use lepton_auth::contacts::{
//!     add_account_email, add_account_phone, mark_account_email_verified,
//!     mark_account_phone_verified, normalize_phone_to_e164, set_account_primary_email,
//!     set_account_primary_phone, set_primary_email, set_primary_phone,
//! };
//! use lepton_auth::trust::{confirm_user, primary_email_verified, primary_phone_verified};
//! use valence::{RecordId, Valence};
//!
//! async fn promote_backup_and_confirm(
//!     v: &Valence,
//!     user: RecordId,
//!     account: RecordId,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     assert!(primary_email_verified(v, &user).await?);
//!
//!     let backup = add_account_email(v, &account, "backup@example.com").await?;
//!     mark_account_email_verified(v, &backup).await?;
//!     let backup_id = backup.id().cloned().ok_or("backup missing id")?;
//!     set_primary_email(v, &user, &backup_id).await?;
//!     set_account_primary_email(v, &account, &backup_id).await?;
//!
//!     let e164 = normalize_phone_to_e164("(555) 555-0100")?;
//!     let phone = add_account_phone(v, &account, &e164).await?;
//!     mark_account_phone_verified(v, &phone).await?;
//!     let phone_id = phone.id().cloned().ok_or("phone missing id")?;
//!     set_primary_phone(v, &user, &phone_id).await?;
//!     set_account_primary_phone(v, &account, &phone_id).await?;
//!     assert!(primary_phone_verified(v, &user).await?);
//!
//!     confirm_user(v, &user).await?;
//!     Ok(())
//! }
//! ```

mod phone_normalize;

pub use phone_normalize::{normalize_phone_to_e164, PhoneNormalizeError};

#[cfg(feature = "ssr")]
mod api;
#[cfg(feature = "ssr")]
mod error;

#[cfg(feature = "ssr")]
pub use api::{
    account_for_user, add_account_email, add_account_phone, find_account_email_by_address,
    mark_account_email_verified, mark_account_phone_verified, set_account_primary_email,
    set_account_primary_phone, set_primary_email, set_primary_phone,
};
#[cfg(feature = "ssr")]
pub use error::ContactError;
