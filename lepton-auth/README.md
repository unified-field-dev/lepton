# lepton-auth

Leptos auth UI, server functions, password policy, and verification delivery for Unified Field hosts.

## Organized by task

| Task | Feature | Start here |
|------|---------|------------|
| **Boot delivery** — inject email/SMS + public URL | `ssr` (+ channels) | [`services`](src/services.rs), crate-root Host wiring |
| **Send verification / reset mail** | `email` | [`email_delivery`](src/email_delivery.rs) |
| **Issue / consume one-time tokens** | `ssr` / `phone` | [`token_helpers`](src/token_helpers/) |
| **Email / SMS OTP / TOTP verify** | `email` / `phone` / `totp` | [`factor`](src/factor/) |
| **Contacts / confirm** | `ssr` | [`contacts`](src/contacts/), [`trust`](src/trust/) |
| **TOTP enroll** | `totp` | [`totp`](src/totp/) |
| **Trusted devices** | `ssr` (+ `webauthn`) | [`devices`](src/devices/) — TrustedBrowser; WebAuthn ceremony with `webauthn`; binding cookie MFA-skip |
| **Login MFA / session bind** | `ssr` (+ `totp` / `webauthn`) | [`session_mfa`](src/session_mfa/), [`session_binding`](src/session_binding/); sign-in actions |
| **Device server fns / Account Settings** | `ssr` (+ `webauthn` on host SSR) | [`actions/devices`](src/actions/devices.rs); hydrate [`webauthn_browser`](src/webauthn_browser.rs); host Account Settings UI |
| **OAuth (mock + live Google/GitHub)** | `ssr` (+ `oauth-google` / `oauth-github`) | [`oauth`](src/oauth/) |
| **SMS adapters** | `phone` | [`../lepton-sms`](../lepton-sms/) |
| **Live status refetch (Photon)** | `ssr` | [`events`](src/events.rs), [`verification`](src/verification.rs) |
| **Sign-in / sign-up / reset UI** | — | [`lepton-auth-ui`](../lepton-auth-ui/), [`actions`](src/actions/) |
| **Password policy / audit** | — | [`security`](src/security.rs) |

## Typical verification flow (backend)

1. Host builds plain `SmtpConfig` / SMS adapters and calls `provide_auth_services` once at SSR boot (only enabled channels).
2. Issue email or SMS OTP via `FactorChallengeService` channel methods (or existing account actions).
3. On success, consume with unique-marker winner checks; set contact `verified_at` (+ primary when unset); publish `VerificationCompleted`.
4. Clients refetch via `verification_status(challenge_id)` (capability key; high entropy).

**Breaking:** `User.email` / `phone` / verified bools / `totp_enabled` removed — use
`AccountEmail` / `AccountPhone` and `TotpFactor`. Login resolves via `AccountEmail.address`.

## Feature flags

| Feature | Role |
|---------|------|
| `ssr` | Server functions, Valence helpers (no delivery deps); includes Signup |
| `email` | Verification / reset **mail** (`lepton-smtp`); not email-as-login |
| `phone` | SMS OTP + `lepton-sms` |
| `totp` / `two_factor` | TOTP verify + enroll (`totp-rs`) |
| `webauthn` | WebAuthn passkey ceremony for `AuthDevice` (`webauthn-rs`) |
| `oauth-google` | Live Google authorize URL + token/userinfo exchange (`ssr` supplies `reqwest`) |
| `oauth-github` | Live GitHub authorize URL + token/user/emails exchange (`ssr` supplies `reqwest`) |
| `full` | `email` + `phone` + `totp` + oauth flags + `webauthn` |
| `spectra` | Auth funnel Spectra counters / `lepton_auth_failure` via `lepton-spectra-telemetry` (ops-id labels only) |
| `hydrate` | Client hydration of auth UI |
| `test-utils` | Publish-capture helpers for integration tests |

Mail hosts: `features = ["ssr", "email"]`. CI / full channels: `ssr,full`.
Ops metrics: add `spectra` after the host boots Spectra (see `lepton-spectra-telemetry`).

## Host env (selected)

| Var | Role |
|-----|------|
| `UF_PUBLIC_BASE_URL` | Origin used in verification / reset links (set via services builder) |
| `UF_TOTP_ISSUER` | otpauth issuer for Account Settings TOTP enroll (default `Unified Field`) |

## Integration checklist

1. `provide_auth_services(LeptonAuthServicesBuilder…)` at boot — **fail closed** (no env rebuild on send).
   For passkeys, also `.webauthn_rp(WebauthnRpConfig { … })` and enable `lepton-auth/webauthn` on the **SSR** host (not on wasm).
2. Supply secrets as **plain strings** from the host (env, Vault, …). This crate does not load a secrets manager.
3. Mount photon-leptos `ws_router` + Origin allowlist when using live verification UI.
4. Prefer the Mailpit harness (`../infra/mailpit`) to validate real SMTP; CI stays Docker-free.

## Further reading

- Crate rustdoc (`cargo doc -p lepton-auth --features ssr,full --open`)
- [`examples/README.md`](examples/README.md)
- [`../docs/VERIFICATION.md`](../docs/VERIFICATION.md)
