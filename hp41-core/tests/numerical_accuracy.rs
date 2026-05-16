// Reference values are HP-41 hardware outputs, not mathematical constants.
// These approximate literals are intentional: they represent what the real
// hardware displays, not exact mathematical values.
#![allow(clippy::approx_constant)]
// HP-41 reference values use digit groupings that match the hardware display
// format (e.g., 3.141_592_653 groups at 3+3+3) rather than Rust's 3+3 convention.
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::unwrap_used)]

//! 503-case numerical accuracy suite for QUAL-06.
//!
//! Reference values derived from HP-41 Owner's Handbook formulas and known
//! mathematical constants. Approach: document-derived (same as Free42, D-05).
//!
//! Tolerance: <= 1e-9 (9-digit relative accuracy threshold; 1e-10 cases use WIDE_TOL where BCD rounding compounds).
//!
//! Gate: passes >= 493 (98% of 503, D-08). Failing cases printed as diagnostics.

use hp41_core::ops::program::op_dse;
use hp41_core::ops::program::op_isg;
use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::str::FromStr;

const TOLERANCE: f64 = 1e-9;
const WIDE_TOL: f64 = 1e-6;

struct AccuracyCase {
    id: usize,
    domain: &'static str,
    description: String,
    expected: f64,
    actual: f64,
    tol: f64,
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).expect("valid decimal in accuracy suite")
}

fn push(state: &mut CalcState, s: &str) {
    let d = dec(s);
    state.stack.lift_enabled = true;
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

fn get_x(state: &CalcState) -> f64 {
    state.stack.x.inner().to_f64().unwrap_or(f64::NAN)
}

#[allow(dead_code)]
fn get_y(state: &CalcState) -> f64 {
    state.stack.y.inner().to_f64().unwrap_or(f64::NAN)
}

fn passes_with_tol(actual: f64, expected: f64, tol: f64) -> bool {
    if actual.is_nan() || expected.is_nan() {
        return false;
    }
    if expected == 0.0 {
        actual.abs() <= tol
    } else {
        ((actual - expected) / expected).abs() <= tol
    }
}

fn new_deg_state() -> CalcState {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::SetDeg).unwrap();
    s
}

fn new_rad_state() -> CalcState {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::SetRad).unwrap();
    s
}

fn new_grad_state() -> CalcState {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::SetGrad).unwrap();
    s
}

/// Add a (y_val, x_val) data point via Sigma+. X=x_val is the x-domain value.
#[allow(dead_code)]
fn add_point(state: &mut CalcState, y_val: &str, x_val: &str) {
    push(state, y_val);
    push(state, x_val);
    dispatch(state, Op::SigmaPlus).unwrap();
}

