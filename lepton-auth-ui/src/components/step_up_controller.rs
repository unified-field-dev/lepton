//! Shared controller so any host action can open the step-up modal and resume
//! after factors are collected.

use leptos::prelude::*;

/// Which factors the step-up dialog must collect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepUpPolicy {
    /// Authenticator code only (default for control-plane / permissions).
    #[default]
    Totp,
    /// Current password and authenticator code.
    PasswordAndTotp,
}

/// Factors collected by the step-up dialog (never log or put in Spectra).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StepUpFactors {
    /// Authenticator (TOTP) code.
    pub totp_code: String,
    /// Current password when [`StepUpPolicy::PasswordAndTotp`].
    pub password: Option<String>,
}

/// Title, copy, and factor policy for one step-up challenge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepUpRequest {
    /// Dialog title (for example "Confirm it's you").
    pub title: String,
    /// Optional supporting sentence under the title.
    pub description: Option<String>,
    /// Which fields to show.
    pub policy: StepUpPolicy,
}

impl Default for StepUpRequest {
    fn default() -> Self {
        Self {
            title: "Confirm it's you".to_string(),
            description: None,
            policy: StepUpPolicy::Totp,
        }
    }
}

/// Reactive handle for opening the step-up dialog without a route change.
///
/// After the user continues, [`Self::request`]'s callback receives [`StepUpFactors`].
/// The dialog stays open until the host calls [`Self::complete_success`] or
/// [`Self::report_error`] (or the user cancels).
#[derive(Clone, Copy)]
pub struct StepUpController {
    open: RwSignal<bool>,
    request: RwSignal<StepUpRequest>,
    error: RwSignal<Option<String>>,
    /// One-shot callback for the active challenge (cleared on cancel / success).
    on_verified: RwSignal<Option<Callback<StepUpFactors>>>,
    submitting: RwSignal<bool>,
}

impl StepUpController {
    /// Create an unbound controller (not yet provided as context).
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: RwSignal::new(false),
            request: RwSignal::new(StepUpRequest::default()),
            error: RwSignal::new(None),
            on_verified: RwSignal::new(None),
            submitting: RwSignal::new(false),
        }
    }

    /// Whether the step-up dialog should be visible.
    #[must_use]
    pub const fn open(&self) -> RwSignal<bool> {
        self.open
    }

    /// Active request (title / description / policy).
    #[must_use]
    pub const fn request_signal(&self) -> RwSignal<StepUpRequest> {
        self.request
    }

    /// Host- or client-visible error message.
    #[must_use]
    pub const fn error(&self) -> RwSignal<Option<String>> {
        self.error
    }

    /// True while the host is finishing the sensitive action after Continue.
    #[must_use]
    pub const fn submitting(&self) -> RwSignal<bool> {
        self.submitting
    }

    /// Open the dialog for `request` and register `on_verified`.
    ///
    /// Replaces any pending challenge. Does not invoke the previous callback.
    pub fn request(&self, request: StepUpRequest, on_verified: Callback<StepUpFactors>) {
        tracing::debug!(
            operation = "lepton_auth_ui.step_up.request",
            policy = ?request.policy,
            "step-up requested"
        );
        self.error.set(None);
        self.submitting.set(false);
        self.request.set(request);
        self.on_verified.set(Some(on_verified));
        self.open.set(true);
    }

    /// Deliver factors to the host callback (dialog stays open).
    pub fn submit_factors(&self, factors: StepUpFactors) {
        self.error.set(None);
        self.submitting.set(true);
        if let Some(cb) = self.on_verified.get_untracked() {
            tracing::debug!(
                operation = "lepton_auth_ui.step_up.verified_client",
                policy = ?self.request.get_untracked().policy,
                "step-up factors handed to host"
            );
            cb.run(factors);
        } else {
            self.submitting.set(false);
        }
    }

    /// Close after the sensitive action succeeded.
    pub fn complete_success(&self) {
        self.submitting.set(false);
        self.error.set(None);
        self.on_verified.set(None);
        self.open.set(false);
    }

    /// Keep the dialog open and show `message` (wrong code, bad password, …).
    pub fn report_error(&self, message: impl Into<String>) {
        self.submitting.set(false);
        self.error.set(Some(message.into()));
    }

    /// Cancel without invoking the verified callback.
    pub fn cancel(&self) {
        tracing::debug!(
            operation = "lepton_auth_ui.step_up.cancel",
            "step-up cancelled"
        );
        self.submitting.set(false);
        self.error.set(None);
        self.on_verified.set(None);
        self.open.set(false);
    }

    /// Hide the dialog and clear pending state (same as cancel).
    pub fn close(&self) {
        self.cancel();
    }
}

