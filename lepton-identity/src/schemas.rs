// Schema modules register Valence tables for identity (user, account, profile, membership, photos).

mod user_schema {
    include!("../schemas/user_valence_schema.rs");
}

mod account_email_schema {
    include!("../schemas/account_email_valence_schema.rs");
}

mod account_phone_schema {
    include!("../schemas/account_phone_valence_schema.rs");
}

mod linked_identity_schema {
    include!("../schemas/linked_identity_valence_schema.rs");
}

mod oauth_pending_state_schema {
    include!("../schemas/oauth_pending_state_valence_schema.rs");
}

mod auth_device_schema {
    include!("../schemas/auth_device_valence_schema.rs");
}

mod auth_device_ceremony_schema {
    include!("../schemas/auth_device_ceremony_valence_schema.rs");
}

mod user_profile_schema {
    include!("../schemas/user_profile_valence_schema.rs");
}

mod account_schema {
    include!("../schemas/account_valence_schema.rs");
}

mod account_membership_schema {
    include!("../schemas/account_membership_valence_schema.rs");
}

mod file_trait {
    include!("../schemas/file_valence_trait.rs");
}

mod profile_photo_schema {
    include!("../schemas/profile_photo_valence_schema.rs");
}

mod user_appearance_schema {
    include!("../schemas/user_appearance_valence_schema.rs");
}
