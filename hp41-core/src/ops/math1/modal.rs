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

    /// Returns `true` if the current modal step expects an alpha label name input
    /// (i.e., the user must type a function label via the XeqByName{CollectForModal}
    /// pending-input flow before numeric R/S submission can advance the modal).
    ///
    /// Only the three FunctionNamePrompt variants return `true` — one per user-callback
    /// program (Integ, Solve, Difeq). All numeric-input steps and all non-callback
    /// programs (Matrix, Poly, Four, Trans) return `false`.
    ///
    /// Used by the auto-open hook (`maybe_auto_open_collect_for_modal`) in hp41-cli
    /// (D-29.9) and by hp41-gui Phase 31 via the same function call (D-25.6 parity).
    ///
    /// Phase 29 / CLI-05 additive public surface — D-29.7 / D-29.9 / D-25.6.
    pub fn requires_alpha_label(&self) -> bool {
        matches!(
            self,
            ModalProgram::Integ(IntegInputStep::FunctionNamePrompt)
                | ModalProgram::Solve(SolveInputStep::FunctionNamePrompt)
                | ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt)
        )
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
            MatrixInputStep::ElementPrompt(r, c) => Some(format!("A{},{}=?", r + 1, c + 1)),
            MatrixInputStep::Ready => None,
            MatrixInputStep::EditPrompt => Some("ROW\u{2191}COL=?".to_string()),
            // SimeqInputPrompt(idx): 0-indexed → 1-indexed for OM-faithful display.
            MatrixInputStep::SimeqInputPrompt(idx) => Some(format!("B{}=?", idx + 1)),
            MatrixInputStep::SimeqDone => None,
        }
    }
}

// ── SolveInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the SOLVE workflow (Plan 28-08).
///
/// Steps follow the HP-41C Math Pac I OM SOLVE program prompting sequence
/// (HP 00041-90034, 1979), Chapter 6 "Root Finder".
///
/// Prompt sequence: FUNCTION NAME? → GUESS 1=? → GUESS 2=? → Ready (computing).
/// Plan 28-08 extends Plan 28-01's 3-variant stub with the `Ready` variant.
#[derive(Debug, Clone, PartialEq)]
pub enum SolveInputStep {
    /// Awaiting user function label name. Prompt: "FUNCTION NAME?"
    /// Source: HP 00041-90034 (1979), Chapter 6, p. 33.
    FunctionNamePrompt,
    /// Awaiting first guess x1. Prompt: "GUESS 1=?"
    /// Source: HP 00041-90034 (1979), Chapter 6, p. 34.
    Guess1Prompt,
    /// Awaiting second guess x2. Prompt: "GUESS 2=?"
    /// Source: HP 00041-90034 (1979), Chapter 6, p. 34.
    Guess2Prompt,
    /// SOLVE workflow complete; computing or done. No prompt displayed.
    /// Added by Plan 28-08 (mirrors IntegInputStep::Ready pattern from Plan 28-07).
    Ready,
}

