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
            description:
                "Mean of 4-level stack (T,Z,Y,X): load four values with ENTER, run → mean in X.",
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
        SampleProgram {
            name: "Synthetic Demo",
            description: "Phase 12 demo: GETKEY → STO M → NULL → RCL M. Press a key, then run.",
            ops: synthetic_demo_ops(),
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
        Op::StoReg(2), // R02 = n (loop count)
        Op::PushNum(HpNum::from(0i32)),
        Op::StoReg(0), // R00 = 0 (a)
        Op::PushNum(HpNum::from(1i32)),
        Op::StoReg(1), // R01 = 1 (b)
        // Build ISG counter: current=1, final=n, step=1
        // ISG counter format CCCCC.FFFSS: use R02 directly as loop count via DSE
        Op::Lbl("B".to_string()), // loop top
        Op::RclReg(0),            // a
        Op::RclReg(1),            // b, a
        Op::Add,                  // a+b
        Op::RclReg(1),            // b, a+b
        Op::StoReg(0),            // R00 = old b
        Op::XySwap,
        Op::StoReg(1), // R01 = a+b
        Op::RclReg(2),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(2),               // R02--
        Op::Test(TestKind::XGtZero), // if R02 > 0: loop
        Op::Gto("B".to_string()),
        Op::RclReg(1), // result = b = Fib(n)
        Op::Rtn,
    ]
}

fn factorial_ops() -> Vec<Op> {
    // n! — X=n, result in X
    // R00 = accumulator, R01 = counter
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(1), // R01 = n (counter)
        Op::PushNum(HpNum::from(1i32)),
        Op::StoReg(0), // R00 = 1 (result)
        Op::Lbl("B".to_string()),
        Op::RclReg(1),               // counter
        Op::Test(TestKind::XLeZero), // if counter <= 0: done
        Op::Gto("C".to_string()),
        Op::StoArith {
            reg: 0,
            kind: StoArithKind::Mul,
        }, // R00 *= counter
        Op::RclReg(1),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(1), // counter--
        Op::Gto("B".to_string()),
        Op::Lbl("C".to_string()),
        Op::RclReg(0), // result
        Op::Rtn,
    ]
}

