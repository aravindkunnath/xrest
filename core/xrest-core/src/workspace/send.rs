//! Preparing requests for sending. The resolution logic lives in [`crate::resolve`]; this only
//! hands it the workspace's state.

use super::Workspace;
use crate::error::Result;
use crate::exec::PreparedSend;
use crate::model::RequestSpec;
use crate::resolve::{self, SendContext};

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
        resolve::prepare_send(
            SendContext {
                root: &self.root,
                service: &self.service,
                environment: self.selected_environment(),
                secrets: self.secrets.as_ref(),
                tokens: self.tokens.clone(),
                http: &self.http,
            },
            spec,
            confirmed,
        )
    }
}
