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
    pub fn current_prompt(&self) -> Option<&'static str> {
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
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            MatrixInputStep::OrderPrompt => Some("ORDER=?"),
            MatrixInputStep::ElementPrompt(_, _) => Some("ELEMENT=?"),
            MatrixInputStep::Ready => None,
            MatrixInputStep::EditPrompt => Some("EDIT=?"),
            MatrixInputStep::SimeqInputPrompt(_) => Some("b=?"),
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
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            SolveInputStep::FunctionNamePrompt => Some("FUNCTION NAME?"),
            SolveInputStep::Guess1Prompt => Some("GUESS 1=?"),
            SolveInputStep::Guess2Prompt => Some("GUESS 2=?"),
        }
    }
}

// ── PolyInputStep ──────────────────────────────────────────────────────────────

/// Per-step state for the POLY workflow (Plan 28-05).
///
/// Steps follow the HP-41C Math Pac I OM POLY program prompting sequence.
/// Extended by Plan 28-05.
#[derive(Debug, Clone, PartialEq)]
pub enum PolyInputStep {
    /// Awaiting polynomial degree n. Prompt: "DEGREE=?"
    DegreePrompt,
    /// Awaiting coefficient k of term x^j. Prompt: "COEF k,j=?"
    CoefficientPrompt(u8, u8),
    /// All coefficients entered; ready to compute roots. Prompt: None.
    Ready,
}

impl PolyInputStep {
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            PolyInputStep::DegreePrompt => Some("DEGREE=?"),
            PolyInputStep::CoefficientPrompt(_, _) => Some("COEFF=?"),
            PolyInputStep::Ready => None,
        }
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
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            IntegInputStep::ModeChoice => Some("MODE?"),
            IntegInputStep::FunctionNamePrompt => Some("FUNCTION NAME?"),
            IntegInputStep::IntervalPrompt => Some("LOWER LIMIT=?"),
            IntegInputStep::SubdivisionPrompt => Some("SUBDIVISIONS=?"),
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
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            DifeqInputStep::FunctionNamePrompt => Some("FUNCTION NAME?"),
            DifeqInputStep::OrderPrompt => Some("ORDER=?"),
            DifeqInputStep::StepSizePrompt => Some("STEP SIZE=?"),
            DifeqInputStep::X0Prompt => Some("X0=?"),
            DifeqInputStep::Y0Prompt => Some("Y0=?"),
            DifeqInputStep::Y1PrimePrompt => Some("Y'0=?"),
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
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            FourInputStep::NumSamplesPrompt => Some("N SAMPLES=?"),
            FourInputStep::NumFreqPrompt => Some("N FREQS=?"),
            FourInputStep::FirstCoeffPrompt => Some("FIRST COEFF=?"),
            FourInputStep::SamplePrompt(_) => Some("SAMPLE=?"),
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
    pub fn current_prompt(&self) -> Option<&'static str> {
        match self {
            TransInputStep::InitPrompt => Some("INIT=?"),
            TransInputStep::ForwardPrompt => Some("FORWARD=?"),
            TransInputStep::InversePrompt => Some("INVERSE=?"),
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
        assert_eq!(mp.current_prompt(), Some("ORDER=?"));
    }

    // Catches: ModalProgram::Solve dispatch regression
    #[test]
    fn solve_function_name_prompt() {
        let mp = ModalProgram::Solve(SolveInputStep::FunctionNamePrompt);
        assert_eq!(mp.current_prompt(), Some("FUNCTION NAME?"));
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
            Some("GUESS 1=?")
        );
        assert_eq!(
            ModalProgram::Solve(SolveInputStep::Guess2Prompt).current_prompt(),
            Some("GUESS 2=?")
        );
    }

    // Catches: DifeqInputStep dispatch regression
    #[test]
    fn difeq_step_prompts() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt).current_prompt(),
            Some("FUNCTION NAME?")
        );
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::Y1PrimePrompt).current_prompt(),
            Some("Y'0=?")
        );
    }

    // Catches: Clone + PartialEq derive regression
    #[test]
    fn modal_program_clone_and_eq() {
        let mp = ModalProgram::Matrix(MatrixInputStep::OrderPrompt);
        assert_eq!(mp.clone(), mp);
    }
}
