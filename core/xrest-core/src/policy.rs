//! Environment selection and the unsafe-environment guardrail.
//!
//! The core never sends to an unsafe environment unless the caller passes `confirmed = true`.
//! Platform UIs ask [`SendPolicy::requires_confirmation`] to decide whether to show their dialog,
//! and pass the user's answer back; the core re-checks on every send regardless.
//!
//! An unsafe environment can limit confirmation to requests that may change data
//! ([`ConfirmScope::Changes`]): `GET`, `HEAD` and `OPTIONS` then send without asking. The scope
//! is judged by the main request's method only; a preflight token request is part of
//! authenticating, not a change.

use crate::error::{Error, Result};
#[cfg(test)]
use crate::model::ConfirmScope;
use crate::model::{Environment, MAX_MIN_COUNTDOWN_SECONDS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendPolicy {
    pub environment: Option<String>,
    pub requires_confirmation: bool,
    /// The workspace's minimum countdown for this environment (0 = none). Only meaningful when
    /// `requires_confirmation`.
    pub min_countdown_seconds: u32,
}

impl SendPolicy {
    /// The policy for any request in `env`, ignoring the method: whether the environment is
    /// unsafe at all. Use [`SendPolicy::for_request`] for a specific send.
    pub fn for_environment(env: Option<&Environment>) -> Self {
        Self::build(env, true)
    }

    /// The policy for a `method` request in `env`.
    pub fn for_request(env: Option<&Environment>, method: &str) -> Self {
        let in_scope = env.is_none_or(|e| e.confirm.applies_to(method));
        Self::build(env, in_scope)
    }

    fn build(env: Option<&Environment>, in_scope: bool) -> Self {
        let requires_confirmation = in_scope && env.is_some_and(|e| e.is_unsafe);
        Self {
            environment: env.map(|e| e.name.clone()),
            requires_confirmation,
            min_countdown_seconds: if requires_confirmation {
                env.map_or(0, |e| {
                    e.min_countdown_seconds.min(MAX_MIN_COUNTDOWN_SECONDS)
                })
            } else {
                0
            },
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
            ..Default::default()
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
        assert!(!SendPolicy::for_request(None, "DELETE").requires_confirmation);
    }

    #[test]
    fn changes_scope_lets_read_only_methods_through() {
        let prod = Environment {
            confirm: ConfirmScope::Changes,
            ..env("PROD", true)
        };
        for m in ["GET", "get", " HEAD ", "OPTIONS"] {
            assert!(
                !SendPolicy::for_request(Some(&prod), m).requires_confirmation,
                "{m}"
            );
        }
        for m in ["POST", "PUT", "PATCH", "DELETE", "TRACE", "PURGE", ""] {
            assert!(
                SendPolicy::for_request(Some(&prod), m).requires_confirmation,
                "{m:?}"
            );
        }
        // The environment as a whole is still unsafe.
        assert!(SendPolicy::for_environment(Some(&prod)).requires_confirmation);
    }

    #[test]
    fn every_scope_confirms_reads_too() {
        let prod = env("PROD", true);
        assert!(SendPolicy::for_request(Some(&prod), "GET").requires_confirmation);
    }

    #[test]
    fn min_countdown_only_when_confirming_and_clamped() {
        let prod = Environment {
            confirm: ConfirmScope::Changes,
            min_countdown_seconds: 99,
            ..env("PROD", true)
        };
        assert_eq!(
            SendPolicy::for_request(Some(&prod), "POST").min_countdown_seconds,
            MAX_MIN_COUNTDOWN_SECONDS
        );
        assert_eq!(
            SendPolicy::for_request(Some(&prod), "GET").min_countdown_seconds,
            0
        );
        let dev = Environment {
            min_countdown_seconds: 5,
            ..env("DEV", false)
        };
        assert_eq!(
            SendPolicy::for_request(Some(&dev), "POST").min_countdown_seconds,
            0
        );
    }
}