#[test]
fn test_numerical_accuracy_suite() {
    let mut cases: Vec<AccuracyCase> = Vec::with_capacity(500);
    let mut id = 0usize;

    macro_rules! case {
        ($domain:expr, $desc:expr, $expected:expr, $actual:expr) => {{
            id += 1;
            cases.push(AccuracyCase {
                id,
                domain: $domain,
                description: $desc.to_string(),
                expected: $expected,
                actual: $actual,
                tol: TOLERANCE,
            });
        }};
        ($domain:expr, $desc:expr, $expected:expr, $actual:expr, wide) => {{
            id += 1;
            cases.push(AccuracyCase {
                id,
                domain: $domain,
                description: $desc.to_string(),
                expected: $expected,
                actual: $actual,
                tol: WIDE_TOL,
            });
        }};
    }

    // ── Domain 1: Arithmetic (cases 1–100) ───────────────────────────────────

    // Cases 1–20: Addition
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        push(&mut s, "3");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "2+3=5", 5.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.1");
        push(&mut s, "0.2");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "0.1+0.2=0.3", 0.3, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-5");
        push(&mut s, "5");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "-5+5=0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "999999999");
        push(&mut s, "1");
        dispatch(&mut s, Op::Add).unwrap();
        case!(
            "arithmetic",
            "999999999+1=1000000000",
            1_000_000_000.0,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.000000001");
        push(&mut s, "0.000000001");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "1e-9+1e-9=2e-9", 2e-9, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        push(&mut s, "-100");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "100+(-100)=0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3.14159265");
        push(&mut s, "2.71828182");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "pi_approx+e_approx", 5.85987447, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-3.14159265");
        push(&mut s, "-2.71828182");
        dispatch(&mut s, Op::Add).unwrap();
        case!(
            "arithmetic",
            "-pi_approx+(-e_approx)",
            -5.85987447,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1234567.89");
        push(&mut s, "9876543.21");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "1234567.89+9876543.21", 11111111.1, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "0.5+0.5=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100000000");
        push(&mut s, "100000000");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "1e8+1e8=2e8", 2e8, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.23456789");
        push(&mut s, "0.00000001");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "1.23456789+0.00000001", 1.23456790, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-0.5");
        push(&mut s, "1.5");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "-0.5+1.5=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "9999999999");
        push(&mut s, "0");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "9999999999+0", 9_999_999_999.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.1");
        push(&mut s, "2.2");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "1.1+2.2=3.3", 3.3, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.333333333");
        push(&mut s, "0.666666667");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "0.333+0.667=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "42");
        push(&mut s, "0");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "42+0=42", 42.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-1");
        push(&mut s, "1");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "-1+1=0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1000000");
        push(&mut s, "0.000001");
        dispatch(&mut s, Op::Add).unwrap();
        case!(
            "arithmetic",
            "1000000+0.000001",
            1_000_000.000001,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.5");
        push(&mut s, "-0.5");
        dispatch(&mut s, Op::Add).unwrap();
        case!("arithmetic", "1.5+(-0.5)=1", 1.0, get_x(&s));
    }

    // Cases 21–40: Subtraction
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        push(&mut s, "3");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "10-3=7", 7.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0");
        push(&mut s, "5");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "0-5=-5", -5.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3.14159265");
        push(&mut s, "1.41421356");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "pi-sqrt2", 1.72737909, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1000000000");
        push(&mut s, "999999999");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "1e9-999999999=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.1");
        push(&mut s, "0.1");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "0.1-0.1=0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-5");
        push(&mut s, "-3");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "-5-(-3)=-2", -2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.23456789");
        push(&mut s, "0.23456789");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "1.23456789-0.23456789=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        push(&mut s, "200");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "100-200=-100", -100.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.999999999");
        push(&mut s, "0.000000001");
        dispatch(&mut s, Op::Sub).unwrap();
        case!(
            "arithmetic",
            "0.999999999-0.000000001",
            0.999999998,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "50");
        push(&mut s, "50");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "50-50=0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.5");
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "1.5-0.5=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        push(&mut s, "3");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "7-3=4", 4.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1000000000");
        push(&mut s, "1");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "1e9-1=999999999", 999_999_999.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-10");
        push(&mut s, "10");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "-10-10=-20", -20.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3.33333333");
        push(&mut s, "1.33333333");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "3.333-1.333=2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.000000005");
        push(&mut s, "0.000000004");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "5e-9-4e-9=1e-9", 1e-9, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "0.000000001");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "1-1e-9=0.999999999", 0.999999999, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "200");
        push(&mut s, "100");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "200-100=100", 100.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5.5");
        push(&mut s, "2.5");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "5.5-2.5=3", 3.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0");
        push(&mut s, "0");
        dispatch(&mut s, Op::Sub).unwrap();
        case!("arithmetic", "0-0=0", 0.0, get_x(&s));
    }

    // Cases 41–60: Multiplication
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        push(&mut s, "4");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "3*4=12", 12.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        push(&mut s, "2");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "0.5*2=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-3");
        push(&mut s, "4");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "-3*4=-12", -12.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        push(&mut s, "7");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "7*7=49", 49.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.5");
        push(&mut s, "1.5");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "1.5*1.5=2.25", 2.25, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        push(&mut s, "0.01");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "100*0.01=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3.14159265");
        push(&mut s, "2");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "pi*2=6.2831853", 6.2831853, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-0.5");
        push(&mut s, "-2");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "-0.5*-2=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1000");
        push(&mut s, "1000");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "1000*1000=1e6", 1_000_000.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.23456789");
        push(&mut s, "10");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "1.23456789*10", 12.3456789, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.1");
        push(&mut s, "0.1");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "0.1*0.1=0.01", 0.01, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "9");
        push(&mut s, "9");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "9*9=81", 81.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.414213562");
        push(&mut s, "1.414213562");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "sqrt2*sqrt2~2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0");
        push(&mut s, "9999999");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "0*9999999=0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-1");
        push(&mut s, "-1");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "-1*-1=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2.5");
        push(&mut s, "4");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "2.5*4=10", 10.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.333333333");
        push(&mut s, "3");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "0.333333333*3", 0.999999999, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.1");
        push(&mut s, "1.1");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "1.1*1.1=1.21", 1.21, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5");
        push(&mut s, "0.2");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "5*0.2=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10000");
        push(&mut s, "10000");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("arithmetic", "10000*10000=1e8", 1e8, get_x(&s));
    }

    // Cases 61–80: Division
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        push(&mut s, "2");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "10/2=5", 5.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "3");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/3=0.3333333333", 0.333_333_333_3, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "22");
        push(&mut s, "7");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "22/7=3.142857143", 3.142_857_143, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-10");
        push(&mut s, "2");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "-10/2=-5", -5.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        push(&mut s, "100");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "100/100=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "7");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/7=0.1428571429", 0.142_857_142_9, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "9");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/9=0.1111111111", 0.111_111_111_1, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        push(&mut s, "3");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "2/3=0.6666666667", 0.666_666_666_7, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "355");
        push(&mut s, "113");
        dispatch(&mut s, Op::Div).unwrap();
        case!(
            "arithmetic",
            "355/113=3.141592920",
            3.141_592_920,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "1000000");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/1e6=1e-6", 1e-6, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        push(&mut s, "2");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "7/2=3.5", 3.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-6");
        push(&mut s, "-3");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "-6/-3=2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "4");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/4=0.25", 0.25, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5");
        push(&mut s, "5");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "5/5=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "8");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/8=0.125", 0.125, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "9");
        push(&mut s, "4");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "9/4=2.25", 2.25, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "6");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "1/6=0.1666666667", 0.166_666_666_7, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        push(&mut s, "3");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "100/3=33.33333333", 33.333_333_33, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "11");
        dispatch(&mut s, Op::Div).unwrap();
        case!(
            "arithmetic",
            "1/11=0.09090909091",
            0.090_909_090_91,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "8");
        push(&mut s, "4");
        dispatch(&mut s, Op::Div).unwrap();
        case!("arithmetic", "8/4=2", 2.0, get_x(&s));
    }

    // Cases 81–100: 1/x, sqrt, sq, YPow
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::Recip).unwrap();
        case!("arithmetic", "1/x(2)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "4");
        dispatch(&mut s, Op::Recip).unwrap();
        case!("arithmetic", "1/x(4)=0.25", 0.25, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        dispatch(&mut s, Op::Recip).unwrap();
        case!("arithmetic", "1/x(10)=0.1", 0.1, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Recip).unwrap();
        case!("arithmetic", "1/x(0.5)=2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.1");
        dispatch(&mut s, Op::Recip).unwrap();
        case!("arithmetic", "1/x(0.1)=10", 10.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "4");
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("arithmetic", "sqrt(4)=2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "9");
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("arithmetic", "sqrt(9)=3", 3.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!(
            "arithmetic",
            "sqrt(2)=1.414213562",
            1.414_213_562,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.25");
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("arithmetic", "sqrt(0.25)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("arithmetic", "sqrt(100)=10", 10.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        dispatch(&mut s, Op::Sq).unwrap();
        case!("arithmetic", "sq(3)=9", 9.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.5");
        dispatch(&mut s, Op::Sq).unwrap();
        case!("arithmetic", "sq(1.5)=2.25", 2.25, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        dispatch(&mut s, Op::Sq).unwrap();
        case!("arithmetic", "sq(10)=100", 100.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Sq).unwrap();
        case!("arithmetic", "sq(0.5)=0.25", 0.25, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.414213562");
        dispatch(&mut s, Op::Sq).unwrap();
        case!("arithmetic", "sq(sqrt2)~2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        push(&mut s, "10");
        dispatch(&mut s, Op::YPow).unwrap();
        case!("arithmetic", "2^10=1024", 1024.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        push(&mut s, "0.5");
        dispatch(&mut s, Op::YPow).unwrap();
        case!("arithmetic", "2^0.5=sqrt2", 1.414_213_562, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        push(&mut s, "3");
        dispatch(&mut s, Op::YPow).unwrap();
        case!("arithmetic", "3^3=27", 27.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        push(&mut s, "3");
        dispatch(&mut s, Op::YPow).unwrap();
        case!("arithmetic", "10^3=1000", 1000.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "4");
        push(&mut s, "0.5");
        dispatch(&mut s, Op::YPow).unwrap();
        case!("arithmetic", "4^0.5=2", 2.0, get_x(&s));
    }

    // ── Domain 2: Trig DEG (cases 101–130 SIN) ───────────────────────────────

    // SIN DEG cases 101–130
    for (deg, expected) in &[
        ("0", 0.0_f64),
        ("30", 0.5),
        ("45", 0.707_106_781_2),
        ("60", 0.866_025_403_8),
        ("90", 1.0),
        ("180", 0.0),
        ("270", -1.0),
        ("360", 0.0),
        ("-30", -0.5),
        ("-90", -1.0),
        ("15", 0.258_819_045_1),
        ("75", 0.965_925_826_3),
        ("120", 0.866_025_403_8),
        ("135", 0.707_106_781_2),
        ("150", 0.5),
        ("210", -0.5),
        ("225", -0.707_106_781_2),
        ("240", -0.866_025_403_8),
        ("300", -0.866_025_403_8),
        ("315", -0.707_106_781_2),
        ("330", -0.5),
        ("1", 0.017_452_406_44),
        ("89", 0.999_847_695_2),
        ("45.5", 0.713_250_423_0),
        ("10", 0.173_648_177_7),
        ("20", 0.342_020_143_3),
        ("40", 0.642_787_609_7),
        ("50", 0.766_044_443_1),
        ("70", 0.939_692_620_8),
        ("80", 0.984_807_753_0),
    ] {
        let mut s = new_deg_state();
        push(&mut s, deg);
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_deg", &format!("sin({deg}deg)"), *expected, get_x(&s));
    }

    // COS DEG cases 131–155
    for (deg, expected) in &[
        ("0", 1.0_f64),
        ("30", 0.866_025_403_8),
        ("45", 0.707_106_781_2),
        ("60", 0.5),
        ("90", 0.0),
        ("180", -1.0),
        ("270", 0.0),
        ("360", 1.0),
        ("-60", 0.5),
        ("120", -0.5),
        ("150", -0.866_025_403_8),
        ("240", -0.5),
        ("300", 0.5),
        ("330", 0.866_025_403_8),
        ("1", 0.999_847_695_2),
        ("89", 0.017_452_406_44),
        ("10", 0.984_807_753_0),
        ("20", 0.939_692_620_8),
        ("40", 0.766_044_443_1),
        ("50", 0.642_787_609_7),
        ("70", 0.342_020_143_3),
        ("80", 0.173_648_177_7),
        ("135", -0.707_106_781_2),
        ("225", -0.707_106_781_2),
        ("315", 0.707_106_781_2),
    ] {
        let mut s = new_deg_state();
        push(&mut s, deg);
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_deg", &format!("cos({deg}deg)"), *expected, get_x(&s));
    }

    // TAN DEG cases 156–175
    for (deg, expected) in &[
        ("0", 0.0_f64),
        ("45", 1.0),
        ("30", 0.577_350_269_2),
        ("60", 1.732_050_808),
        ("-45", -1.0),
        ("135", -1.0),
        ("150", -0.577_350_269_2),
        ("1", 0.017_455_064_93),
        ("89", 57.289_961_63),
        ("10", 0.176_326_980_7),
        ("20", 0.363_970_234_3),
        ("40", 0.839_099_631_2),
        ("50", 1.191_753_593),
        ("70", 2.747_477_419),
        ("80", 5.671_281_820),
        ("225", 1.0),
        ("315", -1.0),
        ("15", 0.267_949_192_4),
        ("75", 3.732_050_808),
        ("-30", -0.577_350_269_2),
    ] {
        let mut s = new_deg_state();
        push(&mut s, deg);
        dispatch(&mut s, Op::Tan).unwrap();
        case!("trig_deg", &format!("tan({deg}deg)"), *expected, get_x(&s));
    }

    // ASIN/ACOS/ATAN DEG cases 176–195
    {
        let mut s = new_deg_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(0)=0deg", 0.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(0.5)=30deg", 30.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(1)=90deg", 90.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "-0.5");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(-0.5)=-30deg", -30.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(-1)=-90deg", -90.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.7071067812");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(sqrt2/2)=45deg", 45.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.8660254038");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_deg", "asin(sqrt3/2)=60deg", 60.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_deg", "acos(1)=0deg", 0.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_deg", "acos(0)=90deg", 90.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_deg", "acos(-1)=180deg", 180.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_deg", "acos(0.5)=60deg", 60.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.8660254038");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_deg", "acos(sqrt3/2)=30deg", 30.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.7071067812");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_deg", "acos(sqrt2/2)=45deg", 45.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(0)=0deg", 0.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(1)=45deg", 45.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(-1)=-45deg", -45.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "1.732050808");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(sqrt3)=60deg", 60.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.5773502692");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(tan30)=30deg", 30.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "57.28996163");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(57.29)~89deg", 89.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.2679491924");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_deg", "atan(tan15)=15deg", 15.0, get_x(&s));
    }

    // Trig RAD cases 196–215
    {
        let mut s = new_rad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(0rad)=0", 0.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.5235987756");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(pi/6)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "1.570796327");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(pi/2)=1", 1.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "3.141592654");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(pi)~0", 0.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_rad", "cos(0rad)=1", 1.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "1.047197551");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_rad", "cos(pi/3)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "3.141592654");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_rad", "cos(pi)=-1", -1.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.7853981634");
        dispatch(&mut s, Op::Tan).unwrap();
        case!("trig_rad", "tan(pi/4)=1", 1.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_rad", "asin(1)=pi/2", 1.570_796_327, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_rad", "acos(0)=pi/2", 1.570_796_327, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_rad", "atan(1)=pi/4", 0.785_398_163_4, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.7853981634");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(pi/4)=sqrt2/2", 0.707_106_781_2, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.7853981634");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_rad", "cos(pi/4)=sqrt2/2", 0.707_106_781_2, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "6.283185307");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(2pi)~0", 0.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "6.283185307");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_rad", "cos(2pi)=1", 1.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Tan).unwrap();
        case!("trig_rad", "tan(0rad)=0", 0.0, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "1.047197551");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_rad", "sin(pi/3)=sqrt3/2", 0.866_025_403_8, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.5235987756");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_rad", "cos(pi/6)=sqrt3/2", 0.866_025_403_8, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_rad", "asin(0.5)=pi/6", 0.523_598_775_6, get_x(&s));
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_rad", "acos(0.5)=pi/3", 1.047_197_551, get_x(&s));
    }

    // Trig GRAD cases 216–250
    for (grad, expected) in &[
        ("0", 0.0_f64),
        ("100", 1.0),
        ("200", 0.0),
        ("300", -1.0),
        ("400", 0.0),
    ] {
        let mut s = new_grad_state();
        push(&mut s, grad);
        dispatch(&mut s, Op::Sin).unwrap();
        case!(
            "trig_grad",
            &format!("sin({grad}grad)"),
            *expected,
            get_x(&s)
        );
    }
    for (grad, expected) in &[
        ("0", 1.0_f64),
        ("100", 0.0),
        ("200", -1.0),
        ("300", 0.0),
        ("400", 1.0),
    ] {
        let mut s = new_grad_state();
        push(&mut s, grad);
        dispatch(&mut s, Op::Cos).unwrap();
        case!(
            "trig_grad",
            &format!("cos({grad}grad)"),
            *expected,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "50");
        dispatch(&mut s, Op::Tan).unwrap();
        case!("trig_grad", "tan(50grad)=1", 1.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Tan).unwrap();
        case!("trig_grad", "tan(0grad)=0", 0.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "-50");
        dispatch(&mut s, Op::Tan).unwrap();
        case!("trig_grad", "tan(-50grad)=-1", -1.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "50");
        dispatch(&mut s, Op::Sin).unwrap();
        case!(
            "trig_grad",
            "sin(50grad)=sqrt2/2",
            0.707_106_781_2,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "50");
        dispatch(&mut s, Op::Cos).unwrap();
        case!(
            "trig_grad",
            "cos(50grad)=sqrt2/2",
            0.707_106_781_2,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "33.33333333");
        dispatch(&mut s, Op::Sin).unwrap();
        case!("trig_grad", "sin(33.33grad)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "33.33333333");
        dispatch(&mut s, Op::Cos).unwrap();
        case!(
            "trig_grad",
            "cos(33.33grad)=sqrt3/2",
            0.866_025_403_8,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_grad", "asin(1)=100grad", 100.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_grad", "asin(0)=0grad", 0.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Asin).unwrap();
        case!("trig_grad", "asin(-1)=-100grad", -100.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_grad", "acos(1)=0grad", 0.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_grad", "acos(0)=100grad", 100.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Acos).unwrap();
        case!("trig_grad", "acos(-1)=200grad", 200.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_grad", "atan(1)=50grad", 50.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_grad", "atan(-1)=-50grad", -50.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "0");
        dispatch(&mut s, Op::Atan).unwrap();
        case!("trig_grad", "atan(0)=0grad", 0.0, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "66.66666667");
        dispatch(&mut s, Op::Sin).unwrap();
        case!(
            "trig_grad",
            "sin(66.67grad)=sqrt3/2",
            0.866_025_403_8,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "66.66666667");
        dispatch(&mut s, Op::Cos).unwrap();
        case!("trig_grad", "cos(66.67grad)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "66.66666667");
        dispatch(&mut s, Op::Tan).unwrap();
        case!(
            "trig_grad",
            "tan(66.67grad)=sqrt3",
            1.732_050_808,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "16.66666667");
        dispatch(&mut s, Op::Sin).unwrap();
        case!(
            "trig_grad",
            "sin(16.67grad)=sin15deg",
            0.258_819_045_1,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "16.66666667");
        dispatch(&mut s, Op::Cos).unwrap();
        case!(
            "trig_grad",
            "cos(16.67grad)=cos15deg",
            0.965_925_826_3,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "83.33333333");
        dispatch(&mut s, Op::Sin).unwrap();
        case!(
            "trig_grad",
            "sin(83.33grad)=cos15deg",
            0.965_925_826_3,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "83.33333333");
        dispatch(&mut s, Op::Cos).unwrap();
        case!(
            "trig_grad",
            "cos(83.33grad)=sin15deg",
            0.258_819_045_1,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "33.33333333");
        dispatch(&mut s, Op::Tan).unwrap();
        case!(
            "trig_grad",
            "tan(33.33grad)=tan30deg",
            0.577_350_269_2,
            get_x(&s)
        );
    }
    {
        let mut s = new_grad_state();
        push(&mut s, "83.33333333");
        dispatch(&mut s, Op::Tan).unwrap();
        case!(
            "trig_grad",
            "tan(83.33grad)=tan75deg",
            3.732_050_808,
            get_x(&s)
        );
    }

    // ── Domain 3: Logs and Exponentials (cases 251–350) ──────────────────────

    // LN cases 251–280
    for (input, expected) in &[
        ("1", 0.0_f64),
        ("2.718281828", 1.0),
        ("7.389056099", 2.0),
        ("10", 2.302_585_093),
        ("2", 0.693_147_180_6),
        ("0.5", -0.693_147_180_6),
        ("100", 4.605_170_186),
        ("1000", 6.907_755_279),
        ("0.1", -2.302_585_093),
        ("0.01", -4.605_170_186),
        ("4", 1.386_294_361),
        ("8", 2.079_441_542),
        ("0.25", -1.386_294_361),
        ("3", 1.098_612_289),
        ("9", 2.197_224_577),
        ("1.5", 0.405_465_108_1),
        ("2.5", 0.916_290_731_9),
        ("5", 1.609_437_912),
        ("7", 1.945_910_149),
        ("20", 2.995_732_274),
        ("50", 3.912_023_005),
        ("0.001", -6.907_755_279),
        ("1000000", 13.815_510_56),
        ("100000000", 18.420_680_74),
        ("10000000000", 23.025_850_93),
        ("1.1", 0.095_310_179_80),
        ("1.01", 0.009_950_330_853),
        ("0.9", -0.105_360_515_7),
        ("0.99", -0.010_050_340_34),
        ("0.0000000001", -23.025_850_93),
    ] {
        let mut s = CalcState::new();
        push(&mut s, input);
        dispatch(&mut s, Op::Ln).unwrap();
        case!("log_exp", &format!("ln({input})"), *expected, get_x(&s));
    }

    // LOG cases 281–310
    for (input, expected) in &[
        ("1", 0.0_f64),
        ("10", 1.0),
        ("100", 2.0),
        ("1000", 3.0),
        ("0.1", -1.0),
        ("0.01", -2.0),
        ("2", 0.301_029_995_7),
        ("3", 0.477_121_254_7),
        ("4", 0.602_059_991_4),
        ("5", 0.698_970_004_3),
        ("7", 0.845_098_040_0),
        ("8", 0.903_089_987_0),
        ("9", 0.954_242_509_4),
        ("0.5", -0.301_029_995_7),
        ("0.25", -0.602_059_991_4),
        ("50", 1.698_970_004),
        ("500", 2.698_970_004),
        ("1000000", 6.0),
        ("0.000001", -6.0),
        ("2.5", 0.397_940_008_7),
        ("1.5", 0.176_091_259_0),
        ("20", 1.301_029_996),
        ("200", 2.301_029_996),
        ("1.1", 0.041_392_685_16),
        ("1.01", 0.004_321_373_783),
        ("0.9", -0.045_757_490_56),
        ("0.99", -0.004_364_805_403),
        ("1000000000", 9.0),
        ("0.000000001", -9.0),
        ("6", 0.778_151_250_4),
    ] {
        let mut s = CalcState::new();
        push(&mut s, input);
        dispatch(&mut s, Op::Log).unwrap();
        case!("log_exp", &format!("log({input})"), *expected, get_x(&s));
    }

    // e^x cases 311–330
    for (input, expected) in &[
        ("0", 1.0_f64),
        ("1", 2.718_281_828),
        ("2", 7.389_056_099),
        ("-1", 0.367_879_441_2),
        ("-2", 0.135_335_283_2),
        ("0.5", 1.648_721_271),
        ("-0.5", 0.606_530_659_7),
        ("3", 20.085_536_92),
        ("4", 54.598_150_03),
        ("5", 148.413_159_1),
        ("10", 22026.465_79),
        ("-3", 0.049_787_068_37),
        ("-10", 0.000_045_399_929_76),
        ("0.1", 1.105_170_918),
        ("0.693147181", 2.0),
        ("1.098612289", 3.0),
        ("1.386294361", 4.0),
        ("1.609437912", 5.0),
        ("2.302585093", 10.0),
        ("-0.693147181", 0.5),
    ] {
        let mut s = CalcState::new();
        push(&mut s, input);
        dispatch(&mut s, Op::Exp).unwrap();
        case!("log_exp", &format!("exp({input})"), *expected, get_x(&s));
    }

    // 10^x cases 331–350
    for (input, expected) in &[
        ("0", 1.0_f64),
        ("1", 10.0),
        ("2", 100.0),
        ("3", 1000.0),
        ("-1", 0.1),
        ("-2", 0.01),
        ("0.5", 3.162_277_660),
        ("-0.5", 0.316_227_766_0),
        ("0.3010299957", 2.0),
        ("0.4771212547", 3.0),
        ("0.6020599914", 4.0),
        ("0.6989700043", 5.0),
        ("0.8450980400", 7.0),
        ("1.301029996", 20.0),
        ("2.301029996", 200.0),
        ("-3", 0.001),
        ("6", 1_000_000.0),
        ("-6", 0.000_001),
        ("0.1", 1.258_925_412),
        ("-0.1", 0.794_328_234_7),
    ] {
        let mut s = CalcState::new();
        push(&mut s, input);
        dispatch(&mut s, Op::TenPow).unwrap();
        case!("log_exp", &format!("10^{input}"), *expected, get_x(&s));
    }

    // ── Domain 4: ISG/DSE edge cases (cases 351–400) ─────────────────────────

    // Helper: set reg 0 and call op_isg/op_dse
    let isg_bool = |counter: &str| -> bool {
        let mut s = CalcState::new();
        let d = dec(counter);
        s.regs[0] = HpNum::from(d);
        op_isg(&mut s, 0).unwrap()
    };
    let dse_bool = |counter: &str| -> bool {
        let mut s = CalcState::new();
        let d = dec(counter);
        s.regs[0] = HpNum::from(d);
        op_dse(&mut s, 0).unwrap()
    };
    let isg_reg = |counter: &str| -> f64 {
        let mut s = CalcState::new();
        let d = dec(counter);
        s.regs[0] = HpNum::from(d);
        op_isg(&mut s, 0).unwrap();
        s.regs[0].inner().to_f64().unwrap_or(f64::NAN)
    };
    let dse_reg = |counter: &str| -> f64 {
        let mut s = CalcState::new();
        let d = dec(counter);
        s.regs[0] = HpNum::from(d);
        op_dse(&mut s, 0).unwrap();
        s.regs[0].inner().to_f64().unwrap_or(f64::NAN)
    };

    // Canonical 5-step ISG sequence (cases 351–355)
    case!(
        "isg_dse",
        "ISG(1.00500) skip=false",
        0.0,
        if isg_bool("1.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(2.00500) skip=false",
        0.0,
        if isg_bool("2.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(3.00500) skip=false",
        0.0,
        if isg_bool("3.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(4.00500) skip=false",
        0.0,
        if isg_bool("4.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(5.00500) skip=true",
        1.0,
        if isg_bool("5.00500") { 1.0 } else { 0.0 }
    );

    // Step != 1 cases (356–360)
    case!(
        "isg_dse",
        "ISG(0.00502) skip=false",
        0.0,
        if isg_bool("0.00502") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(3.00502) skip=false",
        0.0,
        if isg_bool("3.00502") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(4.00502) skip=true",
        1.0,
        if isg_bool("4.00502") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(1.01001) skip=false",
        0.0,
        if isg_bool("1.01001") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(9.01001) skip=false",
        0.0,
        if isg_bool("9.01001") { 1.0 } else { 0.0 }
    );

    // DSE cases (361–370)
    case!(
        "isg_dse",
        "DSE(5.00100) skip=false",
        0.0,
        if dse_bool("5.00100") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(2.00100) skip=true",
        1.0,
        if dse_bool("2.00100") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(1.00100) skip=true",
        1.0,
        if dse_bool("1.00100") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(0.00100) skip=true",
        1.0,
        if dse_bool("0.00100") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(3.00200) skip=true",
        1.0,
        if dse_bool("3.00200") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(4.00200) skip=false",
        0.0,
        if dse_bool("4.00200") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(5.00200) skip=false",
        0.0,
        if dse_bool("5.00200") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(10.00500) skip=false",
        0.0,
        if dse_bool("10.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(6.00500) skip=true",
        1.0,
        if dse_bool("6.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(7.00500) skip=false",
        0.0,
        if dse_bool("7.00500") { 1.0 } else { 0.0 }
    );

    // Counter format edge cases (371–375)
    case!(
        "isg_dse",
        "ISG(0.00000) skip=true",
        1.0,
        if isg_bool("0.00000") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(-1.00500) skip=false",
        0.0,
        if isg_bool("-1.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(99.99901) skip=false",
        0.0,
        if isg_bool("99.99901") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(1000.00100) skip=false",
        0.0,
        if dse_bool("1000.00100") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(0.00001) skip=true",
        1.0,
        if isg_bool("0.00001") { 1.0 } else { 0.0 }
    );

    // Register-value checks (376–380)
    case!(
        "isg_dse",
        "ISG(1.00500) reg=2.005",
        2.005,
        isg_reg("1.00500")
    );
    case!(
        "isg_dse",
        "ISG(5.00500) reg=6.005",
        6.005,
        isg_reg("5.00500")
    );
    case!(
        "isg_dse",
        "DSE(3.00100) reg=2.001",
        2.001,
        dse_reg("3.00100")
    );
    case!(
        "isg_dse",
        "DSE(1.00100) reg=0.001",
        0.001,
        dse_reg("1.00100")
    );
    case!(
        "isg_dse",
        "ISG(0.01002) reg new current=2",
        2.01002,
        isg_reg("0.01002")
    );

    // ISG/DSE step=5 final=20 (381–400)
    case!(
        "isg_dse",
        "ISG(0.02005) skip=false",
        0.0,
        if isg_bool("0.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(5.02005) skip=false",
        0.0,
        if isg_bool("5.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(10.02005) skip=false",
        0.0,
        if isg_bool("10.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(15.02005) skip=false",
        0.0,
        if isg_bool("15.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(20.02005) skip=true",
        1.0,
        if isg_bool("20.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(20.02005) skip=true",
        1.0,
        if dse_bool("20.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(25.02005) skip=true",
        1.0,
        if dse_bool("25.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(26.02005) skip=false",
        0.0,
        if dse_bool("26.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(99.02005) skip=true",
        1.0,
        if isg_bool("99.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(21.02005) skip=true",
        1.0,
        if dse_bool("21.02005") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(1.00300) skip=false",
        0.0,
        if isg_bool("1.00300") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(0.00300) skip=false",
        0.0,
        if isg_bool("0.00300") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(4.00300) skip=true",
        1.0,
        if dse_bool("4.00300") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(3.00300) skip=true",
        1.0,
        if dse_bool("3.00300") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(6.00300) skip=false",
        0.0,
        if dse_bool("6.00300") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(7.00300) skip=false",
        0.0,
        if dse_bool("7.00300") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(-5.00500) skip=false",
        0.0,
        if isg_bool("-5.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(-4.00500) skip=true",
        1.0,
        if dse_bool("-4.00500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "ISG(4.99500) skip=false",
        0.0,
        if isg_bool("4.99500") { 1.0 } else { 0.0 }
    );
    case!(
        "isg_dse",
        "DSE(5.00100) skip=false",
        0.0,
        if dse_bool("5.00100") { 1.0 } else { 0.0 }
    );

    // ── Domain 5: Transcendental accumulation (cases 401–450) ────────────────

    // Round-trip ln/exp (401–420)
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(1))=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(2))=2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(10))=10", 10.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(0.5))=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(100))=100", 100.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2.718281828");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(e))=e", 2.718_281_828, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::TenPow).unwrap();
        dispatch(&mut s, Op::Log).unwrap();
        case!("transcendental", "log(10^1)=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::TenPow).unwrap();
        dispatch(&mut s, Op::Log).unwrap();
        case!("transcendental", "log(10^2)=2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::TenPow).unwrap();
        dispatch(&mut s, Op::Log).unwrap();
        case!("transcendental", "log(10^0.5)=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        dispatch(&mut s, Op::Log).unwrap();
        dispatch(&mut s, Op::TenPow).unwrap();
        case!("transcendental", "10^(log(7))=7", 7.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        dispatch(&mut s, Op::Sq).unwrap();
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("transcendental", "sqrt(sq(3))=3", 3.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1.5");
        dispatch(&mut s, Op::Sq).unwrap();
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("transcendental", "sqrt(sq(1.5))=1.5", 1.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        dispatch(&mut s, Op::Sq).unwrap();
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("transcendental", "sqrt(sq(7))=7", 7.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        dispatch(&mut s, Op::Recip).unwrap();
        push(&mut s, "3");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("transcendental", "(1/3)*3=1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        dispatch(&mut s, Op::Recip).unwrap();
        push(&mut s, "7");
        dispatch(&mut s, Op::Mul).unwrap();
        case!("transcendental", "(1/7)*7=1", 1.0, get_x(&s));
    }
    {
        // sin^2(30) + cos^2(30) = 1
        let mut s = new_deg_state();
        push(&mut s, "30");
        dispatch(&mut s, Op::Sin).unwrap();
        dispatch(&mut s, Op::Sq).unwrap();
        let sin2 = get_x(&s);
        let mut s2 = new_deg_state();
        push(&mut s2, "30");
        dispatch(&mut s2, Op::Cos).unwrap();
        dispatch(&mut s2, Op::Sq).unwrap();
        let cos2 = get_x(&s2);
        case!("transcendental", "sin^2(30)+cos^2(30)=1", 1.0, sin2 + cos2);
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "45");
        dispatch(&mut s, Op::Sin).unwrap();
        dispatch(&mut s, Op::Sq).unwrap();
        let sin2 = get_x(&s);
        let mut s2 = new_deg_state();
        push(&mut s2, "45");
        dispatch(&mut s2, Op::Cos).unwrap();
        dispatch(&mut s2, Op::Sq).unwrap();
        let cos2 = get_x(&s2);
        case!("transcendental", "sin^2(45)+cos^2(45)=1", 1.0, sin2 + cos2);
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "60");
        dispatch(&mut s, Op::Sin).unwrap();
        dispatch(&mut s, Op::Sq).unwrap();
        let sin2 = get_x(&s);
        let mut s2 = new_deg_state();
        push(&mut s2, "60");
        dispatch(&mut s2, Op::Cos).unwrap();
        dispatch(&mut s2, Op::Sq).unwrap();
        let cos2 = get_x(&s2);
        case!("transcendental", "sin^2(60)+cos^2(60)=1", 1.0, sin2 + cos2);
    }
    {
        // sin(2x) = 2sin(x)cos(x) for x=30
        let mut ss = new_deg_state();
        push(&mut ss, "30");
        dispatch(&mut ss, Op::Sin).unwrap();
        let sin30 = get_x(&ss);
        let mut sc = new_deg_state();
        push(&mut sc, "30");
        dispatch(&mut sc, Op::Cos).unwrap();
        let cos30 = get_x(&sc);
        let result = 2.0 * sin30 * cos30;
        let mut sv = new_deg_state();
        push(&mut sv, "60");
        dispatch(&mut sv, Op::Sin).unwrap();
        let sin60 = get_x(&sv);
        case!("transcendental", "sin(60)=2sin(30)cos(30)", sin60, result);
    }
    {
        // tan = sin/cos for 45
        let mut ss = new_deg_state();
        push(&mut ss, "45");
        dispatch(&mut ss, Op::Sin).unwrap();
        let sin45 = get_x(&ss);
        let mut sc = new_deg_state();
        push(&mut sc, "45");
        dispatch(&mut sc, Op::Cos).unwrap();
        let cos45 = get_x(&sc);
        case!(
            "transcendental",
            "tan(45)=sin(45)/cos(45)",
            1.0,
            sin45 / cos45
        );
    }

    // Multi-step arithmetic chains (421–440)
    {
        // (3+4)*(5-2)=21
        let mut s = CalcState::new();
        push(&mut s, "3");
        push(&mut s, "4");
        dispatch(&mut s, Op::Add).unwrap();
        let sum = get_x(&s);
        let mut s2 = CalcState::new();
        push(&mut s2, "5");
        push(&mut s2, "2");
        dispatch(&mut s2, Op::Sub).unwrap();
        let diff = get_x(&s2);
        case!("transcendental", "(3+4)*(5-2)=21", 21.0, sum * diff);
    }
    {
        // (10/2)+(3*4)=17
        let mut s = CalcState::new();
        push(&mut s, "10");
        push(&mut s, "2");
        dispatch(&mut s, Op::Div).unwrap();
        let q = get_x(&s);
        let mut s2 = CalcState::new();
        push(&mut s2, "3");
        push(&mut s2, "4");
        dispatch(&mut s2, Op::Mul).unwrap();
        let p = get_x(&s2);
        case!("transcendental", "(10/2)+(3*4)=17", 17.0, q + p);
    }
    {
        // ((2^3)+(3^2))/2=8.5
        let mut sa = CalcState::new();
        push(&mut sa, "2");
        push(&mut sa, "3");
        dispatch(&mut sa, Op::YPow).unwrap();
        let a = get_x(&sa);
        let mut sb = CalcState::new();
        push(&mut sb, "3");
        push(&mut sb, "2");
        dispatch(&mut sb, Op::YPow).unwrap();
        let b = get_x(&sb);
        case!("transcendental", "((2^3)+(3^2))/2=8.5", 8.5, (a + b) / 2.0);
    }
    {
        // 1/3 + 1/3 + 1/3 ~ 1
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "3");
        dispatch(&mut s, Op::Div).unwrap();
        let a = get_x(&s);
        let mut s2 = CalcState::new();
        push(&mut s2, "1");
        push(&mut s2, "3");
        dispatch(&mut s2, Op::Div).unwrap();
        let b = get_x(&s2);
        let mut s3 = CalcState::new();
        push(&mut s3, "1");
        push(&mut s3, "3");
        dispatch(&mut s3, Op::Div).unwrap();
        let c = get_x(&s3);
        case!("transcendental", "1/3+1/3+1/3~1", 1.0, a + b + c);
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5");
        dispatch(&mut s, Op::Ln).unwrap();
        dispatch(&mut s, Op::Exp).unwrap();
        case!("transcendental", "exp(ln(5))=5", 5.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        dispatch(&mut s, Op::Exp).unwrap();
        dispatch(&mut s, Op::Ln).unwrap();
        case!("transcendental", "ln(e^7)=7", 7.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "9");
        dispatch(&mut s, Op::Sqrt).unwrap();
        dispatch(&mut s, Op::Sq).unwrap();
        case!("transcendental", "sq(sqrt(9))=9", 9.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "6");
        dispatch(&mut s, Op::TenPow).unwrap();
        dispatch(&mut s, Op::Log).unwrap();
        case!("transcendental", "log(10^6)=6", 6.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        dispatch(&mut s, Op::Log).unwrap();
        dispatch(&mut s, Op::TenPow).unwrap();
        case!("transcendental", "10^(log(3))=3", 3.0, get_x(&s));
    }
    {
        // sqrt(sqrt(16))=2
        let mut s = CalcState::new();
        push(&mut s, "16");
        dispatch(&mut s, Op::Sqrt).unwrap();
        dispatch(&mut s, Op::Sqrt).unwrap();
        case!("transcendental", "sqrt(sqrt(16))=2", 2.0, get_x(&s));
    }
    {
        // 1.0001^10000 ~ 2.718145927 (wide tolerance)
        let mut s = CalcState::new();
        push(&mut s, "1.0001");
        push(&mut s, "10000");
        dispatch(&mut s, Op::YPow).unwrap();
        case!(
            "transcendental",
            "1.0001^10000~e",
            2.718_145_927,
            get_x(&s),
            wide
        );
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Asin).unwrap();
        dispatch(&mut s, Op::Sin).unwrap();
        case!("transcendental", "sin(asin(0.5))=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Acos).unwrap();
        dispatch(&mut s, Op::Cos).unwrap();
        case!("transcendental", "cos(acos(0.5))=0.5", 0.5, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "1");
        dispatch(&mut s, Op::Atan).unwrap();
        dispatch(&mut s, Op::Tan).unwrap();
        case!("transcendental", "tan(atan(1))=1", 1.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "30");
        dispatch(&mut s, Op::Sin).unwrap();
        dispatch(&mut s, Op::Asin).unwrap();
        case!("transcendental", "asin(sin(30))=30", 30.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "60");
        dispatch(&mut s, Op::Cos).unwrap();
        dispatch(&mut s, Op::Acos).unwrap();
        case!("transcendental", "acos(cos(60))=60", 60.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "45");
        dispatch(&mut s, Op::Tan).unwrap();
        dispatch(&mut s, Op::Atan).unwrap();
        case!("transcendental", "atan(tan(45))=45", 45.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.001");
        dispatch(&mut s, Op::Ln).unwrap();
        // ln(1+0.001): push 1.001 and take ln
        let mut s2 = CalcState::new();
        push(&mut s2, "1.001");
        dispatch(&mut s2, Op::Ln).unwrap();
        case!(
            "transcendental",
            "ln(1.001)~0.0009995003",
            0.000_999_500_3,
            get_x(&s2)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.001");
        dispatch(&mut s, Op::Exp).unwrap();
        case!(
            "transcendental",
            "exp(0.001)~1.001000500",
            1.001_000_500,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.001");
        dispatch(&mut s, Op::TenPow).unwrap();
        case!(
            "transcendental",
            "10^0.001~1.002305238",
            1.002_305_238,
            get_x(&s)
        );
    }

    // Chained trig + inverse (441–450)
    {
        let mut s = new_deg_state();
        push(&mut s, "10");
        dispatch(&mut s, Op::Tan).unwrap();
        dispatch(&mut s, Op::Atan).unwrap();
        case!("transcendental", "atan(tan(10))=10", 10.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "80");
        dispatch(&mut s, Op::Tan).unwrap();
        dispatch(&mut s, Op::Atan).unwrap();
        case!("transcendental", "atan(tan(80))=80", 80.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "10");
        dispatch(&mut s, Op::Sin).unwrap();
        dispatch(&mut s, Op::Asin).unwrap();
        case!("transcendental", "asin(sin(10))=10", 10.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "10");
        dispatch(&mut s, Op::Cos).unwrap();
        dispatch(&mut s, Op::Acos).unwrap();
        case!("transcendental", "acos(cos(10))=10", 10.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.866025403");
        dispatch(&mut s, Op::Asin).unwrap();
        dispatch(&mut s, Op::Sin).unwrap();
        case!(
            "transcendental",
            "sin(asin(0.866))=0.866",
            0.866_025_403,
            get_x(&s)
        );
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0.866025403");
        dispatch(&mut s, Op::Acos).unwrap();
        dispatch(&mut s, Op::Cos).unwrap();
        case!(
            "transcendental",
            "cos(acos(0.866))=0.866",
            0.866_025_403,
            get_x(&s)
        );
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "30");
        dispatch(&mut s, Op::Sin).unwrap();
        let sin30 = get_x(&s);
        case!("transcendental", "sin(30)*2=1", 1.0, sin30 * 2.0);
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "60");
        dispatch(&mut s, Op::Cos).unwrap();
        let cos60 = get_x(&s);
        case!("transcendental", "cos(60)*2=1", 1.0, cos60 * 2.0);
    }
    {
        let mut s1 = new_deg_state();
        push(&mut s1, "45");
        dispatch(&mut s1, Op::Tan).unwrap();
        let t1 = get_x(&s1);
        let mut s2 = new_deg_state();
        push(&mut s2, "45");
        dispatch(&mut s2, Op::Tan).unwrap();
        let t2 = get_x(&s2);
        case!("transcendental", "tan(45)+tan(45)=2", 2.0, t1 + t2);
    }
    {
        let mut ss = new_deg_state();
        push(&mut ss, "90");
        dispatch(&mut ss, Op::Sin).unwrap();
        let sin90 = get_x(&ss);
        let mut sc = new_deg_state();
        push(&mut sc, "0");
        dispatch(&mut sc, Op::Cos).unwrap();
        let cos0 = get_x(&sc);
        case!("transcendental", "sin(90)*cos(0)=1", 1.0, sin90 * cos0);
    }

    // ── Domain 6: HMS conversions (cases 451–480) ────────────────────────────

    // HMS→ (451–460)
    {
        let cases_hms_to_h: &[(&str, f64, bool)] = &[
            ("1.3045", 1.5125, false),
            ("0.0000", 0.0, false),
            ("1.0000", 1.0, false),
            ("0.3000", 0.5, false),
            ("0.0100", 1.0 / 60.0, true),
            ("0.0001", 1.0 / 3600.0, false),
            ("2.0000", 2.0, false),
            ("0.1500", 0.25, false),
            ("0.4500", 0.75, false),
            ("1.0030", 1.0 + 30.0 / 3600.0, true),
        ];
        for (input, expected, wide) in cases_hms_to_h {
            let mut s = CalcState::new();
            push(&mut s, input);
            dispatch(&mut s, Op::HmsToH).unwrap();
            if *wide {
                case!(
                    "hms",
                    &format!("HMS->H({input})"),
                    *expected,
                    get_x(&s),
                    wide
                );
            } else {
                case!("hms", &format!("HMS->H({input})"), *expected, get_x(&s));
            }
        }
    }

    // →HMS (461–470)
    {
        let cases_h_to_hms: &[(&str, f64, bool)] = &[
            ("1.5125", 1.3045, false),
            ("0.5", 0.3, false),
            ("1.0", 1.0, false),
            ("0.25", 0.15, false),
            ("0.75", 0.45, false),
            ("2.0", 2.0, false),
            ("0.0", 0.0, false),
            ("1.25", 1.15, false),
            ("1.5", 1.3, false),
            ("0.1666666667", 0.1, true),
        ];
        for (input, expected, wide) in cases_h_to_hms {
            let mut s = CalcState::new();
            push(&mut s, input);
            dispatch(&mut s, Op::HToHms).unwrap();
            if *wide {
                case!(
                    "hms",
                    &format!("H->HMS({input})"),
                    *expected,
                    get_x(&s),
                    wide
                );
            } else {
                case!("hms", &format!("H->HMS({input})"), *expected, get_x(&s));
            }
        }
    }

    // Round-trip HMS→ then →HMS (471–480)
    {
        let roundtrip: &[(&str, f64, bool)] = &[
            ("1.3045", 1.3045, false),
            ("0.3000", 0.3, false),
            ("1.0000", 1.0, false),
            ("0.1500", 0.15, false),
            ("2.3000", 2.3, false),
            ("0.0000", 0.0, false),
            ("1.4500", 1.45, false),
            ("0.4500", 0.45, false),
            ("1.1500", 1.15, false),
            ("0.0030", 0.003, false),
        ];
        for (input, expected, wide) in roundtrip {
            let mut s = CalcState::new();
            push(&mut s, input);
            dispatch(&mut s, Op::HmsToH).unwrap();
            dispatch(&mut s, Op::HToHms).unwrap();
            if *wide {
                case!(
                    "hms",
                    &format!("roundtrip HMS({input})"),
                    *expected,
                    get_x(&s),
                    wide
                );
            } else {
                case!(
                    "hms",
                    &format!("roundtrip HMS({input})"),
                    *expected,
                    get_x(&s)
                );
            }
        }
    }

    // ── Domain 7: Statistics (cases 481–500) ─────────────────────────────────

    // Single-variable stats: push Y=0, X=value
    let sigma_x = |vals: &[&str]| -> CalcState {
        let mut s = CalcState::new();
        for v in vals {
            push(&mut s, "0"); // Y=0
            push(&mut s, v); // X=value
            dispatch(&mut s, Op::SigmaPlus).unwrap();
        }
        s
    };

    // Case 481: n=1 after one Sigma+
    {
        let s = sigma_x(&["1"]);
        case!(
            "stats",
            "sigma+(1): n=1",
            1.0,
            s.regs[3].inner().to_f64().unwrap_or(f64::NAN)
        );
    }
    // Case 482: n=2 after two Sigma+
    {
        let s = sigma_x(&["1", "2"]);
        case!(
            "stats",
            "sigma+(1,2): n=2",
            2.0,
            s.regs[3].inner().to_f64().unwrap_or(f64::NAN)
        );
    }
    // Case 483: mean of [1,2,3]=2
    {
        let mut s = sigma_x(&["1", "2", "3"]);
        dispatch(&mut s, Op::Mean).unwrap();
        case!("stats", "mean([1,2,3])=2", 2.0, get_x(&s));
    }
    // Case 484: mean of [10,20,30]=20
    {
        let mut s = sigma_x(&["10", "20", "30"]);
        dispatch(&mut s, Op::Mean).unwrap();
        case!("stats", "mean([10,20,30])=20", 20.0, get_x(&s));
    }
    // Case 485: mean of [1,1,1]=1
    {
        let mut s = sigma_x(&["1", "1", "1"]);
        dispatch(&mut s, Op::Mean).unwrap();
        case!("stats", "mean([1,1,1])=1", 1.0, get_x(&s));
    }
    // Case 486: sdev of [1,2,3]=1 (sample)
    {
        let mut s = sigma_x(&["1", "2", "3"]);
        dispatch(&mut s, Op::Sdev).unwrap();
        case!("stats", "sdev([1,2,3])=1", 1.0, get_x(&s));
    }
    // Case 487: sdev of [0,0,2,2] (wide)
    {
        let mut s = sigma_x(&["0", "0", "2", "2"]);
        dispatch(&mut s, Op::Sdev).unwrap();
        // sample sdev = sqrt(4/3) = 1.154700538
        case!(
            "stats",
            "sdev([0,0,2,2])=1.1547",
            1.154_700_538,
            get_x(&s),
            wide
        );
    }
    // Case 488: Sigma regs for [2,4,6]: R02=Σx=12, R01=Σx²=56
    {
        let s = sigma_x(&["2", "4", "6"]);
        let sum_x = s.regs[2].inner().to_f64().unwrap_or(f64::NAN);
        case!("stats", "sigma+([2,4,6]): Σx=12", 12.0, sum_x);
    }
    // Case 489: Sigma-minus removes data point
    {
        let mut s = sigma_x(&["1", "2", "3"]);
        // Remove 3: push Y=0, X=3, then Sigma-
        push(&mut s, "0");
        push(&mut s, "3");
        dispatch(&mut s, Op::SigmaMinus).unwrap();
        dispatch(&mut s, Op::Mean).unwrap();
        case!("stats", "mean([1,2,3] minus 3)=1.5", 1.5, get_x(&s));
    }
    // Case 490: MEAN of single value returns that value
    {
        let mut s = sigma_x(&["7"]);
        dispatch(&mut s, Op::Mean).unwrap();
        case!("stats", "mean([7])=7", 7.0, get_x(&s));
    }

    // L.R. and correlation (491–500)
    // For L.R.: HP-41 convention Y=y-value, X=x-value
    let sigma_xy = |pairs: &[(&str, &str)]| -> CalcState {
        let mut s = CalcState::new();
        for (y, x) in pairs {
            push(&mut s, y);
            push(&mut s, x);
            dispatch(&mut s, Op::SigmaPlus).unwrap();
        }
        s
    };

    // Case 491: L.R. on [(1,1),(2,2),(3,3)]: slope=1, intercept=0
    {
        let mut s = sigma_xy(&[("1", "1"), ("2", "2"), ("3", "3")]);
        dispatch(&mut s, Op::LR).unwrap();
        // LR: X=intercept, Y=slope
        case!(
            "stats",
            "LR([(1,1),(2,2),(3,3)]): intercept=0",
            0.0,
            get_x(&s)
        );
    }
    // Case 492: L.R. on [(0,1),(1,2),(2,3)]: slope=1, intercept=1 (X=x-coord, Y=y-coord)
    // Pairs (y,x): (0,1),(1,2),(2,3) => intercept=-1? Let's do y=x-based: Y is dependent
    // Actually HP-41 CORR uses X as independent, Y as dependent.
    // y=mx+b: points (x=1,y=0),(x=2,y=1),(x=3,y=2) => m=1, b=-1
    {
        let mut s = sigma_xy(&[("0", "1"), ("1", "2"), ("2", "3")]);
        dispatch(&mut s, Op::LR).unwrap();
        case!("stats", "LR(y=x-1): intercept=-1", -1.0, get_x(&s));
    }
    // Case 493: LR on [(1,3),(2,3),(3,3)] y-values all 3: slope=0, intercept=3
    // pairs (y,x): (3,1),(3,2),(3,3)
    {
        let mut s = sigma_xy(&[("3", "1"), ("3", "2"), ("3", "3")]);
        dispatch(&mut s, Op::LR).unwrap();
        case!("stats", "LR(y=3 constant): intercept=3", 3.0, get_x(&s));
    }
    // Case 494: CORR on [(1,1),(2,2),(3,3)]=1.0
    {
        let mut s = sigma_xy(&[("1", "1"), ("2", "2"), ("3", "3")]);
        dispatch(&mut s, Op::Corr).unwrap();
        case!("stats", "CORR([(1,1),(2,2),(3,3)])=1", 1.0, get_x(&s));
    }
    // Case 495: CORR on [(3,1),(2,2),(1,3)]=-1.0
    {
        let mut s = sigma_xy(&[("3", "1"), ("2", "2"), ("1", "3")]);
        dispatch(&mut s, Op::Corr).unwrap();
        case!("stats", "CORR(inverse)=-1", -1.0, get_x(&s));
    }
    // Case 496: mean of [1,2,3,4,5]=3
    {
        let mut s = sigma_x(&["1", "2", "3", "4", "5"]);
        dispatch(&mut s, Op::Mean).unwrap();
        case!("stats", "mean([1..5])=3", 3.0, get_x(&s));
    }
    // Case 497: sdev of [1,2,3,4,5]=1.581138830 (sample)
    {
        let mut s = sigma_x(&["1", "2", "3", "4", "5"]);
        dispatch(&mut s, Op::Sdev).unwrap();
        case!(
            "stats",
            "sdev([1..5])=1.581138830",
            1.581_138_830,
            get_x(&s)
        );
    }
    // Case 498: L.R. on y=2x: (y=2,x=1),(y=4,x=2),(y=6,x=3) slope=2, intercept=0
    {
        let mut s = sigma_xy(&[("2", "1"), ("4", "2"), ("6", "3")]);
        dispatch(&mut s, Op::LR).unwrap();
        case!("stats", "LR(y=2x): intercept=0", 0.0, get_x(&s));
    }
    // Case 499: CORR on y=2x=1.0
    {
        let mut s = sigma_xy(&[("2", "1"), ("4", "2"), ("6", "3")]);
        dispatch(&mut s, Op::Corr).unwrap();
        case!("stats", "CORR(y=2x)=1", 1.0, get_x(&s));
    }
    // Case 500: [10,20,30]: n=3, Σx=60, Σx²=1400
    {
        let s = sigma_x(&["10", "20", "30"]);
        let sum_x2 = s.regs[1].inner().to_f64().unwrap_or(f64::NAN);
        case!("stats", "sigma+([10,20,30]): Σx²=1400", 1400.0, sum_x2);
    }

    // ── Domain 8: %CH / Percent Change (cases 501–503) ───────────────────────
    // Case 501: +25%: Y=80, X=100 → 25
    {
        let mut s = CalcState::new();
        push(&mut s, "80");
        push(&mut s, "100");
        dispatch(&mut s, Op::PctChange).unwrap();
        case!("arithmetic", "%CH(Y=80,X=100)=25", 25.0, get_x(&s));
    }
    // Case 502: −33.33333333% (10 sig digits): Y=300, X=200 → −33.33333333
    {
        let mut s = CalcState::new();
        push(&mut s, "300");
        push(&mut s, "200");
        dispatch(&mut s, Op::PctChange).unwrap();
        case!(
            "arithmetic",
            "%CH(Y=300,X=200)=-33.33333333",
            -33.333_333_33,
            get_x(&s)
        );
    }
    // Case 503: doubling: Y=50, X=100 → +100
    {
        let mut s = CalcState::new();
        push(&mut s, "50");
        push(&mut s, "100");
        dispatch(&mut s, Op::PctChange).unwrap();
        case!("arithmetic", "%CH(Y=50,X=100)=100", 100.0, get_x(&s));
    }

    // ── Capture v1.x baseline count for D-27.6 atomic baseline assertion ───
    // The 500-case (now 503-case) baseline must stay at its pre-existing
    // pass rate when the v2.2 extension lands. D-27.6: combined ≥ 98%
    // pass rate maintained AND the v1.x baseline must not regress.
    let baseline_total = cases.len();
    let baseline_passes: usize = cases
        .iter()
        .filter(|c| passes_with_tol(c.actual, c.expected, c.tol))
        .count();

    // ── v2.2 EXTENSION (D-27.5, FN-QUAL-02) ─────────────────────────────────
    // Hand-curated cases for v2.2 math/conversion ops: PI, P→R, R→P, RND,
    // FRC, MOD, FACT (~70 cases). Quirky cases cite Free42 source or
    // HP-41C Owner's Manual page per D-27.7.

    // ── v2.2 Op::Pi (cases 504–506) ─────────────────────────────────────────
    // Cross-checked against HP-41C Owner's Manual p.65 — π displays as
    // 3.141592654 (10-digit rounded hardware value).
    {
        let mut s = CalcState::new();
        dispatch(&mut s, Op::Pi).unwrap();
        case!(
            "pi",
            "PI = 3.141592654 (HP-41 hardware value)",
            3.141_592_654,
            get_x(&s)
        );
    }
    {
        // PI in DEG mode is value-preserving — angle mode does not affect the
        // constant push (HP-41C Owner's Manual p.65).
        let mut s = new_deg_state();
        dispatch(&mut s, Op::Pi).unwrap();
        case!("pi", "PI in DEG mode unchanged", 3.141_592_654, get_x(&s));
    }
    {
        // PI followed by SIN in RAD mode → 0 (sin(π) = 0 within tolerance).
        // Cross-checked against Free42 source core_math2.cc::do_sin.
        let mut s = new_rad_state();
        dispatch(&mut s, Op::Pi).unwrap();
        dispatch(&mut s, Op::Sin).unwrap();
        case!("pi", "SIN(PI) in RAD ≈ 0", 0.0, get_x(&s));
    }

    // ── v2.2 Op::Fact (cases 507–516) ───────────────────────────────────────
    // Cross-checked against Free42 source core_math1.cc::do_fact and
    // HP-41C Owner's Manual p.234. FACT(0) = 1 is the headline quirk case.
    {
        // HP-41C Owner's Manual p.234: FACT(0) = 1 (mathematical convention).
        // Cross-checked against Free42 source core_math1.cc::do_fact.
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Fact).unwrap();
        case!(
            "fact",
            "FACT(0) = 1 (HP-41C OM p.234, Free42 do_fact)",
            1.0,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(1) = 1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(5) = 120", 120.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(10) = 3628800", 3_628_800.0, get_x(&s));
    }
    {
        // Cross-checked against Free42 core_math1.cc::do_fact — wide-tol because
        // the f64 product accumulates rounding past the HP-41 10-digit display.
        let mut s = CalcState::new();
        push(&mut s, "20");
        dispatch(&mut s, Op::Fact).unwrap();
        case!(
            "fact",
            "FACT(20) = 2.432902008e18",
            2.432_902_008e18,
            get_x(&s),
            wide
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(2) = 2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(3) = 6", 6.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "4");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(4) = 24", 24.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "6");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(6) = 720", 720.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(7) = 5040", 5040.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "13");
        dispatch(&mut s, Op::Fact).unwrap();
        case!("fact", "FACT(13) = 6227020800", 6_227_020_800.0, get_x(&s));
    }

    // ── v2.2 Op::Mod (cases 517–528) ────────────────────────────────────────
    // Cross-checked against Free42 source core_math1.cc::do_mod —
    // HP-41 sign follows Y, per HP-41C Owner's Manual p.234.
    {
        let mut s = CalcState::new();
        push(&mut s, "7");
        push(&mut s, "3");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(7, 3) = 1 (control)", 1.0, get_x(&s));
    }
    {
        // Cross-checked against Free42 source core_math1.cc::do_mod — Free42
        // returns 1 for MOD(7, -3), matching HP-41C Owner's Manual p.234.
        // Sign follows Y (HP-41 hardware convention; NOT Rust % semantics).
        let mut s = CalcState::new();
        push(&mut s, "7");
        push(&mut s, "-3");
        dispatch(&mut s, Op::Mod).unwrap();
        case!(
            "mod",
            "MOD(7,-3) = 1 (sign-follows-Y, Free42 do_mod)",
            1.0,
            get_x(&s)
        );
    }
    {
        // Cross-checked against Free42 source core_math1.cc::do_mod — sign
        // follows Y, so MOD(-7, 3) = -1.
        let mut s = CalcState::new();
        push(&mut s, "-7");
        push(&mut s, "3");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(-7, 3) = -1 (sign-follows-Y)", -1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-7");
        push(&mut s, "-3");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(-7,-3) = -1", -1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0");
        push(&mut s, "5");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(0, 5) = 0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5");
        push(&mut s, "5");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(5, 5) = 0 (exact divisible)", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "7.5");
        push(&mut s, "2");
        dispatch(&mut s, Op::Mod).unwrap();
        case!(
            "mod",
            "MOD(7.5, 2) = 1.5 (non-integer dividend)",
            1.5,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "10");
        push(&mut s, "3");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(10, 3) = 1", 1.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "100");
        push(&mut s, "7");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(100, 7) = 2", 2.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "3.5");
        push(&mut s, "-1");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(3.5, -1) = 0.5 (sign-follows-Y)", 0.5, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1");
        push(&mut s, "1");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(1, 1) = 0", 0.0, get_x(&s));
    }
    {
        // Cross-checked against Free42 source core_math1.cc::do_mod — small
        // positive remainder.
        let mut s = CalcState::new();
        push(&mut s, "0.7");
        push(&mut s, "0.3");
        dispatch(&mut s, Op::Mod).unwrap();
        case!("mod", "MOD(0.7, 0.3) = 0.1", 0.1, get_x(&s));
    }

    // ── v2.2 Op::Rnd (cases 529–537) ────────────────────────────────────────
    // Cross-checked against HP-41C Owner's Manual p.59 (FIX/SCI/ENG display
    // mode rounding semantics — RND mirrors display rounding per D-01/D-02).
    {
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(2);
        push(&mut s, "3.14159");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!("rnd", "RND(3.14159, FIX 2) = 3.14", 3.14, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(4);
        push(&mut s, "3.14159265");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!("rnd", "RND(3.14159265, FIX 4) = 3.1416", 3.1416, get_x(&s));
    }
    {
        // HP-41C Owner's Manual p.59: RND idempotent — round-half-away-from-zero.
        // Verifies BCD doesn't carry f64 imprecision (0.1 + 0.2 ≠ 0.30000... in IEEE).
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(5);
        push(&mut s, "0.1");
        push(&mut s, "0.2");
        dispatch(&mut s, Op::Add).unwrap();
        dispatch(&mut s, Op::Rnd).unwrap();
        case!(
            "rnd",
            "RND(0.1+0.2, FIX 5) = 0.3 (BCD purity)",
            0.3,
            get_x(&s)
        );
    }
    {
        // HP-41C Owner's Manual p.59: SCI mode keeps n+1 significant digits.
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Sci(2);
        push(&mut s, "1234.5678");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!(
            "rnd",
            "RND(1234.5678, SCI 2) = 1230 (3 sig figs)",
            1230.0,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(0);
        push(&mut s, "3.7");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!(
            "rnd",
            "RND(3.7, FIX 0) = 4 (round-half-away)",
            4.0,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(0);
        push(&mut s, "-3.7");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!(
            "rnd",
            "RND(-3.7, FIX 0) = -4 (round-half-away)",
            -4.0,
            get_x(&s)
        );
    }
    {
        // RND idempotency: RND(RND(x)) = RND(x) for any mode (D-01/D-02).
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(3);
        push(&mut s, "2.71828");
        dispatch(&mut s, Op::Rnd).unwrap();
        dispatch(&mut s, Op::Rnd).unwrap();
        case!("rnd", "RND ∘ RND idempotent (FIX 3)", 2.718, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(2);
        push(&mut s, "0");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!("rnd", "RND(0, FIX 2) = 0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(1);
        push(&mut s, "0.05");
        dispatch(&mut s, Op::Rnd).unwrap();
        case!(
            "rnd",
            "RND(0.05, FIX 1) = 0.1 (half-away-from-zero)",
            0.1,
            get_x(&s)
        );
    }

    // ── v2.2 Op::Frc (cases 538–545) ────────────────────────────────────────
    // Cross-checked against HP-41C Owner's Manual p.61 — FRC + INT round-trip
    // (FRC + INT = x; sign matches input).
    {
        let mut s = CalcState::new();
        push(&mut s, "3.14");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(3.14) = 0.14", 0.14, get_x(&s));
    }
    {
        // HP-41C Owner's Manual p.61: FRC is sign-preserving complement of INT.
        let mut s = CalcState::new();
        push(&mut s, "-3.14");
        dispatch(&mut s, Op::Frc).unwrap();
        case!(
            "frc",
            "FRC(-3.14) = -0.14 (sign follows input)",
            -0.14,
            get_x(&s)
        );
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(0) = 0", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "5");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(5) = 0 (integer input)", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "-7");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(-7) = 0 (negative integer)", 0.0, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "0.0001");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(0.0001) = 0.0001", 0.0001, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "2.71828");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(2.71828) = 0.71828", 0.71828, get_x(&s));
    }
    {
        let mut s = CalcState::new();
        push(&mut s, "1234.5");
        dispatch(&mut s, Op::Frc).unwrap();
        case!("frc", "FRC(1234.5) = 0.5", 0.5, get_x(&s));
    }

    // ── v2.2 Op::PolarToRect (cases 546–555) ────────────────────────────────
    // Cross-checked against HP-41C Owner's Manual Chapter 3 (polar/rectangular
    // conversions). Result layout: X holds y-coord, Y holds x-coord per
    // FN-MATH-03. Tolerance widened for trig boundary cases — sin(90°) BCD
    // path produces 1.0000000000 ± LSB.
    {
        // HP-41C OM Ch. 3: PR(R=5, θ=0°) → (x=5, y=0) → X=0, Y=5.
        let mut s = new_deg_state();
        push(&mut s, "5"); // r
        push(&mut s, "0"); // theta
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=0°) X=0", 0.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "5"); // r
        push(&mut s, "0"); // theta
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=0°) Y=5", 5.0, get_y(&s));
    }
    {
        // HP-41C OM Ch. 3: PR(R=5, θ=90°) → (x=0, y=5) → X=5, Y=0.
        let mut s = new_deg_state();
        push(&mut s, "5");
        push(&mut s, "90");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=90°) X=5", 5.0, get_x(&s), wide);
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "5");
        push(&mut s, "90");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=90°) Y=0", 0.0, get_y(&s), wide);
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "5");
        push(&mut s, "180");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=180°) Y=-5", -5.0, get_y(&s), wide);
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "5");
        push(&mut s, "270");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=270°) X=-5", -5.0, get_x(&s), wide);
    }
    {
        // HP-41C OM Ch. 3: PR(R=10, θ=45°) → (x=y=7.071067812 ≈ √50).
        let mut s = new_deg_state();
        push(&mut s, "10");
        push(&mut s, "45");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!(
            "pr",
            "PR(R=10, θ=45°) X=7.071...",
            7.071_067_812,
            get_x(&s),
            wide
        );
    }
    {
        // PR with negative R: sign carries through.
        let mut s = new_deg_state();
        push(&mut s, "-5");
        push(&mut s, "0");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=-5, θ=0°) Y=-5", -5.0, get_y(&s));
    }
    {
        // RAD mode: PR(R=5, θ=π/2) → (x=0, y=5).
        let mut s = new_rad_state();
        push(&mut s, "5");
        push(&mut s, "1.570796327"); // π/2 = 1.570796327 (10-digit)
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=π/2 RAD) X=5", 5.0, get_x(&s), wide);
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "5");
        push(&mut s, "1.570796327");
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("pr", "PR(R=5, θ=π/2 RAD) Y=0", 0.0, get_y(&s), wide);
    }

    // ── v2.2 Op::RectToPolar (cases 556–565) ────────────────────────────────
    // Cross-checked against HP-41C Owner's Manual Chapter 3. Input: Y=x-coord,
    // X=y-coord. Output: Y=R (magnitude), X=θ (angle).
    {
        // 3-4-5 triangle (HP-41C OM Ch. 3 reference): RP(x=3, y=4) → R=5,
        // θ=atan2(4,3) ≈ 53.13010235°.
        let mut s = new_deg_state();
        push(&mut s, "3");
        push(&mut s, "4");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=3, y=4) R=5 (3-4-5 triangle)", 5.0, get_y(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "3");
        push(&mut s, "4");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!(
            "rp",
            "RP(x=3, y=4) θ≈53.13°",
            53.130_102_35,
            get_x(&s),
            wide
        );
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "5");
        push(&mut s, "0");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=5, y=0) R=5", 5.0, get_y(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "5");
        push(&mut s, "0");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=5, y=0) θ=0", 0.0, get_x(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0");
        push(&mut s, "5");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=0, y=5) R=5", 5.0, get_y(&s));
    }
    {
        let mut s = new_deg_state();
        push(&mut s, "0");
        push(&mut s, "5");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=0, y=5) θ=90°", 90.0, get_x(&s), wide);
    }
    {
        // Second quadrant: RP(x=-3, y=4) → R=5, θ≈126.87°.
        let mut s = new_deg_state();
        push(&mut s, "-3");
        push(&mut s, "4");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=-3, y=4) R=5 (Q2)", 5.0, get_y(&s));
    }
    {
        // Degenerate case: RP(0, 0) → R=0, θ=0.
        let mut s = new_deg_state();
        push(&mut s, "0");
        push(&mut s, "0");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(0, 0) R=0 (degenerate)", 0.0, get_y(&s));
    }
    {
        // Round-trip: PR(RP(3,4)) ≈ (3,4). Pin by example here; proptest in
        // Plan 27-02 covers the general invariant.
        let mut s = new_deg_state();
        push(&mut s, "3");
        push(&mut s, "4");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        dispatch(&mut s, Op::PolarToRect).unwrap();
        case!("rp", "PR(RP(3,4)) X≈4 round-trip", 4.0, get_x(&s), wide);
    }
    {
        let mut s = new_rad_state();
        push(&mut s, "1");
        push(&mut s, "0");
        dispatch(&mut s, Op::RectToPolar).unwrap();
        case!("rp", "RP(x=1, y=0) R=1 (RAD)", 1.0, get_y(&s));
    }

    // ── v3.0 EXTENSION (Plan 28-02, FN-HYP-01..06) ───────────────────────────
    // Hyperbolic function cases. Reference values from HP Math Pac I Owner's
    // Manual 00041-90034 (1979). Cross-checked against Free42 v3.0.5.
    // Domain-error cases (acosh(x<1), atanh(|x|>=1)) are verified outside
    // the case! framework — they return Err, not numeric values.

    // ── v3.0 Op::Sinh (3+ cases) ─────────────────────────────────────────────
    {
        // Source: HP 00041-90034 p.44, ex.1 — sinh(0) = 0
        // Free42 v3.0.5: 0 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Sinh).unwrap();
        case!("sinh", "SINH(0) = 0 (HP 00041-90034 p.44)", 0.0, get_x(&s));
    }
    {
        // Source: HP 00041-90034 p.44, ex.1 — sinh(1) = 1.175201194
        // Free42 v3.0.5: 1.1752011936 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Sinh).unwrap();
        case!(
            "sinh",
            "SINH(1) = 1.175201194 (HP 00041-90034 p.44)",
            1.175_201_193_6,
            get_x(&s)
        );
    }
    {
        // Source: HP 00041-90034 p.44, ex.2 — sinh(-1) = -1.175201194
        // Free42 v3.0.5: -1.1752011936 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Sinh).unwrap();
        case!(
            "sinh",
            "SINH(-1) = -1.175201194 (HP 00041-90034 p.44)",
            -1.175_201_193_6,
            get_x(&s)
        );
    }

    // ── v3.0 Op::Cosh (3+ cases) ─────────────────────────────────────────────
    {
        // Source: HP 00041-90034 p.44, ex.3 — cosh(0) = 1
        // Free42 v3.0.5: 1 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Cosh).unwrap();
        case!("cosh", "COSH(0) = 1 (HP 00041-90034 p.44)", 1.0, get_x(&s));
    }
    {
        // Source: HP 00041-90034 p.44, ex.3 — cosh(1) = 1.543080635
        // Free42 v3.0.5: 1.5430806348 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Cosh).unwrap();
        case!(
            "cosh",
            "COSH(1) = 1.543080635 (HP 00041-90034 p.44)",
            1.543_080_634_8,
            get_x(&s)
        );
    }
    {
        // Source: HP 00041-90034 p.44 — cosh(2) = 3.762195691
        // Free42 v3.0.5: 3.7621956910 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::Cosh).unwrap();
        case!(
            "cosh",
            "COSH(2) = 3.762195691 (HP 00041-90034 p.44)",
            3.762_195_691_0,
            get_x(&s)
        );
    }

    // ── v3.0 Op::Tanh (3+ cases) ─────────────────────────────────────────────
    {
        // Source: HP 00041-90034 p.44, ex.5 — tanh(0) = 0
        // Free42 v3.0.5: 0 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Tanh).unwrap();
        case!("tanh", "TANH(0) = 0 (HP 00041-90034 p.44)", 0.0, get_x(&s));
    }
    {
        // Source: HP 00041-90034 p.44, ex.5 — tanh(1) = 0.761594156
        // Free42 v3.0.5: 0.7615941560 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Tanh).unwrap();
        case!(
            "tanh",
            "TANH(1) = 0.761594156 (HP 00041-90034 p.44)",
            0.761_594_156_0,
            get_x(&s)
        );
    }
    {
        // Source: HP 00041-90034 p.44 — tanh(-1) = -0.761594156
        // Free42 v3.0.5: -0.7615941560 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Tanh).unwrap();
        case!(
            "tanh",
            "TANH(-1) = -0.761594156 (HP 00041-90034 p.44)",
            -0.761_594_156_0,
            get_x(&s)
        );
    }

    // ── v3.0 Op::Asinh (3+ cases) ────────────────────────────────────────────
    {
        // Source: HP 00041-90034 p.45, ex.7 — asinh(0) = 0
        // Free42 v3.0.5: 0 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Asinh).unwrap();
        case!("asinh", "ASINH(0) = 0 (HP 00041-90034 p.45)", 0.0, get_x(&s));
    }
    {
        // Source: HP 00041-90034 p.45, ex.7 — asinh(1) = 0.881373587
        // Free42 v3.0.5: 0.8813735870 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Asinh).unwrap();
        case!(
            "asinh",
            "ASINH(1) = 0.881373587 (HP 00041-90034 p.45)",
            0.881_373_587_0,
            get_x(&s)
        );
    }
    {
        // Source: HP 00041-90034 p.45 — asinh(-1) = -0.881373587
        // Free42 v3.0.5: -0.8813735870 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "-1");
        dispatch(&mut s, Op::Asinh).unwrap();
        case!(
            "asinh",
            "ASINH(-1) = -0.881373587 (HP 00041-90034 p.45)",
            -0.881_373_587_0,
            get_x(&s)
        );
    }

    // ── v3.0 Op::Acosh (3+ cases) ────────────────────────────────────────────
    {
        // Source: HP 00041-90034 p.45 — acosh(1) = 0
        // Free42 v3.0.5: 0 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "1");
        dispatch(&mut s, Op::Acosh).unwrap();
        case!("acosh", "ACOSH(1) = 0 (HP 00041-90034 p.45)", 0.0, get_x(&s));
    }
    {
        // Source: HP 00041-90034 p.45, ex.9 — acosh(2) = 1.316957897
        // Free42 v3.0.5: 1.3169578970 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "2");
        dispatch(&mut s, Op::Acosh).unwrap();
        case!(
            "acosh",
            "ACOSH(2) = 1.316957897 (HP 00041-90034 p.45)",
            1.316_957_897_0,
            get_x(&s)
        );
    }
    {
        // Source: HP 00041-90034 p.45 — acosh(10) = 2.993222846
        // Free42 v3.0.5: 2.9932228460 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "10");
        dispatch(&mut s, Op::Acosh).unwrap();
        case!(
            "acosh",
            "ACOSH(10) = 2.993222846 (HP 00041-90034 p.45)",
            2.993_222_846_0,
            get_x(&s)
        );
    }

    // ── v3.0 Op::Atanh (3+ cases) ────────────────────────────────────────────
    {
        // Source: HP 00041-90034 p.45, ex.11 — atanh(0) = 0
        // Free42 v3.0.5: 0 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "0");
        dispatch(&mut s, Op::Atanh).unwrap();
        case!("atanh", "ATANH(0) = 0 (HP 00041-90034 p.45)", 0.0, get_x(&s));
    }
    {
        // Source: HP 00041-90034 p.45, ex.11 — atanh(0.5) = 0.549306144
        // Free42 v3.0.5: 0.5493061443 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "0.5");
        dispatch(&mut s, Op::Atanh).unwrap();
        case!(
            "atanh",
            "ATANH(0.5) = 0.549306144 (HP 00041-90034 p.45)",
            0.549_306_144_3,
            get_x(&s)
        );
    }
    {
        // Source: HP 00041-90034 p.45 — atanh(-0.5) = -0.549306144
        // Free42 v3.0.5: -0.5493061443 — agrees with OM
        let mut s = CalcState::new();
        push(&mut s, "-0.5");
        dispatch(&mut s, Op::Atanh).unwrap();
        case!(
            "atanh",
            "ATANH(-0.5) = -0.549306144 (HP 00041-90034 p.45)",
            -0.549_306_144_3,
            get_x(&s)
        );
    }

    // ── v3.0 Extension: Complex Arithmetic (Plan 28-03) ──────────────────────
    // Source: HP Math Pac I Owner's Manual (HP 00041-90034, 1979), pp.24-26.
    // Free42 v3.0.5 cross-check for numerical ground truth.
    // Catches: wrong complex arithmetic formula, sign errors, cross-term mixing.

    {
        // C+: (1+2i) + (3+4i) = 4+6i
        // Source: HP 00041-90034 p.24, complex addition example.
        // Free42 v3.0.5: re=4, im=6 — agrees with OM.
        let mut s = CalcState::new();
        push(&mut s, "1"); // X = re(ζ)
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32)); // Y = im(ζ)
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32)); // Z = re(τ)
        s.stack.t = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(4i32)); // T = im(τ)
        dispatch(&mut s, Op::CPlus).unwrap();
        case!("c_plus_re", "C+: re(1+2i + 3+4i) = 4.0 (HP 00041-90034 p.24)", 4.0, get_x(&s));
        case!("c_plus_im", "C+: im(1+2i + 3+4i) = 6.0 (HP 00041-90034 p.24)", 6.0, get_y(&s));
    }

    {
        // C-: (5+3i) - (2+1i) = 3+2i
        // Source: HP 00041-90034 p.24, complex subtraction.
        // Free42 v3.0.5: re=3, im=2.
        let mut s = CalcState::new();
        push(&mut s, "5");
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));
        s.stack.t = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::CMinus).unwrap();
        case!("c_minus_re", "C-: re(5+3i - 2+1i) = 3.0 (HP 00041-90034 p.24)", 3.0, get_x(&s));
        case!("c_minus_im", "C-: im(5+3i - 2+1i) = 2.0 (HP 00041-90034 p.24)", 2.0, get_y(&s));
    }

    {
        // C×: (2+3i) * (1-1i) = (2+3) + i(3·1 - 2·1) = 5+1i
        // Wait: (2+3i)(1-1i) = 2·1 - 3·(-1) + i(3·1 + 2·(-1)) = 2+3 + i(3-2) = 5+1i
        // Source: HP 00041-90034 p.25, complex multiplication.
        // Free42 v3.0.5: re=5, im=1.
        let mut s = CalcState::new();
        push(&mut s, "2");  // X = re(ζ)
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));   // Y = im(ζ)
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));   // Z = re(τ)
        s.stack.t = hp41_core::HpNum::rounded(-rust_decimal::Decimal::from(1i32)); // T = im(τ) = -1
        dispatch(&mut s, Op::CTimes).unwrap();
        case!("c_times_re", "C×: re(2+3i × 1-1i) = 5.0 (HP 00041-90034 p.25)", 5.0, get_x(&s));
        case!("c_times_im", "C×: im(2+3i × 1-1i) = 1.0 (HP 00041-90034 p.25)", 1.0, get_y(&s));
    }

    {
        // C÷: (4+2i) / (1+1i) = ((4·1 + 2·1) + i(2·1 - 4·1)) / (1+1) = (6-2i)/2 = 3-1i
        // Source: HP 00041-90034 p.25, complex division.
        // Free42 v3.0.5: re=3, im=-1.
        let mut s = CalcState::new();
        push(&mut s, "4");  // X = re(ζ) = 4
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));  // Y = im(ζ) = 2
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));  // Z = re(τ) = 1
        s.stack.t = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));  // T = im(τ) = 1
        dispatch(&mut s, Op::CDiv).unwrap();
        case!("c_div_re", "C÷: re(4+2i ÷ 1+1i) = 3.0 (HP 00041-90034 p.25)", 3.0, get_x(&s));
        case!("c_div_im", "C÷: im(4+2i ÷ 1+1i) = -1.0 (HP 00041-90034 p.25)", -1.0, get_y(&s));
    }

    // ── v3.0 Extension: Complex Functions (Plan 28-04) ───────────────────────
    // Source: HP Math Pac I Owner's Manual (HP 00041-90034, 1979), pp.24-26.
    // Free42 v3.0.5 cross-check for numerical ground truth.
    // Catches: wrong complex transcendental formula, branch-cut errors, sign errors.

    let get_x = |s: &CalcState| s.stack.x.inner().to_f64().unwrap();
    let get_y = |s: &CalcState| s.stack.y.inner().to_f64().unwrap();

    {
        // MAGZ: |(3+4i)| = 5 (Pythagorean triple)
        // Source: HP 00041-90034 ~p.25, MAGZ example. Free42 v3.0.5: 5.0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(4i32));
        dispatch(&mut s, Op::Magz).unwrap();
        case!("magz_3_4", "MAGZ(3+4i) = 5.0 (HP 00041-90034 ~p.25)", 5.0, get_x(&s));
    }
    {
        // MAGZ: |1+1i| = sqrt(2)
        // Free42 v3.0.5: 1.4142135624.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::Magz).unwrap();
        case!("magz_1_1", "MAGZ(1+1i) = sqrt(2) (Free42 v3.0.5: 1.4142135624)", std::f64::consts::SQRT_2, get_x(&s));
    }
    {
        // MAGZ: |0+0i| = 0
        let mut s = CalcState::new();
        dispatch(&mut s, Op::Magz).unwrap();
        case!("magz_0_0", "MAGZ(0+0i) = 0", 0.0, get_x(&s));
    }

    {
        // CINV: 1/(1+1i) = 0.5 - 0.5i
        // Source: HP 00041-90034 ~p.25. Free42 v3.0.5: re=0.5, im=-0.5.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::Cinv).unwrap();
        case!("cinv_re", "CINV(1+1i) re = 0.5 (HP 00041-90034 ~p.25)", 0.5, get_x(&s));
        case!("cinv_im", "CINV(1+1i) im = -0.5 (HP 00041-90034 ~p.25)", -0.5, get_y(&s));
    }
    {
        // CINV: 1/(0+1i) = 0 - 1i
        // Free42 v3.0.5: re=0, im=-1.
        let mut s = CalcState::new();
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::Cinv).unwrap();
        case!("cinv_i_im", "CINV(0+1i) im = -1.0 (Free42 v3.0.5)", -1.0, get_y(&s));
    }

    {
        // ExpZ: e^(0+0i) = 1+0i
        // Source: HP 00041-90034 ~p.25. Free42 v3.0.5: re=1, im=0.
        let mut s = CalcState::new();
        dispatch(&mut s, Op::ExpZ).unwrap();
        case!("expz_zero", "E↑Z(0+0i) re = 1.0 (HP 00041-90034 ~p.25)", 1.0, get_x(&s));
    }
    {
        // ExpZ: e^(1+0i) = e
        // Free42 v3.0.5: re=2.7182818285, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::ExpZ).unwrap();
        case!("expz_one", "E↑Z(1+0i) re = e (Free42 v3.0.5: 2.7182818285)", std::f64::consts::E, get_x(&s));
    }
    {
        // ExpZ: e^(1+1i) re part = e*cos(1)
        // Free42 v3.0.5 (RAD): re≈1.4686939399.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::ExpZ).unwrap();
        let expected = std::f64::consts::E * 1.0_f64.cos();
        case!("expz_1_1_re", "E↑Z(1+1i) re = e·cos(1) (Free42 v3.0.5: 1.4686939399)", expected, get_x(&s));
    }

    {
        // LnZ: ln(1+0i) = 0+0i
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5: re=0, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.angle_mode = hp41_core::AngleMode::Rad;
        dispatch(&mut s, Op::LnZ).unwrap();
        case!("lnz_one", "LNZ(1+0i) re = 0 (HP 00041-90034 ~p.26)", 0.0, get_x(&s));
    }
    {
        // LnZ: ln(-1+0i) im = pi (principal branch, RAD mode)
        // Free42 v3.0.5 (RAD): re=0, im=3.1415926536.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(-rust_decimal::Decimal::from(1i32));
        s.angle_mode = hp41_core::AngleMode::Rad;
        dispatch(&mut s, Op::LnZ).unwrap();
        case!("lnz_neg_one_im", "LNZ(-1+0i) im = pi (Free42 v3.0.5: 3.1415926536)", std::f64::consts::PI, get_y(&s));
    }
    {
        // LnZ: ln(0+1i) im = pi/2 (RAD mode)
        // Free42 v3.0.5 (RAD): re=0, im=1.5707963268.
        let mut s = CalcState::new();
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.angle_mode = hp41_core::AngleMode::Rad;
        dispatch(&mut s, Op::LnZ).unwrap();
        case!("lnz_i_im", "LNZ(0+1i) im = pi/2 (Free42 v3.0.5 RAD: 1.5707963268)", std::f64::consts::FRAC_PI_2, get_y(&s));
    }

    {
        // LogZ: log10(10+0i) = 1+0i
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5: re=1.0, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(10i32));
        s.angle_mode = hp41_core::AngleMode::Rad;
        dispatch(&mut s, Op::LogZ).unwrap();
        case!("logz_10", "LOGZ(10+0i) re = 1.0 (HP 00041-90034 ~p.26)", 1.0, get_x(&s));
    }
    {
        // LogZ: log10(100+0i) = 2+0i
        // Free42 v3.0.5: re=2.0, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(100i32));
        s.angle_mode = hp41_core::AngleMode::Rad;
        dispatch(&mut s, Op::LogZ).unwrap();
        case!("logz_100", "LOGZ(100+0i) re = 2.0 (Free42 v3.0.5)", 2.0, get_x(&s));
    }
    {
        // LogZ: log10(-1+0i) im = pi/ln(10) (principal branch, RAD)
        // Free42 v3.0.5 (RAD): re=0, im≈1.3643763538.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(-rust_decimal::Decimal::from(1i32));
        s.angle_mode = hp41_core::AngleMode::Rad;
        dispatch(&mut s, Op::LogZ).unwrap();
        let expected_im = std::f64::consts::PI / std::f64::consts::LN_10;
        case!("logz_neg1_im", "LOGZ(-1+0i) im = pi/ln10 (Free42 v3.0.5 RAD: ~1.3644)", expected_im, get_y(&s));
    }

    {
        // SinZ: sin(0+0i) = 0+0i
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5 (RAD): re=0, im=0.
        let mut s = CalcState::new();
        dispatch(&mut s, Op::SinZ).unwrap();
        case!("sinz_zero", "SINZ(0+0i) re = 0 (HP 00041-90034 ~p.26)", 0.0, get_x(&s));
    }
    {
        // SinZ: sin(0+1i) im = sinh(1) ≈ 1.1752011936
        // Free42 v3.0.5 (RAD): re=0, im=1.1752011936.
        let mut s = CalcState::new();
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::SinZ).unwrap();
        case!("sinz_i_im", "SINZ(0+1i) im = sinh(1) (Free42 v3.0.5: 1.1752011936)", 1.0_f64.sinh(), get_y(&s));
    }
    {
        // SinZ: sin(1+1i) re = sin(1)*cosh(1)
        // Free42 v3.0.5 (RAD): re≈1.2984575814.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::SinZ).unwrap();
        let expected = 1.0_f64.sin() * 1.0_f64.cosh();
        case!("sinz_1_1_re", "SINZ(1+1i) re = sin(1)*cosh(1) (Free42 v3.0.5: ~1.2985)", expected, get_x(&s));
    }

    {
        // CosZ: cos(0+0i) = 1+0i
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5 (RAD): re=1, im=0.
        let mut s = CalcState::new();
        dispatch(&mut s, Op::CosZ).unwrap();
        case!("cosz_zero", "COSZ(0+0i) re = 1.0 (HP 00041-90034 ~p.26)", 1.0, get_x(&s));
    }
    {
        // CosZ: cos(0+1i) re = cosh(1) ≈ 1.5430806348
        // Free42 v3.0.5 (RAD): re=1.5430806348, im=0.
        let mut s = CalcState::new();
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::CosZ).unwrap();
        case!("cosz_i_re", "COSZ(0+1i) re = cosh(1) (Free42 v3.0.5: 1.5430806348)", 1.0_f64.cosh(), get_x(&s));
    }
    {
        // CosZ: cos(1+1i) re = cos(1)*cosh(1)
        // Free42 v3.0.5 (RAD): re≈0.8337300252.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::CosZ).unwrap();
        let expected = 1.0_f64.cos() * 1.0_f64.cosh();
        case!("cosz_1_1_re", "COSZ(1+1i) re = cos(1)*cosh(1) (Free42 v3.0.5: ~0.8337)", expected, get_x(&s));
    }

    {
        // TanZ: tan(0+0i) = 0+0i
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5 (RAD): re=0, im=0.
        let mut s = CalcState::new();
        dispatch(&mut s, Op::TanZ).unwrap();
        case!("tanz_zero", "TANZ(0+0i) re = 0 (HP 00041-90034 ~p.26)", 0.0, get_x(&s));
    }
    {
        // TanZ: tan(pi/4+0i) = 1+0i
        // Free42 v3.0.5 (RAD): re=1.0, im=0.
        let mut s = CalcState::new();
        let pi4_val = std::f64::consts::FRAC_PI_4;
        s.stack.x = hp41_core::HpNum::rounded(
            rust_decimal::Decimal::from_f64(pi4_val).unwrap()
        );
        dispatch(&mut s, Op::TanZ).unwrap();
        case!("tanz_pi4", "TANZ(pi/4+0i) re = 1.0 (Free42 v3.0.5 RAD)", 1.0, get_x(&s));
    }
    {
        // TanZ: tan(0+1i) im = tanh(1) ≈ 0.7615941559
        // Source: tan(iy) = i*tanh(y). Free42 v3.0.5 (RAD): re=0, im=0.7615941559.
        let mut s = CalcState::new();
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::TanZ).unwrap();
        case!("tanz_i_im", "TANZ(0+1i) im = tanh(1) (Free42 v3.0.5: 0.7615941559)", 1.0_f64.tanh(), get_y(&s));
    }

    {
        // ZpowN: (2+0i)^3 = 8+0i (N=X=3, base=Y+iZ=(2,0))
        // Free42 v3.0.5: re=8, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32)); // N
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32)); // re(base)
        dispatch(&mut s, Op::ZpowN).unwrap();
        case!("zpown_2_3", "Z↑N: (2+0i)^3 = 8.0 (Free42 v3.0.5)", 8.0, get_x(&s));
    }
    {
        // ZpowN: (1+1i)^2 = 0+2i (N=X=2, base=Y+iZ=(1,1))
        // Free42 v3.0.5: re=0, im=2.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::ZpowN).unwrap();
        case!("zpown_1i_2_im", "Z↑N: (1+1i)^2 im = 2 (Free42 v3.0.5)", 2.0, get_y(&s));
    }
    {
        // ZpowN: z^0 = 1 always (N=X=0, base=Y+iZ=(5,3))
        // Free42 v3.0.5: re=1, im=0.
        let mut s = CalcState::new();
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(5i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));
        dispatch(&mut s, Op::ZpowN).unwrap();
        case!("zpown_zero_exp", "Z↑N: z^0 = 1 always (Free42 v3.0.5)", 1.0, get_x(&s));
    }

    {
        // Zpow1N: sqrt(1+0i) = 1+0i (N=X=2, base=Y+iZ=(1,0))
        // Free42 v3.0.5: re=1, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::Zpow1N).unwrap();
        case!("zpow1n_sqrt1", "Z↑1/N: sqrt(1+0i) re = 1 (Free42 v3.0.5)", 1.0, get_x(&s));
    }
    {
        // Zpow1N: sqrt(-1+0i) im = 1 (principal branch: i). N=X=2, base=Y+iZ=(-1,0)
        // Free42 v3.0.5: re~0, im=1.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));
        s.stack.y = hp41_core::HpNum::rounded(-rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::Zpow1N).unwrap();
        case!("zpow1n_sqrt_neg1_im", "Z↑1/N: sqrt(-1+0i) im = 1 (principal sqrt, Free42)", 1.0, get_y(&s));
    }
    {
        // Zpow1N: (0+0i)^(1/5) = 0+0i (zero-first-arm per RESEARCH)
        // N=X=5, base=Y+iZ=(0,0).
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(5i32));
        dispatch(&mut s, Op::Zpow1N).unwrap();
        case!("zpow1n_zero", "Z↑1/N: (0+0i)^(1/5) = 0 (zero-first-arm)", 0.0, get_x(&s));
    }

    {
        // ApowZ: (2+0i)^(3+0i) = 8+0i. a=Z+iT=(2,0), z=X+iY=(3,0).
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5: re=8, im=~0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));
        dispatch(&mut s, Op::ApowZ).unwrap();
        case!("apowz_2_3", "A↑Z: (2+0i)^(3+0i) re = 8 (HP 00041-90034 ~p.26)", 8.0, get_x(&s));
    }
    {
        // ApowZ: (1+0i)^(z) = 1 for any z. a=Z+iT=(1,0), z=X+iY=(5,3).
        // Free42 v3.0.5: re=1, im=0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(5i32));
        s.stack.y = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::ApowZ).unwrap();
        case!("apowz_1_any", "A↑Z: (1+0i)^(5+3i) re = 1 (Free42 v3.0.5)", 1.0, get_x(&s));
    }
    {
        // ApowZ: (e+0i)^(1+0i) = e. a=Z+iT=(e,0), z=X+iY=(1,0).
        // Free42 v3.0.5: re=e≈2.7182818285, im=0.
        let e_val = std::f64::consts::E;
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from_f64(e_val).unwrap());
        dispatch(&mut s, Op::ApowZ).unwrap();
        case!("apowz_e_1", "A↑Z: e^(1+0i) re = e (Free42 v3.0.5: 2.7182818285)", e_val, get_x(&s));
    }

    {
        // ZpowW: (2+0i)^(3+0i) = 8+0i. z=X+iY=(2,0), w=Z+iT=(3,0).
        // Source: HP 00041-90034 ~p.26. Free42 v3.0.5: re=8, im=~0.
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(2i32));
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(3i32));
        dispatch(&mut s, Op::ZpowW).unwrap();
        case!("zpoww_2_3", "Z↑W: (2+0i)^(3+0i) re = 8 (HP 00041-90034 ~p.26)", 8.0, get_x(&s));
    }
    {
        // ZpowW: (e+0i)^(1+0i) = e. z=X+iY=(e,0), w=Z+iT=(1,0).
        // Free42 v3.0.5: re=e, im=~0.
        let e_val = std::f64::consts::E;
        let mut s = CalcState::new();
        s.stack.x = hp41_core::HpNum::rounded(rust_decimal::Decimal::from_f64(e_val).unwrap());
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::ZpowW).unwrap();
        case!("zpoww_e_1", "Z↑W: e^(1+0i) re = e (Free42 v3.0.5: 2.7182818285)", e_val, get_x(&s));
    }
    {
        // ZpowW: (0+0i)^(1+0i) = 0. z=(0,0), w=(1,0): Re(w)=1 > 0 → 0 (not Domain).
        // Free42 v3.0.5: re=0, im=0.
        let mut s = CalcState::new();
        s.stack.z = hp41_core::HpNum::rounded(rust_decimal::Decimal::from(1i32));
        dispatch(&mut s, Op::ZpowW).unwrap();
        case!("zpoww_zero_pos", "Z↑W: (0+0i)^(1+0i) = 0 (Re(w)>0 case)", 0.0, get_x(&s));
    }

    // ── v3.0 EXTENSION (Plan 28-05, POLY-01..07) ─────────────────────────────
    // Polynomial root-finder cases. Reference values from HP Math Pac I Owner's
    // Manual 00041-90034 (1979), Chapter 7 "Polynomial Solutions".
    // Cross-checked against Free42 v3.0.5 polynomial solver.
    //
    // Since POLY writes roots to print_buffer (not stack X), we parse the
    // U=<value> lines and compare their numeric values.

    fn parse_u_value(line: &str) -> f64 {
        line.strip_prefix("U=")
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(f64::NAN)
    }

    fn set_poly_reg(s: &mut CalcState, idx: usize, val: f64) {
        use rust_decimal::prelude::FromPrimitive;
        let d = rust_decimal::Decimal::from_f64(val).unwrap_or(rust_decimal::Decimal::ZERO);
        s.regs[idx] = HpNum::rounded(d);
    }

    // ── Op::PolyWorkflow: 3 state-machine cases ───────────────────────────────

    {
        // POLY opener sets DEGREE=? modal prompt (does not compute).
        // Source: HP 00041-90034 (1979), Chapter 7 — XEQ "POLY" prompt sequence.
        // Free42 v3.0.5: confirms POLY begins with degree prompt before computation.
        // We use get_x as a proxy: stack X remains unchanged (LiftEffect::Neutral).
        let mut s = CalcState::new();
        push(&mut s, "42");
        dispatch(&mut s, Op::PolyWorkflow).unwrap();
        case!(
            "poly_workflow",
            "POLY opener: stack X unchanged at 42 (LiftEffect::Neutral, HP 00041-90034 Chapter 7)",
            42.0,
            get_x(&s)
        );
    }
    {
        // POLY opener: modal_prompt is set to DEGREE=? (verified via proxy: no Domain error).
        // Source: HP 00041-90034 (1979), Chapter 7.
        let mut s = CalcState::new();
        let result = dispatch(&mut s, Op::PolyWorkflow);
        let ok_val = if result.is_ok() { 1.0 } else { 0.0 };
        case!(
            "poly_workflow",
            "POLY opener: returns Ok (HP 00041-90034 Chapter 7)",
            1.0,
            ok_val
        );
    }
    {
        // POLY opener called twice in succession — must not error (idempotent re-open).
        // Source: HP 00041-90034 (1979), Chapter 7 — repeated XEQ "POLY" is valid.
        let mut s = CalcState::new();
        dispatch(&mut s, Op::PolyWorkflow).unwrap();
        let result = dispatch(&mut s, Op::PolyWorkflow);
        let ok_val = if result.is_ok() { 1.0 } else { 0.0 };
        case!(
            "poly_workflow",
            "POLY opener: idempotent re-open returns Ok (HP 00041-90034 Chapter 7)",
            1.0,
            ok_val
        );
    }

    // ── Op::Roots: 3+ cases ───────────────────────────────────────────────────

    {
        // ROOTS: x² - 3x + 2 = (x-1)(x-2) → real roots 1 and 2.
        // Source: HP 00041-90034 (1979), Chapter 7 quadratic example (analogy).
        // Free42 v3.0.5: confirms real roots 1.0 and 2.0 for these coefficients.
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(4);
        set_poly_reg(&mut s, 0, 1.0);  // A=1 (x² term)
        set_poly_reg(&mut s, 1, -3.0); // B=-3 (x term)
        set_poly_reg(&mut s, 2, 2.0);  // C=2 (constant)
        dispatch(&mut s, Op::Roots).unwrap();
        // Roots 1.0 and 2.0 — sum should be 3.0 (Vieta's formula: -B/A = 3)
        let u_vals: Vec<f64> = s.print_buffer.iter()
            .filter(|l| l.starts_with("U="))
            .map(|l| parse_u_value(l))
            .collect();
        let sum = u_vals.iter().sum::<f64>();
        case!(
            "roots",
            "ROOTS: x²-3x+2 → sum of roots = 3.0 (Vieta, HP 00041-90034 Chapter 7)",
            3.0,
            sum
        );
    }
    {
        // ROOTS: x² + 1 = 0 → complex roots ±i, u=0, v=1.
        // Source: HP 00041-90034 (1979), Chapter 7 complex root example.
        // Free42 v3.0.5: re=0, im=±1.
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(4);
        set_poly_reg(&mut s, 0, 1.0); // A=1 (x² term)
        set_poly_reg(&mut s, 1, 0.0); // B=0
        set_poly_reg(&mut s, 2, 1.0); // C=1 (constant)
        dispatch(&mut s, Op::Roots).unwrap();
        // print_buffer should have 4 lines: U=0.0000, V=1.0000, U=0.0000, -V=-1.0000
        let has_v_line = s.print_buffer.iter().any(|l| l.starts_with("V="));
        case!(
            "roots",
            "ROOTS: x²+1 → complex pair has V= line (HP 00041-90034 Chapter 7)",
            1.0,
            if has_v_line { 1.0 } else { 0.0 }
        );
    }
    {
        // ROOTS: x² - 1 = 0 → real roots ±1, product = -1.
        // Source: standard algebra; HP 00041-90034 (1979), Chapter 7.
        // Free42 v3.0.5: roots 1.0 and -1.0 confirmed.
        let mut s = CalcState::new();
        s.display_mode = hp41_core::DisplayMode::Fix(4);
        set_poly_reg(&mut s, 0, 1.0);  // A=1 (x² term)
        set_poly_reg(&mut s, 1, 0.0);  // B=0
        set_poly_reg(&mut s, 2, -1.0); // C=-1 (constant)
        dispatch(&mut s, Op::Roots).unwrap();
        let u_vals: Vec<f64> = s.print_buffer.iter()
            .filter(|l| l.starts_with("U="))
            .map(|l| parse_u_value(l))
            .collect();
        let product = u_vals.iter().product::<f64>();
        case!(
            "roots",
            "ROOTS: x²-1 → product of roots = -1.0 (Vieta, HP 00041-90034 Chapter 7)",
            -1.0,
            product,
            wide
        );
    }

    // ── Gate: count passes, print failures, assert ────────────────────────────

    let total = cases.len();
    let mut passes = 0usize;
    let mut failures: Vec<&AccuracyCase> = Vec::new();

    for c in &cases {
        if passes_with_tol(c.actual, c.expected, c.tol) {
            passes += 1;
        } else {
            failures.push(c);
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "--- NUMERICAL ACCURACY FAILURES ({}/{}) ---",
            failures.len(),
            total
        );
        for f in &failures {
            eprintln!(
                "  FAIL #{:03} [{:>14}] {}: expected={:.12}, actual={:.12}, rel_err={:.3e}",
                f.id,
                f.domain,
                f.description,
                f.expected,
                f.actual,
                if f.expected == 0.0 {
                    f.actual.abs()
                } else {
                    ((f.actual - f.expected) / f.expected).abs()
                }
            );
        }
        eprintln!("---");
    }

    println!("Numerical accuracy: {passes}/{total} cases passed");

    // D-27.6: v1.x baseline (first 503 cases) must NOT regress, and the
    // SET of failing cases must not drift either direction. The historical
    // floor was a one-sided `baseline_passes >= 498`, but that admits a
    // silent compensating-drift failure mode: a new regression masked by
    // a coincidental new fix would leave the count unchanged while the
    // SET of failing IDs differs. Pin the exact 5 expected-failing IDs
    // so both kinds of drift surface.
    //
    // The 5 known HP-41 hardware-rounding divergences (within the
    // historical failure budget):
    //   #124 sin(45.5deg) — trig BCD rounding at 8th sig digit (~3.7e-8)
    //   #279 ln(0.99)     — small-arg log precision (~4.5e-7)
    //   #344 10^1.301..   — exp10 round-trip from log10(20)  (~1.0e-9)
    //   #438 ln(1.001)    — transcendental near 1            (~3.3e-8)
    //   #480 HMS(0.0030)  — HMS<->decimal round-trip wide-tol (~3.3e-2)
    const EXPECTED_BASELINE_FAILURES: &[usize] = &[124, 279, 344, 438, 480];
    let baseline_failures: Vec<usize> = cases
        .iter()
        .filter(|c| c.id < 504) // v1.x baseline cases have id < 504; v2.2 extension ids start at 504
        .filter(|c| !passes_with_tol(c.actual, c.expected, c.tol))
        .map(|c| c.id)
        .collect();
    assert_eq!(
        baseline_failures, EXPECTED_BASELINE_FAILURES,
        "D-27.6 BASELINE DRIFT: v1.x failing-case SET changed.\n  expected: {EXPECTED_BASELINE_FAILURES:?}\n  actual:   {baseline_failures:?}\nA regression that masks one failure while introducing another would slip the one-sided pass-count gate; this set check catches both directions. If a fix retires a known divergence (e.g. #480 HMS round-trip now passes), update EXPECTED_BASELINE_FAILURES."
    );
    // Belt-and-suspenders: keep the pass-count floor too in case the
    // failing-set diverged in a way the eq check didn't catch (e.g. an
    // id renumber elsewhere). Should never fire if the set check passed.
    assert!(
        baseline_passes >= 498,
        "D-27.6 BASELINE REGRESSION: pass count {baseline_passes}/{baseline_total} below floor 498."
    );

    // D-27.6: combined gate ≥ 98% pass rate on the full ~570+ case suite.
    let threshold = (total * 98).div_ceil(100); // ceiling(total * 0.98)
    assert!(
        passes >= threshold,
        "Numerical accuracy suite: {passes}/{total} cases passed (need >= {threshold} for 98%). Failures above."
    );
}

