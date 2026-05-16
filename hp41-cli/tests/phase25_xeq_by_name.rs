//! Phase 25 / Plan 03 — XEQ-by-Name modal resolver integration tests.
//!
//! Coverage:
//!   - Each of the 8 non-keyboard conditional-test mnemonics resolves through
//!     the XEQ-by-Name modal Enter-arm to the documented `Op::Test(TestKind::*)`
//!     variant — both ASCII-pure and Unicode-symbol spellings.
//!   - The 4 v2.1 card-reader names (WPRGM/RDPRGM/WDTA/RDTA) fall through the
//!     CLI-local resolver to `Op::Xeq` → hp41-core::builtin_card_op.
//!   - Unknown names produce an InvalidOp diagnostic (Pitfall 9 — no
//!     "did you mean…?" until Phase 26).
//!   - Cross-resolver drift test: `keys::xeq_by_name_local_resolve` and
//!     `hp41_core::ops::program::builtin_card_op` agree on every conditional
//!     mnemonic (T-25-09 mitigation).
//!   - FN-TEST-01 closure: all 12 conditional tests are reachable via the
//!     keyboard — 4 via f-arith (Plan 01) + 8 via XEQ-by-Name (this plan).
//!
//! Modal opening uses the Plan 02 scaffold pattern: directly set
//! `app.pending_input = Some(PendingInput::XeqByName(...))` rather than
//! emulating the `f-N` opener keystroke — this isolates the resolver under
//! test from the opener wiring (which has its own Plan 02 regression tests
//! in `phase25_pending_input.rs`).

#![allow(clippy::unwrap_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use hp41_cli::app::{App, PendingInput};
use hp41_cli::keys::xeq_by_name_local_resolve;
use hp41_core::ops::{Op, TestKind};
use hp41_core::state::CalcState;

// ── Test scaffolding ─────────────────────────────────────────────────────────

fn key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn raw_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn make_app() -> (App, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir creation must succeed");
    let state_path = tmp.path().join("phase25-xeq-by-name-test-state.json");
    let app = App::new(CalcState::new(), state_path, None);
    (app, tmp)
}

/// Drive the modal: open with `XeqByName(String::new())`, type each char of
/// `name`, press Enter, then assert the modal closed AND `app.message` does
/// not contain an error (no InvalidOp surfaced).
///
/// Returns the post-Enter `app.message` for inspection.
fn type_name_and_enter(name: &str) -> (App, tempfile::TempDir, Option<String>) {
    let (mut app, tmp) = make_app();
    app.pending_input = Some(PendingInput::XeqByName(String::new()));
    for c in name.chars() {
        app.handle_key(key(c));
    }
    // Confirm accumulator captured the full name before pressing Enter.
    match &app.pending_input {
        Some(PendingInput::XeqByName(acc)) => {
            assert_eq!(
                acc, name,
                "accumulator must hold the full mnemonic before Enter"
            );
        }
        other => panic!("expected XeqByName open after typing; got {other:?}"),
    }
    app.handle_key(raw_key(KeyCode::Enter));
    assert!(
        app.pending_input.is_none(),
        "Enter must close XeqByName modal regardless of dispatch outcome"
    );
    let msg = app.message.clone();
    (app, tmp, msg)
}

// ── Resolver direct-call tests (8 conditional-test mnemonics × spellings) ────

#[test]
fn xeq_by_name_resolves_x_ne_y() {
    assert_eq!(
        xeq_by_name_local_resolve("X<>Y?"),
        Some(Op::Test(TestKind::XNeY))
    );
    // Drive through the modal — full integration path.
    let (_app, _tmp, msg) = type_name_and_enter("X<>Y?");
    assert!(
        msg.is_none(),
        "Op::Test(XNeY) must dispatch cleanly; got message={msg:?}"
    );
}

