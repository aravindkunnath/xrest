//! Preparing requests for sending. The resolution logic lives in [`crate::resolve`]; this only
//! hands it the workspace's state.

use super::Workspace;
use crate::codegen::{self, CodeLanguage};
use crate::error::Result;
use crate::exec::{PreparedPreflightTest, PreparedSend};
use crate::model::{Preflight, RequestSpec};
use crate::preflight::PreflightTestResult;
use crate::resolve::{self, ExportContext, SendContext};

impl Workspace {
    pub fn prepare_send_endpoint(&self, id: &str, confirmed: bool) -> Result<PreparedSend> {
        self.prepare_send(&self.endpoint(id)?.spec(), confirmed)
    }

    pub fn prepare_send_scratch(&self, id: &str, confirmed: bool) -> Result<PreparedSend> {
        self.prepare_send(&self.scratch(id)?.spec, confirmed)
    }

    /// Resolve `spec` against the selected environment and service auth.
    ///
    /// Enforces the unsafe-environment policy: returns
    /// [`Error::ConfirmationRequired`](crate::Error::ConfirmationRequired) unless `confirmed` is
    /// true. Nothing touches the network until [`PreparedSend::execute`].
    pub fn prepare_send(&self, spec: &RequestSpec, confirmed: bool) -> Result<PreparedSend> {
        let env = self.selected_environment();
        let layers = self.variable_layers(env);
        resolve::prepare_send(
            SendContext {
                root: &self.root,
                service: &self.service,
                environment: env,
                layers: &layers,
                secrets: self.secrets.as_ref(),
                tokens: self.tokens.clone(),
                http: &self.http,
            },
            spec,
            confirmed,
        )
    }

    /// Resolve `preflight` (or the service preflight when `None`) for a test run, enforcing the
    /// unsafe-environment policy like [`Workspace::prepare_send`]. No I/O happens here.
    pub fn prepare_test_preflight(
        &self,
        preflight: Option<Preflight>,
        confirmed: bool,
    ) -> Result<PreparedPreflightTest> {
        let env = self.selected_environment();
        let layers = self.variable_layers(env);
        resolve::prepare_test_preflight(
            SendContext {
                root: &self.root,
                service: &self.service,
                environment: env,
                layers: &layers,
                secrets: self.secrets.as_ref(),
                tokens: self.tokens.clone(),
                http: &self.http,
            },
            preflight,
            confirmed,
        )
    }

    /// Run a preflight request once and report the exchange and extracted token. Never reads or
    /// writes the token cache. A token-extraction failure is a result with `error` set.
    pub async fn test_preflight(
        &self,
        preflight: Option<Preflight>,
        confirmed: bool,
    ) -> Result<PreflightTestResult> {
        self.prepare_test_preflight(preflight, confirmed)?
            .execute()
            .await
    }

    /// Render `spec` as code for the selected environment: variables, path and query params,
    /// service auth and options applied. Does no I/O (no preflight request, no file reads; file
    /// fields keep their path), ignores the unsafe-environment policy, and writes the
    /// preflight header as `<preflight token>`. Secrets stay `{{secret.KEY}}` placeholders
    /// unless `include_secrets` is true, in which case a missing secret is
    /// [`Error::MissingSecrets`](crate::Error::MissingSecrets).
    pub fn export_code(
        &self,
        spec: &RequestSpec,
        language: CodeLanguage,
        include_secrets: bool,
    ) -> Result<String> {
        let env = self.selected_environment();
        let layers = self.variable_layers(env);
        let req = resolve::resolve_for_export(
            ExportContext {
                service: &self.service,
                environment: env,
                layers: &layers,
                secrets: self.secrets.as_ref(),
            },
            spec,
            include_secrets,
        )?;
        Ok(codegen::generate(&req, language))
    }
}
