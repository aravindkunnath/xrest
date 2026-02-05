pub mod variable;
pub mod resolver;
pub mod resolvers;

pub use variable::Variable;
pub use resolver::{Resolver, VariableResolver, ResolveError};
pub use resolvers::{
    AwsResolver, AzureResolver, EnvFileResolver, GcpResolver, KeychainResolver, SystemEnvResolver,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_resolver_order() -> Result<(), ResolveError> {
        let mut resolver = Resolver::new();
        
        // 1. Env File Resolver
        let mut env_vars = HashMap::new();
        env_vars.insert("LOCAL_VAR".to_string(), "local_value".to_string());
        resolver.add_resolver(Box::new(EnvFileResolver::new(env_vars)));

        // 2. System Env Resolver
        unsafe {
            std::env::set_var("SYS_VAR", "sys_value");
        }
        resolver.add_resolver(Box::new(SystemEnvResolver));

        // 3. Keychain
        resolver.add_resolver(Box::new(KeychainResolver));

        // 4. GCP
        resolver.add_resolver(Box::new(GcpResolver));

        // Test Local Env
        let v1 = Variable::new("K1".into(), "{{ LOCAL_VAR }}".into());
        assert_eq!(resolver.resolve_variable(&v1).await?, "local_value");

        // Test System Env
        let v2 = Variable::new("K2".into(), "{{ SYS_VAR }}".into());
        assert_eq!(resolver.resolve_variable(&v2).await?, "sys_value");

        // Test Keychain
        let v3 = Variable::new("K3".into(), "{{ secret:my-api-key }}".into());
        assert_eq!(resolver.resolve_variable(&v3).await?, "mock-keychain-value-for-my-api-key");

        // Test GCP
        let v4 = Variable::new("K4".into(), "{{ gcp:projects/xyz/secrets/abc }}".into());
        assert_eq!(resolver.resolve_variable(&v4).await?, "mock-gcp-value-for-projects/xyz/secrets/abc");

        Ok(())
    }

    #[tokio::test]
    async fn test_resolver_order_env_vs_system() -> Result<(), ResolveError> {
        let mut resolver = Resolver::new();
        
        // Mock a variable that exists in both .env and system env
        let key = "CONFLICT_VAR";
        let env_value = "from_env_file";
        let system_value = "from_system";

        // 1. Add Env File Resolver FIRST
        let mut env_vars = HashMap::new();
        env_vars.insert(key.to_string(), env_value.to_string());
        resolver.add_resolver(Box::new(EnvFileResolver::new(env_vars)));

        // 2. Add System Env Resolver SECOND
        unsafe {
            std::env::set_var(key, system_value);
        }
        resolver.add_resolver(Box::new(SystemEnvResolver));

        let v = Variable::new("K".into(), format!("{{{{ {} }}}}", key));
        
        // Should return the value from .env file because it's first in the chain
        assert_eq!(resolver.resolve_variable(&v).await?, env_value);

        Ok(())
    }
}