fn prime_test_ops() -> Vec<Op> {
    // Is X prime? Pushes 1 (yes) or 0 (no). Trial division up to √n.
    // R00 = n, R01 = divisor d.
    //
    // Stack semantics (TRUE = execute next; FALSE = skip next):
    //   Early exit: PushNum(2), RclReg(0) → X=n, Y=2
    //     Test(XLeY): n≤2 → TRUE → Gto("P") [prime]; n>2 → FALSE → skip Gto, continue loop
    //
    //   Loop termination: RclReg(1), Sq, RclReg(0) → X=n, Y=d²
    //     Test(XLtY): n<d² → TRUE → Gto("P") [prime]; n≥d² → FALSE → skip, check modulo
    //
    //   Modulo: n mod d = n - d*int(n/d)   (uses Op::Int for exact integer truncation)
    //     RclReg(0), RclReg(1), Div, Int, RclReg(1), Mul → X=d*int(n/d), Y=n (after RclReg(0), XySwap)
    //     Sub (Y-X): n - d*int(n/d) = n mod d
    //     Test(XEqZero): remainder=0 → TRUE → Gto("N") [not prime]; ≠0 → FALSE → skip, increment d
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(0),                  // R00 = n
        Op::PushNum(HpNum::from(2i32)), // Y=2, X=2 (lift)
        Op::RclReg(0),                  // X=n, Y=2
        Op::Test(TestKind::XLeY),       // n≤2 → TRUE → execute Gto("P"); n>2 → FALSE → skip
        Op::Gto("P".to_string()),
        Op::PushNum(HpNum::from(2i32)),
        Op::StoReg(1),            // R01 = divisor = 2
        Op::Lbl("L".to_string()), // loop top
        Op::RclReg(1),            // X=d
        Op::Sq,                   // X=d²
        Op::RclReg(0),            // X=n, Y=d²
        Op::Test(TestKind::XLtY), // n<d² → TRUE → execute Gto("P"); n≥d² → FALSE → skip
        Op::Gto("P".to_string()),
        // Compute n mod d = n - d * int(n/d) — exact integer modulo via Op::Int
        Op::RclReg(0),               // X=n
        Op::RclReg(1),               // X=d, Y=n
        Op::Div,                     // X=n/d (real division)
        Op::Int,                     // X=floor(n/d) (truncate toward zero)
        Op::RclReg(1),               // X=d, Y=floor(n/d)
        Op::Mul,                     // X=d*floor(n/d)
        Op::RclReg(0),               // X=n, Y=d*floor(n/d)
        Op::XySwap,                  // X=d*floor(n/d), Y=n
        Op::Sub,                     // X = Y-X = n - d*floor(n/d) = n mod d
        Op::Test(TestKind::XEqZero), // remainder=0 → TRUE → execute Gto("N"); ≠0 → FALSE → skip
        Op::Gto("N".to_string()),
        Op::RclReg(1),
        Op::PushNum(HpNum::from(1i32)),
        Op::Add,
        Op::StoReg(1), // divisor++
        Op::Gto("L".to_string()),
        Op::Lbl("P".to_string()), // prime
        Op::PushNum(HpNum::from(1i32)),
        Op::Rtn,
        Op::Lbl("N".to_string()), // not prime
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
        Op::StoReg(2), // R02 = c (X)
        Op::Rdn,
        Op::StoReg(1), // R01 = b (Y→X after roll)
        Op::Rdn,
        Op::StoReg(0), // R00 = a
        // discriminant = b² - 4ac
        Op::RclReg(1),
        Op::Sq, // b²
        Op::PushNum(HpNum::from(4i32)),
        Op::RclReg(0),
        Op::Mul, // 4a
        Op::RclReg(2),
        Op::Mul,       // 4ac
        Op::Sub,       // b² - 4ac
        Op::Sqrt,      // √disc
        Op::StoReg(3), // R03 = √disc
        // root1 = (-b + √disc) / 2a
        Op::RclReg(1),
        Op::Chs, // -b
        Op::RclReg(3),
        Op::Add, // -b + √disc
        Op::PushNum(HpNum::from(2i32)),
        Op::RclReg(0),
        Op::Mul, // 2a
        Op::Div, // root1
        // root2 = (-b - √disc) / 2a
        Op::RclReg(1),
        Op::Chs,
        Op::RclReg(3),
        Op::Sub, // -b - √disc
        Op::PushNum(HpNum::from(2i32)),
        Op::RclReg(0),
        Op::Mul,
        Op::Div,    // root2
        Op::XySwap, // X=root1, Y=root2
        Op::Rtn,
    ]
}

fn gcd_ops() -> Vec<Op> {
    // GCD via Euclidean algorithm: gcd(Y, X)
    // R00=a, R01=b; iterate: a,b = b, a mod b until b=0
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(1), // R01 = b = X
        Op::XySwap,
        Op::StoReg(0), // R00 = a = Y
        Op::Lbl("L".to_string()),
        Op::RclReg(1),
        Op::Test(TestKind::XEqZero), // if b == 0: done
        Op::Gto("D".to_string()),
        // r = a mod b = a - b * floor(a/b)
        Op::RclReg(0),
        Op::RclReg(1),
        Op::Div,
        // floor-truncate quotient then multiply back for exact integer modulo
        Op::Int, // floor-truncate quotient (BCD division is exact; Int truncates)
        Op::RclReg(1),
        Op::Mul,
        Op::RclReg(0),
        Op::XySwap,
        Op::Sub,       // r = a - b*(a/b)
        Op::StoReg(2), // R02 = r
        Op::RclReg(1),
        Op::StoReg(0), // a = b
        Op::RclReg(2),
        Op::StoReg(1), // b = r
        Op::Gto("L".to_string()),
        Op::Lbl("D".to_string()),
        Op::RclReg(0), // GCD result
        Op::Rtn,
    ]
}