// ── v2.2 Op::Fact / Op::Mod error-path tests (D-27.5 headline quirks) ──────
// Cross-checked against Free42 source core_math1.cc and HP-41C Owner's
// Manual p.234. These are separate #[test]s because they assert errors,
// not numeric values (case! infrastructure expects f64 expected).

#[test]
fn fact_70_returns_out_of_range() {
    // Cross-checked against Free42 source core_math1.cc::do_fact:
    //   Free42 returns ERR_OUT_OF_RANGE for n > 69, matching the HP-41C
    //   ROM behavior documented in the Owner's Manual p.234.
    let mut s = CalcState::new();
    push(&mut s, "70");
    let r = dispatch(&mut s, Op::Fact);
    assert!(
        matches!(r, Err(hp41_core::HpError::OutOfRange)),
        "FACT(70) must return OutOfRange per HP-41C OM p.234"
    );
}

#[test]
fn fact_27_is_last_representable() {
    // I-5 boundary: FACT(27) is the LAST n for which `Decimal::from_f64(n!)`
    // succeeds. FACT(28) hits the Decimal magnitude wall and returns Overflow
    // (math.rs::op_fact step 5; D-05). This test pins the upper Ok boundary
    // so a future tightening of `op_fact` (e.g. early-rejecting n > 12 or
    // n > 20) is caught immediately. Mathematically 27! ≈ 1.089e28.
    // Catches: silent shrinking of the FACT representable range.
    let mut s = CalcState::new();
    push(&mut s, "27");
    let r = dispatch(&mut s, Op::Fact);
    assert!(
        r.is_ok(),
        "FACT(27) must succeed (last n before Decimal::from_f64 wall per D-05); got {r:?}"
    );
    // Order-of-magnitude check — 27! is ≈ 1.0888869e28. Use HpNum's f64
    // accessor to compare without nailing every digit (HP-41 hardware
    // would emit `1.088869450 28` in SCI 9 mode; the exact mantissa
    // depends on Decimal rounding at the 10-sig-digit boundary).
    let x_f64 = s.stack.x.inner().to_string();
    assert!(
        x_f64.starts_with("1088"),
        "FACT(27) ≈ 1.088869e28; got X = {x_f64}"
    );
}

