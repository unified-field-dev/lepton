# lepton-test-support

Test-only identity builders, named seed scenarios, and optional Axum seed HTTP
for Lepton harnesses (`publish = false`).

Use this when a Playwright or integration test needs a user in a known state
without walking the signup UI. For the real signup → confirm pipeline, use
[`lepton-e2e`](../lepton-e2e/) instead.

## Quick start

```rust,ignore
use lepton_test_support::TestUserBuilder;
use lepton_e2e::boot::boot_valence;

# async fn demo() -> Result<(), lepton_test_support::SeedError> {
let v = boot_valence("demo").await?;
let user = TestUserBuilder::new()
    .email("alice@example.test")
    .verified_email()
    .with_totp()
    .build(&v)
    .await?;
assert!(user.totp_secret.is_some());
# Ok(())
# }
```

Named scenarios (same JSON as Playwright):

| Scenario | Result |
|----------|--------|
| `auth_basic_user` | Verified Active user |
| `auth_unverified_user` | Unverified email |
| `auth_confirm_email_only` | Verified email (confirm mid-state) |
| `auth_confirm_ready` | Verified email + phone |
| `auth_confirm_done` | Confirmed (trust) |
| `auth_reset_token` | Reset token in response |
| `auth_user_with_totp` | TOTP secret in response |

```rust,ignore
use lepton_test_support::{run_seed, SeedRequest};

# async fn demo(v: &valence::Valence) -> Result<(), lepton_test_support::SeedError> {
let out = run_seed(v, SeedRequest {
    scenario: "auth_basic_user".into(),
    email: Some("u@example.test".into()),
    password: None,
}).await?;
# Ok(())
# }
```

## Axum mount (`features = ["axum"]`)

Implement `SeedValence` on host state and route `POST /api/test/seed-data` to
`seed_data::<YourState>`. The auth UI harness does this. Do not mount the route
on production product binaries.

Responses may include plaintext passwords, reset tokens, and TOTP secrets.

## Playwright helpers

Shared TypeScript fixtures live next to the harness:

`lepton-auth-ui-e2e/end2end/shared/` (`seedTestData`, `signInAs`, Mailpit/SMS helpers).

## Verify

```bash
cargo test -p lepton-test-support --all-features
```