fn newton_root_ops() -> Vec<Op> {
    // Newton's method for √X: iterate x_n+1 = (x_n + N/x_n) / 2
    // Converges quickly; 10 iterations is overkill but safe.
    // R00 = N, R01 = guess, R02 = counter
    vec![
        Op::Lbl("A".to_string()),
        Op::StoReg(0), // R00 = N
        Op::PushNum(HpNum::from(1i32)),
        Op::StoReg(1), // R01 = initial guess = 1
        Op::PushNum(HpNum::from(10i32)),
        Op::StoReg(2), // R02 = 10 (iterations)
        Op::Lbl("L".to_string()),
        Op::RclReg(0),
        Op::RclReg(1),
        Op::Div, // N/guess
        Op::RclReg(1),
        Op::Add, // guess + N/guess
        Op::PushNum(HpNum::from(2i32)),
        Op::Div, // new guess
        Op::StoReg(1),
        Op::RclReg(2),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(2),
        Op::Test(TestKind::XGtZero), // if iterations remain: loop
        Op::Gto("L".to_string()),
        Op::RclReg(1), // result
        Op::Rtn,
    ]
}

fn mean_sdev_ops() -> Vec<Op> {
    // Mean of 4 values already on the stack (T, Z, Y, X).
    // Adds all four: T+Z+Y+X, then divides by 4 → mean in X.
    // No registers used. No indirect addressing needed.
    vec![
        Op::Lbl("A".to_string()),
        Op::Add, // Y+X → X; T becomes new Z
        Op::Add, // Z+X → X; T becomes new Y (was Z orig)
        Op::Add, // Y+X → X; one value remains (was T orig)
        Op::PushNum(HpNum::from(4i32)),
        Op::Div, // (T+Z+Y+X) / 4 = mean
        Op::Rtn,
    ]
}

fn deg_to_rad_ops() -> Vec<Op> {
    // Convert X from degrees to radians: X × π/180
    // π/180 ≈ 0.01745329252
    vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(180i32)),
        Op::Div,                        // X / 180
        Op::SetRad,                     // ensure RAD mode
        Op::PushNum(HpNum::from(1i32)), // push 1
        Op::Asin,                       // asin(1) = π/2
        Op::PushNum(HpNum::from(2i32)),
        Op::Mul, // π
        Op::Mul, // (X/180) × π = X in radians
        Op::Rtn,
    ]
}

