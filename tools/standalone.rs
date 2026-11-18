// Standalone crate root for `cargo test`.
// Compiles the pure Rust core without game dependencies.

#[path = "../core/mod.rs"]
pub mod fpvdrone_core;

pub use fpvdrone_core::*;