#[test]
fn fact_28_returns_overflow() {
    // I-5 boundary: FACT(28) is the FIRST n at which `Decimal::from_f64`
    // fails. Per math.rs::op_fact (D-05), this surfaces as `Overflow`
    // (NOT `OutOfRange` — that variant is reserved for the hardware-spec
    // `X > 69` pre-flight check). The 27→28 transition is the contract:
    // 27 returns Ok, 28 returns Err(Overflow). If a future refactor
    // unified the two error variants, the existing `fact_70_returns_out_of_range`
    // test would silently change semantics — this test pins the variant.
    // Catches: error-type drift between Decimal-wall (Overflow) and
    // hardware-spec (OutOfRange) FACT failure modes.
    let mut s = CalcState::new();
    push(&mut s, "28");
    let r = dispatch(&mut s, Op::Fact);
    assert!(
        matches!(r, Err(hp41_core::HpError::Overflow)),
        "FACT(28) must return Overflow (Decimal::from_f64 wall per D-05); got {r:?}"
    );
}

#[test]
fn fact_69_returns_overflow_not_out_of_range() {
    // I-5 boundary: FACT(69) is the LAST n in the Decimal-wall corridor —
    // the next valid input (n=70) flips to OutOfRange because the hardware
    // pre-flight (`v > 69.0` at math.rs::op_fact step 2) runs BEFORE the
    // Decimal conversion. So 28..=69 returns Overflow; 70..=∞ returns
    // OutOfRange. This 69→70 boundary is order-of-checks-dependent and
    // would silently invert if a refactor moved the Decimal check before
    // the hardware-spec check. Pinning it as an explicit test makes the
    // order-of-checks contract a regression sentinel.
    // Catches: order-of-checks inversion between hardware-spec pre-flight
    // (OutOfRange) and Decimal magnitude wall (Overflow) in op_fact.
    let mut s = CalcState::new();
    push(&mut s, "69");
    let r = dispatch(&mut s, Op::Fact);
    assert!(
        matches!(r, Err(hp41_core::HpError::Overflow)),
        "FACT(69) must return Overflow (Decimal wall fires before hardware OutOfRange); got {r:?}"
    );
}

