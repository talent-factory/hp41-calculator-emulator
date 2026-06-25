use crate::ResolveError;
use hp41_core::ops::{backspace_entry, dispatch};
use hp41_core::{CalcState, HpError};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Identifies whether a key mutated frontend-style entry/mode state directly
/// or dispatched a real calculator operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    StateOnly,
    Operation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DispatchError {
    Resolve(ResolveError),
    Core(HpError),
}

impl Display for DispatchError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve(error) => formatter.write_str(&error.message),
            Self::Core(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for DispatchError {}

impl From<ResolveError> for DispatchError {
    fn from(error: ResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl From<HpError> for DispatchError {
    fn from(error: HpError) -> Self {
        Self::Core(error)
    }
}

/// Apply one canonical GUI key identifier to calculator state.
///
/// This is the shared, framework-independent equivalent of the former Tauri
/// `handle_op_prepare` input/dispatch section. Card I/O is deliberately left to
/// the caller because native and Tauri shells use different storage adapters.
pub fn dispatch_key(state: &mut CalcState, key_id: &str) -> Result<DispatchOutcome, DispatchError> {
    // Any key exits the live clock display, matching the CLI and Tauri shells.
    state.clock_active = false;

    // Native transport aliases belong at the framework-independent boundary so
    // every adapter observes identical ALPHA and contextual-delete behavior.
    let normalized_key;
    let key_id = if key_id == "backspace" {
        if state.alpha_mode {
            "alpha_backspace"
        } else {
            "entry_backspace"
        }
    } else if let Some(value) = key_id.strip_prefix("alpha:") {
        normalized_key = format!("alpha_{value}");
        &normalized_key
    } else {
        key_id
    };

    if key_id == "sw_exit" {
        state.stopwatch_keyboard_mode = false;
        return Ok(DispatchOutcome::StateOnly);
    }

    if key_id.len() == 1 && key_id.as_bytes()[0].is_ascii_digit() {
        if let Some(e_pos) = state.entry_buf.find('e') {
            let exponent_digits = state.entry_buf[e_pos + 1..]
                .chars()
                .filter(char::is_ascii_digit)
                .count();
            if exponent_digits >= 2 {
                return Ok(DispatchOutcome::StateOnly);
            }
        }
        state.entry_buf.push(char::from(key_id.as_bytes()[0]));
        return Ok(DispatchOutcome::StateOnly);
    }

    if key_id == "." {
        if !state.entry_buf.contains('.') && !state.entry_buf.contains('e') {
            if state.entry_buf.is_empty() {
                state.entry_buf.push_str("0.");
            } else {
                state.entry_buf.push('.');
            }
        }
        return Ok(DispatchOutcome::StateOnly);
    }

    if key_id == "e" {
        if !state.entry_buf.contains('e') {
            if state.entry_buf.is_empty() {
                state.entry_buf.push_str("1e");
            } else {
                state.entry_buf.push('e');
            }
        }
        return Ok(DispatchOutcome::StateOnly);
    }

    if key_id == "eex_chs" {
        if let Some(e_pos) = state.entry_buf.find('e') {
            if state.entry_buf[e_pos + 1..].starts_with('-') {
                state.entry_buf.remove(e_pos + 1);
            } else {
                state.entry_buf.insert(e_pos + 1, '-');
            }
        }
        return Ok(DispatchOutcome::StateOnly);
    }

    if key_id == "entry_backspace" {
        backspace_entry(state);
        return Ok(DispatchOutcome::StateOnly);
    }

    let op = crate::resolve(key_id)?;
    dispatch(state, op)?;
    Ok(DispatchOutcome::Operation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponent_entry_is_capped_and_sign_toggles() {
        let mut state = CalcState::new();
        for key in ["1", "e", "2", "3", "4"] {
            dispatch_key(&mut state, key).unwrap();
        }
        assert_eq!(state.entry_buf, "1e23");

        dispatch_key(&mut state, "eex_chs").unwrap();
        assert_eq!(state.entry_buf, "1e-23");
        dispatch_key(&mut state, "eex_chs").unwrap();
        assert_eq!(state.entry_buf, "1e23");
    }

    #[test]
    fn decimal_and_backspace_follow_gui_contract() {
        let mut state = CalcState::new();
        for key in [".", "1", ".", "entry_backspace"] {
            dispatch_key(&mut state, key).unwrap();
        }
        assert_eq!(state.entry_buf, "0.");
    }

    #[test]
    fn native_alpha_and_contextual_backspace_aliases_are_shared() {
        let mut state = CalcState::new();
        for key in ["alpha:1", "alpha:0", "alpha:1"] {
            dispatch_key(&mut state, key).unwrap();
        }
        assert_eq!(state.alpha_reg, "101");
        state.alpha_mode = true;
        dispatch_key(&mut state, "backspace").unwrap();
        assert_eq!(state.alpha_reg, "10");

        state.alpha_mode = false;
        state.entry_buf = "42".to_string();
        dispatch_key(&mut state, "backspace").unwrap();
        assert_eq!(state.entry_buf, "4");
    }

    #[test]
    fn dispatches_named_and_parameterized_operations() {
        let mut state = CalcState::new();
        for key in ["2", "chs", "abs", "fix_2"] {
            dispatch_key(&mut state, key).unwrap();
        }
        assert_eq!(
            hp41_core::format_hpnum(&state.stack.x, &state.display_mode),
            "2.00"
        );
    }

    #[test]
    fn every_key_exits_clock_and_stopwatch_exit_is_state_only() {
        let mut state = CalcState::new();
        state.clock_active = true;
        state.stopwatch_keyboard_mode = true;
        assert_eq!(
            dispatch_key(&mut state, "sw_exit").unwrap(),
            DispatchOutcome::StateOnly
        );
        assert!(!state.clock_active);
        assert!(!state.stopwatch_keyboard_mode);
    }

    #[test]
    fn preserves_resolver_and_core_error_kinds() {
        let mut state = CalcState::new();
        assert!(matches!(
            dispatch_key(&mut state, "not_a_key"),
            Err(DispatchError::Resolve(_))
        ));

        state.entry_buf = "0".to_string();
        dispatch_key(&mut state, "enter").unwrap();
        state.entry_buf = "0".to_string();
        assert!(matches!(
            dispatch_key(&mut state, "div"),
            Err(DispatchError::Core(HpError::DivideByZero))
        ));
    }
}
