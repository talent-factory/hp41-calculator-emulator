//! PRGM mode step display — implemented in Plan 04-03.

use hp41_core::CalcState;

/// Format the current program step as "{step_num:03} {op_name}".
/// Fully implemented in Plan 04-03; this stub satisfies the module declaration.
pub fn format_step(state: &CalcState) -> String {
    format!("{:03} END", state.pc)
}
