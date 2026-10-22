//! Environment selection and the unsafe-environment guardrail.
//!
//! The core never sends to an unsafe environment unless the caller passes `confirmed = true`.
//! Platform UIs ask [`SendPolicy::requires_confirmation`] to decide whether to show their dialog,
//! and pass the user's answer back; the core re-checks on every send regardless.

use crate::error::{Error, Result};
use crate::model::Environment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendPolicy {
    pub environment: Option<String>,
    pub requires_confirmation: bool,
}

impl SendPolicy {
    pub fn for_environment(env: Option<&Environment>) -> Self {
        Self {
            environment: env.map(|e| e.name.clone()),
            requires_confirmation: env.is_some_and(|e| e.is_unsafe),
        }
    }

    /// `Ok(())` if a send may proceed given the caller's `confirmed` flag.
    pub fn check(&self, confirmed: bool) -> Result<()> {
        if self.requires_confirmation && !confirmed {
            return Err(Error::ConfirmationRequired {
                environment: self.environment.clone().unwrap_or_default(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str, is_unsafe: bool) -> Environment {
        Environment {
            name: name.into(),
            is_unsafe,
            variables: vec![],
        }
    }

    #[test]
    fn safe_env_needs_no_confirmation() {
        let p = SendPolicy::for_environment(Some(&env("DEV", false)));
        assert!(!p.requires_confirmation);
        assert!(p.check(false).is_ok());
    }

    #[test]
    fn unsafe_env_requires_confirmed() {
        let p = SendPolicy::for_environment(Some(&env("PROD", true)));
        assert!(p.requires_confirmation);
        assert!(matches!(
            p.check(false),
            Err(Error::ConfirmationRequired { environment }) if environment == "PROD"
        ));
        assert!(p.check(true).is_ok());
    }

    #[test]
    fn no_environment_is_safe() {
        assert!(!SendPolicy::for_environment(None).requires_confirmation);
    }
}
