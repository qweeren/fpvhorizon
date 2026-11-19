// Standalone crate root for `cargo test`.
// Compiles the pure Rust core without game dependencies.

#[path = "../hmod/src/drone/mod.rs"]
pub mod fpvhorizon_core;

pub use fpvhorizon_core::*;
