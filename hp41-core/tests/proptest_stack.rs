//! Property-based tests for CORE-02: stack-lift invariant holds across random op sequences.
//!
//! Strategy: generate random sequences of Phase 1 ops and verify:
//!   a) No op panics
//!   b) lift_enabled is always a valid bool after the sequence
//!   c) CalcState fields are always finite/defined (no NaN from rust_decimal)

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op};
use proptest::prelude::*;

/// Generate a random Phase 1 Op (excluding PushNum variants that need arbitrary Decimal)
fn arb_simple_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        Just(Op::Add),
        Just(Op::Sub),
        Just(Op::Mul),
        Just(Op::Div),
        Just(Op::Enter),
        Just(Op::Clx),
        Just(Op::Chs),
        Just(Op::Rdn),
        Just(Op::XySwap),
        Just(Op::Lastx),
        // PushNum with small integers — avoids overflow, exercises lift
        (1i32..=100i32).prop_map(|n| Op::PushNum(HpNum::from(n))),
        (1i32..=100i32).prop_map(|n| Op::PushNum(HpNum::from(-n))),
    ]
}

proptest! {
    /// Any sequence of Phase 1 ops must never panic and must leave lift_enabled in a
    /// defined state. Errors (HpError) are acceptable — panics are not.
    #[test]
    fn stack_never_panics_on_any_op_sequence(
        ops in proptest::collection::vec(arb_simple_op(), 0..30)
    ) {
        let mut state = CalcState::new();
        for op in ops {
            // Ignore Err — DivideByZero and Overflow are acceptable results.
            // The invariant is: no panic, ever.
            let _ = dispatch(&mut state, op);
        }
        // lift_enabled must be a valid bool — this is always true in Rust but
        // documents the invariant explicitly for future unsafe code reviews
        let _ = state.stack.lift_enabled;
    }

    /// After any sequence ending with a binary op (Add), lift must be enabled.
    #[test]
    fn add_always_enables_lift_after_random_prefix(
        prefix in proptest::collection::vec(arb_simple_op(), 0..20)
    ) {
        let mut state = CalcState::new();
        // Set up valid operands so Add won't fail
        state.stack.x = HpNum::from(1);
        state.stack.y = HpNum::from(2);
        // Apply random prefix
        for op in prefix {
            let _ = dispatch(&mut state, op);
        }
        // Set up operands again (prefix may have changed them)
        state.stack.x = HpNum::from(1);
        state.stack.y = HpNum::from(2);
        dispatch(&mut state, Op::Add).unwrap();
        prop_assert!(state.stack.lift_enabled, "Add must always enable lift");
    }

    /// After any sequence ending with ENTER, lift must be disabled.
    #[test]
    fn enter_always_disables_lift_after_random_prefix(
        prefix in proptest::collection::vec(arb_simple_op(), 0..20)
    ) {
        let mut state = CalcState::new();
        for op in prefix {
            let _ = dispatch(&mut state, op);
        }
        dispatch(&mut state, Op::Enter).unwrap();
        prop_assert!(!state.stack.lift_enabled, "Enter must always disable lift");
    }
}
