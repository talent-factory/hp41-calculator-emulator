// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Modal state-machine layer for prompt-driven Math Pac I workflows.
//!
//! `ModalProgram` is the carrier enum: one variant per top-level Math Pac I program
//! that requires multi-step user input. Each variant wraps a per-program step enum.
//!
//! The `current_prompt()` accessor returns the OM-cited prompt text for the current
//! (program, step) pair. Exhaustive match only — `_ =>` is FORBIDDEN per FN-CLI-04
//! (inherited from Phase 25 `pending_prompt()` invariant).
//!
//! Lifecycle: set on modal-open, advanced on R/S submit (D-28.5), cleared on
//! completion or modal-cancel. `modal_prompt: Option<String>` on `CalcState`
//! carries the current prompt text; CLI renders it in `pending_prompt()` (Phase 29);
//! GUI renders it as an overlay banner (Phase 31).

/// Top-level Math Pac I modal program identifier.
///
/// Each variant wraps the per-program step state. Sub-enum variants are extended
/// by Plans 28-05..28-10 as the programs are implemented.
/// D-28.4 / XROM-09: prompts written to `CalcState::modal_prompt`, not `print_buffer`.
#[derive(Debug, Clone, PartialEq)]
pub enum ModalProgram {
    /// MATRIX workflow (Plan 28-06): matrix operations menu + element-by-element input.
    Matrix(MatrixInputStep),
    /// SOLVE workflow (Plan 28-08): root-finding via secant method.
    Solve(SolveInputStep),
    /// POLY workflow (Plan 28-05): polynomial root-finding by coefficient input.
    Poly(PolyInputStep),
    /// INTG workflow (Plan 28-07): numerical integration setup.
    Integ(IntegInputStep),
    /// DIFEQ workflow (Plan 28-09): differential equation solver setup.
    Difeq(DifeqInputStep),
    /// FOUR workflow (Plan 28-10): Fourier analysis setup.
    Four(FourInputStep),
    /// TRANS workflow (Plan 28-10): coordinate transform setup.
    Trans(TransInputStep),
}

impl ModalProgram {
    /// Returns the OM-cited prompt text for the current (program, step) pair.
    ///
    /// Returns `None` if the program is in a non-prompting state (e.g., computing
    /// or waiting for a non-modal input). Exhaustive match — no `_ =>` arm.
    /// FN-CLI-04: adding a new `ModalProgram` variant without updating this match
    /// is a compile error.
    ///
    /// Returns `Option<String>` (owned) because Matrix ElementPrompt and
    /// SimeqInputPrompt generate dynamic strings (e.g., "A1,1=?", "B3=?").
    /// Pre-allocating a 14×14 const table was rejected per Plan 28-06 Task 1 note (b).
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            ModalProgram::Matrix(step) => step.current_prompt(),
            ModalProgram::Solve(step) => step.current_prompt(),
            ModalProgram::Poly(step) => step.current_prompt(),
            ModalProgram::Integ(step) => step.current_prompt(),
            ModalProgram::Difeq(step) => step.current_prompt(),
            ModalProgram::Four(step) => step.current_prompt(),
            ModalProgram::Trans(step) => step.current_prompt(),
        }
    }
}

// ── MatrixInputStep ────────────────────────────────────────────────────────────

/// Per-step state for the MATRIX workflow (Plan 28-06).
///
/// Steps follow the HP-41C Math Pac I OM (HP 00041-90034, 1979) MATRIX program
/// prompting sequence, Chapter 3. Extended by Plan 28-06.
#[derive(Debug, Clone, PartialEq)]
pub enum MatrixInputStep {
    /// Awaiting matrix order (dimension n×n or m×n). Prompt: "ORDER=?"
    OrderPrompt,
    /// Awaiting matrix element A(i,j). Prompt: "Ai,j=?" (1-indexed).
    ElementPrompt(u8, u8),
    /// Matrix fully entered; ready for operation selection. Prompt: None (computing).
    Ready,
    /// Awaiting element value during matrix edit mode. Prompt: "A i,j=?"
    EditPrompt,
    /// SIMEQ: awaiting right-hand-side vector element b(i). Prompt: "bi=?"
    SimeqInputPrompt(u8),
    /// SIMEQ: system solved; solution in stack. Prompt: None (done).
    SimeqDone,
}

