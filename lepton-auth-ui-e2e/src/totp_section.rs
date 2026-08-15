//! E2e harness for TOTP enroll (same server fns / testids as product UI).

use lepton_auth::actions::totp::{
    begin_totp_enroll_ui, confirm_totp_enroll_ui, disable_totp_ui, get_totp_settings_status,
    regenerate_totp_recovery_codes_ui, PendingTotpEnrollView,
};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use orbital_base_components::{input_event_value, SkeletonItemSize};
use orbital_core_components::{Body1, Caption1, Subtitle1};
use orbital_primitives::{
    Badge, BadgeAppearance, Button, ButtonAppearance, Card, CardContent, CardHeader, Checkbox,
    CheckboxSize, Code, Field, Flex, FlexAlign, FlexGap, FlexWrap, MessageBar, MessageBarIntent,
    Skeleton, SkeletonItem,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TotpUiStep {
    Idle,
    Scan,
    Confirm,
    Recovery,
    Enabled,
    Disable,
    RegenConfirm,
}

/// Authenticator section for the auth-ui e2e host (Orbital card layout).
#[component]
#[allow(clippy::too_many_lines)] // mirrors product TotpSection for e2e
pub fn E2eTotpSection() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let status = Resource::new(
        move || refresh.get(),
        |_| async move { get_totp_settings_status().await },
    );
    let step = RwSignal::new(TotpUiStep::Idle);
    let pending = RwSignal::new(Option::<PendingTotpEnrollView>::None);
    let recovery_codes = RwSignal::new(Vec::<String>::new());
    let code_input = RwSignal::new(String::new());
    let recovery_ack = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);
    let bump = move || refresh.update(|n| *n = n.wrapping_add(1));

    Effect::new(move |_| {
        if let Some(Ok(s)) = status.get() {
            match step.get_untracked() {
                TotpUiStep::Idle | TotpUiStep::Enabled => {
                    step.set(if s.totp_enabled {
                        TotpUiStep::Enabled
                    } else {
                        TotpUiStep::Idle
                    });
                }
                _ => {}
            }
        }
    });

    view! {
        <Card>
            <CardHeader>
                <Flex align=FlexAlign::Center gap=FlexGap::Small wrap=FlexWrap::Wrap>
                    <Subtitle1>"Authenticator app"</Subtitle1>
                    {move || match step.get() {
                        TotpUiStep::Enabled => view! {
                            <Badge appearance=BadgeAppearance::Filled>"enabled"</Badge>
                        }.into_any(),
                        TotpUiStep::Idle => view! {
                            <Badge appearance=BadgeAppearance::Outline>"not set up"</Badge>
                        }.into_any(),
                        _ => ().into_any(),
                    }}
                </Flex>
            </CardHeader>
            <CardContent>
                <div data-testid="totp-settings-section">
                    <Flex vertical=true gap=FlexGap::Medium>
                        <Show when=move || error.get().is_some()>
                            <MessageBar intent=MessageBarIntent::Error>
                                <div data-testid="totp-settings-error">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </MessageBar>
                        </Show>

                        <Suspense fallback=move || view! {
                            <Skeleton>
                                <SkeletonItem size=Signal::from(SkeletonItemSize::S48) />
                            </Skeleton>
                        }>
                            {move || {
                                let _ = status.get();
                                match step.get() {
                                    TotpUiStep::Idle => view! {
                                        <div data-testid="totp-settings-idle">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <Caption1>
                                                    "Add a time-based code from an authenticator app. You'll need it when you sign in, and for sensitive account actions."
                                                </Caption1>
                                                <div data-testid="totp-settings-setup">
                                                    <Button
                                                        disabled=Signal::derive(move || busy.get())
                                                        on_click=Callback::new(move |_| {
                                                            busy.set(true);
                                                            error.set(None);
                                                            spawn_local_scoped(async move {
                                                                match begin_totp_enroll_ui().await {
                                                                    Ok(view) => {
                                                                        pending.set(Some(view));
                                                                        step.set(TotpUiStep::Scan);
                                                                    }
                                                                    Err(e) => error.set(Some(e.to_string())),
                                                                }
                                                                busy.set(false);
                                                            });
                                                        })
                                                    >
                                                        "Set up authenticator"
                                                    </Button>
                                                </div>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                    TotpUiStep::Scan => view! {
                                        <div data-testid="totp-settings-scan">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <Body1>"Scan this QR with your authenticator app."</Body1>
                                                {move || pending.get().map(|p| view! {
                                                    <div data-testid="totp-settings-qr" data-otpauth=p.otpauth_uri.clone() inner_html=p.qr_svg.clone()></div>
                                                    <Caption1>"Can't scan? Enter this key manually:"</Caption1>
                                                    <div data-testid="totp-settings-manual-secret">
                                                        <Code text=p.manual_secret />
                                                    </div>
                                                })}
                                                <Flex gap=FlexGap::Small>
                                                    <Button
                                                        appearance=ButtonAppearance::Secondary
                                                        on_click=Callback::new(move |_| {
                                                            pending.set(None);
                                                            step.set(TotpUiStep::Idle);
                                                        })
                                                    >
                                                        "Cancel"
                                                    </Button>
                                                    <div data-testid="totp-settings-continue">
                                                        <Button
                                                            on_click=Callback::new(move |_| {
                                                                code_input.set(String::new());
                                                                error.set(None);
                                                                step.set(TotpUiStep::Confirm);
                                                            })
                                                        >
                                                            "Continue"
                                                        </Button>
                                                    </div>
                                                </Flex>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                    TotpUiStep::Confirm => view! {
                                        <div data-testid="totp-settings-confirm">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <Body1>"Enter the 6-digit code from your app to finish setup."</Body1>
                                                <Field label="Code" required=true>
                                                    <input
                                                        type="text"
                                                        inputmode="numeric"
                                                        autocomplete="one-time-code"
                                                        data-testid="totp-settings-code"
                                                        prop:value=move || code_input.get()
                                                        on:input=move |ev| {
                                                            if let Some(v) = input_event_value(&ev) {
                                                                code_input.set(v);
                                                            }
                                                        }
                                                    />
                                                </Field>
                                                <Flex gap=FlexGap::Small>
                                                    <Button
                                                        appearance=ButtonAppearance::Secondary
                                                        on_click=Callback::new(move |_| step.set(TotpUiStep::Scan))
                                                    >
                                                        "Back"
                                                    </Button>
                                                    <div data-testid="totp-settings-confirm-submit">
                                                        <Button
                                                            disabled=Signal::derive(move || busy.get())
                                                            on_click=Callback::new(move |_| {
                                                                let Some(p) = pending.get() else {
                                                                    error.set(Some("Setup expired.".into()));
                                                                    step.set(TotpUiStep::Idle);
                                                                    return;
                                                                };
                                                                let factor_id = p.factor_id;
                                                                let code = code_input.get();
                                                                busy.set(true);
                                                                error.set(None);
                                                                spawn_local_scoped(async move {
                                                                    match confirm_totp_enroll_ui(factor_id, code).await {
                                                                        Ok(codes) => {
                                                                            recovery_codes.set(codes);
                                                                            recovery_ack.set(false);
                                                                            pending.set(None);
                                                                            step.set(TotpUiStep::Recovery);
                                                                        }
                                                                        Err(e) => error.set(Some(e.to_string())),
                                                                    }
                                                                    busy.set(false);
                                                                });
                                                            })
                                                        >
                                                            "Confirm"
                                                        </Button>
                                                    </div>
                                                </Flex>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                    TotpUiStep::Recovery => view! {
                                        <div data-testid="totp-settings-recovery">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <MessageBar intent=MessageBarIntent::Warning>
                                                    "Save these recovery codes now. We only show them once."
                                                </MessageBar>
                                                <div data-testid="totp-settings-recovery-list">
                                                    {move || {
                                                        let text = recovery_codes.get().join("\n");
                                                        view! { <Code text=text /> }
                                                    }}
                                                </div>
                                                <div data-testid="totp-settings-recovery-ack">
                                                    <Checkbox
                                                        checked=recovery_ack
                                                        label="I saved these codes".to_string()
                                                        size=Signal::from(CheckboxSize::Medium)
                                                    />
                                                </div>
                                                <div data-testid="totp-settings-recovery-done">
                                                    <Button
                                                        disabled=Signal::derive(move || !recovery_ack.get())
                                                        on_click=Callback::new(move |_| {
                                                            recovery_codes.set(Vec::new());
                                                            recovery_ack.set(false);
                                                            step.set(TotpUiStep::Enabled);
                                                            bump();
                                                        })
                                                    >
                                                        "Done"
                                                    </Button>
                                                </div>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                    TotpUiStep::Enabled => view! {
                                        <div data-testid="totp-settings-enabled">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <Caption1>
                                                    "Sign-in and sensitive actions can ask for a code from your app."
                                                </Caption1>
                                                <Flex gap=FlexGap::Small wrap=FlexWrap::Wrap>
                                                    <div data-testid="totp-settings-regen-start">
                                                        <Button
                                                            appearance=ButtonAppearance::Secondary
                                                            on_click=Callback::new(move |_| {
                                                                code_input.set(String::new());
                                                                error.set(None);
                                                                step.set(TotpUiStep::RegenConfirm);
                                                            })
                                                        >
                                                            "Get new recovery codes"
                                                        </Button>
                                                    </div>
                                                    <div data-testid="totp-settings-disable-start">
                                                        <Button
                                                            appearance=ButtonAppearance::Secondary
                                                            on_click=Callback::new(move |_| {
                                                                code_input.set(String::new());
                                                                error.set(None);
                                                                step.set(TotpUiStep::Disable);
                                                            })
                                                        >
                                                            "Disable authenticator"
                                                        </Button>
                                                    </div>
                                                </Flex>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                    TotpUiStep::Disable => view! {
                                        <div data-testid="totp-settings-disable">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <Body1>"Disable authenticator? You can set it up again later."</Body1>
                                                <Field label="Current code" required=true>
                                                    <input
                                                        type="text"
                                                        inputmode="numeric"
                                                        autocomplete="one-time-code"
                                                        data-testid="totp-settings-disable-code"
                                                        prop:value=move || code_input.get()
                                                        on:input=move |ev| {
                                                            if let Some(v) = input_event_value(&ev) {
                                                                code_input.set(v);
                                                            }
                                                        }
                                                    />
                                                </Field>
                                                <Flex gap=FlexGap::Small>
                                                    <Button
                                                        appearance=ButtonAppearance::Secondary
                                                        on_click=Callback::new(move |_| step.set(TotpUiStep::Enabled))
                                                    >
                                                        "Keep enabled"
                                                    </Button>
                                                    <div data-testid="totp-settings-disable-submit">
                                                        <Button
                                                            disabled=Signal::derive(move || busy.get())
                                                            on_click=Callback::new(move |_| {
                                                                let code = code_input.get();
                                                                busy.set(true);
                                                                error.set(None);
                                                                spawn_local_scoped(async move {
                                                                    match disable_totp_ui(code).await {
                                                                        Ok(()) => {
                                                                            step.set(TotpUiStep::Idle);
                                                                            bump();
                                                                        }
                                                                        Err(e) => error.set(Some(e.to_string())),
                                                                    }
                                                                    busy.set(false);
                                                                });
                                                            })
                                                        >
                                                            "Disable"
                                                        </Button>
                                                    </div>
                                                </Flex>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                    TotpUiStep::RegenConfirm => view! {
                                        <div data-testid="totp-settings-regen">
                                            <Flex vertical=true gap=FlexGap::Medium>
                                                <Field label="Current code" required=true>
                                                    <input
                                                        type="text"
                                                        inputmode="numeric"
                                                        autocomplete="one-time-code"
                                                        data-testid="totp-settings-regen-code"
                                                        prop:value=move || code_input.get()
                                                        on:input=move |ev| {
                                                            if let Some(v) = input_event_value(&ev) {
                                                                code_input.set(v);
                                                            }
                                                        }
                                                    />
                                                </Field>
                                                <Flex gap=FlexGap::Small>
                                                    <Button
                                                        appearance=ButtonAppearance::Secondary
                                                        on_click=Callback::new(move |_| step.set(TotpUiStep::Enabled))
                                                    >
                                                        "Cancel"
                                                    </Button>
                                                    <div data-testid="totp-settings-regen-submit">
                                                        <Button
                                                            disabled=Signal::derive(move || busy.get())
                                                            on_click=Callback::new(move |_| {
                                                                let code = code_input.get();
                                                                busy.set(true);
                                                                error.set(None);
                                                                spawn_local_scoped(async move {
                                                                    match regenerate_totp_recovery_codes_ui(code).await {
                                                                        Ok(codes) => {
                                                                            recovery_codes.set(codes);
                                                                            recovery_ack.set(false);
                                                                            step.set(TotpUiStep::Recovery);
                                                                        }
                                                                        Err(e) => error.set(Some(e.to_string())),
                                                                    }
                                                                    busy.set(false);
                                                                });
                                                            })
                                                        >
                                                            "Generate"
                                                        </Button>
                                                    </div>
                                                </Flex>
                                            </Flex>
                                        </div>
                                    }.into_any(),
                                }
                            }}
                        </Suspense>
                    </Flex>
                </div>
            </CardContent>
        </Card>
    }
}
