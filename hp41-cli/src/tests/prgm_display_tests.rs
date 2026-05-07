//! Unit tests for prgm_display::format_step().

use hp41_core::{
    ops::{dispatch, Op},
    CalcState,
};

use crate::prgm_display::format_step;

#[test]
fn empty_program_shows_end() {
    let state = CalcState::new(); // pc=0, program=[]
    assert_eq!(format_step(&state), "000 END");
}

#[test]
fn program_step_at_add() {
    let mut state = CalcState::new();
    state.prgm_mode = true;
    dispatch(&mut state, Op::Add).unwrap();
    state.prgm_mode = false;
    state.pc = 0;
    assert_eq!(format_step(&state), "000 + ");
}

#[test]
fn program_step_at_sin() {
    let mut state = CalcState::new();
    state.prgm_mode = true;
    dispatch(&mut state, Op::Sin).unwrap();
    state.prgm_mode = false;
    state.pc = 0;
    assert_eq!(format_step(&state), "000 SIN");
}

#[test]
fn program_step_at_lbl() {
    let mut state = CalcState::new();
    state.prgm_mode = true;
    dispatch(&mut state, Op::Lbl("A".to_string())).unwrap();
    state.prgm_mode = false;
    state.pc = 0;
    assert_eq!(format_step(&state), "000 LBL A");
}

#[test]
fn step_number_zero_padded_three_digits() {
    let mut state = CalcState::new();
    state.prgm_mode = true;
    // Record 5 Add ops
    for _ in 0..5 {
        dispatch(&mut state, Op::Add).unwrap();
    }
    state.prgm_mode = false;
    state.pc = 4;
    assert_eq!(format_step(&state), "004 + ");
}

#[test]
fn pc_beyond_program_shows_end() {
    let mut state = CalcState::new();
    state.prgm_mode = true;
    dispatch(&mut state, Op::Add).unwrap();
    state.prgm_mode = false;
    state.pc = 99; // beyond the program
    assert_eq!(format_step(&state), "099 END");
}
