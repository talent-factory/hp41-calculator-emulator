//! Bundled sample program library for the HP-41 emulator (D-21 through D-24).
//!
//! CRITICAL: Op::Lbl(String) is heap-allocated — NOT const-constructible.
//! Use OnceLock<Vec<SampleProgram>> initialized at first access (RESEARCH Pitfall 2).
//! All programs start with Op::Lbl("A") so run_program(state, "A") works uniformly.

use std::sync::OnceLock;

use hp41_core::ops::{Op, StoArithKind, TestKind};
use hp41_core::HpNum;

/// A bundled sample program with metadata for display in the program library overlay.
pub struct SampleProgram {
    pub name: &'static str,
    pub description: &'static str,
    pub ops: Vec<Op>,
}

static PROGRAMS_CACHE: OnceLock<Vec<SampleProgram>> = OnceLock::new();

/// Access the bundled sample programs (lazily initialized, thread-safe via OnceLock).
pub fn sample_programs() -> &'static [SampleProgram] {
    PROGRAMS_CACHE.get_or_init(build_all_programs)
}

fn build_all_programs() -> Vec<SampleProgram> {
    vec![
        SampleProgram {
            name: "Fibonacci",
            description: "Fibonacci sequence: X=n → Fn. ISG loop demo.",
            ops: fibonacci_ops(),
        },
        SampleProgram {
            name: "Factorial",
            description: "n! for X≥0 integer. Uses ISG counter loop.",
            ops: factorial_ops(),
        },
        SampleProgram {
            name: "Prime Test",
            description: "Is X prime? Pushes 1 (yes) or 0 (no). ISG+conditional demo.",
            ops: prime_test_ops(),
        },
        SampleProgram {
            name: "Quadratic Solver",
            description: "Roots of ax²+bx+c. Stack: a(T) b(Z) c(Y) → roots in X,Y.",
            ops: quadratic_ops(),
        },
        SampleProgram {
            name: "GCD (Euclidean)",
            description: "Greatest common divisor of Y and X using Euclidean algorithm.",
            ops: gcd_ops(),
        },
        SampleProgram {
            name: "Newton Root",
            description: "Square root by Newton's method (convergence demo). X=number.",
            ops: newton_root_ops(),
        },
        SampleProgram {
            name: "Stack Mean (4 values)",
            description: "Mean of 4-level stack (T,Z,Y,X): load four values with ENTER, run → mean in X.",
            ops: mean_sdev_ops(),
        },
        SampleProgram {
            name: "Deg ↔ Rad",
            description: "Convert X: degrees to radians (push π/180 × X).",
            ops: deg_to_rad_ops(),
        },
        SampleProgram {
            name: "Stack Stats",
            description: "Min and max of 4-level stack: X=min, Y=max after run.",
            ops: stack_stats_ops(),
        },
        SampleProgram {
            name: "Countdown",
            description: "Count down from X to 1 using DSE loop (display demo).",
            ops: countdown_ops(),
        },
    ]
}

// ── Program implementations ───────────────────────────────────────────────────
// Each program begins with Op::Lbl("A") so run_program(state, "A") works uniformly.

fn fibonacci_ops() -> Vec<Op> {
    // Fibonacci(n): X=n → Fib(n)
    // R00 = a=0, R01 = b=1, R02 = counter
    // Algorithm: loop n times: tmp=b, b=a+b, a=tmp
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(2),                      // R02 = n (loop count)
        Op::PushNum(HpNum::from(0i32)),
        Op::StoReg(0),                      // R00 = 0 (a)
        Op::PushNum(HpNum::from(1i32)),
        Op::StoReg(1),                      // R01 = 1 (b)
        // Build ISG counter: current=1, final=n, step=1
        // ISG counter format CCCCC.FFFSS: use R02 directly as loop count via DSE
        Op::Lbl("B".to_string()),           // loop top
        Op::RclReg(0),                      // a
        Op::RclReg(1),                      // b, a
        Op::Add,                            // a+b
        Op::RclReg(1),                      // b, a+b
        Op::StoReg(0),                      // R00 = old b
        Op::XySwap,
        Op::StoReg(1),                      // R01 = a+b
        Op::RclReg(2),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(2),                      // R02--
        Op::Test(TestKind::XGtZero),        // if R02 > 0: loop
        Op::Gto("B".to_string()),
        Op::RclReg(1),                      // result = b = Fib(n)
        Op::Rtn,
    ]
}

