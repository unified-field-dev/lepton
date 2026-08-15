//! Build script.

// Build script: compiles `schemas/*_valence_schema.rs` into generated Valence models
// under `OUT_DIR`, included by `src/generated.rs`. Not part of the published API surface.
#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let schemas_dir = std::path::PathBuf::from("schemas");

    println!("cargo:rerun-if-changed=schemas/");

    valence_codegen::generate_models(&valence_codegen::CodegenConfig {
        schemas_dir,
        out_dir,
        file_suffix: "_valence_schema.rs",
        trait_file_suffix: "_valence_trait.rs",
    })?;
    Ok(())
}
