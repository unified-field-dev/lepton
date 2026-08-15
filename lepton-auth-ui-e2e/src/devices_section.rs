//! E2e harness for Security devices (same server fns / testids as product UI).

#[cfg(feature = "webauthn-ui")]
use lepton_auth::actions::devices::begin_passkey_registration;
#[cfg(all(feature = "webauthn-ui", feature = "hydrate"))]
use lepton_auth::actions::devices::finish_passkey_registration;
use lepton_auth::actions::devices::{
    confirm_trusted_browser, list_my_auth_devices, register_trusted_browser, revoke_my_auth_device,
};
use lepton_auth::devices::{AuthDeviceKind, AuthDeviceView};
#[cfg(all(feature = "hydrate", feature = "webauthn-ui"))]
use lepton_auth::webauthn_browser::credentials_create_json;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use orbital_base_components::SkeletonItemSize;
use orbital_core_components::{Body1, Caption1, Subtitle1};
use orbital_primitives::{
    Button, ButtonAppearance, Card, CardContent, CardHeader, Field, Flex, FlexAlign, FlexGap,
    FlexWrap, Input, InputAppearance, InputBind, MessageBar, MessageBarIntent, Skeleton,
    SkeletonItem,
};

const fn kind_label(kind: AuthDeviceKind) -> &'static str {
    match kind {
        AuthDeviceKind::TrustedBrowser => "Trusted browser",
        AuthDeviceKind::WebAuthn => "Passkey",
    }
}

const fn device_status(device: &AuthDeviceView) -> &'static str {
    if device.revoked_at.is_some() {
        "Revoked"
    } else if device.trusted_at.is_some() {
        "Trusted"
    } else {
        "Pending"
    }
}

