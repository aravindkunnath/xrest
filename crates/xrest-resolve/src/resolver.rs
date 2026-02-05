use async_trait::async_trait;
use thiserror::Error;

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

/// Orchestrates the resolution of variables using a chain of resolvers.
pub struct Resolver {
    pub(crate) resolvers: Vec<Box<dyn VariableResolver>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { resolvers: Vec::new() }
    }

    pub fn add_resolver(&mut self, resolver: Box<dyn VariableResolver>) {
        self.resolvers.push(resolver);
    }

    /// Resolves a variable by trying each resolver in order.
    pub async fn resolve_variable(&self, variable: &crate::variable::Variable) -> Result<String, ResolveError> {
        let content = match variable.get_template_content() {
            Some(c) => c,
            None => return Ok(variable.raw_value.clone()), // Return literal if not a template
        };

        for resolver in &self.resolvers {
            if let Some(resolved) = resolver.resolve(content).await? {
                return Ok(resolved);
            }
        }

        Err(ResolveError::NotFound(content.to_string()))
    }
}
