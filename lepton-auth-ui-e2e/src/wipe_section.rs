//! E2e harness for account wipe (same server fn / testids as the host wipe UI).

use lepton_auth::account_api::WIPE_CONFIRM_PHRASE;
use lepton_auth::actions::account::WipeAccount;
use leptos::prelude::*;
use orbital_core_components::{Body1, Caption1, Subtitle1};
use orbital_primitives::{
    Button, ButtonAppearance, ButtonType, Card, CardContent, CardHeader, Field, Flex, FlexGap,
    Input, InputAppearance, InputBind, InputType, MessageBar, MessageBarIntent,
};

/// Wipe form for the auth-ui e2e host (Orbital card layout).
#[component]
#[allow(clippy::too_many_lines)] // phrase / password / totp fields in one card
pub fn E2eWipeSection() -> impl IntoView {
    let wipe_action = ServerAction::<WipeAccount>::new();
    let confirm_phrase = RwSignal::new(String::new());
    let current_password = RwSignal::new(String::new());
    let totp_code = RwSignal::new(String::new());

    view! {
        <Card>
            <CardHeader>
                <Subtitle1>"Delete account"</Subtitle1>
            </CardHeader>
            <CardContent>
                <div data-testid="account-wipe-section">
                    <Flex vertical=true gap=FlexGap::Medium>
                        <Body1>
                            "This permanently deletes your account, emails, personas, and sign-in data. It cannot be undone."
                        </Body1>
                        <Caption1>
                            "Type " {WIPE_CONFIRM_PHRASE} " to confirm. Enter your current password. If you use an authenticator app, enter a code too."
                        </Caption1>
                        <Show when=move || matches!(wipe_action.value().get(), Some(Err(_)))>
                            <div data-testid="account-wipe-error">
                                <MessageBar intent=MessageBarIntent::Error>
                                    {move || {
                                        wipe_action
                                            .value()
                                            .get()
                                            .and_then(Result::err)
                                            .map_or_else(
                                                || "Unable to delete the account right now.".to_string(),
                                                |e| e.to_string(),
                                            )
                                    }}
                                </MessageBar>
                            </div>
                        </Show>
                        <ActionForm action=wipe_action>
                            <div data-testid="account-wipe-form">
                                <Flex vertical=true gap=FlexGap::Medium>
                                    <Field label="Type DELETE to confirm" required=true>
                                        <div data-testid="account-wipe-confirm-phrase">
                                            <Input
                                                bind={
                                                    let mut bind = InputBind::new(confirm_phrase);
                                                    bind.name = "confirm_phrase".into();
                                                    bind
                                                }
                                                appearance=InputAppearance {
                                                    input_type: Signal::from(InputType::Text),
                                                    ..Default::default()
                                                }
                                            />
                                        </div>
                                    </Field>
                                    <Field label="Current password" required=true>
                                        <div data-testid="account-wipe-current-password">
                                            <Input
                                                bind={
                                                    let mut bind = InputBind::new(current_password);
                                                    bind.name = "current_password".into();
                                                    bind
                                                }
                                                appearance=InputAppearance {
                                                    input_type: Signal::from(InputType::Password),
                                                    ..Default::default()
                                                }
                                            />
                                        </div>
                                    </Field>
                                    <Field label="Authenticator code (if enabled)">
                                        <div data-testid="account-wipe-totp">
                                            <Input
                                                bind={
                                                    let mut bind = InputBind::new(totp_code);
                                                    bind.name = "totp_code".into();
                                                    bind
                                                }
                                                appearance=InputAppearance {
                                                    input_type: Signal::from(InputType::Text),
                                                    ..Default::default()
                                                }
                                            />
                                        </div>
                                    </Field>
                                    <Flex gap=FlexGap::Small>
                                        <div data-testid="account-wipe-submit">
                                            <Button
                                                button_type=ButtonType::Submit
                                                appearance=ButtonAppearance::Primary
                                                disabled=wipe_action.pending()
                                            >
                                                {move || {
                                                    if wipe_action.pending().get() {
                                                        "Deleting…"
                                                    } else {
                                                        "Delete account"
                                                    }
                                                }}
                                            </Button>
                                        </div>
                                    </Flex>
                                </Flex>
                            </div>
                        </ActionForm>
                    </Flex>
                </div>
            </CardContent>
        </Card>
    }
}