impl MatrixInputStep {
    /// Returns the OM-cited prompt text for the current matrix workflow step.
    ///
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3 "Matrix Functions".
    /// ElementPrompt uses 1-indexed row/col per OM convention ("A1,1=?" not "A0,0=?").
    /// SimeqInputPrompt uses 1-indexed vector element ("B1=?" not "B0=?").
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            MatrixInputStep::OrderPrompt => Some("ORDER=?".to_string()),
            // ElementPrompt(row, col): 0-indexed → 1-indexed for OM-faithful display.
            // Column-major iteration: col varies slowest in the OM prompt sequence.
            MatrixInputStep::ElementPrompt(r, c) => {
                Some(format!("A{},{}=?", r + 1, c + 1))
            }
            MatrixInputStep::Ready => None,
            MatrixInputStep::EditPrompt => Some("ROW\u{2191}COL=?".to_string()),
            // SimeqInputPrompt(idx): 0-indexed → 1-indexed for OM-faithful display.
            MatrixInputStep::SimeqInputPrompt(idx) => {
                Some(format!("B{}=?", idx + 1))
            }
            MatrixInputStep::SimeqDone => None,
        }
    }
}

// ── SolveInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the SOLVE workflow (Plan 28-08).
///
/// Steps follow the HP-41C Math Pac I OM SOLVE program prompting sequence.
/// Extended by Plan 28-08.
#[derive(Debug, Clone, PartialEq)]
pub enum SolveInputStep {
    /// Awaiting user function label name. Prompt: "FUNCTION NAME?"
    FunctionNamePrompt,
    /// Awaiting first guess. Prompt: "GUESS 1=?"
    Guess1Prompt,
    /// Awaiting second guess. Prompt: "GUESS 2=?"
    Guess2Prompt,
}

impl SolveInputStep {
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            SolveInputStep::FunctionNamePrompt => Some("FUNCTION NAME?".to_string()),
            SolveInputStep::Guess1Prompt => Some("GUESS 1=?".to_string()),
            SolveInputStep::Guess2Prompt => Some("GUESS 2=?".to_string()),
        }
    }
}

// ── PolyInputStep ──────────────────────────────────────────────────────────────

/// Per-step state for the POLY workflow (Plan 28-05).
///
/// Steps follow the HP-41C Math Pac I OM POLY program prompting sequence
/// (HP 00041-90034, 1979), Chapter 7 "Polynomial Solutions".
///
/// Prompt sequence: DEGREE=? → A=? → B=? → ... → F=? (max degree 5) → Ready.
/// CoefficientPrompt(degree, current_idx): degree = total degree n (2..=5),
/// current_idx = coefficient index (0=A, 1=B, ..., 5=F).
#[derive(Debug, Clone, PartialEq)]
pub enum PolyInputStep {
    /// Awaiting polynomial degree n (2..=5). Prompt: "DEGREE=?"
    DegreePrompt,
    /// Awaiting coefficient at index current_idx. Prompt: "A=?" through "F=?".
    /// Field 0: total degree (2..=5). Field 1: current coefficient index (0..=5).
    /// Coefficient naming per OM Chapter 7: A=highest-order coeff, B=next, etc.
    CoefficientPrompt(u8, u8),
    /// All coefficients entered; ready to compute roots. Prompt: None.
    Ready,
}

impl PolyInputStep {
    /// Returns the OM-cited prompt text for the current step.
    ///
    /// Source: HP 00041-90034 (1979), Chapter 7 prompt sequence.
    /// A=? = leading coefficient (x^n term), B=? = x^(n-1), ..., F=? = constant term.
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            PolyInputStep::DegreePrompt => Some("DEGREE=?".to_string()),
            PolyInputStep::CoefficientPrompt(_degree, idx) => match idx {
                0 => Some("A=?".to_string()),
                1 => Some("B=?".to_string()),
                2 => Some("C=?".to_string()),
                3 => Some("D=?".to_string()),
                4 => Some("E=?".to_string()),
                5 => Some("F=?".to_string()),
                // Defensive fallback: idx > 5 is a logic error (degree cap is 5).
                _ => Some("?=?".to_string()),
            },
            PolyInputStep::Ready => None,
        }
    }

    /// Ergonomic helper: wrap this step in a ModalProgram for tests.
    pub fn into_modal(self) -> ModalProgram {
        ModalProgram::Poly(self)
    }
}

// ── IntegInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the INTG workflow (Plan 28-07).
///
/// Steps follow the HP-41C Math Pac I OM INTG program prompting sequence.
/// Extended by Plan 28-07.
#[derive(Debug, Clone, PartialEq)]
pub enum IntegInputStep {
    /// Awaiting integration mode choice (automatic vs. manual). Prompt: "MODE?"
    ModeChoice,
    /// Awaiting user function label name. Prompt: "FUNCTION NAME?"
    FunctionNamePrompt,
    /// Awaiting integration interval (lower bound a). Prompt: "LOWER LIMIT=?"
    IntervalPrompt,
    /// Awaiting number of initial subdivisions. Prompt: "SUBDIVISIONS=?"
    SubdivisionPrompt,
}

impl IntegInputStep {
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            IntegInputStep::ModeChoice => Some("MODE?".to_string()),
            IntegInputStep::FunctionNamePrompt => Some("FUNCTION NAME?".to_string()),
            IntegInputStep::IntervalPrompt => Some("LOWER LIMIT=?".to_string()),
            IntegInputStep::SubdivisionPrompt => Some("SUBDIVISIONS=?".to_string()),
        }
    }
}

// ── DifeqInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the DIFEQ workflow (Plan 28-09).
///
/// Steps follow the HP-41C Math Pac I OM DIFEQ program prompting sequence.
/// Extended by Plan 28-09.
#[derive(Debug, Clone, PartialEq)]
pub enum DifeqInputStep {
    /// Awaiting differential equation function label name. Prompt: "FUNCTION NAME?"
    FunctionNamePrompt,
    /// Awaiting ODE order n. Prompt: "ORDER=?"
    OrderPrompt,
    /// Awaiting step size h. Prompt: "STEP SIZE=?"
    StepSizePrompt,
    /// Awaiting initial x value x0. Prompt: "X0=?"
    X0Prompt,
    /// Awaiting initial y value y(x0). Prompt: "Y0=?"
    Y0Prompt,
    /// Awaiting initial y' value y'(x0) (for 2nd-order ODEs). Prompt: "Y'0=?"
    Y1PrimePrompt,
}

impl DifeqInputStep {
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            DifeqInputStep::FunctionNamePrompt => Some("FUNCTION NAME?".to_string()),
            DifeqInputStep::OrderPrompt => Some("ORDER=?".to_string()),
            DifeqInputStep::StepSizePrompt => Some("STEP SIZE=?".to_string()),
            DifeqInputStep::X0Prompt => Some("X0=?".to_string()),
            DifeqInputStep::Y0Prompt => Some("Y0=?".to_string()),
            DifeqInputStep::Y1PrimePrompt => Some("Y'0=?".to_string()),
        }
    }
}

// ── FourInputStep ──────────────────────────────────────────────────────────────

/// Per-step state for the FOUR (Fourier analysis) workflow (Plan 28-10).
///
/// Steps follow the HP-41C Math Pac I OM FOUR program prompting sequence.
/// Extended by Plan 28-10.
#[derive(Debug, Clone, PartialEq)]
pub enum FourInputStep {
    /// Awaiting number of samples N. Prompt: "N SAMPLES=?"
    NumSamplesPrompt,
    /// Awaiting number of frequencies to compute. Prompt: "N FREQS=?"
    NumFreqPrompt,
    /// Awaiting first coefficient input. Prompt: "FIRST COEFF=?"
    FirstCoeffPrompt,
    /// Awaiting sample value n. Prompt: "SAMPLE n=?"
    SamplePrompt(u8),
    /// All samples entered; computing. Prompt: None.
    Ready,
}