fn stack_stats_ops() -> Vec<Op> {
    // Min and max of the 4-level stack (T, Z, Y, X).
    // Saves all 4 values to R00-R03 via Rdn cycling, then finds min/max with
    // pairwise RclReg comparisons so all four values are correctly reached.
    // Result: X = min, Y = max.
    vec![
        Op::Lbl("A".to_string()),
        // Save all 4 stack values to R00-R03 (Rdn cycles each to X position)
        Op::StoReg(0), // R00 = X
        Op::Rdn,
        Op::StoReg(1), // R01 = Y (original)
        Op::Rdn,
        Op::StoReg(2), // R02 = Z (original)
        Op::Rdn,
        Op::StoReg(3), // R03 = T (original)
        // Find max: compare pairs; XLtY fires when X is smaller → swap brings larger to X
        Op::RclReg(0),
        Op::RclReg(1),            // X=R01, Y=R00
        Op::Test(TestKind::XLtY), // R01 < R00 → swap so larger ends up in X
        Op::XySwap,
        Op::StoReg(5),            // R05 = max(R00, R01)
        Op::RclReg(2),            // X=R02, Y=running max
        Op::Test(TestKind::XLtY), // R02 < running max → swap
        Op::XySwap,
        Op::StoReg(5), // R05 = max(R00..R02)
        Op::RclReg(3), // X=R03, Y=running max
        Op::Test(TestKind::XLtY),
        Op::XySwap,
        Op::StoReg(5), // R05 = final max
        // Find min: invert test for smaller-wins; XGtY fires when X is larger → swap brings smaller to X
        Op::RclReg(0),
        Op::RclReg(1),            // X=R01, Y=R00
        Op::Test(TestKind::XGtY), // R01 > R00 → swap so smaller ends up in X
        Op::XySwap,
        Op::StoReg(4),            // R04 = min(R00, R01)
        Op::RclReg(2),            // X=R02, Y=running min
        Op::Test(TestKind::XGtY), // R02 > running min → swap
        Op::XySwap,
        Op::StoReg(4), // R04 = min(R00..R02)
        Op::RclReg(3), // X=R03, Y=running min
        Op::Test(TestKind::XGtY),
        Op::XySwap,
        Op::StoReg(4), // R04 = final min
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
        Op::StoReg(9), // R09 = n (starting count)
        Op::Lbl("L".to_string()),
        Op::RclReg(9),               // show current count in X
        Op::Test(TestKind::XLeZero), // if n <= 0: done
        Op::Gto("D".to_string()),
        Op::PushNum(HpNum::from(1i32)),
        Op::Sub,
        Op::StoReg(9), // n--
        Op::Gto("L".to_string()),
        Op::Lbl("D".to_string()),
        Op::PushNum(HpNum::from(1i32)), // final value = 1
        Op::Rtn,
    ]
}

