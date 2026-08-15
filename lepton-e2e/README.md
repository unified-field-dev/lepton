# lepton-e2e

CI e2e coverage for signup → email → phone → confirm, trusted-browser
device + TOTP, and OAuth signup/login (mock provider). Interactive CLIs:

- `lepton-live-verify` — live Twilio email (SendGrid) + SMS signup/confirm
- `lepton-live-totp` — test user setup (no Twilio), real Google Authenticator via printed
  `otpauth://` URI and stdin codes
- `lepton-live-oauth` — live Google or GitHub OAuth signup then login (loopback callback)
- `lepton-mock-oidc` — lab OIDC IdP on `127.0.0.1:5556`
- `lepton-sms-sink` — lab SMS HTTP capture on `127.0.0.1:8099`

This crate is `publish = false`. Default CI never calls live Twilio, Google, GitHub, or
prompts for authenticator codes. Sidecar modules (`mock_oidc`, `sms_sink`) run under
default `cargo test` on ephemeral ports.

## CI e2e

```bash
cargo test -p lepton-e2e --lib --tests
```

Uses in-memory Valence, Noop email, and [`TestSmsAdapter`](../lepton-sms) so OTP
bodies are captured without a network. After confirm, device + TOTP tests call
[`run_device_totp_challenge_flow`](src/flow.rs) with [`TestTotpCodeSource`](src/flow.rs)
(or the sad-path APIs directly).

Lab sidecars (also covered by CI-always HTTP tests):

```bash
cargo run -p lepton-e2e --bin lepton-mock-oidc   # :5556
cargo run -p lepton-e2e --bin lepton-sms-sink    # :8099
```

See [`infra/mailpit/README.md`](../infra/mailpit/README.md) for smoke scripts and
`UF_SMS_SINK` / `UF_MOCK_OIDC_URL` wiring.

## Live TOTP CLI (Google Authenticator)

No Twilio. Creates a confirmed user and prompts for enroll and challenge codes. The
manual setup-key **secret** and `otpauth://` URI are hidden by default; set
`UF_LEPTON_LIVE_REVEAL_SECRET=1` in an interactive terminal to display them (the CLI
refuses to print the secret to a redirected or piped stream).

```bash
export UF_LEPTON_LIVE_TOTP=1
export UF_LIVE_VERIFY_LEGAL_NAME="Alex Rivera"
export UF_LIVE_VERIFY_EMAIL=you@example.com
export UF_LIVE_VERIFY_PHONE=+15555550199   # test SMS only; not a real send
# Generate a unique throwaway password per run — never reuse a real credential:
export UF_LIVE_VERIFY_PASSWORD="$(openssl rand -base64 18)Aa1!"
# optional: export UF_LIVE_VERIFY_DEVICE_LABEL="My Laptop"
# optional: export UF_LIVE_VERIFY_TOTP_ISSUER="Acme Site"   # shown in Authenticator

cargo run -p lepton-e2e --bin lepton-live-totp
```

1. CLI creates and confirms the test account (automatic email + SMS).
2. Trusts a browser device automatically.
3. With `UF_LEPTON_LIVE_REVEAL_SECRET=1`, prints Account name, Key (secret), and Type — use
   **Enter a setup key** in Google Authenticator (do not paste the otpauth URI into a browser).
4. Paste the 6-digit enroll code, then a current challenge code.

| Var | Role |
|-----|------|
| `UF_LEPTON_LIVE_TOTP` | Must be `1` or the binary exits |
| `UF_LIVE_VERIFY_LEGAL_NAME` / `EMAIL` / `PHONE` / `PASSWORD` | Test subject |
| `UF_LIVE_VERIFY_DEVICE_LABEL` | Optional device label (default `Live TOTP Browser`) |
| `UF_LIVE_VERIFY_TOTP_ISSUER` | Optional otpauth issuer for this CLI run (default `Lepton Auth`) |
| `UF_LEPTON_LIVE_REVEAL_SECRET=1` | Print manual-entry key + otpauth URI (interactive tty only) |