/// Security devices section for the auth-ui e2e host (Orbital card layout).
#[component]
#[allow(clippy::too_many_lines)] // mirrors product DevicesSection for e2e
pub fn E2eDevicesSection() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let devices = Resource::new(
        move || refresh.get(),
        |_| async move { list_my_auth_devices().await },
    );
    let error = RwSignal::new(Option::<String>::None);
    let pending_confirm = RwSignal::new(Option::<(String, String)>::None);
    let browser_label = RwSignal::new("This browser".to_string());
    let confirm_code_input = RwSignal::new(String::new());
    #[cfg(feature = "webauthn-ui")]
    let passkey_label = RwSignal::new("Passkey".to_string());
    let busy = RwSignal::new(false);
    let bump_refresh = move || refresh.update(|n| *n = n.wrapping_add(1));

    view! {
        <Card>
            <CardHeader>
                <Subtitle1>"Security devices"</Subtitle1>
            </CardHeader>
            <CardContent>
                <div data-testid="devices-section">
                    <Flex vertical=true gap=FlexGap::Medium>
                        <Caption1>
                            "Remember this browser or add a passkey after you sign in. These devices are for trust on this account — they are not a sign-in method yet."
                        </Caption1>

                        <Show when=move || error.get().is_some()>
                            <MessageBar intent=MessageBarIntent::Error>
                                <div data-testid="devices-error">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </MessageBar>
                        </Show>

                        <Suspense fallback=move || view! {
                            <Skeleton>
                                <SkeletonItem size=Signal::from(SkeletonItemSize::S48) />
                            </Skeleton>
                        }>
                            {move || match devices.get() {
                                Some(Ok(list)) => {
                                    let active: Vec<_> = list
                                        .into_iter()
                                        .filter(|d| d.revoked_at.is_none())
                                        .collect();
                                    if active.is_empty() {
                                        view! {
                                            <div data-testid="devices-empty">
                                                <Body1>"No trusted devices yet."</Body1>
                                            </div>
                                        }
                                        .into_any()
                                    } else {
                                        view! {
                                            <div data-testid="devices-list">
                                                <Flex vertical=true gap=FlexGap::Small>
                                                    {active
                                                        .into_iter()
                                                        .map(|device| {
                                                            let id = device.id.clone();
                                                            let label = device.label.clone();
                                                            let kind = kind_label(device.kind);
                                                            let status = device_status(&device);
                                                            view! {
                                                                <div data-testid="devices-row">
                                                                    <Flex
                                                                        align=FlexAlign::Center
                                                                        gap=FlexGap::Small
                                                                        wrap=FlexWrap::Wrap
                                                                    >
                                                                        <Body1>
                                                                            {label}
                                                                            " · "
                                                                            {kind}
                                                                            " · "
                                                                            {status}
                                                                        </Body1>
                                                                        <div data-testid="devices-revoke">
                                                                            <Button
                                                                                appearance=ButtonAppearance::Secondary
                                                                                disabled=Signal::derive(move || busy.get())
                                                                                on_click=Callback::new({
                                                                                    let bump_refresh = bump_refresh;
                                                                                    move |_| {
                                                                                        let device_id = id.clone();
                                                                                        busy.set(true);
                                                                                        error.set(None);
                                                                                        spawn_local_scoped(async move {
                                                                                            match revoke_my_auth_device(device_id).await {
                                                                                                Ok(()) => bump_refresh(),
                                                                                                Err(e) => error.set(Some(e.to_string())),
                                                                                            }
                                                                                            busy.set(false);
                                                                                        });
                                                                                    }
                                                                                })
                                                                            >
                                                                                "Revoke"
                                                                            </Button>
                                                                        </div>
                                                                    </Flex>
                                                                </div>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </Flex>
                                            </div>
                                        }
                                        .into_any()
                                    }
                                }
                                Some(Err(err)) => view! {
                                    <MessageBar intent=MessageBarIntent::Error>
                                        <div data-testid="devices-error">
                                            {err.to_string()}
                                        </div>
                                    </MessageBar>
                                }
                                .into_any(),
                                None => ().into_any(),
                            }}
                        </Suspense>

                        <Flex vertical=true gap=FlexGap::Small>
                            <Field label="Browser label">
                                <div data-testid="devices-browser-label">
                                    <Input
                                        bind=InputBind::new(browser_label)
                                        appearance=InputAppearance::with_placeholder("This browser")
                                    />
                                </div>
                            </Field>
                            <div data-testid="devices-remember-browser">
                                <Button
                                    appearance=ButtonAppearance::Secondary
                                    disabled=Signal::derive(move || busy.get())
                                    on_click=Callback::new(move |_| {
                                        let label = browser_label.get();
                                        busy.set(true);
                                        error.set(None);
                                        spawn_local_scoped(async move {
                                            match register_trusted_browser(label).await {
                                                Ok(pending) => {
                                                    confirm_code_input.set(pending.confirm_code.clone());
                                                    pending_confirm.set(Some((
                                                        pending.device_id,
                                                        pending.confirm_code,
                                                    )));
                                                }
                                                Err(e) => error.set(Some(e.to_string())),
                                            }
                                            busy.set(false);
                                        });
                                    })
                                >
                                    "Remember this browser"
                                </Button>
                            </div>
                        </Flex>

                        <Show when=move || pending_confirm.get().is_some()>
                            <Flex vertical=true gap=FlexGap::Small>
                                <MessageBar intent=MessageBarIntent::Info>
                                    "Enter the confirm code to finish trusting this browser."
                                </MessageBar>
                                <Field label="Confirm code" required=true>
                                    <div data-testid="devices-confirm-code">
                                        <Input
                                            bind=InputBind::new(confirm_code_input)
                                            appearance=InputAppearance {
                                                autocomplete: MaybeProp::<String>::from(
                                                    "off".to_string(),
                                                ),
                                                ..Default::default()
                                            }
                                        />
                                    </div>
                                </Field>
                                <div data-testid="devices-confirm-browser">
                                    <Button
                                        disabled=Signal::derive(move || busy.get())
                                        on_click=Callback::new({
                                            let bump_refresh = bump_refresh;
                                            move |_| {
                                                let Some((device_id, _)) = pending_confirm.get() else {
                                                    return;
                                                };
                                                let code = confirm_code_input.get();
                                                busy.set(true);
                                                error.set(None);
                                                spawn_local_scoped(async move {
                                                    match confirm_trusted_browser(device_id, code).await {
                                                        Ok(()) => {
                                                            pending_confirm.set(None);
                                                            confirm_code_input.set(String::new());
                                                            bump_refresh();
                                                        }
                                                        Err(e) => error.set(Some(e.to_string())),
                                                    }
                                                    busy.set(false);
                                                });
                                            }
                                        })
                                    >
                                        "Confirm browser"
                                    </Button>
                                </div>
                            </Flex>
                        </Show>

                        {
                            #[cfg(feature = "webauthn-ui")]
                            {
                                view! {
                                    <Flex vertical=true gap=FlexGap::Small>
                                        <Field label="Passkey label">
                                            <div data-testid="devices-passkey-label">
                                                <Input
                                                    bind=InputBind::new(passkey_label)
                                                    appearance=InputAppearance::with_placeholder(
                                                        "Passkey",
                                                    )
                                                />
                                            </div>
                                        </Field>
                                        <div data-testid="devices-add-passkey">
                                            <Button
                                                disabled=Signal::derive(move || busy.get())
                                                on_click=Callback::new({
                                                    let bump_refresh = bump_refresh;
                                                    move |_| {
                                                        let label = passkey_label.get();
                                                        busy.set(true);
                                                        error.set(None);
                                                        spawn_local_scoped(async move {
                                                            match enroll_passkey(label).await {
                                                                Ok(()) => bump_refresh(),
                                                                Err(msg) => error.set(Some(msg)),
                                                            }
                                                            busy.set(false);
                                                        });
                                                    }
                                                })
                                            >
                                                "Add passkey"
                                            </Button>
                                        </div>
                                    </Flex>
                                }
                                .into_any()
                            }
                            #[cfg(not(feature = "webauthn-ui"))]
                            {
                                ().into_any()
                            }
                        }
                    </Flex>
                </div>
            </CardContent>
        </Card>
    }
}

#[cfg(feature = "webauthn-ui")]
async fn enroll_passkey(label: String) -> Result<(), String> {
    let pending = begin_passkey_registration(label)
        .await
        .map_err(|e| e.to_string())?;
    #[cfg(feature = "hydrate")]
    {
        let attestation = credentials_create_json(&pending.creation_options)
            .await
            .map_err(|e| e.to_string())?;
        finish_passkey_registration(pending.ceremony_id, attestation)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = pending;
        Err("Passkey enrollment requires a browser.".into())
    }
}