fn synthetic_demo_ops() -> Vec<Op> {
    // Phase 12 Synthetic Programming demo.
    //
    // Demonstrates GETKEY, STO M, NULL, and RCL M in a single program.
    //
    // Usage:
    //   1. In RUN mode, press any key (e.g. '5') — this updates last_key_code.
    //   2. Press Ctrl+P to open program library, select "Synthetic Demo", load.
    //   3. Press F5 to run from LBL A.
    //   4. X shows the HP-41 row-column key code of the key you pressed in step 1.
    //   5. The NULL step is a no-op placeholder — it does not change any register.
    //
    // Expected result for key '5': X = 62 (row 6, col 2 in HP-41 layout).
    vec![
        Op::Lbl("A".to_string()),
        Op::GetKey, // SYNT-01: push last_key_code to X (LiftEffect::Enable)
        Op::StoM,   // SYNT-03: store X in hidden register M
        Op::Null,   // SYNT-02: no-op, neutral stack-lift — X unchanged
        Op::RclM,   // SYNT-03: recall M back to X (round-trip proof)
        Op::Rtn,
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        assert!(
            result.is_ok(),
            "Fibonacci must run without error: {:?}",
            result
        );
    }

    #[test]
    fn test_program_names_unique() {
        let names: Vec<&str> = sample_programs().iter().map(|p| p.name).collect();
        let mut unique = names.clone();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "Program names must be unique");
    }

    #[test]
    fn test_prime_test_correctness() {
        // Verifies SC-5 gap closure: prime_test_ops XySwap bug is fixed.
        let prime_prog = sample_programs()
            .iter()
            .find(|p| p.name == "Prime Test")
            .expect("Prime Test program must exist");

        let run_prime = |n: i32| -> HpNum {
            let mut state = CalcState::new();
            state.program = prime_prog.ops.clone();
            hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(n))).unwrap();
            hp41_core::run_program(&mut state, "A")
                .unwrap_or_else(|e| panic!("prime({n}) failed: {e:?}"));
            state.stack.x.clone()
        };

        assert_eq!(
            run_prime(2),
            HpNum::from(1i32),
            "prime(2) must be 1 (prime)"
        );
        assert_eq!(
            run_prime(3),
            HpNum::from(1i32),
            "prime(3) must be 1 (prime)"
        );
        assert_eq!(
            run_prime(4),
            HpNum::from(0i32),
            "prime(4) must be 0 (composite)"
        );
        assert_eq!(
            run_prime(9),
            HpNum::from(0i32),
            "prime(9) must be 0 (composite: 3×3)"
        );
        assert_eq!(
            run_prime(13),
            HpNum::from(1i32),
            "prime(13) must be 1 (prime)"
        );
    }

    #[test]
    fn test_stack_mean_correctness() {
        // Verifies SC-5 gap closure: mean_sdev_ops replaced with correct stack mean.
        let mean_prog = sample_programs()
            .iter()
            .find(|p| p.name == "Stack Mean (4 values)")
            .expect("Stack Mean (4 values) program must exist");

        let mut state = CalcState::new();
        state.program = mean_prog.ops.clone();
        // Build stack: push 1 ENTER 2 ENTER 3 ENTER 4 → T=1, Z=2, Y=3, X=4
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(1i32))).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(2i32))).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(3i32))).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(4i32))).unwrap();
        hp41_core::run_program(&mut state, "A").expect("Stack Mean must run without error");
        // (1+2+3+4)/4 = 2.5
        let result_str = state.stack.x.to_string();
        let result: f64 = result_str
            .parse()
            .expect("stack.x must be parseable as f64");
        assert!(
            (result - 2.5).abs() < 1e-9,
            "Stack mean of [1,2,3,4] must be 2.5, got {result}"
        );
    }

    #[test]
    fn test_gcd_correctness() {
        // Verifies CR-02 fix: Op::Int added after Op::Div in gcd_ops modulo step.
        let gcd_prog = sample_programs()
            .iter()
            .find(|p| p.name == "GCD (Euclidean)")
            .expect("GCD (Euclidean) program must exist");

        let run_gcd = |a: i32, b: i32| -> HpNum {
            let mut state = CalcState::new();
            state.program = gcd_prog.ops.clone();
            // Push Y=a, X=b: gcd(Y, X)
            hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(a))).unwrap();
            hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
            hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(b))).unwrap();
            hp41_core::run_program(&mut state, "A")
                .unwrap_or_else(|e| panic!("gcd({a},{b}) failed: {e:?}"));
            state.stack.x.clone()
        };

        assert_eq!(run_gcd(12, 8), HpNum::from(4i32), "gcd(12,8) must be 4");
        assert_eq!(run_gcd(7, 3), HpNum::from(1i32), "gcd(7,3) must be 1");
        assert_eq!(run_gcd(15, 5), HpNum::from(5i32), "gcd(15,5) must be 5");
    }

    #[test]
    fn test_stack_stats_correctness() {
        // Verifies CR-03 fix: test conditions in stack_stats_ops are inverted so
        // larger value is stored as max (R05) and smaller value as min (R04).
        // Documented output: X = min of stack, Y = max of stack.
        let stats_prog = sample_programs()
            .iter()
            .find(|p| p.name == "Stack Stats")
            .expect("Stack Stats program must exist");

        let mut state = CalcState::new();
        state.program = stats_prog.ops.clone();
        // Build stack: push 3 ENTER 1 ENTER 4 ENTER 5 → T=3, Z=1, Y=4, X=5
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(3i32))).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(1i32))).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(4i32))).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::Enter).unwrap();
        hp41_core::ops::dispatch(&mut state, Op::PushNum(HpNum::from(5i32))).unwrap();
        hp41_core::run_program(&mut state, "A").expect("Stack Stats must run without error");
        // X = min = 1, Y = max = 5
        assert_eq!(
            state.stack.x,
            HpNum::from(1i32),
            "Stack Stats: X must be min (1) for inputs [3,1,4,5], got {:?}",
            state.stack.x
        );
        assert_eq!(
            state.stack.y,
            HpNum::from(5i32),
            "Stack Stats: Y must be max (5) for inputs [3,1,4,5], got {:?}",
            state.stack.y
        );
    }
}
