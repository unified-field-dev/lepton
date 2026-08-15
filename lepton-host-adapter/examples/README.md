# lepton-host-adapter examples

Canonical teaching path for axum-login `Backend` wiring and
`session_snapshot_middleware` — SQLite `:memory:`, no external services.

Examples in this directory:

| Example | Role |
|---------|------|
| `axum_session_snapshot` | Login → session cookie → `SessionSnapshot` |

## 1. Session snapshot — `axum_session_snapshot`

When to use: wire an SSR host so authenticated requests expose
`Extension<higgs_identity::SessionSnapshot>` for higgs-host / higgs.

```bash
CARGO_BUILD_JOBS=1 cargo run -p lepton-host-adapter --example axum_session_snapshot --features ssr
```

The binary sets `VALENCE_OWNERSHIP_UNIFIED_FETCH=0` so SQLite can load the
session user (`Model::get` without the unified ownership RETURN query).

Success: stdout prints `axum_session_snapshot: OK — login → SessionSnapshot`
(exit code 0).

API path exercised (in order):

1. `Backend::new` — same `HiggsValenceFactory` Arc the host would put on `HiggsConfig`
2. `SessionManagerLayer` — cookie flags (`HttpOnly`, `SameSite`; `Secure=false` for this HTTP smoke)
3. `AuthManagerLayerBuilder` — axum-login over that session store
4. `session_snapshot_middleware` — copies the logged-in user into `SessionSnapshot`
5. Follow-up request with the session cookie — handler reads `Extension<SessionSnapshot>`

Look next at `axum_session_snapshot.rs`, then the cookie table in workspace
`SECURITY.md`. Auth UI and password/token helpers live under
`lepton-auth/examples/`.