Product enroll (Account Settings TOTP) reads `UF_TOTP_ISSUER` instead (default
`Unified Field`) when building the otpauth issuer string.

Never run in CI. Do not commit filled env files.

## Live Google / GitHub OAuth CLI

Requires a Google Cloud OAuth **Web** client or a GitHub OAuth App. Authorized
redirect URI must match `{UF_PUBLIC_BASE_URL}{UF_OAUTH_REDIRECT_PATH}` (default
`http://127.0.0.1:8765/auth/oauth/callback`).

### Google

```bash
export UF_LEPTON_LIVE_OAUTH=1
export UF_OAUTH_PROVIDER=google
export UF_OAUTH_GOOGLE_CLIENT_ID=...
export UF_OAUTH_GOOGLE_CLIENT_SECRET=...
export UF_PUBLIC_BASE_URL=http://127.0.0.1:8765
# optional: export UF_OAUTH_REDIRECT_PATH=/auth/oauth/callback
# optional: export UF_OAUTH_CALLBACK_PORT=8765

cargo run -p lepton-e2e --bin lepton-live-oauth --features live-oauth
```

### GitHub

```bash
export UF_LEPTON_LIVE_OAUTH=1
export UF_OAUTH_PROVIDER=github
export UF_OAUTH_GITHUB_CLIENT_ID=...
export UF_OAUTH_GITHUB_CLIENT_SECRET=...
export UF_PUBLIC_BASE_URL=http://127.0.0.1:8765

cargo run -p lepton-e2e --bin lepton-live-oauth --features live-oauth
```

1. CLI listens on loopback for the OAuth callback.
2. Open the printed authorize URL; sign in with the provider and allow access.
3. Browser redirects to localhost; close the tab when prompted.
4. CLI completes signup, then repeats for login with the same account.
5. Exit 0 when both succeed.

| Var | Role |
|-----|------|
| `UF_LEPTON_LIVE_OAUTH` | Must be `1` or the binary exits |
| `UF_OAUTH_PROVIDER` | `google` (default) or `github` |
| `UF_OAUTH_GOOGLE_CLIENT_ID` / `UF_OAUTH_GOOGLE_CLIENT_SECRET` | Google OAuth client (when provider is google) |
| `UF_OAUTH_GITHUB_CLIENT_ID` / `UF_OAUTH_GITHUB_CLIENT_SECRET` | GitHub OAuth App (when provider is github) |
| `UF_PUBLIC_BASE_URL` | Redirect base (default `http://127.0.0.1:{port}`) |
| `UF_OAUTH_REDIRECT_PATH` | Callback path (default `/auth/oauth/callback`) |
| `UF_OAUTH_CALLBACK_PORT` | Loopback port (default `8765`) |

Never run in CI. Do not commit filled env files. The CLI never prints auth codes,
access tokens, or the client secret.

## Live Twilio CLI

Requires inbox + phone you control, Twilio credentials, and an explicit gate:

```bash
set -a
source infra/mailpit/mailpit.env.example   # then fill Twilio + LIVE_VERIFY_* vars
export UF_EMAIL_DRIVER=twilio
export UF_LEPTON_LIVE_TWILIO=1
export UF_LIVE_VERIFY_LEGAL_NAME="Alex Rivera"
export UF_LIVE_VERIFY_EMAIL=you@example.com
export UF_LIVE_VERIFY_PHONE=+15551234567
# Generate a unique throwaway password per run — never reuse a real credential:
export UF_LIVE_VERIFY_PASSWORD="$(openssl rand -base64 18)Aa1!"
# Optional: skip SendGrid + email paste; jump straight to SMS OTP prompt
# export UF_LIVE_VERIFY_SKIP_EMAIL=1
set +a

cargo run -p lepton-e2e --bin lepton-live-verify --features live-twilio
```

The binary sends a verification email, prompts for the pasteable code,
sends an SMS OTP, prompts for the code, then asserts both primaries are verified
and `confirm_user` succeeds. For authenticator MFA after a test user, use
`lepton-live-totp` instead.

