# xrest-resolve

A flexible and extensible variable resolution library for Rust, designed to handle secrets and environment configurations from multiple sources.

## Features

- **Multi-source Resolution**: Resolve variables from `.env` files, system environment variables, and OS keychains.
- **Priority-based Chain**: Order your resolution strategies to control which source takes precedence.
- **Secure Secret Handing**: Native integration with OS-level secure storage (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux).
- **Template Support**: Detects and resolves variables within `{{ template }}` strings.
- **Extensible Architecture**: Easily add new resolution backends (like AWS Secrets Manager or GCP Secret Manager).
- **Test-friendly**: Includes built-in mock backends and traits for isolated unit testing.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xrest-resolve = { path = "../xrest-resolve" } # Or the appropriate path/version
```

## Usage

```rust
use xrest_resolve::{Resolver, ResolverStrategy, Variable, EnvFileResolver, SystemEnvResolver, KeychainResolver};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let mut resolver = Resolver::new();

    // 1. Add Env File Strategy (Highest priority)
    let mut env_map = HashMap::new();
    env_map.insert("API_URL".to_string(), "https://api.example.com".to_string());
    resolver.add_strategy(ResolverStrategy::EnvFile(EnvFileResolver::new(env_map)));

    // 2. Add System Env Strategy
    resolver.add_strategy(ResolverStrategy::SystemEnv(SystemEnvResolver));

    // 3. Add Keychain Strategy (Resolves keys with 'secret:' prefix)
    resolver.add_strategy(ResolverStrategy::Keychain(KeychainResolver::default()));

    // Resolve a variable
    let var = Variable::new("DB_PASS".into(), "{{ secret:database_password }}".into());
    let value = resolver.resolve_variable(&var).await.unwrap();
    
    println!("Resolved value: {}", value);
}
```

## Resolution Strategies

### 1. EnvFile (`EnvFileResolver`)
Resolves variables from a provided `HashMap<String, String>`. Typically used for `.env` files or local overrides.

### 2. System Environment (`SystemEnvResolver`)
Resolves variables using standard `std::env::var`. Supports both raw names and `{{ VAR }}` templates.

### 3. OS Keychain (`KeychainResolver`)
Interacts with the native OS secure storage. It looks for keys prefixed with `secret:`.
- **macOS**: Apple Keychain
- **Windows**: Credential Manager
- **Linux**: Secret Service (libsecret)

## Extensibility

You can implement your own resolver by implementing the `VariableResolver` trait:

```rust
#[async_trait]
pub trait VariableResolver: Send + Sync {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError>;
}
```

Then add it to the `ResolverStrategy` enum and update the dispatch logic in `resolver.rs`.

## Testing

For testing, you can use the `MockKeychainBackend` to avoid dependencies on real OS hardware:

```rust
let mut mock_data = HashMap::new();
mock_data.insert("test_key".into(), "secret_val".into());
let backend = Arc::new(MockKeychainBackend::new(mock_data));
let resolver = KeychainResolver::new(backend);
```

## License

MIT / Apache-2.0