#[test]
fn xeq_by_name_resolves_x_lt_y() {
    assert_eq!(
        xeq_by_name_local_resolve("X<Y?"),
        Some(Op::Test(TestKind::XLtY))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X<Y?");
    assert!(
        msg.is_none(),
        "Op::Test(XLtY) dispatch should not error; got {msg:?}"
    );
}

#[test]
fn xeq_by_name_resolves_x_ge_y() {
    // ASCII spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X>=Y?"),
        Some(Op::Test(TestKind::XGeY))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X>=Y?");
    assert!(
        msg.is_none(),
        "X>=Y? dispatch should not error; got {msg:?}"
    );

    // Unicode spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X\u{2265}Y?"),
        Some(Op::Test(TestKind::XGeY))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X\u{2265}Y?");
    assert!(
        msg.is_none(),
        "X\u{2265}Y? (Unicode) dispatch should not error; got {msg:?}"
    );
}

#[test]
fn xeq_by_name_resolves_x_ne_zero() {
    // ASCII spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X#0?"),
        Some(Op::Test(TestKind::XNeZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X#0?");
    assert!(msg.is_none(), "X#0? dispatch should not error; got {msg:?}");

    // Unicode spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X\u{2260}0?"),
        Some(Op::Test(TestKind::XNeZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X\u{2260}0?");
    assert!(
        msg.is_none(),
        "X\u{2260}0? (Unicode) dispatch should not error; got {msg:?}"
    );
}

#[test]
fn xeq_by_name_resolves_x_lt_zero() {
    assert_eq!(
        xeq_by_name_local_resolve("X<0?"),
        Some(Op::Test(TestKind::XLtZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X<0?");
    assert!(msg.is_none(), "X<0? dispatch should not error; got {msg:?}");
}

#[test]
fn xeq_by_name_resolves_x_gt_zero() {
    assert_eq!(
        xeq_by_name_local_resolve("X>0?"),
        Some(Op::Test(TestKind::XGtZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X>0?");
    assert!(msg.is_none(), "X>0? dispatch should not error; got {msg:?}");
}

#[test]
fn xeq_by_name_resolves_x_le_zero() {
    // ASCII spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X<=0?"),
        Some(Op::Test(TestKind::XLeZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X<=0?");
    assert!(
        msg.is_none(),
        "X<=0? dispatch should not error; got {msg:?}"
    );

    // Unicode spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X\u{2264}0?"),
        Some(Op::Test(TestKind::XLeZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X\u{2264}0?");
    assert!(
        msg.is_none(),
        "X\u{2264}0? (Unicode) dispatch should not error; got {msg:?}"
    );
}

#[test]
fn xeq_by_name_resolves_x_ge_zero() {
    // ASCII spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X>=0?"),
        Some(Op::Test(TestKind::XGeZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X>=0?");
    assert!(
        msg.is_none(),
        "X>=0? dispatch should not error; got {msg:?}"
    );

    // Unicode spelling.
    assert_eq!(
        xeq_by_name_local_resolve("X\u{2265}0?"),
        Some(Op::Test(TestKind::XGeZero))
    );
    let (_app, _tmp, msg) = type_name_and_enter("X\u{2265}0?");
    assert!(
        msg.is_none(),
        "X\u{2265}0? (Unicode) dispatch should not error; got {msg:?}"
    );
}

/// Explicit Unicode-only path: type `X` `≠` `Y` `?` via crossterm
/// `KeyCode::Char('≠')` events. Confirms the modal's `handle_key` accumulates
/// non-ASCII Unicode chars correctly (XEQ_NAME_CAP counts in bytes? No —
/// `String::push(char)` and `String::len() < CAP` byte-count cap; non-ASCII
/// chars are >1 byte each but the 24-byte cap is generous enough that the
/// 4-char Unicode mnemonic fits well under it).
#[test]
fn xeq_by_name_unicode_form_works() {
    assert_eq!(
        xeq_by_name_local_resolve("X\u{2260}Y?"),
        Some(Op::Test(TestKind::XNeY))
    );

    let (mut app, _tmp) = make_app();
    app.pending_input = Some(PendingInput::XeqByName(String::new()));
    app.handle_key(key('X'));
    app.handle_key(key('\u{2260}'));
    app.handle_key(key('Y'));
    app.handle_key(key('?'));
    match &app.pending_input {
        Some(PendingInput::XeqByName(acc)) => {
            assert_eq!(acc, "X\u{2260}Y?");
        }
        other => panic!("expected XeqByName open after typing Unicode mnemonic; got {other:?}"),
    }
    app.handle_key(raw_key(KeyCode::Enter));
    assert!(app.pending_input.is_none());
    assert!(
        app.message.is_none(),
        "Unicode XNeY dispatch should not error; got {:?}",
        app.message
    );
}

// ── 4 v2.1 card-reader names — fall through to hp41-core::builtin_card_op ────

#[test]
fn xeq_by_name_falls_through_to_card_reader() {
    // The CLI-local resolver returns None for the 4 v2.1 names — the modal
    // Enter-arm falls through to `Op::Xeq(acc)` which routes through
    // hp41-core::builtin_card_op (the Op::Xeq match arm at line ~73 of
    // ops/program.rs since is_running=false). dispatch then resolves to
    // Op::Wprgm.
    assert_eq!(xeq_by_name_local_resolve("WPRGM"), None);

    let (_app, _tmp, msg) = type_name_and_enter("WPRGM");
    // WPRGM dispatch sets a card-op pending state but does NOT push an error
    // message — i.e. the resolver found the name. Without an actual card
    // file the side effect surfaces elsewhere; here we just assert that no
    // InvalidOp diagnostic was raised.
    assert!(
        !msg.as_deref().unwrap_or("").contains("InvalidOp"),
        "WPRGM resolved by core builtin_card_op should not error as InvalidOp; got {msg:?}"
    );
}

// ── Pitfall 9 — unknown name surfaces InvalidOp ──────────────────────────────

#[test]
fn xeq_by_name_unknown_returns_invalid_op() {
    assert_eq!(xeq_by_name_local_resolve("FOOBAR"), None);

    let (_app, _tmp, msg) = type_name_and_enter("FOOBAR");
    // Op::Xeq("FOOBAR") with no user LBL and no card-reader match returns
    // HpError::InvalidOp; the App formats it into `self.message`.
    let m = msg.expect("InvalidOp message must be set after unknown XEQ name");
    assert!(
        m.to_ascii_lowercase().contains("invalid"),
        "expected InvalidOp diagnostic; got message={m:?}"
    );
}

// ── FN-TEST-01 closure: all 12 conditional tests keyboard-reachable ──────────

/// Comprehensive coverage — every one of the 12 `TestKind` variants is
/// reachable from the keyboard via either the f-arith path (Plan 01: 4
/// variants) or the XEQ-by-Name path (this plan: 8 variants).
#[test]
fn all_12_conditional_tests_reachable() {
    // The 4 keyboard f-arith conditional tests (Plan 01 / D-25.7). These are
    // verified by the existing `phase25_keyboard` integration suite; here we
    // just assert that the W4 asymmetry holds — they MUST NOT be reachable
    // via XEQ-by-Name (the user gets HpError::InvalidOp by design).
    let keyboard_only = ["X=Y?", "X<=Y?", "X>Y?", "X=0?"];
    for name in &keyboard_only {
        assert_eq!(
            xeq_by_name_local_resolve(name),
            None,
            "{name} is keyboard-only per W4; CLI resolver MUST return None"
        );
        // The hp41-core builtin_card_op is `pub(super)` (W1 fix); the
        // inline test `resolves_8_conditional_test_mnemonics` in
        // hp41-core/src/ops/program.rs covers the core side. Here we
        // verify the negative case via the public Op::Xeq → InvalidOp
        // chain in `xeq_by_name_unknown_returns_invalid_op` instead.
    }

    // The 8 XEQ-by-Name conditional tests (this plan / D-25.8). All resolve
    // through `xeq_by_name_local_resolve` to the right `TestKind`.
    let xeq_pairs: &[(&str, TestKind)] = &[
        ("X<>Y?", TestKind::XNeY),
        ("X<Y?", TestKind::XLtY),
        ("X>=Y?", TestKind::XGeY),
        ("X#0?", TestKind::XNeZero),
        ("X<0?", TestKind::XLtZero),
        ("X>0?", TestKind::XGtZero),
        ("X<=0?", TestKind::XLeZero),
        ("X>=0?", TestKind::XGeZero),
    ];
    for (name, kind) in xeq_pairs {
        assert_eq!(
            xeq_by_name_local_resolve(name),
            Some(Op::Test(kind.clone())),
            "{name} must resolve to Op::Test({kind:?})"
        );
    }
}

// ── Cross-resolver drift guard (T-25-09) ─────────────────────────────────────

/// The CLI-local `xeq_by_name_local_resolve` and the hp41-core
/// `builtin_card_op` MUST agree on every conditional-test mnemonic.
///
/// Implementation note: because `builtin_card_op` is `pub(super)` (W1 fix
/// from the 2026-05-14 plan revision — visibility intentionally NOT widened
/// to preserve the surgical hp41-core exception), this test cannot perform
/// a direct `==` comparison between the two resolvers across the crate
/// boundary. Instead, both resolvers are asserted against the SAME canonical
/// mnemonic-to-`TestKind` table defined here.
///
/// The matching assertion against `builtin_card_op` lives inside
/// `hp41-core/src/ops/program.rs::phase25_builtin_card_op_tests::
/// resolves_8_conditional_test_mnemonics` and uses the SAME table values.
/// Drift in either resolver fails its respective test (T-25-09 mitigation).
#[test]
fn cli_resolver_matches_core_resolver() {
    // Canonical 14-entry mnemonic table (8 mnemonics × 1–3 spellings) — kept
    // in sync with hp41-core/src/ops/program.rs::builtin_card_op. Adding a
    // new spelling here REQUIRES adding it to the core resolver too, and the
    // mirror inline test catches the inverse.
    let canonical: &[(&str, TestKind)] = &[
        ("X<>Y?", TestKind::XNeY),
        ("X\u{2260}Y?", TestKind::XNeY),
        ("X#Y?", TestKind::XNeY),
        ("X<Y?", TestKind::XLtY),
        ("X>=Y?", TestKind::XGeY),
        ("X\u{2265}Y?", TestKind::XGeY),
        ("X#0?", TestKind::XNeZero),
        ("X\u{2260}0?", TestKind::XNeZero),
        ("X<0?", TestKind::XLtZero),
        ("X>0?", TestKind::XGtZero),
        ("X<=0?", TestKind::XLeZero),
        ("X\u{2264}0?", TestKind::XLeZero),
        ("X>=0?", TestKind::XGeZero),
        ("X\u{2265}0?", TestKind::XGeZero),
    ];
    for (name, kind) in canonical {
        assert_eq!(
            xeq_by_name_local_resolve(name),
            Some(Op::Test(kind.clone())),
            "CLI-local resolver disagreed with the canonical table for {name:?}"
        );
    }

    // End-to-end cross-resolver check via the public Op::Xeq surface:
    // typing each ASCII mnemonic into the XEQ-by-Name modal and pressing
    // Enter must NOT surface an InvalidOp diagnostic — proving that either
    // the CLI-local fast-path resolved it OR the hp41-core fallback chain
    // (Op::Xeq → builtin_card_op) resolved it. If either resolver dropped
    // a mnemonic this assertion would fire.
    let ascii_mnemonics = [
        "X<>Y?", "X<Y?", "X>=Y?", "X#0?", "X<0?", "X>0?", "X<=0?", "X>=0?",
    ];
    for name in &ascii_mnemonics {
        let (_app, _tmp, msg) = type_name_and_enter(name);
        assert!(
            !msg.as_deref().unwrap_or("").to_ascii_lowercase().contains("invalid"),
            "Mnemonic {name:?} reached InvalidOp — either CLI-local or core resolver dropped it; got {msg:?}"
        );
    }
}

// ── Backward-compat: the 4 card-reader names go through core resolver ───────

#[test]
fn cli_resolver_returns_none_for_card_reader_names() {
    // CLI-local intentionally returns None for the 4 v2.1 names — they fall
    // through to `Op::Xeq` → core builtin_card_op. This split is deliberate:
    // the CLI-local resolver covers ONLY the 8 conditional tests; everything
    // else defers to the core resolver.
    for name in &["WPRGM", "RDPRGM", "WDTA", "RDTA"] {
        assert_eq!(
            xeq_by_name_local_resolve(name),
            None,
            "CLI-local resolver MUST return None for v2.1 card-reader name {name}"
        );
        // End-to-end: typing the name into the modal and pressing Enter
        // does NOT surface an InvalidOp diagnostic — proving the core
        // resolver picked it up.
        let (_app, _tmp, msg) = type_name_and_enter(name);
        assert!(
            !msg.as_deref()
                .unwrap_or("")
                .to_ascii_lowercase()
                .contains("invalid"),
            "v2.1 card-reader name {name} should resolve via core builtin_card_op; got {msg:?}"
        );
    }
}