impl SolveInputStep {
    /// Returns the OM-cited prompt text for the current SOLVE workflow step.
    ///
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 6, pp. 33–34.
    /// `Ready` returns `None` because no user input is needed during computation.
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            SolveInputStep::FunctionNamePrompt => Some("FUNCTION NAME?".to_string()),
            SolveInputStep::Guess1Prompt => Some("GUESS 1=?".to_string()),
            SolveInputStep::Guess2Prompt => Some("GUESS 2=?".to_string()),
            SolveInputStep::Ready => None,
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
/// Steps follow the HP-41C Math Pac I OM INTG program prompting sequence
/// (HP 00041-90034, 1979), Chapter 3 "Numerical Integration", pp. 33–42.
///
/// Prompt choices:
/// - ModeChoice → "INTG MODE?" (explicit per plan behavior; OM pp. 33-34)
/// - FunctionNamePrompt → "FUNCTION NAME?" (OM pp. 35/38: prompts for user LBL)
/// - IntervalPrompt → "(A,B)=?" (OM p. 36: integration interval — lower then upper)
/// - SubdivisionPrompt → "N=?" (OM p. 37: subdivision count for explicit mode)
/// - Ready → None (computing; no prompt)
///
/// Plan 28-07 extends Plan 28-01's 4-variant stub with the `Ready` variant.
#[derive(Debug, Clone, PartialEq)]
pub enum IntegInputStep {
    /// Awaiting integration mode choice (Discrete vs. Explicit).
    /// Prompt: "INTG MODE?" (OM p. 34 — user selects C/D for discrete, FUNCTION for explicit).
    ModeChoice,
    /// Awaiting user function label name (Explicit mode only).
    /// Prompt: "FUNCTION NAME?" (OM p. 38: XEQ label of the integrand function).
    FunctionNamePrompt,
    /// Awaiting integration interval bounds (a, b) (Explicit mode).
    /// Prompt: "(A,B)=?" (OM p. 36: lower bound a and upper bound b entered as A in X, B in Y).
    IntervalPrompt,
    /// Awaiting subdivision count N for the interval.
    /// Prompt: "N=?" (OM p. 37: number of subdivisions; cap 32768 per INTG-07/ADR-004).
    SubdivisionPrompt,
    /// INTG workflow complete; computing or done. No prompt displayed.
    Ready,
}

impl IntegInputStep {
    /// Returns the OM-cited prompt text for the current INTG workflow step.
    ///
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3, pp. 33–42.
    /// `Ready` returns `None` because no user input is needed during computation.
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            IntegInputStep::ModeChoice => Some("INTG MODE?".to_string()),
            IntegInputStep::FunctionNamePrompt => Some("FUNCTION NAME?".to_string()),
            IntegInputStep::IntervalPrompt => Some("(A,B)=?".to_string()),
            IntegInputStep::SubdivisionPrompt => Some("N=?".to_string()),
            IntegInputStep::Ready => None,
        }
    }
}

// ── DifeqInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the DIFEQ workflow (Plan 28-09).
///
/// Steps follow the HP-41C Math Pac I OM DIFEQ program prompting sequence
/// (HP 00041-90034, 1979), Chapter 7 "Differential Equations".
///
/// Prompt sequence:
/// - ORDER=1 (5 prompts): FUNCTION NAME? → ORDER=? → STEP SIZE=? → X0=? → Y0=? → Ready
/// - ORDER=2 (6 prompts): FUNCTION NAME? → ORDER=? → STEP SIZE=? → X0=? → Y0=? → Y'0=? → Ready
///
/// Y1PrimePrompt is only routed when the OrderPrompt user-entered value equals 2;
/// the routing logic lives in op_difeq_run_loop (Plan 28-09). Plan 28-09 stores
/// ORDER=1 vs ORDER=2 in DifeqState.order and skips/visits Y1PrimePrompt accordingly.
///
/// Plan 28-09 extends Plan 28-01's 6-variant stub with the `Ready` variant.
#[derive(Debug, Clone, PartialEq)]
pub enum DifeqInputStep {
    /// Awaiting differential equation function label name.
    /// Prompt: "FUNCTION NAME?"
    /// Source: HP 00041-90034 (1979), Chapter 7, DIFEQ program.
    FunctionNamePrompt,
    /// Awaiting ODE order (1 or 2). Prompt: "ORDER=?"
    /// Source: HP 00041-90034 (1979), Chapter 7 prompt sequence.
    OrderPrompt,
    /// Awaiting step size h. Prompt: "STEP SIZE=?"
    /// Source: HP 00041-90034 (1979), Chapter 7 prompt sequence.
    StepSizePrompt,
    /// Awaiting initial x value x0. Prompt: "X0=?"
    /// Source: HP 00041-90034 (1979), Chapter 7 prompt sequence.
    X0Prompt,
    /// Awaiting initial y value y(x0). Prompt: "Y0=?"
    /// Source: HP 00041-90034 (1979), Chapter 7 prompt sequence.
    Y0Prompt,
    /// Awaiting initial y' value y'(x0) (for 2nd-order ODEs only). Prompt: "Y'0=?"
    /// Only visited when ORDER=2; skipped when ORDER=1.
    /// Source: HP 00041-90034 (1979), Chapter 7 prompt sequence (2nd-order extension).
    Y1PrimePrompt,
    /// DIFEQ workflow complete; computing or done. No prompt displayed.
    /// Added by Plan 28-09 (mirrors IntegInputStep::Ready + SolveInputStep::Ready pattern).
    Ready,
}

