# lepton-auth examples

Teaching binaries mapped to crate-root **Host recipes**. Rustdoc module pages hold
the highlight sketches; these commands are the runnable / checkable layer.

| Product job | Example | Features | rustdoc |
|-------------|---------|----------|---------|
| Password policy + PHC / token secret | `password_and_token` | `ssr` | [`security`](../src/security.rs), [`token_helpers`](../src/token_helpers/mod.rs) |
| Step-up TOTP + password re-check | `step_up_totp` | `ssr,totp` | [`factor`](../src/factor/mod.rs) |
| TOTP enroll | `auth_totp_enroll` | `ssr,totp` | [`totp`](../src/totp/mod.rs) |
| Credential + noop SMTP envelopes | `auth_flows_noop_smtp` | `ssr,email` | `lepton-smtp` envelopes (not `provide_auth_services`) |
| Credential + Mailpit SMTP | `auth_flows_smtp_mailpit` | `ssr,email` | same |
| Contacts + confirm | `auth_contacts_confirm` | `ssr` | [`contacts`](../src/contacts/mod.rs), [`trust`](../src/trust/mod.rs) |
| Trust / id-verify | `auth_trust_confirm` | `ssr` | [`trust`](../src/trust/mod.rs) |
| Trusted browser device | `auth_devices` | `ssr` | [`devices`](../src/devices/mod.rs) |
| WebAuthn passkey device | `auth_webauthn` | `ssr,webauthn` | [`devices`](../src/devices/mod.rs) |
| OAuth mock provider | `auth_oauth_mock` | `ssr,oauth-github` | [`oauth`](../src/oauth/mod.rs) |

Leptos injection (`provide_auth_services`), stock `email_delivery::send_*`, and Photon
status are documented on their modules / crate-root Host recipes — these binaries do
not spin a full SSR host.

## 1. Policy + token — `password_and_token`

```bash
CARGO_BUILD_JOBS=1 cargo run -p lepton-auth --example password_and_token --features ssr
```

Success: stderr prints `password_and_token: OK — policy, token verify, redirect sanitize`.

## 2. Step-up / re-check — `step_up_totp`

```bash
CARGO_BUILD_JOBS=1 cargo run -p lepton-auth --example step_up_totp --features ssr,totp
```

Success: `step_up_totp: OK — password re-check + TOTP step-up sketch`.

## 3. Credential + email — `auth_flows_noop_smtp`

Builder-constructed noop delivery (envelopes / adapters). Does not call
`provide_auth_services`.

```bash
CARGO_BUILD_JOBS=1 \
  cargo run -p lepton-auth --example auth_flows_noop_smtp --features ssr,email
```

Success: stdout prints `auth_flows_noop_smtp: OK — signup/login/reset + noop SMTP`.

## 4. Real SMTP — `auth_flows_smtp_mailpit`

Requires Mailpit (`infra/mailpit`). Prefer the validating integ orchestrator:

```bash
./infra/mailpit/smtp_smoke.sh
```

Or run the example after `docker compose -f infra/mailpit/docker-compose.yml up -d`:

```bash
CARGO_BUILD_JOBS=1 \
  cargo run -p lepton-auth --example auth_flows_smtp_mailpit --features ssr,email
```

Success: `provider=smtp` receipt; messages visible at http://127.0.0.1:8025.

## Look next

Inject `LeptonAuthServices` at host boot (`provide_auth_services` — see `services`
rustdoc). Customize mail via hand-built `lepton_smtp::EmailEnvelope`. Mount
auth pages on a Higgs SSR host; subscribe to `verification_status` via photon-leptos.
