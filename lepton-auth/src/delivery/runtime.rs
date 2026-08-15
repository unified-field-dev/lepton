//! Process-global delivery adapters for Boson workers.

use std::sync::{Arc, RwLock};

use thiserror::Error;

#[cfg(feature = "phone")]
use lepton_sms::SmsDeliveryService;
#[cfg(feature = "email")]
use lepton_smtp::EmailDeliveryService;

static RUNTIME: RwLock<Option<DeliveryRuntime>> = RwLock::new(None);

/// Errors resolving the installed [`DeliveryRuntime`].
#[derive(Debug, Error)]
pub enum DeliveryRuntimeError {
    /// [`DeliveryRuntime::install`] was never called in this process.
    #[error("reason_class=runtime: DeliveryRuntime not installed")]
    NotInstalled,
    /// Requested channel adapter is missing from the installed runtime.
    #[error("reason_class=runtime: delivery channel not configured")]
    ChannelMissing,
    /// Runtime lock poisoned.
    #[error("reason_class=runtime: delivery runtime lock poisoned")]
    LockPoisoned,
}

/// Process-wide email/SMS adapters for durable delivery task handlers.
#[derive(Clone, Default)]
pub struct DeliveryRuntime {
    #[cfg(feature = "email")]
    email: Option<Arc<dyn EmailDeliveryService>>,
    #[cfg(feature = "phone")]
    sms: Option<Arc<dyn SmsDeliveryService>>,
}

impl DeliveryRuntime {
    /// Start a builder.
    #[must_use]
    pub fn builder() -> DeliveryRuntimeBuilder {
        DeliveryRuntimeBuilder::default()
    }

    /// Install (or replace) the process runtime at host boot / in tests.
    ///
    /// # Errors
    ///
    /// [`DeliveryRuntimeError::LockPoisoned`] when the lock is poisoned.
    pub fn install(runtime: Self) -> Result<(), DeliveryRuntimeError> {
        let mut guard = RUNTIME
            .write()
            .map_err(|_| DeliveryRuntimeError::LockPoisoned)?;
        *guard = Some(runtime);
        drop(guard);
        Ok(())
    }

    /// Clone the installed runtime.
    ///
    /// # Errors
    ///
    /// Not installed or lock poisoned.
    pub fn get() -> Result<Self, DeliveryRuntimeError> {
        let guard = RUNTIME
            .read()
            .map_err(|_| DeliveryRuntimeError::LockPoisoned)?;
        guard.clone().ok_or(DeliveryRuntimeError::NotInstalled)
    }

    /// Email adapter from this runtime.
    ///
    /// # Errors
    ///
    /// Email channel missing.
    #[cfg(feature = "email")]
    pub fn email(&self) -> Result<Arc<dyn EmailDeliveryService>, DeliveryRuntimeError> {
        self.email
            .clone()
            .ok_or(DeliveryRuntimeError::ChannelMissing)
    }

    /// SMS adapter from this runtime.
    ///
    /// # Errors
    ///
    /// SMS channel missing.
    #[cfg(feature = "phone")]
    pub fn sms(&self) -> Result<Arc<dyn SmsDeliveryService>, DeliveryRuntimeError> {
        self.sms.clone().ok_or(DeliveryRuntimeError::ChannelMissing)
    }
}

/// Builder for [`DeliveryRuntime`].
#[derive(Default)]
pub struct DeliveryRuntimeBuilder {
    #[cfg(feature = "email")]
    email: Option<Arc<dyn EmailDeliveryService>>,
    #[cfg(feature = "phone")]
    sms: Option<Arc<dyn SmsDeliveryService>>,
}

impl DeliveryRuntimeBuilder {
    /// Set the email adapter.
    #[cfg(feature = "email")]
    #[must_use]
    pub fn email(mut self, email: Arc<dyn EmailDeliveryService>) -> Self {
        self.email = Some(email);
        self
    }

    /// Set the SMS adapter.
    #[cfg(feature = "phone")]
    #[must_use]
    pub fn sms(mut self, sms: Arc<dyn SmsDeliveryService>) -> Self {
        self.sms = Some(sms);
        self
    }

    /// Build the runtime (does not install).
    #[must_use]
    pub fn build(self) -> DeliveryRuntime {
        DeliveryRuntime {
            #[cfg(feature = "email")]
            email: self.email,
            #[cfg(feature = "phone")]
            sms: self.sms,
        }
    }
}

#[cfg(all(test, feature = "ssr", feature = "email"))]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use lepton_smtp::EmailServiceBuilder;

    #[test]
    fn runtime_missing_channel_sad() {
        let rt = DeliveryRuntime::builder().build();
        assert!(matches!(
            rt.email(),
            Err(DeliveryRuntimeError::ChannelMissing)
        ));
    }

    #[test]
    fn runtime_email_happy() {
        let email = EmailServiceBuilder::new().noop().build().expect("noop");
        let rt = DeliveryRuntime::builder().email(email).build();
        assert!(rt.email().is_ok());
    }
}