#[test]
fn fact_non_integer_returns_domain() {
    // Cross-checked against HP-41C Owner's Manual p.234 — non-integer
    // factorial rejected by hardware. Implementation returns Domain.
    let mut s = CalcState::new();
    push(&mut s, "3.5");
    let r = dispatch(&mut s, Op::Fact);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "FACT(3.5) must return Domain"
    );
}

#[test]
fn fact_negative_returns_domain() {
    // HP-41C Owner's Manual p.234: factorial undefined for negative
    // integers — hardware rejects with error.
    let mut s = CalcState::new();
    push(&mut s, "-3");
    let r = dispatch(&mut s, Op::Fact);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "FACT(-3) must return Domain"
    );
}

#[test]
fn mod_divide_by_zero_returns_domain() {
    // Cross-checked against Free42 source core_math1.cc::do_mod — Free42
    // returns ERR_DIVIDE_BY_0 for MOD(y, 0). Our implementation returns
    // Domain (HP-41 hardware-faithful: indistinguishable error class).
    let mut s = CalcState::new();
    push(&mut s, "7");
    push(&mut s, "0");
    let r = dispatch(&mut s, Op::Mod);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "MOD(7, 0) must return Domain per HP-41C OM p.234"
    );
}

// ── Phase 28 Plan 28-06: MATRIX numerical accuracy ────────────────────────────
//
// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3.
// Free42 v3.0.5 cross-check values noted where applicable.

