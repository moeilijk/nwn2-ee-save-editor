//! Debugging / diagnostic test suite.
//!
//! These tests are diagnostics — most are `#[ignore]` and exist for inspecting
//! real save data. Run with:
//!   cargo test --test debugging -- --ignored --nocapture
//!
//! Add new diagnostics by creating `tests/debugging/<name>.rs` and adding a
//! `mod <name>;` line below.

#[path = "debugging/diagnostic_list_types.rs"]
mod diagnostic_list_types;

#[path = "debugging/dump_armor_meshes.rs"]
mod dump_armor_meshes;

#[path = "debugging/diagnose_item_models.rs"]
mod diagnose_item_models;

#[path = "debugging/diagnose_model_scale.rs"]
mod diagnose_model_scale;
