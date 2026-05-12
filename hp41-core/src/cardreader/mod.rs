//! Card Reader (Phase 19) — HP 82104A peripheral emulation.
//!
//! Provides UI-agnostic codecs and state helpers for the four MVP operations:
//! `WDTA`, `RDTA`, `WPRGM`, `RDPRGM`. Disk I/O is performed by the frontends
//! (hp41-cli, hp41-gui); core only encodes/decodes and stages requests via
//! `CalcState::pending_card_op`.
//!
//! Formats:
//! - Programs: V41/Free42 bare `.raw` byte stream — community-standard, byte-
//!   exact instruction sequence with a trailing END marker (D-19).
//! - Data: `.card.json` — our own format (no de-facto standard exists for
//!   emulator data cards) with a `format: "hp41-data-v1"` magic header.

pub mod data;
pub mod raw;

pub use data::{decode_data, encode_data, DataCard};
pub use raw::{decode_program, encode_program};

use crate::ops::Op;
use crate::state::CalcState;

/// Pending card I/O request staged by an `Op::Wdta`/`Op::Rdta`/`Op::Wprgm`/`Op::Rdprgm`
/// handler. The frontend drains this after each `dispatch()` and performs the disk I/O.
///
/// `name` is taken verbatim from the ALPHA register at the time the op runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardOpRequest {
    /// WPRGM "name" — write current program bytes to `<name>.raw`.
    WriteProgram { name: String },
    /// WDTA "name" — write current data registers to `<name>.card.json`.
    WriteData { name: String },
    /// RDPRGM "name" — read `<name>.raw` and insert ops per RDPRGM semantics.
    ReadProgram { name: String },
    /// RDTA "name" — read `<name>.card.json` and replace data registers.
    ReadData { name: String },
}

/// Insert decoded program ops per RDPRGM semantics (decision: 2026-05-12):
/// - If `state.program` is empty → replace (also resets `pc` to 0).
/// - Otherwise → insert immediately AFTER `state.pc` (caller may advance pc afterwards).
///
/// The frontend calls this after `decode_program()` succeeds.
pub fn insert_program_ops(state: &mut CalcState, ops: Vec<Op>) {
    if state.program.is_empty() {
        state.program = ops;
        state.pc = 0;
    } else {
        let insert_at = (state.pc + 1).min(state.program.len());
        for (i, op) in ops.into_iter().enumerate() {
            state.program.insert(insert_at + i, op);
        }
    }
}

/// Capture the data registers R00..R(SIZE-1) into a `DataCard`.
/// SIZE = `state.regs.len()` (the HP-41 SIZE allocation is reflected as the Vec length).
pub fn capture_data_card(state: &CalcState) -> DataCard {
    DataCard {
        format: data::FORMAT_TAG.to_string(),
        version: data::FORMAT_VERSION,
        registers: state.regs.clone(),
    }
}

/// Load a `DataCard` into `state.regs`. Resizes `state.regs` to match the card's SIZE
/// (HP-41 hardware reallocates SIZE to fit the card on RDTA).
pub fn load_data_card(state: &mut CalcState, card: DataCard) {
    state.regs = card.registers;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use crate::ops::Op;
    use crate::state::CalcState;

    #[test]
    fn insert_into_empty_program_replaces_and_resets_pc() {
        let mut state = CalcState::new();
        state.pc = 5; // stale pc; must be reset to 0 on replace
        let ops = vec![Op::Add, Op::Sub];
        insert_program_ops(&mut state, ops);
        assert_eq!(state.program, vec![Op::Add, Op::Sub]);
        assert_eq!(state.pc, 0);
    }

    #[test]
    fn insert_into_nonempty_inserts_after_pc() {
        let mut state = CalcState::new();
        state.program = vec![Op::Lbl("A".into()), Op::Add, Op::Rtn];
        state.pc = 1; // currently at the Add op
        let ops = vec![Op::Mul, Op::Sub];
        insert_program_ops(&mut state, ops);
        // Expected: LBL A, Add, Mul, Sub, Rtn
        assert_eq!(
            state.program,
            vec![Op::Lbl("A".into()), Op::Add, Op::Mul, Op::Sub, Op::Rtn]
        );
    }

    #[test]
    fn insert_at_program_end_appends() {
        let mut state = CalcState::new();
        state.program = vec![Op::Add];
        state.pc = 1; // at END marker (one past last op)
        insert_program_ops(&mut state, vec![Op::Sub]);
        assert_eq!(state.program, vec![Op::Add, Op::Sub]);
    }

    #[test]
    fn capture_then_load_round_trips_registers() {
        let mut state = CalcState::new();
        state.regs[0] = HpNum::from(42i32);
        state.regs[7] = HpNum::from(-3i32);
        let card = capture_data_card(&state);
        let mut state2 = CalcState::new();
        load_data_card(&mut state2, card);
        assert_eq!(state2.regs[0], HpNum::from(42i32));
        assert_eq!(state2.regs[7], HpNum::from(-3i32));
        assert_eq!(state2.regs.len(), 100);
    }

    #[test]
    fn load_data_card_resizes_regs() {
        let mut state = CalcState::new(); // 100 regs
        let small_card = DataCard {
            format: data::FORMAT_TAG.to_string(),
            version: data::FORMAT_VERSION,
            registers: vec![HpNum::from(1i32); 16],
        };
        load_data_card(&mut state, small_card);
        assert_eq!(state.regs.len(), 16);
    }
}
