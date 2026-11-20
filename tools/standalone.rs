// Standalone crate root for `cargo test`.
// Compiles the pure Rust core without game dependencies.

#[path = "../hmod/src/drone/mod.rs"]
pub mod fpvhorizon_core;

pub use fpvhorizon_core::*;
/// The core as the mod's crate names it (`crate::drone`), for the modules that use it.
pub use fpvhorizon_core as drone;

/// The setup screen's settings table and maths.
#[path = "../hmod/src/setup/model.rs"]
pub mod setup_model;
