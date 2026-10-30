//! Secret lookup for `{{secret.KEY}}` placeholders and variables with a `secretKey`.
//!
//! The core only defines the trait. Platform apps are expected to supply an implementation backed
//! by the OS credential store (macOS Keychain, libsecret on Linux). `xrest-ffi` exposes a
//! foreign-implementable `SecretProvider` callback interface and adapts it to [`SecretStore`];
//! no credential-store code lives in Rust. [`MemorySecretStore`] backs tests.

use std::collections::HashMap;
use std::sync::Mutex;

pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;

    /// Whether a value exists, without fetching it. Previews use this so that typing in the UI
    /// never reads secret material from the credential store.
    fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

/// Process-lifetime secret store; also used by tests.
#[derive(Default)]
pub struct MemorySecretStore {
    data: Mutex<HashMap<String, String>>,
}

impl MemorySecretStore {
    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) {
        self.data
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(key.into(), value.into());
    }

    pub fn remove(&self, key: &str) {
        self.data
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(key);
    }
}

impl SecretStore for MemorySecretStore {
    fn get(&self, key: &str) -> Option<String> {
        self.data
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(key)
            .cloned()
    }

    fn contains(&self, key: &str) -> bool {
        self.data
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(key)
    }
}