impl DifeqInputStep {
    /// Returns the OM-cited prompt text for the current DIFEQ workflow step.
    ///
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 7, DIFEQ program.
    /// `Ready` returns `None` because no user input is needed during computation.
    /// `Y1PrimePrompt` returns `"Y'0=?"` — only displayed for ORDER=2 ODEs.
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            DifeqInputStep::FunctionNamePrompt => Some("FUNCTION NAME?".to_string()),
            DifeqInputStep::OrderPrompt => Some("ORDER=?".to_string()),
            DifeqInputStep::StepSizePrompt => Some("STEP SIZE=?".to_string()),
            DifeqInputStep::X0Prompt => Some("X0=?".to_string()),
            DifeqInputStep::Y0Prompt => Some("Y0=?".to_string()),
            DifeqInputStep::Y1PrimePrompt => Some("Y'0=?".to_string()),
            DifeqInputStep::Ready => None,
        }
    }
}

// ── FourInputStep ──────────────────────────────────────────────────────────────

/// Per-step state for the FOUR (Fourier analysis) workflow (Plan 28-10).
///
/// Prompt sequence per HP-41C Math Pac I OM:
/// NO. SAMPLES=? → NO. FREQ=? → 1ST COEFF=? → [RECT?] → Y1=? … YN=? → Ready.
///
/// Plan 28-10 adds `RectTogglePrompt` (FOUR-03) and uses OM-faithful prompt strings.
/// Plan 28-01's placeholder prompt strings (N SAMPLES=?, N FREQS=?, FIRST COEFF=?)
/// are updated to match OM wording: "NO. SAMPLES=?", "NO. FREQ=?", "1ST COEFF=?".
#[derive(Debug, Clone, PartialEq)]
pub enum FourInputStep {
    /// Awaiting number of samples N. Prompt: "NO. SAMPLES=?"
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), FOUR program.
    NumSamplesPrompt,
    /// Awaiting number of frequencies L to compute (1..=10). Prompt: "NO. FREQ=?"
    /// Source: HP-41C Math Pac I OM FOUR program. Cap: MAX_FOURIER_PAIRS = 10.
    NumFreqPrompt,
    /// Awaiting first coefficient number (starting index). Prompt: "1ST COEFF=?"
    /// Source: HP-41C Math Pac I OM FOUR program.
    FirstCoeffPrompt,
    /// Awaiting rectangular/polar output choice. Prompt: "RECT?"
    /// FOUR-03: if user presses R/S (yes) → rectangular (aₙ, bₙ) form;
    /// if user presses any other key → polar (cₙ, φₙ) form.
    /// Added by Plan 28-10 (FOUR-03).
    RectTogglePrompt,
    /// Awaiting sample value Yk (0-indexed). Prompt: "Yk=?" where k = idx+1.
    /// Source: HP-41C Math Pac I OM FOUR program, sample-input sequence.
    SamplePrompt(u8),
    /// All samples entered and DFT computed. No prompt displayed (computing/done state).
    Ready,
}

impl FourInputStep {
    /// Returns the OM-cited prompt text for the current FOUR workflow step.
    ///
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), FOUR program prompting.
    /// `Ready` returns `None` — computation in progress, no user input needed.
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            FourInputStep::NumSamplesPrompt => Some("NO. SAMPLES=?".to_string()),
            FourInputStep::NumFreqPrompt => Some("NO. FREQ=?".to_string()),
            FourInputStep::FirstCoeffPrompt => Some("1ST COEFF=?".to_string()),
            FourInputStep::RectTogglePrompt => Some("RECT?".to_string()),
            // SamplePrompt(idx): 0-indexed → 1-indexed for OM-faithful display ("Y1=?", "Y2=?", ...).
            FourInputStep::SamplePrompt(idx) => Some(format!("Y{}=?", idx + 1)),
            FourInputStep::Ready => None,
        }
    }
}

// ── TransInputStep ─────────────────────────────────────────────────────────────

