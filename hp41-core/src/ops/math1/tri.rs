// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Triangle Solutions — Law of Sines and Law of Cosines solvers.
//!
//! ## Source
//! HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter on Triangle Solutions, p. 46.
//!
//! ## Stack convention
//! All triangle ops read inputs from the stack (X/Y/Z/T) and write formatted output
//! lines to `state.print_buffer`. Stack consumption: 3 or 4 input values depending
//! on the solver type. LiftEffect: Disable (per HP-41 convention for print ops).
//!
//! ## Angle mode
//! All angle inputs are read from the stack in `state.angle_mode` units.
//! All angle outputs are formatted in `state.angle_mode` units.
//! Internal computation uses f64 radians.
//!
//! ## Print buffer format
//! All output lines pushed to `state.print_buffer` use uppercase labels:
//! "A=<value>", "B=<value>", "C=<value>", "a=<value>", "b=<value>", "c=<value>"
//! (uppercase for angles, lowercase for sides — OM convention).
//!
//! ## SSA Ambiguous Case (TRI-05)
//!
//! The SSA (Side-Side-Angle) case is the classical "ambiguous case" in trigonometry.
//! Given sides a, b and angle A:
//!
//! ```text
//! Let h = b · sin(A)  (altitude from C to side c)
//! - a < h         → NO SOLUTION (side too short to reach the base)
//! - a == h        → ONE solution (right triangle, B = 90°)
//! - h < a < b     → TWO solutions (ambiguous case)
//! - a >= b        → ONE solution (unique)
//! ```
//!
//! When two solutions exist, BOTH are output to `print_buffer` per the OM display
//! sequence (HP 00041-90034, 1979, p. 46 worked example):
//!
//! ```text
//! Solution 1 (acute B):
//!   B1=<value>
//!   C1=<value>
//!   c1=<value>
//!
//! Solution 2 (obtuse B = π - B1):
//!   B2=<value>
//!   C2=<value>
//!   c2=<value>
//! ```
//!
//! NOTE: The OM uses the labeling "Solution 1 / Solution 2" in its worked example.
//! Plan 28-10 transcribes this as "B1=", "C1=", "c1=", "B2=", "C2=", "c2=" per the
//! pattern observed in the OM's tabular display. The "NO SOLUTION" case outputs one
//! line to print_buffer.

use std::f64::consts::PI;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::num::HpNum;
use crate::stack::apply_lift_effect;
use crate::stack::LiftEffect;
use crate::state::{AngleMode, CalcState};

// ── Angle conversion helpers (f64 bridge) ────────────────────────────────────

/// Convert angle from CalcState.angle_mode to radians (f64).
fn to_radians(angle: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Rad => angle,
        AngleMode::Deg => angle.to_radians(),
        AngleMode::Grad => angle * (PI / 200.0),
    }
}

/// Convert angle from radians (f64) to CalcState.angle_mode.
fn from_radians(rad: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Rad => rad,
        AngleMode::Deg => rad.to_degrees(),
        AngleMode::Grad => rad * (200.0 / PI),
    }
}

/// Convert an f64 to HpNum, returning Domain on NaN/infinity.
fn f64_to_hpnum(v: f64) -> Result<HpNum, HpError> {
    Decimal::from_f64(v)
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)
}

/// Format a triangle angle in the current angle_mode for print_buffer output.
fn fmt_angle(rad: f64, state: &CalcState) -> String {
    let v = f64_to_hpnum(from_radians(rad, state.angle_mode)).unwrap_or_else(|_| HpNum::zero());
    format_hpnum(&v, &state.display_mode)
}

/// Format a triangle side length for print_buffer output.
fn fmt_side(v: f64, state: &CalcState) -> String {
    let n = f64_to_hpnum(v).unwrap_or_else(|_| HpNum::zero());
    format_hpnum(&n, &state.display_mode)
}

// ── Op::TriSss — SSS (Law of Cosines: three sides → three angles) ────────────

