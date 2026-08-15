//! Validate a password and one-time token without a database or SMTP server.
//!
//! Run with:
//! `CARGO_BUILD_JOBS=1 cargo run -p lepton-auth --example password_and_token --features ssr`

#![allow(clippy::print_stderr)]

use lepton_auth::{
    routes::sanitize_referer_path,
    security::{password_policy_error_message, random_token_part},
    token_helpers::verify_token_secret,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = "CorrectHorseBattery1!";
    assert!(password_policy_error_message(password).is_none());

    // Store only a PHC hash. The plaintext value represents the secret carried
    // in a verification or password-reset link.
    let token = random_token_part(12);
    let token_hash = lepton_host_adapter::auth::hash_password(&token)?;
    verify_token_secret(&token, &token_hash)
        .map_err(|error| std::io::Error::other(error.message()))?;

    // Keep a post-auth redirect inside the application.
    assert_eq!(
        sanitize_referer_path(Some("/account".to_string())),
        "/account"
    );
    assert_eq!(
        sanitize_referer_path(Some("//example.invalid".to_string())),
        "/"
    );

    eprintln!("password_and_token: OK — policy, token verify, redirect sanitize");
    Ok(())
}
