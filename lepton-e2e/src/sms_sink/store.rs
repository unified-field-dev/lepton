//! Bounded in-memory message store for the SMS HTTP capture sink.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Maximum JSON body size accepted on `POST /v1/messages` (8 KiB).
pub const MAX_BODY_BYTES: usize = 8 * 1024;

/// Maximum recorded messages before further POSTs are rejected.
pub const MAX_STORE_MESSAGES: usize = 1_000;

/// One captured SMS envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedSms {
    /// Destination E.164.
    pub to_e164: String,
    /// Message body.
    pub body: String,
    /// Optional discrete OTP code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub otp_code: Option<String>,
}

/// Thread-safe bounded store.
#[derive(Debug, Default)]
pub struct MessageStore {
    messages: Mutex<Vec<CapturedSms>>,
}

impl MessageStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a message.
    ///
    /// # Errors
    ///
    /// Returns `Err("store_full")` when at [`MAX_STORE_MESSAGES`].
    pub fn push(&self, msg: CapturedSms) -> Result<(), &'static str> {
        let mut guard = self
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.len() >= MAX_STORE_MESSAGES {
            return Err("store_full");
        }
        guard.push(msg);
        Ok(())
    }

    /// Snapshot of recorded messages (insert order).
    #[must_use]
    pub fn list(&self) -> Vec<CapturedSms> {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Clear all messages.
    pub fn clear(&self) {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    /// Current count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// True when empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sms_sink_store_push_list_clear_happy() {
        let store = MessageStore::new();
        store
            .push(CapturedSms {
                to_e164: "+15551234567".into(),
                body: "hi".into(),
                otp_code: Some("123456".into()),
            })
            .expect("push");
        assert_eq!(store.len(), 1);
        assert_eq!(store.list()[0].body, "hi");
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn sms_sink_store_cap_sad() {
        let store = MessageStore::new();
        for i in 0..MAX_STORE_MESSAGES {
            store
                .push(CapturedSms {
                    to_e164: "+15551234567".into(),
                    body: format!("m{i}"),
                    otp_code: None,
                })
                .expect("push");
        }
        let err = store
            .push(CapturedSms {
                to_e164: "+15551234567".into(),
                body: "overflow".into(),
                otp_code: None,
            })
            .expect_err("cap");
        assert_eq!(err, "store_full");
    }
}
