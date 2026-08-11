//! Bake a source fingerprint for dogfood stale-build detection.

use std::path::PathBuf;

#[path = "src/source_fingerprint.rs"]
mod source_fingerprint;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    // crates/duckboard → repo root
    let root = manifest_dir
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.clone());

    let id = source_fingerprint::resolve_source_id(&root);
    println!("cargo:rustc-env=DUCKBOARD_SOURCE_ID={id}");
    // Fingerprint depends on VCS state, not only this file.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/source_fingerprint.rs");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");
}
