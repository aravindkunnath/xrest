use async_trait::async_trait;
use thiserror::Error;
use crate::resolvers::{
    AwsResolver, AzureResolver, EnvFileResolver, GcpResolver, KeychainResolver, SystemEnvResolver,
};

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("Variable not found: {0}")]
    NotFound(String),
    #[error("Resolution error: {0}")]
    Error(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Trait for resolving variable values based on different strategies.
#[async_trait]
pub trait VariableResolver: Send + Sync {
    /// Attempts to resolve a key to a value.
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError>;
}

/// Enum-based dispatch for different resolution strategies.
pub enum ResolverStrategy {
    EnvFile(EnvFileResolver),
    SystemEnv(SystemEnvResolver),
    Keychain(KeychainResolver),
    Gcp(GcpResolver),
    Aws(AwsResolver),
    Azure(AzureResolver),
}

impl ResolverStrategy {
    pub async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        match self {
            Self::EnvFile(r) => r.resolve(content).await,
            Self::SystemEnv(r) => r.resolve(content).await,
            Self::Keychain(r) => r.resolve(content).await,
            Self::Gcp(r) => r.resolve(content).await,
            Self::Aws(r) => r.resolve(content).await,
            Self::Azure(r) => r.resolve(content).await,
        }
    }
}

/// Orchestrates the resolution of variables using a chain of prioritized strategies.
pub struct Resolver {
    strategies: Vec<ResolverStrategy>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { strategies: Vec::new() }
    }

    pub fn add_strategy(&mut self, strategy: ResolverStrategy) {
        self.strategies.push(strategy);
    }

    /// Resolves a variable by trying each strategy in order.
    pub async fn resolve_variable(&self, variable: &crate::variable::Variable) -> Result<String, ResolveError> {
        let content = match variable.get_template_content() {
            Some(c) => c,
            None => return Ok(variable.raw_value.clone()), // Return literal if not a template
        };

        for strategy in &self.strategies {
            if let Some(resolved) = strategy.resolve(content).await? {
                return Ok(resolved);
            }
        }

        Err(ResolveError::NotFound(content.to_string()))
    }
}
