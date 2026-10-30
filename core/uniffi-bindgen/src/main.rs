//! Thin wrapper so the workspace pins the exact `uniffi-bindgen` version.
//! Run via `cargo run -p uniffi-bindgen -- generate ...` (see scripts/generate-bindings.sh).

fn main() {
    uniffi::uniffi_bindgen_main()
}
