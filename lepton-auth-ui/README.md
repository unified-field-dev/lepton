# lepton-auth-ui

Leptos auth form components built on Orbital. Depends on [`lepton-auth`](../lepton-auth/) for `#[server]` actions, paths, and password-policy helpers.

## Features

| Feature | Role |
|---------|------|
| `ssr` | SSR of auth UI (includes `SignupContent`) |
| `hydrate` | Client hydration |
| `oauth-google` | Continue with Google and the “or” divider |
| `oauth-github` | Continue with GitHub and the “or” divider |

## Public components

`AuthDialog`, `AuthModalShell`, `SigninContent`, `SignupContent`, `LogoutContent`,
`OAuthProviderButtons`, `OAuthCallbackContent`, `PasswordResetDialog`,
`PasswordResetRequestContent`, `PasswordResetConfirmContent`,
`provide_step_up_controller`, `StepUpDialog`, `StepUpController`, `StepUpPolicy`,
`ConfirmAccountPrompt`, `ConfirmAccountPage`.

Signup collects **legal name** and **display name** separately. With
`oauth-google` / `oauth-github` enabled on SSR and hydrate, sign-in and sign-up
show Continue with Google / GitHub (same-window redirect via `BeginOAuth` /
`CompleteOAuthCallback`). Without those features, the OAuth block is omitted.

Mount, routes, and step-up examples: `cargo doc -p lepton-auth-ui --open`.

Library browser e2e and leptos-lints (`cargo dylint` on hydrate): see workspace
`lepton-auth-ui-e2e` / [`docs/VERIFICATION.md`](../docs/VERIFICATION.md).
