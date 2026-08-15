//! AccountMembership delete side effects.

pub mod clear_account_primary_on_membership_delete;

pub use clear_account_primary_on_membership_delete::{
    clear_account_primaries_if_login_matched, ClearAccountPrimaryOnMembershipDelete,
};

#[cfg(feature = "test-utils")]
pub use clear_account_primary_on_membership_delete::force_primary_clear_failure;
