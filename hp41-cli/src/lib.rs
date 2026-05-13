//! Public library surface for hp41-cli — exists so that integration tests
//! under `tests/` can import the cards module. The CLI itself runs via
//! `main.rs`; this lib.rs exposes nothing that `main.rs` doesn't already
//! declare.

pub mod cards;