/// Per-step state for the TRANS (coordinate transform) workflow (Plan 28-10).
///
/// Plan 28-10 revises Plan 28-01's generic `InitPrompt/ForwardPrompt/InversePrompt`
/// with 2D/3D-specific variants per TRANS-01..04. The schema revision is SAFE:
/// the Plan 28-01 placeholder variants had no downstream consumers (no CLI/GUI
/// wiring existed for TRANS before Plan 28-10). All TRANS modal routing lands in
/// Phases 29 (CLI) and 31 (GUI) per the standard v3.0 plan-boundary convention.
///
/// **TransInputStep schema revision note (Plan 28-10):**
/// - `InitPrompt` → split into `Init2dPrompt` (TRANS-01 A-entry 2D) and
///   `Init3dOriginPrompt` + `Init3dAxisPrompt` (TRANS-03 A/B-entry 3D).
/// - `ForwardPrompt` and `InversePrompt` retained (now carry both 2D/3D semantics).
/// - New `Ready` variant added (computation/done state, no user input).
#[derive(Debug, Clone, PartialEq)]
pub enum TransInputStep {
    /// TRANS-01 A-entry: Awaiting 2D transform parameters (x₀, y₀, θ). Prompt: "X0,Y0,θ?"
    /// Source: HP-41C Math Pac I OM, TRANS program, 2D initialization sequence.
    Init2dPrompt,
    /// TRANS-03 A-entry: Awaiting 3D origin point (x₀, y₀, z₀). Prompt: "ORIGIN?"
    /// Source: HP-41C Math Pac I OM, TRANS program, 3D initialization — origin entry.
    Init3dOriginPrompt,
    /// TRANS-03 B-entry: Awaiting 3D rotation axis and angle (a, b, c, θ). Prompt: "AXIS+θ?"
    /// Source: HP-41C Math Pac I OM, TRANS program, 3D initialization — axis+angle entry.
    Init3dAxisPrompt,
    /// TRANS-02/04 C-entry: Awaiting forward-transform input point. Prompt: "FWD?"
    /// Valid for both 2D and 3D modes after initialization.
    ForwardPrompt,
    /// TRANS-02/04 E-entry: Awaiting inverse-transform input point. Prompt: "INV?"
    /// Valid for both 2D and 3D modes after initialization.
    InversePrompt,
    /// Transform computation complete or initialized; ready for FWD/INV queries. No prompt.
    Ready,
}

impl TransInputStep {
    /// Returns the OM-cited prompt text for the current TRANS workflow step.
    ///
    /// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), TRANS program prompting.
    /// `Ready` returns `None` — no user input needed in this state.
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            TransInputStep::Init2dPrompt => Some("X0,Y0,\u{03B8}?".to_string()),
            TransInputStep::Init3dOriginPrompt => Some("ORIGIN?".to_string()),
            TransInputStep::Init3dAxisPrompt => Some("AXIS+\u{03B8}?".to_string()),
            TransInputStep::ForwardPrompt => Some("FWD?".to_string()),
            TransInputStep::InversePrompt => Some("INV?".to_string()),
            TransInputStep::Ready => None,
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