impl FourInputStep {
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            FourInputStep::NumSamplesPrompt => Some("N SAMPLES=?".to_string()),
            FourInputStep::NumFreqPrompt => Some("N FREQS=?".to_string()),
            FourInputStep::FirstCoeffPrompt => Some("FIRST COEFF=?".to_string()),
            FourInputStep::SamplePrompt(_) => Some("SAMPLE=?".to_string()),
            FourInputStep::Ready => None,
        }
    }
}

// ── TransInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the TRANS (coordinate transform) workflow (Plan 28-10).
///
/// Steps follow the HP-41C Math Pac I OM TRANS program prompting sequence.
/// Extended by Plan 28-10.
#[derive(Debug, Clone, PartialEq)]
pub enum TransInputStep {
    /// Awaiting transform initialization parameters. Prompt: "INIT=?"
    InitPrompt,
    /// Awaiting forward-transform input. Prompt: "FORWARD=?"
    ForwardPrompt,
    /// Awaiting inverse-transform input. Prompt: "INVERSE=?"
    InversePrompt,
}

impl TransInputStep {
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            TransInputStep::InitPrompt => Some("INIT=?".to_string()),
            TransInputStep::ForwardPrompt => Some("FORWARD=?".to_string()),
            TransInputStep::InversePrompt => Some("INVERSE=?".to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: ModalProgram::Matrix dispatch regression
    #[test]
    fn matrix_order_prompt() {
        let mp = ModalProgram::Matrix(MatrixInputStep::OrderPrompt);
        assert_eq!(mp.current_prompt(), Some("ORDER=?".to_string()));
    }

    // Catches: ModalProgram::Solve dispatch regression
    #[test]
    fn solve_function_name_prompt() {
        let mp = ModalProgram::Solve(SolveInputStep::FunctionNamePrompt);
        assert_eq!(mp.current_prompt(), Some("FUNCTION NAME?".to_string()));
    }

    // Catches: MatrixInputStep::Ready should return None (not prompting)
    #[test]
    fn matrix_ready_returns_none() {
        let mp = ModalProgram::Matrix(MatrixInputStep::Ready);
        assert_eq!(mp.current_prompt(), None);
    }

    // Catches: SolveInputStep multi-step regression
    #[test]
    fn solve_guess_prompts() {
        assert_eq!(
            ModalProgram::Solve(SolveInputStep::Guess1Prompt).current_prompt(),
            Some("GUESS 1=?".to_string())
        );
        assert_eq!(
            ModalProgram::Solve(SolveInputStep::Guess2Prompt).current_prompt(),
            Some("GUESS 2=?".to_string())
        );
    }

    // Catches: DifeqInputStep dispatch regression
    #[test]
    fn difeq_step_prompts() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt).current_prompt(),
            Some("FUNCTION NAME?".to_string())
        );
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::Y1PrimePrompt).current_prompt(),
            Some("Y'0=?".to_string())
        );
    }

    // Catches: Clone + PartialEq derive regression
    #[test]
    fn modal_program_clone_and_eq() {
        let mp = ModalProgram::Matrix(MatrixInputStep::OrderPrompt);
        assert_eq!(mp.clone(), mp);
    }

    // ── Plan 28-05: PolyInputStep prompt tests ────────────────────────────────

    // Catches: DegreePrompt returning wrong text
    #[test]
    fn poly_degree_prompt() {
        let mp = PolyInputStep::DegreePrompt.into_modal();
        assert_eq!(mp.current_prompt(), Some("DEGREE=?".to_string()));
    }

    // Catches: CoefficientPrompt idx=0 not returning "A=?"
    #[test]
    fn poly_coeff_prompt_a() {
        let mp = PolyInputStep::CoefficientPrompt(5, 0).into_modal();
        assert_eq!(mp.current_prompt(), Some("A=?".to_string()));
    }

    // Catches: CoefficientPrompt idx=1 not returning "B=?"
    #[test]
    fn poly_coeff_prompt_b() {
        let mp = PolyInputStep::CoefficientPrompt(5, 1).into_modal();
        assert_eq!(mp.current_prompt(), Some("B=?".to_string()));
    }

    // Catches: CoefficientPrompt idx=2 not returning "C=?"
    #[test]
    fn poly_coeff_prompt_c() {
        let mp = PolyInputStep::CoefficientPrompt(5, 2).into_modal();
        assert_eq!(mp.current_prompt(), Some("C=?".to_string()));
    }

    // Catches: CoefficientPrompt idx=3 not returning "D=?"
    #[test]
    fn poly_coeff_prompt_d() {
        let mp = PolyInputStep::CoefficientPrompt(5, 3).into_modal();
        assert_eq!(mp.current_prompt(), Some("D=?".to_string()));
    }

    // Catches: CoefficientPrompt idx=4 not returning "E=?"
    #[test]
    fn poly_coeff_prompt_e() {
        let mp = PolyInputStep::CoefficientPrompt(5, 4).into_modal();
        assert_eq!(mp.current_prompt(), Some("E=?".to_string()));
    }

    // Catches: CoefficientPrompt idx=5 not returning "F=?"
    #[test]
    fn poly_coeff_prompt_f() {
        let mp = PolyInputStep::CoefficientPrompt(5, 5).into_modal();
        assert_eq!(mp.current_prompt(), Some("F=?".to_string()));
    }

    // Catches: Ready not returning None
    #[test]
    fn poly_ready_returns_none() {
        let mp = PolyInputStep::Ready.into_modal();
        assert_eq!(mp.current_prompt(), None);
    }

    // Catches: Clone + PartialEq on PolyInputStep variants
    #[test]
    fn poly_input_step_clone_and_eq() {
        let step = PolyInputStep::CoefficientPrompt(3, 2);
        assert_eq!(step.clone(), step);
    }

    // ── Plan 28-06: MatrixInputStep prompt tests ──────────────────────────────

    // Catches: ElementPrompt 0-indexed offset wrong (must add 1 for OM-faithful display)
    #[test]
    fn matrix_element_prompt_1_1() {
        let mp = ModalProgram::Matrix(MatrixInputStep::ElementPrompt(0, 0));
        assert_eq!(mp.current_prompt(), Some("A1,1=?".to_string()));
    }

    // Catches: ElementPrompt column index wrong
    #[test]
    fn matrix_element_prompt_2_3() {
        let mp = ModalProgram::Matrix(MatrixInputStep::ElementPrompt(1, 2));
        assert_eq!(mp.current_prompt(), Some("A2,3=?".to_string()));
    }

    // Catches: ElementPrompt maximum indices (14×14)
    #[test]
    fn matrix_element_prompt_max() {
        let mp = ModalProgram::Matrix(MatrixInputStep::ElementPrompt(13, 13));
        assert_eq!(mp.current_prompt(), Some("A14,14=?".to_string()));
    }

    // Catches: EditPrompt returning wrong text (ROW↑COL=? uses Unicode ↑)
    #[test]
    fn matrix_edit_prompt() {
        let mp = ModalProgram::Matrix(MatrixInputStep::EditPrompt);
        assert_eq!(mp.current_prompt(), Some("ROW\u{2191}COL=?".to_string()));
    }

    // Catches: SimeqInputPrompt 0-indexed offset wrong (must add 1)
    #[test]
    fn matrix_simeq_input_prompt_first() {
        let mp = ModalProgram::Matrix(MatrixInputStep::SimeqInputPrompt(0));
        assert_eq!(mp.current_prompt(), Some("B1=?".to_string()));
    }

    // Catches: SimeqInputPrompt mid-sequence (e.g., B3=? for idx=2)
    #[test]
    fn matrix_simeq_input_prompt_third() {
        let mp = ModalProgram::Matrix(MatrixInputStep::SimeqInputPrompt(2));
        assert_eq!(mp.current_prompt(), Some("B3=?".to_string()));
    }

    // Catches: SimeqDone should return None (done state, no prompt)
    #[test]
    fn matrix_simeq_done_returns_none() {
        let mp = ModalProgram::Matrix(MatrixInputStep::SimeqDone);
        assert_eq!(mp.current_prompt(), None);
    }
}