### Env vars

| Var | Role |
|-----|------|
| `UF_LEPTON_LIVE_TWILIO` | Must be `1` or the binary exits |
| `UF_EMAIL_DRIVER=twilio` | SendGrid Mail Send path |
| `UF_TWILIO_EMAIL_API_KEY` | SendGrid API key |
| `UF_EMAIL_FROM` / `UF_EMAIL_FROM_NAME` | From identity |
| `UF_TWILIO_ACCOUNT_SID` / `UF_TWILIO_API_KEY` / `UF_TWILIO_API_SECRET` | Twilio auth (API key preferred) |
| `UF_TWILIO_VERIFY_SERVICE_SID` | Prefer **Twilio Verify** custom-code SMS when set (`VA…`; enable Custom Verification Code on the service) |
| `UF_TWILIO_FROM` | Required only for Messages fallback (when Verify SID unset) |
| `UF_PUBLIC_BASE_URL` | Link origin in the verification email |
| `UF_LIVE_VERIFY_LEGAL_NAME` / `UF_LIVE_VERIFY_EMAIL` / `UF_LIVE_VERIFY_PHONE` / `UF_LIVE_VERIFY_PASSWORD` | Operator subject |
| `UF_LIVE_VERIFY_SKIP_EMAIL=1` | Harness-only: auto-verify email token (no send/paste); SMS still interactive |
| `UF_LEPTON_LIVE_REVEAL_PII=1` | Print full operator email/phone in the console (masked by default) |

Do not commit filled env files. Load secrets in your shell only.

SMS OTP is a **6-digit** code (Valence still verifies). Prefer Verify for OTP-only
accounts that have not finished A2P 10DLC. If SMS fails, the CLI prints Twilio HTTP
status and numeric `code` (not the recipient).

**Auth smoke test (Standard API keys):** do **not** GET `/Accounts/{SID}.json` —
Twilio returns `20003` for Standard keys on that resource even when the key is
valid. With `UF_TWILIO_*` already exported in your shell, probe Messages instead:

```bash
curl -sS -o /dev/null -w '%{http_code}\n' \
  -u "$UF_TWILIO_API_KEY:$UF_TWILIO_API_SECRET" \
  "https://api.twilio.com/2010-04-01/Accounts/$UF_TWILIO_ACCOUNT_SID/Messages.json?PageSize=1"
```

Expect `200`, then run `lepton-live-verify`.

## Library entry points

| Task | API |
|------|-----|
| Boot Valence | [`boot_valence`](src/boot.rs) |
| Valence + services (+ Boson) | [`boot_lab`](src/boot.rs) (default `boson-delivery`) |
| Test services (sync only) | [`boot_services_test`](src/boot.rs) |
| Live Twilio lab | [`boot_lab_twilio`](src/boot.rs) (`live-twilio`) |
| Signup → confirm | [`run_signup_verify_flow`](src/flow.rs) |
| Device + TOTP challenge | [`run_device_totp_challenge_flow`](src/flow.rs) |
| OAuth signup → login | [`run_oauth_signup_login_flow`](src/oauth_flow.rs) |
| Mock OAuth codes | [`MockOAuthCodeSource`](src/oauth_flow.rs) |
| Live Google callback | [`LocalhostOAuthCodeSource`](src/oauth_callback.rs) |
| Test TOTP codes | [`TestTotpCodeSource`](src/flow.rs) |
| Live authenticator codes | [`StdinTotpCodeSource`](src/flow.rs) |
| Parse email paste | [`email_token_from_input`](src/parse.rs) |
| Parse otpauth secret | [`totp_secret_from_otpauth_uri`](src/parse.rs) |
| Parse manual entry fields | [`totp_manual_entry_from_otpauth_uri`](src/parse.rs) |

## See also

- [`infra/mailpit/README.md`](../infra/mailpit/README.md)
- [`docs/VERIFICATION.md`](../docs/VERIFICATION.md)
