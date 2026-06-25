use hp41_core::{CalcState, HpError};

/// Advance the program counter by one, clamped to the END row.
pub fn sst_step(state: &mut CalcState) {
    if state.pc < state.program.len() {
        state.pc += 1;
    }
}

/// Move the program counter back by one, clamped at zero.
pub fn bst_step(state: &mut CalcState) {
    state.pc = state.pc.saturating_sub(1);
}

/// Toggle the low-level run/stop flag without entering the interpreter loop.
pub fn run_stop(state: &mut CalcState) {
    state.is_running = !state.is_running;
}

pub fn run_program(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    hp41_core::ops::program::run_program(state, label)
}

pub fn resume_program(state: &mut CalcState) -> Result<(), HpError> {
    hp41_core::ops::program::resume_program(state)
}

pub fn resume_program_with_key(state: &mut CalcState, keycode: u8) -> Result<(), HpError> {
    hp41_core::ops::program::resume_program_with_key(state, keycode)
}

pub fn submit_modal(state: &mut CalcState) -> Result<(), HpError> {
    hp41_core::ops::math1::submit_modal(state)
}

pub fn cancel_modal(state: &mut CalcState) {
    hp41_core::ops::math1::cancel_modal(state);
}

pub fn submit_modal_with_label(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    hp41_core::ops::math1::submit_modal_with_label(state, label)
}

/// Refresh time-module alarms before the shell projects a live state view.
pub fn tick_time(state: &mut CalcState) {
    hp41_core::ops::time::alarm::check_alarms(state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp41_core::ops::math1::modal::{ModalProgram, SolveInputStep};
    use hp41_core::ops::Op;
    use hp41_core::state::YieldKind;
    use hp41_core::HpNum;

    #[test]
    fn stepping_clamps_at_both_ends() {
        let mut state = CalcState::new();
        state.program = vec![Op::Enter];
        sst_step(&mut state);
        sst_step(&mut state);
        assert_eq!(state.pc, 1);
        bst_step(&mut state);
        bst_step(&mut state);
        assert_eq!(state.pc, 0);
    }

    #[test]
    fn run_stop_toggles_without_executing() {
        let mut state = CalcState::new();
        run_stop(&mut state);
        assert!(state.is_running);
        run_stop(&mut state);
        assert!(!state.is_running);
    }

    #[test]
    fn labeled_run_executes_to_completion() {
        let mut state = CalcState::new();
        state.program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum::from(2)),
            Op::Enter,
            Op::PushNum(HpNum::from(3)),
            Op::Add,
        ];
        run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum::from(5));
        assert!(!state.is_running);
    }

    #[test]
    fn getkey_yield_resumes_with_captured_code() {
        let mut state = CalcState::new();
        state.program = vec![
            Op::Lbl("A".to_string()),
            Op::GetKey,
            Op::PushNum(HpNum::from(1)),
            Op::Add,
        ];
        run_program(&mut state, "A").unwrap();
        assert_eq!(
            state
                .pending_yield
                .as_ref()
                .map(|yield_state| yield_state.kind.clone()),
            Some(YieldKind::WaitForKey)
        );

        resume_program_with_key(&mut state, 11).unwrap();
        assert_eq!(state.stack.x, HpNum::from(12));
        assert!(state.pending_yield.is_none());
    }

    #[test]
    fn modal_label_submit_and_cancel_share_core_semantics() {
        let mut state = CalcState::new();
        state.modal_program = Some(ModalProgram::Solve(SolveInputStep::FunctionNamePrompt));
        state.modal_prompt = Some("FUNCTION NAME?".to_string());
        submit_modal_with_label(&mut state, " test ").unwrap();
        assert_eq!(state.alpha_reg, "TEST");
        assert_eq!(
            state.modal_program,
            Some(ModalProgram::Solve(SolveInputStep::Guess1Prompt))
        );

        state.entry_buf = "42".to_string();
        cancel_modal(&mut state);
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
        assert!(state.entry_buf.is_empty());
    }

    #[test]
    fn invalid_program_and_modal_actions_preserve_error_kind() {
        let mut state = CalcState::new();
        assert_eq!(run_program(&mut state, "MISSING"), Err(HpError::InvalidOp));
        assert_eq!(resume_program(&mut state), Err(HpError::InvalidOp));
        assert_eq!(submit_modal(&mut state), Err(HpError::InvalidOp));
    }
}
