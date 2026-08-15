//! Shared Orbital page chrome for the Playwright host.

use lepton_auth_ui::{AuthDialog, AuthDialogCallbacks, AuthDialogKind};
use leptos::prelude::*;
use orbital_primitives::{
    AppBar, AppBarLeading, AppBarMaterial, AppBarPosition, AppBarTrailing, Button,
    ButtonAppearance, Flex, FlexAlign, FlexGap, FlexWrap, Layout, LayoutHeader, LayoutMain,
    MaterialCorners, MaterialElevation, MaterialVariant, SpacingInset, Title3,
};

/// Layout + AppBar + centered content column used by fixture pages.
#[component]
pub fn E2ePageShell(
    #[prop(optional)] data_testid: Option<&'static str>,
    #[prop(optional, default = true)] show_chrome: bool,
    children: Children,
) -> impl IntoView {
    let test_id = data_testid.unwrap_or("e2e-page");
    view! {
        <Layout overlay_header=true page_scrollport=true>
            <LayoutHeader slot>
                <AppBar position=AppBarPosition::Sticky>
                    <AppBarMaterial
                        slot
                        variant=MaterialVariant::Frost
                        elevation=MaterialElevation::Flat
                        corners=MaterialCorners::Square
                    />
                    <AppBarLeading slot>
                        <Title3>"lepton-auth-ui"</Title3>
                    </AppBarLeading>
                    <AppBarTrailing slot>
                        <Show when=move || show_chrome>
                            <ShellChrome/>
                        </Show>
                    </AppBarTrailing>
                </AppBar>
            </LayoutHeader>
            <LayoutMain slot>
                <div
                    data-testid=test_id
                    style="width: min(900px, 100%); margin-inline: auto; box-sizing: border-box;"
                >
                    <Flex
                        vertical=true
                        gap=FlexGap::Large
                        full_width=true
                        padding=SpacingInset::all_l()
                    >
                        {children()}
                    </Flex>
                </div>
            </LayoutMain>
        </Layout>
    }
}

/// App-bar account / sign-in / sign-up controls plus shared auth dialog.
#[component]
pub fn ShellChrome() -> impl IntoView {
    let open = RwSignal::new(false);
    let kind = RwSignal::new(AuthDialogKind::Signin);
    let referer = Signal::derive(|| "/welcome".to_string());

    view! {
        <div data-testid="shell-chrome">
            <Flex gap=FlexGap::Small align=FlexAlign::Center wrap=FlexWrap::Wrap>
                // Orbital Button: wrap for data-testid (do not put attr:data-testid on Button).
                <div data-testid="user-avatar">
                    <Button
                        appearance=ButtonAppearance::Transparent
                        on_click=Callback::new(move |_| open.set(true))
                    >
                        "Account"
                    </Button>
                </div>
                <div data-testid="user-menu-signin">
                    <Button
                        appearance=ButtonAppearance::Secondary
                        on_click=Callback::new(move |_| {
                            kind.set(AuthDialogKind::Signin);
                            open.set(true);
                        })
                    >
                        "Sign in"
                    </Button>
                </div>
                <div data-testid="user-menu-signup">
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=Callback::new(move |_| {
                            kind.set(AuthDialogKind::Signup);
                            open.set(true);
                        })
                    >
                        "Sign up"
                    </Button>
                </div>
            </Flex>
        </div>
        <AuthDialog
            open=open.into()
            kind=kind.into()
            referer=referer
            callbacks=AuthDialogCallbacks {
                on_close: Some(Callback::new(move |()| open.set(false))),
                on_switch_signin: Some(Callback::new(move |()| kind.set(AuthDialogKind::Signin))),
                on_switch_signup: Some(Callback::new(move |()| kind.set(AuthDialogKind::Signup))),
                on_success: Some(Callback::new(move |()| open.set(false))),
            }
        />
    }
}
