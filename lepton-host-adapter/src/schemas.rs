//! Runtime trait registration for token schemas.
//!
//! Model schemas are registered by codegen (`OUT_DIR/generated_models.rs`). Including
//! `*_valence_schema.rs` here would submit a second `SchemaMetadataInit` without merged
//! trait fields and overwrite the full schema in [`valence::SchemaRegistry`].

#[cfg(feature = "ssr")]
mod one_time_token_lifecycle_trait {
    include!("../schemas/one_time_token_lifecycle_valence_trait.rs");
}