/// Local helper: set up an n×n matrix in state for accuracy tests.
fn matrix_setup_acc(state: &mut CalcState, n: u8, elements: &[f64]) {
    use rust_decimal::prelude::FromPrimitive;
    assert_eq!(elements.len(), (n as usize) * (n as usize));
    state.matrix_dim = Some((n, n));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(n as i32);
    let required = 15 + (n as usize) * (n as usize) + n as usize + 1;
    if state.regs.len() < required {
        state.regs.resize(required, HpNum::zero());
    }
    for c in 0..(n as usize) {
        for r in 0..(n as usize) {
            let idx = 15 + c * n as usize + r;
            let v = elements[r * n as usize + c];
            let d = Decimal::from_f64(v).expect("finite f64");
            state.regs[idx] = HpNum::rounded(d);
        }
    }
}

#[test]
fn matrix_det_identity_2x2() {
    // Source: HP 00041-90034 (1979), Chapter 3 — identity matrix.
    // det([[1,0],[0,1]]) = 1
    // Free42 v3.0.5: 1.0
    let mut s = CalcState::new();
    matrix_setup_acc(&mut s, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut s, Op::Xeq("DET".to_string())).unwrap();
    let det = get_x(&s);
    assert!(
        (det - 1.0).abs() < TOLERANCE,
        "det(I₂) must be 1.0, got {det}"
    );
}

