use crate::core::traits::SecretStore;
use keyring::Entry;

pub struct KeyringSecretStore;

impl SecretStore for KeyringSecretStore {
    fn get(&self, key: &str) -> Result<String, String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
        entry
            .get_password()
            .map_err(|e| format!("Failed to get secret from keyring: {}", e))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
        entry
            .set_password(value)
            .map_err(|e| format!("Failed to set secret in keyring: {}", e))
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
        let _ = entry.delete_credential();
        Ok(())
    }
}
