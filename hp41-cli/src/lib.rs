//! Public library surface for hp41-cli — exists so that integration tests
//! under `tests/` can drive the App event loop and call key-mapping helpers.
//! The CLI itself runs via `main.rs`; this lib.rs re-exposes the same modules
//! `main.rs` declares so integration tests can import them.

pub mod app;
pub mod cards;
pub mod help_data;
pub mod keys;
pub mod persistence;
pub mod prgm_display;
pub mod programs;
pub mod ui;
