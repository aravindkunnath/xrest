use std::sync::Arc;
use xrest_resolve::{GcpResolver, RealGcpBackend, Resolver, ResolverStrategy, Variable};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing GCP Resolver...");

    // 1. Initialize the real GCP backend
    let backend = Arc::new(RealGcpBackend::new().await);
    let gcp_resolver = GcpResolver::new(backend);

    // 2. Setup the orchestrator
    let mut resolver = Resolver::new();
    resolver.add_strategy(ResolverStrategy::Gcp(gcp_resolver));

    // 3. Define the variable with the GCP path
    // Note: Secret Manager usually requires /versions/latest or a specific version number
    let secret_path = "projects/989702536491/secrets/pubsub_topic/versions/latest";
    let var = Variable::new(
        "GCP_SECRET".into(),
        format!("{{{{ gcp:{} }}}}", secret_path),
    );

    println!("Attempting to resolve: {}", secret_path);

    // 4. Resolve
    match resolver.resolve_variable(&var).await {
        Ok(value) => {
            println!("✅ Success! Resolved value: {}", value);
        }
        Err(e) => {
            eprintln!("❌ Failed to resolve GCP secret.");
            eprintln!("Error: {}", e);
            eprintln!("\nMake sure you have active Application Default Credentials:");
            eprintln!("  gcloud auth application-default login");
        }
    }

    Ok(())
}
