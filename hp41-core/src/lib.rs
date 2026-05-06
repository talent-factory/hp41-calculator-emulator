//! hp41-core — HP-41 calculator behavioral emulation library.
//!
//! Zero UI/CLI dependencies. All state is in [`state::CalcState`].

pub mod error;
pub mod num;
pub mod state;
pub mod stack;
pub mod ops;

// Convenience re-exports for consumers
pub use error::HpError;
pub use num::HpNum;
pub use state::{CalcState, Stack, AngleMode, DisplayMode};
pub use stack::LiftEffect;

#[cfg(test)]
mod tests;