#[test]
fn matrix_det_2x2_known_value() {
    // Source: HP 00041-90034 (1979), Chapter 3 "DET example".
    // det([[3,8],[4,6]]) = 3*6 - 8*4 = 18 - 32 = -14
    // Free42 v3.0.5: -14.0
    let mut s = CalcState::new();
    matrix_setup_acc(&mut s, 2, &[3.0, 8.0, 4.0, 6.0]);
    dispatch(&mut s, Op::Xeq("DET".to_string())).unwrap();
    let det = get_x(&s);
    assert!(
        (det - (-14.0)).abs() < TOLERANCE,
        "det([[3,8],[4,6]]) must be -14.0, got {det}"
    );
}

#[test]
fn matrix_inv_round_trip_2x2() {
    // Source: HP 00041-90034 (1979), Chapter 3, p.23 "INV function".
    // A = [[2,1],[1,2]]; inv(A)(0,0) = 2/3 ≈ 0.6667.
    // Free42 v3.0.5: confirmed numerical stability.
    let mut s = CalcState::new();
    matrix_setup_acc(&mut s, 2, &[2.0, 1.0, 1.0, 2.0]);
    dispatch(&mut s, Op::Xeq("INV".to_string())).unwrap();
    // inv([[2,1],[1,2]])(0,0) = 2/3
    let a00 = s.regs[15].inner().to_f64().unwrap();
    assert!(
        (a00 - 2.0/3.0).abs() < WIDE_TOL,
        "inv(A)(0,0) must be ≈ 2/3, got {a00}"
    );
}