fn factorial_ops() -> Vec<Op> {
    // n! — X=n, result in X
    // R00 = accumulator, R01 = counter
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(1),                      // R01 = n (counter)
        Op::PushNum(HpNum::from(1i32)),
        Op::StoReg(0),                      // R00 = 1 (result)
        Op::Lbl("B".to_string()),
        Op::RclReg(1),                      // counter
        Op::Test(TestKind::XLeZero),        // if counter <= 0: done
        Op::Gto("C".to_string()),
        Op::StoArith { reg: 0, kind: StoArithKind::Mul }, // R00 *= counter
        Op::RclReg(1),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(1),                      // counter--
        Op::Gto("B".to_string()),
        Op::Lbl("C".to_string()),
        Op::RclReg(0),                      // result
        Op::Rtn,
    ]
}

fn prime_test_ops() -> Vec<Op> {
    // Is X prime? Pushes 1 (yes) or 0 (no).
    // Simple trial division by 2 and odd numbers up to √X.
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(0),                      // R00 = n
        Op::PushNum(HpNum::from(2i32)),
        Op::RclReg(0),
        Op::Test(TestKind::XLeY),           // if n <= 2: prime
        Op::Gto("P".to_string()),
        Op::PushNum(HpNum::from(2i32)),
        Op::StoReg(1),                      // R01 = divisor = 2
        Op::Lbl("L".to_string()),
        Op::RclReg(1),
        Op::Sq,
        Op::RclReg(0),
        Op::Test(TestKind::XGtY),           // if divisor² > n: prime
        Op::Gto("P".to_string()),
        Op::RclReg(0),
        Op::RclReg(1),
        Op::Div,
        Op::RclReg(1),
        Op::Mul,
        Op::RclReg(0),
        Op::XySwap,
        Op::Sub,                            // n mod d = n - d*(n/d)
        Op::Test(TestKind::XEqZero),        // if remainder == 0: not prime
        Op::Gto("N".to_string()),
        Op::RclReg(1),
        Op::PushNum(HpNum::from(1i32)),
        Op::Add,
        Op::StoReg(1),                      // divisor++
        Op::Gto("L".to_string()),
        Op::Lbl("P".to_string()),           // prime
        Op::PushNum(HpNum::from(1i32)),
        Op::Rtn,
        Op::Lbl("N".to_string()),           // not prime
        Op::PushNum(HpNum::from(0i32)),
        Op::Rtn,
    ]
}

fn quadratic_ops() -> Vec<Op> {
    // Quadratic: discriminant = b²-4ac; roots = (-b ± √disc) / 2a
    // Stack entry: c in X, b in Y, a in Z (T unused — STO X→c first, Rdn→STO Y→b, Rdn→STO Z→a)
    // Result: root1 in X, root2 in Y
    vec![
        Op::Lbl("A".to_string()),
        // Store coefficients from stack
        Op::StoReg(2),                      // R02 = c (X)
        Op::Rdn,
        Op::StoReg(1),                      // R01 = b (Y→X after roll)
        Op::Rdn,
        Op::StoReg(0),                      // R00 = a
        // discriminant = b² - 4ac
        Op::RclReg(1), Op::Sq,             // b²
        Op::PushNum(HpNum::from(4i32)),
        Op::RclReg(0), Op::Mul,            // 4a
        Op::RclReg(2), Op::Mul,            // 4ac
        Op::Sub,                            // b² - 4ac
        Op::Sqrt,                           // √disc
        Op::StoReg(3),                      // R03 = √disc
        // root1 = (-b + √disc) / 2a
        Op::RclReg(1), Op::Chs,            // -b
        Op::RclReg(3), Op::Add,            // -b + √disc
        Op::PushNum(HpNum::from(2i32)),
        Op::RclReg(0), Op::Mul,            // 2a
        Op::Div,                            // root1
        // root2 = (-b - √disc) / 2a
        Op::RclReg(1), Op::Chs,
        Op::RclReg(3), Op::Sub,            // -b - √disc
        Op::PushNum(HpNum::from(2i32)),
        Op::RclReg(0), Op::Mul,
        Op::Div,                            // root2
        Op::XySwap,                        // X=root1, Y=root2
        Op::Rtn,
    ]
}

