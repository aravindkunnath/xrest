#!/usr/bin/env bash
set -e

echo "Running Local Guardrails..."

# 1. Secret Scanning (gitleaks)
if command -v gitleaks &> /dev/null; then
    echo "Running gitleaks..."
    gitleaks detect --source . -v
else
    echo "gitleaks not found. Skipping secret scan."
fi

# Navigate to rust project
cd src-tauri

# 2. SAST: clippy
echo "Running cargo clippy..."
cargo clippy --workspace -- -D warnings

# 3. SAST: audit
if command -v cargo-audit &> /dev/null; then
    echo "Running cargo audit..."
    cargo audit
else
    echo "cargo-audit not found. Run 'cargo install cargo-audit' to enable."
fi

# 4. Tests
echo "Running tests..."
cargo test --workspace

# 5. Coverage
if command -v cargo-tarpaulin &> /dev/null; then
    echo "Running cargo tarpaulin..."
    cargo tarpaulin --workspace --out Html
else
    echo "cargo-tarpaulin not found. Run 'cargo install cargo-tarpaulin' to enable."
fi

# 6. Mutation Testing
# if command -v cargo-mutants &> /dev/null; then
#     echo "Running cargo mutants on xrest-core..."
#     cargo mutants -d crates/xrest-core
# else
#     echo "cargo-mutants not found. Run 'cargo install cargo-mutants' to enable."
# fi

echo "All guardrails passed successfully!"