    // Catches: DifeqInputStep dispatch regression (Plan 28-09 expands to all 7 variants)
    #[test]
    fn difeq_step_function_name_prompt() {
        // Source: HP 00041-90034 (1979), Chapter 7, DIFEQ program prompt sequence.
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt).current_prompt(),
            Some("FUNCTION NAME?".to_string())
        );
    }

    // Catches: OrderPrompt returning wrong text or returning None
    #[test]
    fn difeq_step_order_prompt() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::OrderPrompt).current_prompt(),
            Some("ORDER=?".to_string())
        );
    }

    // Catches: StepSizePrompt returning wrong text
    #[test]
    fn difeq_step_step_size_prompt() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::StepSizePrompt).current_prompt(),
            Some("STEP SIZE=?".to_string())
        );
    }

    // Catches: X0Prompt returning wrong text
    #[test]
    fn difeq_step_x0_prompt() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::X0Prompt).current_prompt(),
            Some("X0=?".to_string())
        );
    }

    // Catches: Y0Prompt returning wrong text
    #[test]
    fn difeq_step_y0_prompt() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::Y0Prompt).current_prompt(),
            Some("Y0=?".to_string())
        );
    }

    // Catches: Y1PrimePrompt returning wrong text (ORDER=2 path)
    #[test]
    fn difeq_step_y1_prime_prompt() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::Y1PrimePrompt).current_prompt(),
            Some("Y'0=?".to_string())
        );
    }

    // Catches: Ready variant missing or returning Some(...) instead of None
    // This variant was added in Plan 28-09 (mirrors IntegInputStep::Ready pattern).
    #[test]
    fn difeq_step_ready_returns_none() {
        assert_eq!(
            ModalProgram::Difeq(DifeqInputStep::Ready).current_prompt(),
            None,
            "DifeqInputStep::Ready must return None (computing state, no prompt)"
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

    // ── Plan 28-08: SolveInputStep prompt tests ──────────────────────────────

    // Catches: SolveInputStep::FunctionNamePrompt returning wrong text
    #[test]
    fn solve_function_name_prompt_text() {
        let mp = ModalProgram::Solve(SolveInputStep::FunctionNamePrompt);
        assert_eq!(mp.current_prompt(), Some("FUNCTION NAME?".to_string()));
    }

    // Catches: SolveInputStep::Guess1Prompt returning wrong text
    #[test]
    fn solve_guess1_prompt_text() {
        let mp = ModalProgram::Solve(SolveInputStep::Guess1Prompt);
        assert_eq!(mp.current_prompt(), Some("GUESS 1=?".to_string()));
    }

    // Catches: SolveInputStep::Guess2Prompt returning wrong text
    #[test]
    fn solve_guess2_prompt_text() {
        let mp = ModalProgram::Solve(SolveInputStep::Guess2Prompt);
        assert_eq!(mp.current_prompt(), Some("GUESS 2=?".to_string()));
    }

    // Catches: SolveInputStep::Ready not returning None (computing state must show no prompt)
    #[test]
    fn solve_ready_returns_none() {
        let mp = ModalProgram::Solve(SolveInputStep::Ready);
        assert_eq!(mp.current_prompt(), None);
    }

    // Catches: Clone + PartialEq derive regression on SolveInputStep including Ready variant
    #[test]
    fn solve_input_step_clone_and_eq() {
        let step = SolveInputStep::Ready;
        assert_eq!(step.clone(), step);
        let step2 = SolveInputStep::Guess1Prompt;
        assert_ne!(step, step2);
    }

    // Catches: ModalProgram::Solve dispatch regression — all 4 variants in one round-trip
    #[test]
    fn solve_modal_dispatch_round_trip() {
        let variants = [
            (
                SolveInputStep::FunctionNamePrompt,
                Some("FUNCTION NAME?".to_string()),
            ),
            (SolveInputStep::Guess1Prompt, Some("GUESS 1=?".to_string())),
            (SolveInputStep::Guess2Prompt, Some("GUESS 2=?".to_string())),
            (SolveInputStep::Ready, None),
        ];
        for (step, expected) in variants {
            let mp = ModalProgram::Solve(step.clone());
            assert_eq!(mp.current_prompt(), expected, "failed for step: {step:?}");
        }
    }

    // ── Plan 28-10: FourInputStep prompt tests ────────────────────────────────

    // Catches: FourInputStep::NumSamplesPrompt returning wrong text (OM: "NO. SAMPLES=?")
    #[test]
    fn four_num_samples_prompt() {
        let mp = ModalProgram::Four(FourInputStep::NumSamplesPrompt);
        assert_eq!(mp.current_prompt(), Some("NO. SAMPLES=?".to_string()));
    }

    // Catches: FourInputStep::NumFreqPrompt returning wrong text (OM: "NO. FREQ=?")
    #[test]
    fn four_num_freq_prompt() {
        let mp = ModalProgram::Four(FourInputStep::NumFreqPrompt);
        assert_eq!(mp.current_prompt(), Some("NO. FREQ=?".to_string()));
    }

    // Catches: FourInputStep::FirstCoeffPrompt returning wrong text (OM: "1ST COEFF=?")
    #[test]
    fn four_first_coeff_prompt() {
        let mp = ModalProgram::Four(FourInputStep::FirstCoeffPrompt);
        assert_eq!(mp.current_prompt(), Some("1ST COEFF=?".to_string()));
    }

    // Catches: FourInputStep::RectTogglePrompt returning wrong text (FOUR-03: "RECT?")
    #[test]
    fn four_rect_toggle_prompt() {
        let mp = ModalProgram::Four(FourInputStep::RectTogglePrompt);
        assert_eq!(mp.current_prompt(), Some("RECT?".to_string()));
    }

    // Catches: SamplePrompt 0-indexed offset wrong (must add 1 for OM-faithful "Y1=?"..."YN=?")
    #[test]
    fn four_sample_prompt_first() {
        let mp = ModalProgram::Four(FourInputStep::SamplePrompt(0));
        assert_eq!(mp.current_prompt(), Some("Y1=?".to_string()));
    }

    // Catches: SamplePrompt index 7 returning wrong label (e.g., "Y8=?")
    #[test]
    fn four_sample_prompt_eighth() {
        let mp = ModalProgram::Four(FourInputStep::SamplePrompt(7));
        assert_eq!(mp.current_prompt(), Some("Y8=?".to_string()));
    }

    // Catches: FourInputStep::Ready not returning None (computing state must show no prompt)
    #[test]
    fn four_ready_returns_none() {
        let mp = ModalProgram::Four(FourInputStep::Ready);
        assert_eq!(mp.current_prompt(), None);
    }

    // Catches: Clone + PartialEq derive regression on FourInputStep including RectTogglePrompt
    #[test]
    fn four_input_step_clone_and_eq() {
        let step = FourInputStep::SamplePrompt(3);
        assert_eq!(step.clone(), step);
        let step2 = FourInputStep::RectTogglePrompt;
        assert_ne!(step, step2);
    }

    // Catches: ModalProgram::Four dispatch regression — all 6 variants in one round-trip
    #[test]
    fn four_modal_dispatch_round_trip() {
        let variants: Vec<(FourInputStep, Option<String>)> = vec![
            (
                FourInputStep::NumSamplesPrompt,
                Some("NO. SAMPLES=?".to_string()),
            ),
            (FourInputStep::NumFreqPrompt, Some("NO. FREQ=?".to_string())),
            (
                FourInputStep::FirstCoeffPrompt,
                Some("1ST COEFF=?".to_string()),
            ),
            (FourInputStep::RectTogglePrompt, Some("RECT?".to_string())),
            (FourInputStep::SamplePrompt(0), Some("Y1=?".to_string())),
            (FourInputStep::SamplePrompt(9), Some("Y10=?".to_string())),
            (FourInputStep::Ready, None),
        ];
        for (step, expected) in variants {
            let mp = ModalProgram::Four(step.clone());
            assert_eq!(mp.current_prompt(), expected, "failed for step: {step:?}");
        }
    }

    // ── Plan 28-10: TransInputStep prompt tests ───────────────────────────────

    // Catches: TransInputStep::Init2dPrompt returning wrong text (TRANS-01: "X0,Y0,θ?")
    #[test]
    fn trans_init2d_prompt() {
        let mp = ModalProgram::Trans(TransInputStep::Init2dPrompt);
        // Unicode theta (θ = U+03B8)
        assert_eq!(mp.current_prompt(), Some("X0,Y0,\u{03B8}?".to_string()));
    }

    // Catches: TransInputStep::Init3dOriginPrompt returning wrong text (TRANS-03: "ORIGIN?")
    #[test]
    fn trans_init3d_origin_prompt() {
        let mp = ModalProgram::Trans(TransInputStep::Init3dOriginPrompt);
        assert_eq!(mp.current_prompt(), Some("ORIGIN?".to_string()));
    }

    // Catches: TransInputStep::Init3dAxisPrompt returning wrong text (TRANS-03: "AXIS+θ?")
    #[test]
    fn trans_init3d_axis_prompt() {
        let mp = ModalProgram::Trans(TransInputStep::Init3dAxisPrompt);
        assert_eq!(mp.current_prompt(), Some("AXIS+\u{03B8}?".to_string()));
    }

    // Catches: TransInputStep::ForwardPrompt returning wrong text (TRANS-02: "FWD?")
    #[test]
    fn trans_forward_prompt() {
        let mp = ModalProgram::Trans(TransInputStep::ForwardPrompt);
        assert_eq!(mp.current_prompt(), Some("FWD?".to_string()));
    }

    // Catches: TransInputStep::InversePrompt returning wrong text (TRANS-02/04: "INV?")
    #[test]
    fn trans_inverse_prompt() {
        let mp = ModalProgram::Trans(TransInputStep::InversePrompt);
        assert_eq!(mp.current_prompt(), Some("INV?".to_string()));
    }

    // Catches: TransInputStep::Ready not returning None (done state must show no prompt)
    #[test]
    fn trans_ready_returns_none() {
        let mp = ModalProgram::Trans(TransInputStep::Ready);
        assert_eq!(mp.current_prompt(), None);
    }

    // Catches: Clone + PartialEq derive regression on TransInputStep
    #[test]
    fn trans_input_step_clone_and_eq() {
        let step = TransInputStep::Init2dPrompt;
        assert_eq!(step.clone(), step);
        let step2 = TransInputStep::Init3dOriginPrompt;
        assert_ne!(step, step2);
    }

    // Catches: ModalProgram::Trans dispatch regression — all 6 variants in one round-trip
    #[test]
    fn trans_modal_dispatch_round_trip() {
        let variants: Vec<(TransInputStep, Option<String>)> = vec![
            (
                TransInputStep::Init2dPrompt,
                Some("X0,Y0,\u{03B8}?".to_string()),
            ),
            (
                TransInputStep::Init3dOriginPrompt,
                Some("ORIGIN?".to_string()),
            ),
            (
                TransInputStep::Init3dAxisPrompt,
                Some("AXIS+\u{03B8}?".to_string()),
            ),
            (TransInputStep::ForwardPrompt, Some("FWD?".to_string())),
            (TransInputStep::InversePrompt, Some("INV?".to_string())),
            (TransInputStep::Ready, None),
        ];
        for (step, expected) in variants {
            let mp = ModalProgram::Trans(step.clone());
            assert_eq!(mp.current_prompt(), expected, "failed for step: {step:?}");
        }
    }

    // ── Plan 28-07: IntegInputStep prompt tests ───────────────────────────────

    // Catches: ModeChoice returning wrong text (must be "INTG MODE?" not "MODE?")
    #[test]
    fn integ_mode_choice_prompt() {
        let mp = ModalProgram::Integ(IntegInputStep::ModeChoice);
        assert_eq!(mp.current_prompt(), Some("INTG MODE?".to_string()));
    }

    // Catches: FunctionNamePrompt returning wrong text
    #[test]
    fn integ_function_name_prompt() {
        let mp = ModalProgram::Integ(IntegInputStep::FunctionNamePrompt);
        assert_eq!(mp.current_prompt(), Some("FUNCTION NAME?".to_string()));
    }

    // Catches: IntervalPrompt returning wrong text (must be "(A,B)=?" not "LOWER LIMIT=?")
    #[test]
    fn integ_interval_prompt() {
        let mp = ModalProgram::Integ(IntegInputStep::IntervalPrompt);
        assert_eq!(mp.current_prompt(), Some("(A,B)=?".to_string()));
    }

    // Catches: SubdivisionPrompt returning wrong text (must be "N=?" not "SUBDIVISIONS=?")
    #[test]
    fn integ_subdivision_prompt() {
        let mp = ModalProgram::Integ(IntegInputStep::SubdivisionPrompt);
        assert_eq!(mp.current_prompt(), Some("N=?".to_string()));
    }

    // Catches: Ready not returning None (computing state must show no prompt)
    #[test]
    fn integ_ready_returns_none() {
        let mp = ModalProgram::Integ(IntegInputStep::Ready);
        assert_eq!(mp.current_prompt(), None);
    }

    // Catches: Clone + PartialEq derive regression on IntegInputStep
    #[test]
    fn integ_input_step_clone_and_eq() {
        let step = IntegInputStep::SubdivisionPrompt;
        assert_eq!(step.clone(), step);
    }

    // Catches: ModalProgram::Integ dispatch regression via current_prompt
    #[test]
    fn integ_modal_dispatch_round_trip() {
        let variants = [
            (IntegInputStep::ModeChoice, Some("INTG MODE?".to_string())),
            (
                IntegInputStep::FunctionNamePrompt,
                Some("FUNCTION NAME?".to_string()),
            ),
            (IntegInputStep::IntervalPrompt, Some("(A,B)=?".to_string())),
            (IntegInputStep::SubdivisionPrompt, Some("N=?".to_string())),
            (IntegInputStep::Ready, None),
        ];
        for (step, expected) in variants {
            let mp = ModalProgram::Integ(step.clone());
            assert_eq!(mp.current_prompt(), expected, "failed for step: {step:?}");
        }
    }
}