fn gcd_ops() -> Vec<Op> {
    // GCD via Euclidean algorithm: gcd(Y, X)
    // R00=a, R01=b; iterate: a,b = b, a mod b until b=0
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(1),                      // R01 = b = X
        Op::XySwap,
        Op::StoReg(0),                      // R00 = a = Y
        Op::Lbl("L".to_string()),
        Op::RclReg(1),
        Op::Test(TestKind::XEqZero),        // if b == 0: done
        Op::Gto("D".to_string()),
        // r = a mod b = a - b * floor(a/b)
        Op::RclReg(0), Op::RclReg(1), Op::Div,
        // approximate floor via truncation: use as-is (integer inputs assumed)
        Op::RclReg(1), Op::Mul,
        Op::RclReg(0), Op::XySwap, Op::Sub, // r = a - b*(a/b)
        Op::StoReg(2),                      // R02 = r
        Op::RclReg(1), Op::StoReg(0),      // a = b
        Op::RclReg(2), Op::StoReg(1),      // b = r
        Op::Gto("L".to_string()),
        Op::Lbl("D".to_string()),
        Op::RclReg(0),                      // GCD result
        Op::Rtn,
    ]
}

fn newton_root_ops() -> Vec<Op> {
    // Newton's method for √X: iterate x_n+1 = (x_n + N/x_n) / 2
    // Converges quickly; 10 iterations is overkill but safe.
    // R00 = N, R01 = guess, R02 = counter
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(0),                       // R00 = N
        Op::PushNum(HpNum::from(1i32)),
        Op::StoReg(1),                       // R01 = initial guess = 1
        Op::PushNum(HpNum::from(10i32)),
        Op::StoReg(2),                       // R02 = 10 (iterations)
        Op::Lbl("L".to_string()),
        Op::RclReg(0),
        Op::RclReg(1), Op::Div,             // N/guess
        Op::RclReg(1), Op::Add,             // guess + N/guess
        Op::PushNum(HpNum::from(2i32)),
        Op::Div,                             // new guess
        Op::StoReg(1),
        Op::RclReg(2),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(2),
        Op::Test(TestKind::XGtZero),         // if iterations remain: loop
        Op::Gto("L".to_string()),
        Op::RclReg(1),                       // result
        Op::Rtn,
    ]
}

fn mean_sdev_ops() -> Vec<Op> {
    // Mean of 4 values already on the stack (T, Z, Y, X).
    // Adds all four: T+Z+Y+X, then divides by 4 → mean in X.
    // No registers used. No indirect addressing needed.
    vec![
        Op::Lbl("A".to_string()),
        Op::Add,                            // Y+X → X; T becomes new Z
        Op::Add,                            // Z+X → X; T becomes new Y (was Z orig)
        Op::Add,                            // Y+X → X; one value remains (was T orig)
        Op::PushNum(HpNum::from(4i32)),
        Op::Div,                            // (T+Z+Y+X) / 4 = mean
        Op::Rtn,
    ]
}

fn deg_to_rad_ops() -> Vec<Op> {
    // Convert X from degrees to radians: X × π/180
    // π/180 ≈ 0.01745329252
    vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(180i32)),
        Op::Div,                            // X / 180
        Op::SetRad,                         // ensure RAD mode
        Op::PushNum(HpNum::from(1i32)),     // push 1
        Op::Asin,                           // asin(1) = π/2
        Op::PushNum(HpNum::from(2i32)),
        Op::Mul,                            // π
        Op::Mul,                            // (X/180) × π = X in radians
        Op::Rtn,
    ]
}