/// SSS triangle solver: three sides (a, b, c) in stack X/Y/Z → three angles via Law of Cosines.
///
/// Stack input:
/// - X = a (side opposite angle A)
/// - Y = b (side opposite angle B)
/// - Z = c (side opposite angle C)
///
/// Algorithm (Law of Cosines):
/// ```text
/// A = acos((b² + c² - a²) / (2·b·c))
/// B = acos((a² + c² - b²) / (2·a·c))
/// C = π - A - B
/// ```
///
/// Domain check: cos argument outside [-1, 1] → triangle inequality violated → Domain error.
///
/// Output to print_buffer: "A=<v>", "B=<v>", "C=<v>" (angles in state.angle_mode).
/// LiftEffect: Disable. Source: HP 00041-90034 p.46 (TRI-01).
pub fn op_tri_sss(state: &mut CalcState) -> Result<(), HpError> {
    let a = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let b = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let c = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    // Domain check: sides must be positive
    if a <= 0.0 || b <= 0.0 || c <= 0.0 {
        return Err(HpError::Domain);
    }

    // Law of Cosines
    let cos_a = (b * b + c * c - a * a) / (2.0 * b * c);
    let cos_b = (a * a + c * c - b * b) / (2.0 * a * c);

    if !(-1.0..=1.0).contains(&cos_a) || !(-1.0..=1.0).contains(&cos_b) {
        return Err(HpError::Domain); // triangle inequality violated
    }

    let angle_a_rad = cos_a.acos();
    let angle_b_rad = cos_b.acos();
    let angle_c_rad = PI - angle_a_rad - angle_b_rad;

    state
        .print_buffer
        .push(format!("A={}", fmt_angle(angle_a_rad, state)));
    state
        .print_buffer
        .push(format!("B={}", fmt_angle(angle_b_rad, state)));
    state
        .print_buffer
        .push(format!("C={}", fmt_angle(angle_c_rad, state)));

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

// ── Op::TriAsa — ASA (Angle-Side-Angle via Law of Sines) ──────────────────────

/// ASA triangle solver: Angle-Side-Angle (A, c, B) → remaining side and angle.
///
/// Stack input:
/// - X = A (first angle, in angle_mode)
/// - Y = c (side between the two angles)
/// - Z = B (second angle, in angle_mode)
///
/// Algorithm (Law of Sines):
/// ```text
/// C = π - A - B
/// a = c · sin(A) / sin(C)
/// b = c · sin(B) / sin(C)
/// ```
///
/// Domain check: A + B ≥ π → invalid angles (no triangle) → Domain error.
///
/// Output to print_buffer: "C=<v>", "a=<v>", "b=<v>".
/// LiftEffect: Disable. Source: HP 00041-90034 p.46 (TRI-02).
pub fn op_tri_asa(state: &mut CalcState) -> Result<(), HpError> {
    let angle_a_user = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let side_c = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let angle_b_user = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    let angle_a_rad = to_radians(angle_a_user, state.angle_mode);
    let angle_b_rad = to_radians(angle_b_user, state.angle_mode);

    if side_c <= 0.0 || angle_a_rad <= 0.0 || angle_b_rad <= 0.0 {
        return Err(HpError::Domain);
    }
    if angle_a_rad + angle_b_rad >= PI {
        return Err(HpError::Domain); // angles sum to ≥ 180°
    }

    let angle_c_rad = PI - angle_a_rad - angle_b_rad;
    let sin_c = angle_c_rad.sin();
    if sin_c.abs() < 1e-14 {
        return Err(HpError::Domain);
    }

    let side_a = side_c * angle_a_rad.sin() / sin_c;
    let side_b = side_c * angle_b_rad.sin() / sin_c;

    state
        .print_buffer
        .push(format!("C={}", fmt_angle(angle_c_rad, state)));
    state
        .print_buffer
        .push(format!("a={}", fmt_side(side_a, state)));
    state
        .print_buffer
        .push(format!("b={}", fmt_side(side_b, state)));

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

// ── Op::TriSaa — SAA (Side-Angle-Angle via Law of Sines) ─────────────────────

/// SAA triangle solver: Side-Angle-Angle (a, A, B) → remaining sides and angle.
///
/// Stack input:
/// - X = a (side, in user units)
/// - Y = A (angle opposite side a, in angle_mode)
/// - Z = B (second known angle, in angle_mode)
///
/// Algorithm (Law of Sines):
/// ```text
/// C = π - A - B
/// b = a · sin(B) / sin(A)
/// c = a · sin(C) / sin(A)
/// ```
///
/// Domain check: A + B ≥ π → Domain. sin(A) ≈ 0 → Domain.
///
/// Output to print_buffer: "C=<v>", "b=<v>", "c=<v>".
/// LiftEffect: Disable. Source: HP 00041-90034 p.46 (TRI-03).
pub fn op_tri_saa(state: &mut CalcState) -> Result<(), HpError> {
    let side_a = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let angle_a_user = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let angle_b_user = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    let angle_a_rad = to_radians(angle_a_user, state.angle_mode);
    let angle_b_rad = to_radians(angle_b_user, state.angle_mode);

    if side_a <= 0.0 || angle_a_rad <= 0.0 || angle_b_rad <= 0.0 {
        return Err(HpError::Domain);
    }
    if angle_a_rad + angle_b_rad >= PI {
        return Err(HpError::Domain);
    }

    let sin_a = angle_a_rad.sin();
    if sin_a.abs() < 1e-14 {
        return Err(HpError::Domain);
    }

    let angle_c_rad = PI - angle_a_rad - angle_b_rad;
    let side_b = side_a * angle_b_rad.sin() / sin_a;
    let side_c = side_a * angle_c_rad.sin() / sin_a;

    state
        .print_buffer
        .push(format!("C={}", fmt_angle(angle_c_rad, state)));
    state
        .print_buffer
        .push(format!("b={}", fmt_side(side_b, state)));
    state
        .print_buffer
        .push(format!("c={}", fmt_side(side_c, state)));

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

// ── Op::TriSas — SAS (Side-Angle-Side via Law of Cosines) ────────────────────

/// SAS triangle solver: Side-Angle-Side (b, A, c) → third side and remaining angles.
///
/// Stack input:
/// - X = b (first side)
/// - Y = A (included angle between b and c, in angle_mode)
/// - Z = c (second side)
///
/// Algorithm (Law of Cosines, then Law of Sines):
/// ```text
/// a² = b² + c² - 2·b·c·cos(A)
/// B = asin(b·sin(A)/a)   [from Law of Sines]
/// C = π - A - B
/// ```
///
/// Domain check: a² ≤ 0 → Domain (degenerate triangle).
///
/// Output to print_buffer: "a=<v>", "B=<v>", "C=<v>".
/// LiftEffect: Disable. Source: HP 00041-90034 p.46 (TRI-04).
pub fn op_tri_sas(state: &mut CalcState) -> Result<(), HpError> {
    let side_b = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let angle_a_user = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let side_c = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    let angle_a_rad = to_radians(angle_a_user, state.angle_mode);

    if side_b <= 0.0 || side_c <= 0.0 || angle_a_rad <= 0.0 || angle_a_rad >= PI {
        return Err(HpError::Domain);
    }

    // Law of Cosines: a² = b² + c² - 2·b·c·cos(A)
    let a_sq = side_b * side_b + side_c * side_c - 2.0 * side_b * side_c * angle_a_rad.cos();
    if a_sq <= 0.0 {
        return Err(HpError::Domain); // degenerate or impossible
    }
    let side_a = a_sq.sqrt();

    // Law of Sines: sin(B) = b·sin(A)/a
    let sin_b = side_b * angle_a_rad.sin() / side_a;
    if !(-1.0..=1.0).contains(&sin_b) {
        return Err(HpError::Domain);
    }
    let angle_b_rad = sin_b.asin();
    let angle_c_rad = PI - angle_a_rad - angle_b_rad;

    state
        .print_buffer
        .push(format!("a={}", fmt_side(side_a, state)));
    state
        .print_buffer
        .push(format!("B={}", fmt_angle(angle_b_rad, state)));
    state
        .print_buffer
        .push(format!("C={}", fmt_angle(angle_c_rad, state)));

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

// ── Op::TriSsa — SSA (Side-Side-Angle — AMBIGUOUS CASE) ──────────────────────

/// SSA triangle solver: Side-Side-Angle (a, b, A) — the ambiguous case (TRI-05).
///
/// Stack input:
/// - X = a (side opposite angle A)
/// - Y = b (second known side)
/// - Z = A (known angle, in angle_mode)
///
/// ## Ambiguous Case Analysis (OM p.46)
///
/// ```text
/// Let h = b · sin(A)  (altitude from vertex C to base side c)
///
/// a < h             → NO SOLUTION
/// a == h            → ONE solution (right triangle, B = 90°)
/// h < a < b         → TWO solutions (ambiguous case — the "SSA ambiguity")
/// a >= b            → ONE solution (unique)
/// ```
///
/// ## Output format (per OM p.46 display sequence, Plan 28-10)
///
/// NO SOLUTION case: pushes "NO SOLUTION" to print_buffer (1 line).
///
/// ONE solution case: pushes 3 lines:
/// "B=<v>", "C=<v>", "c=<v>"
///
/// TWO solutions case: pushes 6 lines (both solutions without separator headers,
/// using B1/C1/c1 and B2/C2/c2 labels to distinguish them per OM notation):
/// "B1=<v>", "C1=<v>", "c1=<v>",
/// "B2=<v>", "C2=<v>", "c2=<v>"
///
/// LiftEffect: Disable. Source: HP 00041-90034 p.46 (TRI-05).
pub fn op_tri_ssa(state: &mut CalcState) -> Result<(), HpError> {
    let side_a = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let side_b = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let angle_a_user = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    let angle_a_rad = to_radians(angle_a_user, state.angle_mode);

    if side_a <= 0.0 || side_b <= 0.0 || angle_a_rad <= 0.0 || angle_a_rad >= PI {
        return Err(HpError::Domain);
    }

    // Height from C to base c (altitude of the triangle)
    let h = side_b * angle_a_rad.sin();

    if side_a < h - 1e-10 {
        // NO SOLUTION: side a is too short to reach the base
        state.print_buffer.push("NO SOLUTION".to_string());
    } else if (side_a - h).abs() < 1e-10 {
        // ONE solution: right triangle (B = 90°)
        let angle_b_rad = PI / 2.0;
        let angle_c_rad = PI - angle_a_rad - angle_b_rad;
        let side_c = side_b * angle_c_rad.sin() / angle_a_rad.sin();
        state
            .print_buffer
            .push(format!("B={}", fmt_angle(angle_b_rad, state)));
        state
            .print_buffer
            .push(format!("C={}", fmt_angle(angle_c_rad, state)));
        state
            .print_buffer
            .push(format!("c={}", fmt_side(side_c, state)));
    } else if side_a < side_b {
        // TWO solutions: h < a < b (ambiguous case)
        // Solution 1: acute angle B1
        let sin_b = side_b * angle_a_rad.sin() / side_a;
        if !(-1.0..=1.0).contains(&sin_b) {
            return Err(HpError::Domain);
        }
        let angle_b1_rad = sin_b.asin(); // acute B
        let angle_c1_rad = PI - angle_a_rad - angle_b1_rad;
        let side_c1 = side_a * angle_c1_rad.sin() / angle_a_rad.sin();

        // Solution 2: obtuse angle B2 = π - B1
        let angle_b2_rad = PI - angle_b1_rad;
        let angle_c2_rad = PI - angle_a_rad - angle_b2_rad;
        let side_c2 = side_a * angle_c2_rad.sin() / angle_a_rad.sin();

        // Output: both solutions with B1/C1/c1 and B2/C2/c2 labels (OM p.46 style)
        state
            .print_buffer
            .push(format!("B1={}", fmt_angle(angle_b1_rad, state)));
        state
            .print_buffer
            .push(format!("C1={}", fmt_angle(angle_c1_rad, state)));
        state
            .print_buffer
            .push(format!("c1={}", fmt_side(side_c1, state)));
        state
            .print_buffer
            .push(format!("B2={}", fmt_angle(angle_b2_rad, state)));
        state
            .print_buffer
            .push(format!("C2={}", fmt_angle(angle_c2_rad, state)));
        state
            .print_buffer
            .push(format!("c2={}", fmt_side(side_c2, state)));
    } else {
        // ONE solution: a >= b (unique solution)
        let sin_b = side_b * angle_a_rad.sin() / side_a;
        if !(-1.0..=1.0).contains(&sin_b) {
            return Err(HpError::Domain);
        }
        let angle_b_rad = sin_b.asin();
        let angle_c_rad = PI - angle_a_rad - angle_b_rad;
        let side_c = side_a * angle_c_rad.sin() / angle_a_rad.sin();
        state
            .print_buffer
            .push(format!("B={}", fmt_angle(angle_b_rad, state)));
        state
            .print_buffer
            .push(format!("C={}", fmt_angle(angle_c_rad, state)));
        state
            .print_buffer
            .push(format!("c={}", fmt_side(side_c, state)));
    }

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::{AngleMode, CalcState, DisplayMode};
    use rust_decimal::Decimal;

    const TOLERANCE: f64 = 1e-5;

    fn approx_eq(actual: f64, expected: f64, tol: f64) -> bool {
        (actual - expected).abs() < tol
    }

    /// Parse a formatted number from print_buffer label (e.g., "A=45.0000" → 45.0)
    fn parse_line(line: &str) -> f64 {
        line.split('=')
            .nth(1)
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap()
    }

    fn set_stack_xyz(state: &mut CalcState, x: f64, y: f64, z: f64) {
        state.stack.x = HpNum::rounded(Decimal::from_f64(x).unwrap());
        state.stack.y = HpNum::rounded(Decimal::from_f64(y).unwrap());
        state.stack.z = HpNum::rounded(Decimal::from_f64(z).unwrap());
    }

    // ── SSS Tests ────────────────────────────────────────────────────────────

    // Catches: SSS equilateral triangle wrong (all angles should be 60°)
    // Source: equilateral identity — all angles equal 60°.
    #[test]
    fn sss_equilateral() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=a=1, Y=b=1, Z=c=1
        set_stack_xyz(&mut state, 1.0, 1.0, 1.0);
        op_tri_sss(&mut state).unwrap();
        assert_eq!(state.print_buffer.len(), 3);
        let angle_a = parse_line(&state.print_buffer[0]);
        let angle_b = parse_line(&state.print_buffer[1]);
        let angle_c = parse_line(&state.print_buffer[2]);
        assert!(
            approx_eq(angle_a, 60.0, 0.01),
            "A should be 60°, got {angle_a}"
        );
        assert!(
            approx_eq(angle_b, 60.0, 0.01),
            "B should be 60°, got {angle_b}"
        );
        assert!(
            approx_eq(angle_c, 60.0, 0.01),
            "C should be 60°, got {angle_c}"
        );
    }

    // Catches: SSS right triangle angles wrong (3-4-5)
    // Source: Pythagorean triple 3-4-5: A = acos(4²+5²-3²)/(2·4·5) = acos(32/40) ≈ 36.87°
    #[test]
    fn sss_right_triangle() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=a=3, Y=b=4, Z=c=5 → A≈36.87°, B≈53.13°, C=90°
        set_stack_xyz(&mut state, 3.0, 4.0, 5.0);
        op_tri_sss(&mut state).unwrap();
        let angle_a = parse_line(&state.print_buffer[0]);
        let angle_b = parse_line(&state.print_buffer[1]);
        let angle_c = parse_line(&state.print_buffer[2]);
        assert!(approx_eq(angle_a, 36.8699, 0.01), "A≈36.87°, got {angle_a}");
        assert!(approx_eq(angle_b, 53.1301, 0.01), "B≈53.13°, got {angle_b}");
        assert!(approx_eq(angle_c, 90.0, 0.01), "C=90°, got {angle_c}");
    }

    // Catches: SSS triangle inequality violation not detected
    #[test]
    fn sss_triangle_inequality_violation() {
        let mut state = CalcState::new();
        // a=10, b=2, c=2 violates triangle inequality (10 > 2+2)
        set_stack_xyz(&mut state, 10.0, 2.0, 2.0);
        let result = op_tri_sss(&mut state);
        assert!(
            result.is_err(),
            "triangle inequality violation should return Domain error"
        );
    }

    // Catches: SSS print_buffer count wrong (must be exactly 3 lines)
    #[test]
    fn sss_output_three_lines() {
        let mut state = CalcState::new();
        set_stack_xyz(&mut state, 3.0, 4.0, 5.0);
        op_tri_sss(&mut state).unwrap();
        assert_eq!(state.print_buffer.len(), 3, "SSS must push exactly 3 lines");
    }

    // Catches: SSS angle_mode Rad not respected
    #[test]
    fn sss_angle_mode_radians() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Rad;
        state.display_mode = DisplayMode::Fix(6);
        set_stack_xyz(&mut state, 1.0, 1.0, 1.0);
        op_tri_sss(&mut state).unwrap();
        // All angles should be π/3 ≈ 1.047198 rad
        let angle_a = parse_line(&state.print_buffer[0]);
        assert!(
            approx_eq(angle_a, PI / 3.0, 0.001),
            "A in radians should be π/3 ≈ 1.047, got {angle_a}"
        );
    }

    // Catches: SSS angle_mode Grad not respected
    #[test]
    fn sss_angle_mode_gradians() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Grad;
        state.display_mode = DisplayMode::Fix(4);
        set_stack_xyz(&mut state, 1.0, 1.0, 1.0);
        op_tri_sss(&mut state).unwrap();
        // All angles should be 66.6667 grad (= 60° in gradians)
        let angle_a = parse_line(&state.print_buffer[0]);
        assert!(
            approx_eq(angle_a, 200.0 / 3.0, 0.1),
            "A in gradians should be ≈66.67 grad, got {angle_a}"
        );
    }

    // ── ASA Tests ────────────────────────────────────────────────────────────

    // Catches: ASA equilateral triangle (A=60°, c=10, B=60° → all 60°, a=b=10)
    #[test]
    fn asa_equilateral() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=A=60°, Y=c=10, Z=B=60°
        set_stack_xyz(&mut state, 60.0, 10.0, 60.0);
        op_tri_asa(&mut state).unwrap();
        let angle_c = parse_line(&state.print_buffer[0]);
        let side_a = parse_line(&state.print_buffer[1]);
        let side_b = parse_line(&state.print_buffer[2]);
        assert!(
            approx_eq(angle_c, 60.0, 0.01),
            "C should be 60°, got {angle_c}"
        );
        assert!(
            approx_eq(side_a, 10.0, 0.01),
            "a should be 10, got {side_a}"
        );
        assert!(
            approx_eq(side_b, 10.0, 0.01),
            "b should be 10, got {side_b}"
        );
    }

    // Catches: ASA angles summing to ≥ 180° not detected
    #[test]
    fn asa_invalid_angles_sum() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        // A=120°, c=1, B=90° → A+B=210° > 180° → Domain
        set_stack_xyz(&mut state, 120.0, 1.0, 90.0);
        let result = op_tri_asa(&mut state);
        assert!(result.is_err(), "A+B ≥ 180° should return Domain error");
    }

    // Catches: ASA output lines count wrong
    #[test]
    fn asa_output_three_lines() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 60.0, 10.0, 60.0);
        op_tri_asa(&mut state).unwrap();
        assert_eq!(state.print_buffer.len(), 3, "ASA must push exactly 3 lines");
    }

    // Catches: ASA 30-60-90 triangle wrong
    // A=30°, c=2, B=60° → C=90°, a=1, b=√3≈1.7321
    #[test]
    fn asa_30_60_90() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        set_stack_xyz(&mut state, 30.0, 2.0, 60.0);
        op_tri_asa(&mut state).unwrap();
        let angle_c = parse_line(&state.print_buffer[0]);
        let side_a = parse_line(&state.print_buffer[1]);
        let side_b = parse_line(&state.print_buffer[2]);
        assert!(
            approx_eq(angle_c, 90.0, 0.01),
            "C should be 90°, got {angle_c}"
        );
        assert!(approx_eq(side_a, 1.0, 0.01), "a should be 1, got {side_a}");
        assert!(
            approx_eq(side_b, 3.0_f64.sqrt(), 0.01),
            "b should be √3, got {side_b}"
        );
    }

    // Catches: ASA zero side error not detected
    #[test]
    fn asa_zero_side_error() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 60.0, 0.0, 60.0);
        let result = op_tri_asa(&mut state);
        assert!(result.is_err(), "zero side should return Domain error");
    }

    // ── SAA Tests ────────────────────────────────────────────────────────────

    // Catches: SAA basic case wrong (a=10, A=30°, B=60° → C=90°, b≈17.32, c=20)
    // Source: Law of Sines: b/sin(B) = a/sin(A) → b = 10·sin(60°)/sin(30°) = 10·√3/0.5 ≈ 17.32
    #[test]
    fn saa_basic() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=a=10, Y=A=30°, Z=B=60°
        set_stack_xyz(&mut state, 10.0, 30.0, 60.0);
        op_tri_saa(&mut state).unwrap();
        let angle_c = parse_line(&state.print_buffer[0]);
        let side_b = parse_line(&state.print_buffer[1]);
        let side_c = parse_line(&state.print_buffer[2]);
        assert!(
            approx_eq(angle_c, 90.0, 0.01),
            "C should be 90°, got {angle_c}"
        );
        assert!(
            approx_eq(side_b, 10.0 * 3.0_f64.sqrt(), 0.1),
            "b should be ≈17.32, got {side_b}"
        );
        assert!(approx_eq(side_c, 20.0, 0.1), "c should be 20, got {side_c}");
    }

    // Catches: SAA output lines count wrong
    #[test]
    fn saa_output_three_lines() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 10.0, 30.0, 60.0);
        op_tri_saa(&mut state).unwrap();
        assert_eq!(state.print_buffer.len(), 3, "SAA must push exactly 3 lines");
    }

    // Catches: SAA angle sum error not detected
    #[test]
    fn saa_invalid_angle_sum() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 10.0, 100.0, 100.0);
        let result = op_tri_saa(&mut state);
        assert!(result.is_err(), "A+B≥180° should be Domain error");
    }

    // Catches: SAA equilateral case wrong
    #[test]
    fn saa_equilateral() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // a=5, A=60°, B=60° → C=60°, b=5, c=5
        set_stack_xyz(&mut state, 5.0, 60.0, 60.0);
        op_tri_saa(&mut state).unwrap();
        let angle_c = parse_line(&state.print_buffer[0]);
        let side_b = parse_line(&state.print_buffer[1]);
        let side_c = parse_line(&state.print_buffer[2]);
        assert!(
            approx_eq(angle_c, 60.0, 0.01),
            "C should be 60°, got {angle_c}"
        );
        assert!(approx_eq(side_b, 5.0, 0.01), "b should be 5, got {side_b}");
        assert!(approx_eq(side_c, 5.0, 0.01), "c should be 5, got {side_c}");
    }

    // Catches: SAA zero side error
    #[test]
    fn saa_zero_side_error() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 0.0, 30.0, 60.0);
        let result = op_tri_saa(&mut state);
        assert!(result.is_err());
    }

    // ── SAS Tests ────────────────────────────────────────────────────────────

    // Catches: SAS basic case wrong (b=3, A=60°, c=4 → a²=9+16-24·0.5=13, a≈3.606)
    // Source: Law of Cosines: a² = b² + c² - 2bc·cos(A)
    #[test]
    fn sas_basic() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=b=3, Y=A=60°, Z=c=4
        set_stack_xyz(&mut state, 3.0, 60.0, 4.0);
        op_tri_sas(&mut state).unwrap();
        let side_a = parse_line(&state.print_buffer[0]);
        assert!(
            approx_eq(side_a, 13.0_f64.sqrt(), 0.01),
            "a should be √13 ≈ 3.606, got {side_a}"
        );
    }

    // Catches: SAS output lines count wrong
    #[test]
    fn sas_output_three_lines() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 3.0, 60.0, 4.0);
        op_tri_sas(&mut state).unwrap();
        assert_eq!(state.print_buffer.len(), 3, "SAS must push exactly 3 lines");
    }

    // Catches: SAS with A=90° (Pythagorean result)
    // b=3, A=90°, c=4 → a=5, B=asin(3/5)≈36.87°, C=asin(4/5)≈53.13°
    #[test]
    fn sas_right_angle() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        set_stack_xyz(&mut state, 3.0, 90.0, 4.0);
        op_tri_sas(&mut state).unwrap();
        let side_a = parse_line(&state.print_buffer[0]);
        assert!(
            approx_eq(side_a, 5.0, 0.01),
            "a should be 5 (Pythagorean), got {side_a}"
        );
    }

    // Catches: SAS zero side error
    #[test]
    fn sas_zero_side_error() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 0.0, 60.0, 4.0);
        let result = op_tri_sas(&mut state);
        assert!(result.is_err());
    }

    // Catches: SAS invalid angle (A >= 180°)
    #[test]
    fn sas_invalid_angle() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 3.0, 180.0, 4.0);
        let result = op_tri_sas(&mut state);
        assert!(result.is_err());
    }

    // ── SSA Tests (TRI-05 — AMBIGUOUS CASE) ──────────────────────────────────

    // Catches: SSA unique solution (a >= b case) not computed correctly
    // a=10, b=5, A=30° → unique solution (a > b)
    #[test]
    fn ssa_single_solution_a_gt_b() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=a=10, Y=b=5, Z=A=30°
        set_stack_xyz(&mut state, 10.0, 5.0, 30.0);
        op_tri_ssa(&mut state).unwrap();
        // One solution: B, C, c
        assert_eq!(
            state.print_buffer.len(),
            3,
            "unique SSA: must push exactly 3 lines"
        );
        // sin(B) = b·sin(A)/a = 5·0.5/10 = 0.25 → B≈14.48°
        let angle_b = parse_line(&state.print_buffer[0]);
        assert!(
            approx_eq(angle_b, 14.4775, 0.1),
            "B should be ≈14.48°, got {angle_b}"
        );
    }

    // Catches: SSA no-solution case not detected (TRI-05 / RESEARCH line 605)
    // a=1, b=5, A=30° → h = 5·sin(30°) = 2.5 > a=1 → NO SOLUTION
    #[test]
    fn ssa_no_solution() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        // X=a=1, Y=b=5, Z=A=30°
        set_stack_xyz(&mut state, 1.0, 5.0, 30.0);
        op_tri_ssa(&mut state).unwrap();
        assert_eq!(
            state.print_buffer.len(),
            1,
            "no-solution SSA: must push exactly 1 line"
        );
        assert_eq!(state.print_buffer[0], "NO SOLUTION");
    }

    // Catches: SSA ambiguous two-solution case not computed (TRI-05 — PRIMARY REQUIREMENT)
    // a=5, b=8, A=30° → h = 8·sin(30°) = 4.0, h < a=5 < b=8 → TWO solutions
    // Source: HP-41C Math Pac I OM p.46 (TRI-05 worked example — RESEARCH line 605).
    #[test]
    fn ssa_ambiguous_two_solutions() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=a=5, Y=b=8, Z=A=30°
        set_stack_xyz(&mut state, 5.0, 8.0, 30.0);
        op_tri_ssa(&mut state).unwrap();
        // Must push exactly 6 lines (B1/C1/c1 + B2/C2/c2)
        assert_eq!(
            state.print_buffer.len(),
            6,
            "ambiguous SSA: must push exactly 6 lines (both solutions)"
        );
        // Verify B1 (acute): sin(B1) = b·sin(A)/a = 8·0.5/5 = 0.8 → B1 = asin(0.8) ≈ 53.13°
        let b1_line = &state.print_buffer[0];
        assert!(
            b1_line.starts_with("B1="),
            "first line should start with B1=, got {b1_line}"
        );
        let angle_b1 = parse_line(b1_line);
        assert!(
            approx_eq(angle_b1, 53.13, 0.1),
            "B1 should be ≈53.13° (asin(0.8)), got {angle_b1}"
        );
        // Verify B2 (obtuse): B2 = 180° - B1 ≈ 126.87°
        let b2_line = &state.print_buffer[3];
        assert!(
            b2_line.starts_with("B2="),
            "fourth line should start with B2=, got {b2_line}"
        );
        let angle_b2 = parse_line(b2_line);
        assert!(
            approx_eq(angle_b2, 126.87, 0.1),
            "B2 should be ≈126.87° (180°-B1), got {angle_b2}"
        );
    }

    // Catches: SSA right-triangle edge case (a == h) wrong
    // a=4, b=8, A=30° → h = 8·sin(30°) = 4.0 = a → ONE solution (right triangle, B=90°)
    #[test]
    fn ssa_right_triangle_edge() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(4);
        // X=a=4, Y=b=8, Z=A=30°
        set_stack_xyz(&mut state, 4.0, 8.0, 30.0);
        op_tri_ssa(&mut state).unwrap();
        // ONE solution: right triangle (B=90°)
        assert_eq!(
            state.print_buffer.len(),
            3,
            "right-triangle edge SSA: must push exactly 3 lines"
        );
        let angle_b = parse_line(&state.print_buffer[0]);
        assert!(
            approx_eq(angle_b, 90.0, 0.1),
            "B should be 90° for right-triangle edge case, got {angle_b}"
        );
    }

    // Catches: SSA output line labels wrong (must use B1/C1/c1 + B2/C2/c2 for 2-solution case)
    #[test]
    fn ssa_two_solution_labels() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 5.0, 8.0, 30.0);
        op_tri_ssa(&mut state).unwrap();
        assert!(state.print_buffer[0].starts_with("B1="));
        assert!(state.print_buffer[1].starts_with("C1="));
        assert!(state.print_buffer[2].starts_with("c1="));
        assert!(state.print_buffer[3].starts_with("B2="));
        assert!(state.print_buffer[4].starts_with("C2="));
        assert!(state.print_buffer[5].starts_with("c2="));
    }

    // Catches: SSA zero inputs not rejected
    #[test]
    fn ssa_zero_inputs_rejected() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        set_stack_xyz(&mut state, 0.0, 5.0, 30.0);
        assert!(op_tri_ssa(&mut state).is_err());
    }
}
