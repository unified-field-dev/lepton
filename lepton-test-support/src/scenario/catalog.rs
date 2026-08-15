//! Named scenario → [`TestUserBuilder`] compositions.

use crate::builder::{TestUserBuilder, DEFAULT_PASSWORD};
use crate::error::SeedError;
use crate::http::{SeedRequest, SeedResponse};
use valence::Valence;

use super::{
    AUTH_BASIC_USER, AUTH_CONFIRM_DONE, AUTH_CONFIRM_EMAIL_ONLY, AUTH_CONFIRM_READY,
    AUTH_RESET_TOKEN, AUTH_UNVERIFIED_USER, AUTH_USER_WITH_TOTP,
};

pub(super) async fn run_catalog(
    valence: &Valence,
    request: SeedRequest,
) -> Result<SeedResponse, SeedError> {
    let email = request
        .email
        .unwrap_or_else(|| "user@example.com".to_string());
    let password = request
        .password
        .unwrap_or_else(|| DEFAULT_PASSWORD.to_string());
    let scenario = request.scenario;

    let builder = match scenario.as_str() {
        AUTH_BASIC_USER | AUTH_CONFIRM_EMAIL_ONLY => TestUserBuilder::new()
            .email(email.clone())
            .password(password.clone())
            .verified_email(),
        AUTH_UNVERIFIED_USER => TestUserBuilder::new()
            .email(email.clone())
            .password(password.clone())
            .unverified_email(),
        AUTH_CONFIRM_READY => TestUserBuilder::new()
            .email(email.clone())
            .password(password.clone())
            .verified_email()
            .with_verified_phone(),
        AUTH_CONFIRM_DONE => TestUserBuilder::new()
            .email(email.clone())
            .password(password.clone())
            .verified_email()
            .with_verified_phone()
            .confirmed(),
        AUTH_RESET_TOKEN => TestUserBuilder::new()
            .email(email.clone())
            .password(password.clone())
            .verified_email()
            .with_reset_token(),
        AUTH_USER_WITH_TOTP => TestUserBuilder::new()
            .email(email.clone())
            .password(password.clone())
            .verified_email()
            .with_totp(),
        other => {
            return Err(SeedError::UnknownScenario {
                scenario: other.to_string(),
            });
        }
    };

    let seeded = builder.build(valence).await?;
    Ok(SeedResponse {
        scenario,
        email: seeded.email,
        password: seeded.password,
        reset_token: seeded.reset_token,
        totp_secret: seeded.totp_secret,
    })
}