impl Default for StepUpController {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide [`StepUpController`] for the current component subtree.
pub fn provide_step_up_controller() -> StepUpController {
    let controller = StepUpController::new();
    provide_context(controller);
    controller
}

/// Optional access to a provided [`StepUpController`].
#[must_use]
pub fn use_step_up_controller() -> Option<StepUpController> {
    use_context::<StepUpController>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_owner(f: impl FnOnce()) {
        Owner::new().with(f);
    }

    #[test]
    fn request_opens_and_cancel_clears_without_callback() {
        with_owner(|| {
            let ctrl = StepUpController::new();
            let called = RwSignal::new(false);
            ctrl.request(
                StepUpRequest {
                    title: "Confirm".into(),
                    description: Some("desc".into()),
                    policy: StepUpPolicy::Totp,
                },
                Callback::new(move |_| called.set(true)),
            );
            assert!(ctrl.open().get_untracked());
            assert_eq!(ctrl.request_signal().get_untracked().title, "Confirm");
            ctrl.cancel();
            assert!(!ctrl.open().get_untracked());
            assert!(ctrl.on_verified.get_untracked().is_none());
            assert!(!called.get_untracked());
        });
    }

    #[test]
    fn report_error_keeps_dialog_open() {
        with_owner(|| {
            let ctrl = StepUpController::new();
            ctrl.request(StepUpRequest::default(), Callback::new(move |_| {}));
            ctrl.submit_factors(StepUpFactors {
                totp_code: "000000".into(),
                password: None,
            });
            ctrl.report_error("Authenticator code is incorrect");
            assert!(ctrl.open().get_untracked());
            assert_eq!(
                ctrl.error().get_untracked().as_deref(),
                Some("Authenticator code is incorrect")
            );
            assert!(!ctrl.submitting().get_untracked());
        });
    }

    #[test]
    fn complete_success_closes() {
        with_owner(|| {
            let ctrl = StepUpController::new();
            ctrl.request(StepUpRequest::default(), Callback::new(move |_| {}));
            ctrl.complete_success();
            assert!(!ctrl.open().get_untracked());
            assert!(ctrl.error().get_untracked().is_none());
        });
    }

    #[test]
    fn second_request_replaces_pending() {
        with_owner(|| {
            let ctrl = StepUpController::new();
            let first = RwSignal::new(0);
            let second = RwSignal::new(0);
            ctrl.request(
                StepUpRequest {
                    title: "first".into(),
                    description: None,
                    policy: StepUpPolicy::Totp,
                },
                Callback::new(move |_| first.update(|n| *n += 1)),
            );
            ctrl.request(
                StepUpRequest {
                    title: "second".into(),
                    description: None,
                    policy: StepUpPolicy::PasswordAndTotp,
                },
                Callback::new(move |_| second.update(|n| *n += 1)),
            );
            assert_eq!(ctrl.request_signal().get_untracked().title, "second");
            ctrl.submit_factors(StepUpFactors {
                totp_code: "111111".into(),
                password: Some("x".into()),
            });
            assert_eq!(first.get_untracked(), 0);
            assert_eq!(second.get_untracked(), 1);
        });
    }
}
