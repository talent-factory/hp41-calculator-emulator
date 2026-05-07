//! Key → Op mapping table — implemented in Plan 04-03.

use crossterm::event::KeyEvent;
use hp41_core::ops::Op;
use crate::app::App;

/// Map a crossterm KeyEvent to an hp41-core Op. Returns None for unmapped keys.
/// Fully implemented in Plan 04-03; this stub satisfies the module declaration.
pub fn key_to_op(_key: KeyEvent, _app: &App) -> Option<Op> {
    None
}

/// Key-reference table shown in the right panel of the TUI.
/// Populated in Plan 04-03.
pub const KEY_REF_TABLE: &[(&str, &str)] = &[];
