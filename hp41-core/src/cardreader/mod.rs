//! Card Reader — HP 82104A peripheral emulation.
//!
//! Provides UI-agnostic codecs and state helpers for the four MVP operations:
//! `WDTA`, `RDTA`, `WPRGM`, `RDPRGM`. Disk I/O is performed by the frontends
//! (hp41-cli, hp41-gui); core only encodes/decodes and stages requests via
//! `CalcState::pending_card_op`.
//!
//! Formats:
//! - Programs: bare `.raw` byte stream — community-standard instruction
//!   sequence with a trailing END marker.
//! - Data: `.card.json` — our own format (no de-facto standard exists for
//!   emulator data cards) with a `format: "hp41-data-v1"` magic header.

pub mod data;
pub mod raw;

pub use data::{decode_data, encode_data, DataCard};
pub use raw::{decode_program, encode_program};

use crate::num::HpNum;
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

/// Insert decoded program ops per RDPRGM semantics:
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

/// Minimum register vector length maintained after `load_data_card`. The rest
/// of the engine gates `STO`/`RCL` on `reg < 100` (not `reg < regs.len()`), so
/// shrinking below 100 would turn `STO 50` after a small-card load into a
/// runtime index panic.
const MIN_REGS_AFTER_LOAD: usize = 100;

/// Load a `DataCard` into `state.regs`.
///
/// Replaces the live registers with the card's payload, then zero-pads up to
/// `MIN_REGS_AFTER_LOAD` so subsequent `STO`/`RCL nn` operations stay in
/// bounds. The HP-41 hardware grows SIZE to fit a larger card; we additionally
/// guarantee it never shrinks below 100 to keep the rest of the engine sound.
pub fn load_data_card(state: &mut CalcState, card: DataCard) {
    state.regs = card.registers;
    if state.regs.len() < MIN_REGS_AFTER_LOAD {
        state.regs.resize(MIN_REGS_AFTER_LOAD, HpNum::zero());
    }
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
    fn load_data_card_pads_small_card_to_min_size() {
        let mut state = CalcState::new(); // 100 regs
        let small_card = DataCard {
            format: data::FORMAT_TAG.to_string(),
            version: data::FORMAT_VERSION,
            registers: vec![HpNum::from(1i32); 16],
        };
        load_data_card(&mut state, small_card);
        assert_eq!(
            state.regs.len(),
            MIN_REGS_AFTER_LOAD,
            "load_data_card must keep regs.len() >= MIN_REGS_AFTER_LOAD so STO/RCL nn stays in bounds"
        );
        assert_eq!(state.regs[0], HpNum::from(1i32));
        assert_eq!(state.regs[15], HpNum::from(1i32));
        assert_eq!(
            state.regs[50],
            HpNum::zero(),
            "padded slots must be zero, not garbage"
        );
    }

    #[test]
    fn load_data_card_keeps_oversize_card_intact() {
        // A card with > MIN_REGS_AFTER_LOAD regs must not be truncated.
        let mut state = CalcState::new();
        let big_card = DataCard {
            format: data::FORMAT_TAG.to_string(),
            version: data::FORMAT_VERSION,
            registers: vec![HpNum::from(7i32); 150],
        };
        load_data_card(&mut state, big_card);
        assert_eq!(state.regs.len(), 150);
        assert_eq!(state.regs[149], HpNum::from(7i32));
    }

    #[test]
    fn sto_after_small_card_load_does_not_panic() {
        // Regression guard for the review-flagged out-of-bounds panic:
        // load_data_card with a 16-register card used to leave regs.len() == 16,
        // and a subsequent op_sto on register 50 would index-panic on raw access
        // (op_sto gates on reg < 100, not on reg < state.regs.len()).
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(42i32);
        let small_card = DataCard {
            format: data::FORMAT_TAG.to_string(),
            version: data::FORMAT_VERSION,
            registers: vec![HpNum::zero(); 16],
        };
        load_data_card(&mut state, small_card);
        crate::ops::registers::op_sto(&mut state, 50).expect("op_sto must succeed after card load");
        assert_eq!(state.regs[50], HpNum::from(42i32));
    }
}