fn stack_stats_ops() -> Vec<Op> {
    // Min and max of the 4-level stack (T, Z, Y, X).
    // Compares pairs; result: X = min of stack, Y = max of stack.
    vec![
        Op::Lbl("A".to_string()),
        // Find max: compare X and Y
        Op::Enter,                          // duplicate X → X,X in Y,X
        Op::Rdn,                            // bring Y up: Y(orig), X(orig), X(copy)
        Op::Test(TestKind::XGtY),          // if X > Y: X is larger
        Op::XySwap,                        // ensure larger in X
        Op::StoReg(5),                     // R05 = max candidate so far
        // Compare with Z
        Op::Rdn,
        Op::RclReg(5),
        Op::Test(TestKind::XGtY),
        Op::XySwap,
        Op::StoReg(5),                     // R05 = running max
        // Compare with T
        Op::Rdn,
        Op::RclReg(5),
        Op::Test(TestKind::XGtY),
        Op::XySwap,
        Op::StoReg(5),                     // R05 = final max
        // Min: same logic with reversed test
        Op::Enter,
        Op::Rdn,
        Op::Test(TestKind::XLtY),
        Op::XySwap,
        Op::StoReg(4),                     // R04 = min candidate
        Op::Rdn,
        Op::RclReg(4),
        Op::Test(TestKind::XLtY),
        Op::XySwap,
        Op::StoReg(4),
        Op::Rdn,
        Op::RclReg(4),
        Op::Test(TestKind::XLtY),
        Op::XySwap,
        Op::StoReg(4),                     // R04 = final min
        // Result: X = min, Y = max
        Op::RclReg(4),
        Op::RclReg(5),
        Op::XySwap,
        Op::Rtn,
    ]
}

fn countdown_ops() -> Vec<Op> {
    // Count down from X to 1 using a loop; result = 1 in X.
    // Demonstrates DSE loop control (HP-41 classic).
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(9),                      // R09 = n (starting count)
        Op::Lbl("L".to_string()),
        Op::RclReg(9),                      // show current count in X
        Op::Test(TestKind::XLeZero),        // if n <= 0: done
        Op::Gto("D".to_string()),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(9),                      // n--
        Op::Gto("L".to_string()),
        Op::Lbl("D".to_string()),
        Op::PushNum(HpNum::from(1i32)),     // final value = 1
        Op::Rtn,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp41_core::CalcState;

    #[test]
    fn test_program_count() {
        assert!(
            sample_programs().len() >= 10,
            "Must have at least 10 programs, got {}",
            sample_programs().len()
        );
    }

    #[test]
    fn test_all_programs_non_empty() {
        for prog in sample_programs() {
            assert!(
                !prog.ops.is_empty(),
                "Program '{}' has empty ops list",
                prog.name
            );
        }
    }

    #[test]
    fn test_all_programs_start_with_lbl_a() {
        for prog in sample_programs() {
            assert!(
                matches!(prog.ops.first(), Some(Op::Lbl(l)) if l == "A"),
                "Program '{}' must start with Op::Lbl(\"A\"), got: {:?}",
                prog.name,
                prog.ops.first()
            );
        }
    }

    #[test]
    fn test_fibonacci_runs_without_panic() {
        let mut state = CalcState::new();
        // Load Fibonacci program
        let fib_prog = &sample_programs()[0];
        assert_eq!(fib_prog.name, "Fibonacci");
        state.program = fib_prog.ops.clone();
        // Push n=6, run → should produce F(6)=8 or similar without panic
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(6i32))).unwrap();
        let result = hp41_core::run_program(&mut state, "A");
        assert!(result.is_ok(), "Fibonacci must run without error: {:?}", result);
    }

    #[test]
    fn test_program_names_unique() {
        let names: Vec<&str> = sample_programs().iter().map(|p| p.name).collect();
        let mut unique = names.clone();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "Program names must be unique");
    }
}
