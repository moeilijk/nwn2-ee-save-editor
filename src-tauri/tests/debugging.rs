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
