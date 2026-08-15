//! Bounded authorization-code and access-token store.

use std::collections::HashMap;
use std::sync::Mutex;

use super::identity::identity_from_code;

/// Maximum pending codes / tokens before eviction of oldest inserts.
pub const MAX_STORE_ENTRIES: usize = 1_024;

#[derive(Clone, Debug)]
pub struct Identity {
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub provider: String,
}

#[derive(Debug, Default)]
pub struct CodeStore {
    /// code → identity
    codes: Mutex<HashMap<String, Identity>>,
    /// access_token → identity
    tokens: Mutex<HashMap<String, Identity>>,
    order: Mutex<Vec<StoreKey>>,
}

#[derive(Clone, Debug)]
enum StoreKey {
    Code(String),
    Token(String),
}

impl CodeStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Issue a new opaque code for `provider` / logical `code_hint` (browser `code` param).
    ///
    /// When `code_hint` is empty, a random-ish code is generated. Identity follows
    /// [`identity_from_code`] using the issued code string.
    pub fn issue_code(
        &self,
        provider: &str,
        code_hint: Option<&str>,
    ) -> Result<String, &'static str> {
        // Default `mock-code` matches prior app-route mock + Playwright fixtures.
        let code = match code_hint.map(str::trim).filter(|s| !s.is_empty()) {
            Some(c) => c.to_string(),
            None => "mock-code".to_string(),
        };
        let (subject, email, name) = identity_from_code(provider, &code)?;
        let identity = Identity {
            subject,
            email,
            name,
            provider: provider.to_string(),
        };
        self.insert_code(code.clone(), identity);
        Ok(code)
    }

    fn insert_code(&self, code: String, identity: Identity) {
        {
            let mut codes = self
                .codes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            codes.insert(code.clone(), identity);
        }
        self.track(StoreKey::Code(code));
        self.evict_if_needed();
    }

    /// Exchange authorization code for an opaque access token (single use for code).
    pub fn exchange_code(&self, code: &str) -> Result<String, &'static str> {
        let identity = {
            let mut codes = self
                .codes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            codes.remove(code).ok_or("unknown_code")?
        };
        let token = format!("tok-{}", next_nonce());
        {
            let mut tokens = self
                .tokens
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tokens.insert(token.clone(), identity);
        }
        self.track(StoreKey::Token(token.clone()));
        self.evict_if_needed();
        Ok(token)
    }

    /// Look up identity for a Bearer access token.
    pub fn identity_for_token(&self, token: &str) -> Option<Identity> {
        self.tokens
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(token)
            .cloned()
    }

    fn track(&self, key: StoreKey) {
        self.order
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(key);
    }

    fn evict_if_needed(&self) {
        let mut order = self
            .order
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while order.len() > MAX_STORE_ENTRIES {
            match order.remove(0) {
                StoreKey::Code(c) => {
                    self.codes
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .remove(&c);
                }
                StoreKey::Token(t) => {
                    self.tokens
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .remove(&t);
                }
            }
        }
    }

    /// Total codes + tokens (for tests).
    #[must_use]
    pub fn len(&self) -> usize {
        let codes = self
            .codes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let tokens = self
            .tokens
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        codes + tokens
    }

    /// True when no codes or tokens are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn next_nonce() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_oidc_store_exchange_happy() {
        let store = CodeStore::new();
        let code = store
            .issue_code("google", Some("mock-code"))
            .expect("issue");
        let token = store.exchange_code(&code).expect("exchange");
        let id = store.identity_for_token(&token).expect("token");
        assert_eq!(id.subject, "mock:google:mock-code");
        assert_eq!(id.email.as_deref(), Some("mock-code@oauth.mock.test"));
        assert!(store.exchange_code(&code).is_err(), "replay");
    }

    #[test]
    fn mock_oidc_store_evicts_or_rejects_at_cap_sad() {
        let store = CodeStore::new();
        for i in 0..(MAX_STORE_ENTRIES + 10) {
            let _ = store.issue_code("google", Some(&format!("c{i}")));
        }
        assert!(store.len() <= MAX_STORE_ENTRIES);
    }
}