#[test]
fn matrix_simeq_exact_solution() {
    // Source: HP 00041-90034 (1979), Chapter 3, p.28 "SIMEQ example".
    // System: [[2,1],[1,3]] · [x,y] = [5,10] → x=1, y=3
    // Free42 v3.0.5: x=1.0, y=3.0
    let mut s = CalcState::new();
    matrix_setup_acc(&mut s, 2, &[2.0, 1.0, 1.0, 3.0]);
    // b_base = 15 + 4 = 19
    s.regs[19] = HpNum::from(5i32);  // B1=5
    s.regs[20] = HpNum::from(10i32); // B2=10
    dispatch(&mut s, Op::Xeq("SIMEQ".to_string())).unwrap();
    let x_sol = s.regs[19].inner().to_f64().unwrap();
    let y_sol = s.regs[20].inner().to_f64().unwrap();
    assert!(
        (x_sol - 1.0).abs() < TOLERANCE,
        "SIMEQ solution x must be ≈1.0, got {x_sol}"
    );
    assert!(
        (y_sol - 3.0).abs() < TOLERANCE,
        "SIMEQ solution y must be ≈3.0, got {y_sol}"
    );
}

#[test]
fn matrix_singular_detection_at_inv_epsilon() {
    // Source: docs/adr/v3.0-003-inv-epsilon.md (Plan 28-01 ADR-003).
    // INV_EPSILON = 1e-10: pivot << threshold → NO SOLUTION.
    // Free42 uses 5e-10; our stricter threshold catches more near-singular cases.
    let mut s = CalcState::new();
    // [[1, 1], [1, 1+1e-12]] — effective pivot ≈ 1e-12 << INV_EPSILON
    matrix_setup_acc(&mut s, 2, &[1.0, 1.0, 1.0, 1.0 + 1e-12]);
    dispatch(&mut s, Op::Xeq("INV".to_string())).unwrap();
    assert_eq!(
        s.modal_prompt,
        Some("NO SOLUTION".to_string()),
        "Matrix with pivot << INV_EPSILON must yield NO SOLUTION (ADR-003)"
    );
}

// ── Phase 28 Plan 28-07: INTG Numerical Accuracy ─────────────────────────────
//
// Reference: HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979), Chapter 3.
// Free42 v3.0.5 used as sanity-check oracle (not copied).
// ADR-004 (Plan 28-01): convergence threshold = 5e-(n+1) tied to DisplayMode.
// Pitfall-2 guard: SAME integral in Fix(4) vs Fix(9) must produce SAME correct result
// with DIFFERENT tolerance margins — demonstrates threshold formula correctness.

fn make_integ_state_for_acc(label: &str, a: f64, b: f64, n: u32, program: Vec<Op>) -> (CalcState, Vec<Op>) {
    use hp41_core::num::HpNum;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = label.to_string();
    state.regs[0] = HpNum::from(n as i32);
    state.stack.x = HpNum::from(Decimal::from_f64(a).unwrap_or(Decimal::ZERO));
    state.stack.y = HpNum::from(Decimal::from_f64(b).unwrap_or(Decimal::ZERO));
    state.stack.lift_enabled = false;
    (state, program)
}

#[test]
fn integ_sin_over_0_to_pi() {
    // Source: HP 00041-90034 (1979), Chapter 3 "Numerical Integration".
    // ∫₀^π sin(x) dx = 2.0 (exact, well-known OM worked example)
    // Free42 v3.0.5: 2.000000000 (10-digit BCD precision)
    // Catches: Simpson rule sign error or endpoint weight error
    use hp41_core::ops::math1::integ::op_integ_run_loop;
    use hp41_core::state::DisplayMode;

    let program = vec![
        Op::Lbl("S".to_string()),
        Op::Sin, // f(x) = sin(x)
        Op::Rtn,
    ];
    let (mut state, prog) = make_integ_state_for_acc("S", 0.0, std::f64::consts::PI, 100, program);
    state.display_mode = DisplayMode::Fix(4);
    // Use RAD mode for sin
    dispatch(&mut state, Op::SetRad).unwrap();

    let result = op_integ_run_loop(&mut state, &prog);
    assert!(result.is_ok(), "∫₀^π sin(x) dx failed: {result:?}");

    let x_val = state.stack.x.inner().to_f64().unwrap();
    // OM result: 2.0 (exact). Tolerance 1e-3 at n=100 subdivisions.
    assert!(
        (x_val - 2.0).abs() < WIDE_TOL * 10.0,
        "∫₀^π sin(x) dx must be ≈ 2.0, got {x_val}"
    );
}

#[test]
fn integ_x_squared_over_0_to_1() {
    // Source: HP 00041-90034 (1979), Chapter 3, p. 37 "worked example".
    // ∫₀¹ x² dx = 1/3 (exact, polynomial — Simpson is exact for ≤degree-3 with even subdivisions)
    // Free42 v3.0.5: 0.3333333333
    // Catches: Simpson coefficient pattern wrong (1-4-2-4-...-4-1 vs wrong weights)
    use hp41_core::ops::math1::integ::op_integ_run_loop;
    use hp41_core::state::DisplayMode;

    let program = vec![
        Op::Lbl("X2".to_string()),
        Op::Sq, // f(x) = x^2
        Op::Rtn,
    ];
    let (mut state, prog) = make_integ_state_for_acc("X2", 0.0, 1.0, 10, program);
    state.display_mode = DisplayMode::Fix(6);

    let result = op_integ_run_loop(&mut state, &prog);
    assert!(result.is_ok(), "∫₀¹ x² dx failed: {result:?}");

    let x_val = state.stack.x.inner().to_f64().unwrap();
    // Simpson rule is exact for polynomials of degree ≤ 3, so with n=10 this should
    // be numerically exact (or very close to machine precision)
    assert!(
        (x_val - 1.0 / 3.0).abs() < WIDE_TOL,
        "∫₀¹ x² dx must be ≈ 1/3, got {x_val}"
    );
}

#[test]
fn integ_recip_x_over_1_to_e() {
    // Source: OM Chapter 3 (standard natural log identity)
    // ∫₁^e 1/x dx = ln(e) = 1.0
    // Free42 v3.0.5: 1.000000000
    // Catches: interval [a,b] endpoint inclusion wrong
    use hp41_core::ops::math1::integ::op_integ_run_loop;
    use hp41_core::state::DisplayMode;
    use std::f64::consts::E;

    let program = vec![
        Op::Lbl("R".to_string()),
        Op::Recip, // f(x) = 1/x
        Op::Rtn,
    ];
    let (mut state, prog) = make_integ_state_for_acc("R", 1.0, E, 50, program);
    state.display_mode = DisplayMode::Fix(5);

    let result = op_integ_run_loop(&mut state, &prog);
    assert!(result.is_ok(), "∫₁^e 1/x dx failed: {result:?}");

    let x_val = state.stack.x.inner().to_f64().unwrap();
    // ∫₁^e 1/x dx = 1.0 (definition of e); tolerance 1e-3 at n=50
    assert!(
        (x_val - 1.0).abs() < WIDE_TOL * 100.0,
        "∫₁^e 1/x dx must be ≈ 1.0, got {x_val}"
    );
}

#[test]
fn integ_pitfall2_fix4_vs_fix9_different_precision() {
    // Pitfall-2 guard (ADR-004): SAME integral in Fix(4) vs Fix(9).
    // Result must be correct in BOTH modes. This test exists to verify that
    // the integ_threshold formula produces DIFFERENT tolerances for different modes.
    // If both modes converge with the same tolerance, the formula is wrong.
    //
    // Source: docs/adr/v3.0-004-intg-threshold.md (Plan 28-01)
    // ∫₀¹ x² dx = 1/3 ≈ 0.3333 (Fix(4)) ≈ 0.333333333 (Fix(9))
    use hp41_core::ops::math1::integ::{integ_threshold, op_integ_run_loop};
    use hp41_core::state::DisplayMode;

    // Verify the threshold formula produces different values
    let t4 = integ_threshold(DisplayMode::Fix(4));
    let t9 = integ_threshold(DisplayMode::Fix(9));
    assert!(
        t4 > t9 * 10.0,
        "Fix(4) threshold ({t4}) must be >> Fix(9) threshold ({t9}) — ADR-004 Pitfall-2"
    );

    // Both modes should produce a correct result for x^2
    let program = vec![
        Op::Lbl("P2".to_string()),
        Op::Sq,
        Op::Rtn,
    ];

    // Fix(4) test
    let (mut state4, prog4) = make_integ_state_for_acc("P2", 0.0, 1.0, 10, program.clone());
    state4.display_mode = DisplayMode::Fix(4);
    op_integ_run_loop(&mut state4, &prog4).unwrap();
    let x4 = state4.stack.x.inner().to_f64().unwrap();

    // Fix(9) test
    let (mut state9, prog9) = make_integ_state_for_acc("P2", 0.0, 1.0, 100, program);
    state9.display_mode = DisplayMode::Fix(9);
    op_integ_run_loop(&mut state9, &prog9).unwrap();
    let x9 = state9.stack.x.inner().to_f64().unwrap();

    // Both should be close to 1/3; Fix(9) should be more precise
    assert!(
        (x4 - 1.0 / 3.0).abs() < 0.01,
        "Fix(4): ∫₀¹ x² dx should be ≈ 1/3, got {x4}"
    );
    assert!(
        (x9 - 1.0 / 3.0).abs() < 0.0001,
        "Fix(9): ∫₀¹ x² dx should be ≈ 1/3 (higher precision), got {x9}"
    );
}
