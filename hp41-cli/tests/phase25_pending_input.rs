//! Phase 25 / Plan 02 — PendingInput Hybrid modal architecture integration tests.
//!
//! Tests cover:
//! - 6 new PendingInput variants (FlagPrompt, RegisterPrompt, ClpLabel, DelCount,
//!   TonePrompt, XeqByName) construct and pattern-match correctly.
//! - pending_prompt() is exhaustive (no `_ =>`) and renders each variant with
//!   the right mnemonic prefix (FN-CLI-04 compile-time guarantee).
//! - IND-toggle via shift-0 (D-25.12 + RESEARCH Pitfall 10) flips `ind` and
//!   reuses `App.shift_armed` from Plan 01 (W2 fix — no separate field).
//! - F-shifted modal openers (SF/CF/FS?/FC?/FS?C/FC?C, VIEW, ARCL, ASTO, ISG,
//!   DSE) populate the right PendingInput variant via `shifted_key_to_op`.
//! - Specialty modal scaffolds (ClpLabel cap-at-7, DelCount silent-clamp,
//!   TonePrompt auto-dispatch, XeqByName text-input) handle their accumulator
//!   shapes correctly.
//! - Esc cancels every new modal variant uniformly.

#![allow(clippy::unwrap_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use hp41_cli::app::{App, PendingInput};
use hp41_cli::keys::{FlagPromptKind, RegisterOpKind};
use hp41_core::ops::{FlagTestKind, Op, StoArithKind};
use hp41_core::state::CalcState;
use hp41_core::HpNum;

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
    let state_path = tmp.path().join("phase25-pending-input-test-state.json");
    let app = App::new(CalcState::new(), state_path, None);
    (app, tmp)
}

// ── Task 1: variant compile checks + pending_prompt exhaustive coverage ──────

/// Construct each of the 6 new variants once to force the compiler to verify
/// they exist with the documented shape. Pure compile-time guarantee — the
/// runtime asserts only confirm pattern destructuring.
#[test]
fn pending_input_variants_compile() {
    let v1 = PendingInput::FlagPrompt {
        kind: FlagPromptKind::SetFlag,
        ind: false,
        acc: String::new(),
    };
    let v2 = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Sto,
        ind: false,
        acc: String::new(),
    };
    let v3 = PendingInput::ClpLabel("HELLO".to_string());
    let v4 = PendingInput::DelCount("123".to_string());
    let v5 = PendingInput::TonePrompt;
    let v6 = PendingInput::XeqByName { acc: "FOO".to_string(), mode: hp41_cli::app::XeqByNameMode::Normal };

    assert!(matches!(
        v1,
        PendingInput::FlagPrompt {
            kind: FlagPromptKind::SetFlag,
            ind: false,
            ..
        }
    ));
    assert!(matches!(
        v2,
        PendingInput::RegisterPrompt {
            op: RegisterOpKind::Sto,
            ind: false,
            ..
        }
    ));
    assert!(matches!(v3, PendingInput::ClpLabel(_)));
    assert!(matches!(v4, PendingInput::DelCount(_)));
    assert!(matches!(v5, PendingInput::TonePrompt));
    assert!(matches!(v6, PendingInput::XeqByName { .. }));
}

/// pending_prompt() must render every new variant with the right mnemonic
/// prefix. The exhaustiveness of the match is guaranteed at compile time
/// (FN-CLI-04 — no `_ =>` catch-all in ui.rs::pending_prompt).
#[test]
fn pending_prompt_exhaustive() {
    use hp41_cli::ui::pending_prompt;

    // FlagPrompt
    let p = PendingInput::FlagPrompt {
        kind: FlagPromptKind::SetFlag,
        ind: false,
        acc: String::new(),
    };
    assert!(
        pending_prompt(Some(&p), None).starts_with("SF "),
        "got: {:?}",
        pending_prompt(Some(&p), None)
    );
    let p = PendingInput::FlagPrompt {
        kind: FlagPromptKind::ClearFlag,
        ind: false,
        acc: "1".to_string(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("CF "));
    let p = PendingInput::FlagPrompt {
        kind: FlagPromptKind::Test(FlagTestKind::IsSet),
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("FS?"));
    let p = PendingInput::FlagPrompt {
        kind: FlagPromptKind::Test(FlagTestKind::IsSetThenClear),
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("FS?C"));

    // IND-indication in the status text
    let p = PendingInput::FlagPrompt {
        kind: FlagPromptKind::SetFlag,
        ind: true,
        acc: "1".to_string(),
    };
    assert!(
        pending_prompt(Some(&p), None).contains("IND"),
        "ind: true must produce 'IND' in status — got {:?}",
        pending_prompt(Some(&p), None)
    );

    // RegisterPrompt
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Sto,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("STO "));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Sto,
        ind: true,
        acc: "0".to_string(),
    };
    assert!(pending_prompt(Some(&p), None).contains("STO IND"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Rcl,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("RCL "));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::StoArith(StoArithKind::Add),
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("STO+"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::StoArith(StoArithKind::Mul),
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("STO\u{00D7}"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::View,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("VIEW"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Arcl,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("ARCL"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Asto,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("ASTO"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Isg,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("ISG"));
    let p = PendingInput::RegisterPrompt {
        op: RegisterOpKind::Dse,
        ind: false,
        acc: String::new(),
    };
    assert!(pending_prompt(Some(&p), None).starts_with("DSE"));

    // Specialty variants
    let p = PendingInput::ClpLabel("AB".to_string());
    assert!(pending_prompt(Some(&p), None).starts_with("CLP"));
    let p = PendingInput::DelCount("12".to_string());
    assert!(pending_prompt(Some(&p), None).starts_with("DEL"));
    let p = PendingInput::TonePrompt;
    assert!(pending_prompt(Some(&p), None).starts_with("TONE"));
    let p = PendingInput::XeqByName { acc: "HELLO".to_string(), mode: hp41_cli::app::XeqByNameMode::Normal };
    assert!(pending_prompt(Some(&p), None).starts_with("XEQ"));
}

// ── Task 2: handle_pending_input IND-toggle + dispatch correctness ──────────

/// W2 fix / D-25.12 / RESEARCH Pitfall 10: with an open modal AND
/// `app.shift_armed == true` (set by pressing 'f' inside the modal), pressing
/// '0' MUST flip `ind` and clear `shift_armed` — the '0' is consumed by the
/// IND-toggle, NOT pushed into the digit accumulator. Subsequent `0` `5`
/// digits dispatch `Op::StoInd(5)`.
#[test]
fn test_ind_toggle_via_shift_0() {
    let (mut app, _tmp) = make_app();
    // Seed register 5 to a sentinel so STO 05 results in a verifiable state.
    app.state.regs[5] = HpNum::from(99);
    app.state.stack.x = HpNum::from(42);

    // Open the STO modal.
    app.pending_input = Some(PendingInput::RegisterPrompt {
        op: RegisterOpKind::Sto,
        ind: false,
        acc: String::new(),
    });

    // Press 'f' inside the modal — arms shift via the existing App.shift_armed
    // one-shot bit (Plan 01); modal stays open with ind unchanged.
    app.handle_key(key('f'));
    assert!(
        app.shift_armed,
        "pressing 'f' inside a modal must arm App.shift_armed (Plan 01 reuse, W2)"
    );
    match &app.pending_input {
        Some(PendingInput::RegisterPrompt { op, ind, acc }) => {
            assert!(matches!(op, RegisterOpKind::Sto), "op unchanged after 'f'");
            assert!(!ind, "ind still false right after pressing 'f'");
            assert!(acc.is_empty(), "acc unchanged after 'f'");
        }
        other => panic!("expected open RegisterPrompt after 'f'; got {other:?}"),
    }

    // Press '0' — the IND-toggle keystroke. Must flip ind AND clear shift_armed.
    app.handle_key(key('0'));
    assert!(
        !app.shift_armed,
        "shift_armed must clear after the IND-toggle consumes the '0'"
    );
    match &app.pending_input {
        Some(PendingInput::RegisterPrompt { op, ind, acc }) => {
            assert!(matches!(op, RegisterOpKind::Sto));
            assert!(*ind, "ind must flip to true after shift-0 IND-toggle");
            assert!(
                acc.is_empty(),
                "'0' was consumed by IND-toggle, NOT pushed into acc"
            );
        }
        other => panic!("expected RegisterPrompt {{ ind:true }}; got {other:?}"),
    }

    // Now accumulate two digits 05 and verify Op::StoInd(5) dispatched.
    app.handle_key(key('0'));
    app.handle_key(key('5'));
    assert!(
        app.pending_input.is_none(),
        "after 2-digit accumulation, modal must close"
    );
    // STO IND 05 means: store X into the register pointed to by regs[5].
    // We did not seed an indirect pointer; regs[5] = 99 (from the seed above).
    // op_sto_ind resolves the integer part of regs[5] = 99 and stores X there.
    assert_eq!(
        app.state.regs[99],
        HpNum::from(42),
        "STO IND 05 must store X=42 into regs[regs[5].int_part] = regs[99]"
    );
}

/// FlagPrompt direct (SF 12) dispatches Op::SfFlag(12) which sets bit 12 in
/// state.flags. The modal opens via shifted_key_to_op (f-shifted '7' per the
/// HP-41CV reference card — see PLAN.md mapping table) but for this unit
/// test we open it directly and exercise the 2-digit accumulator.
#[test]
fn test_flag_prompt_dispatches() {
    // SF 12 — direct (no IND).
    let (mut app, _tmp) = make_app();
    app.pending_input = Some(PendingInput::FlagPrompt {
        kind: FlagPromptKind::SetFlag,
        ind: false,
        acc: String::new(),
    });
    app.handle_key(key('1'));
    app.handle_key(key('2'));
    assert!(app.pending_input.is_none(), "modal closes after 2 digits");
    assert!(
        app.state.flags & (1u64 << 12) != 0,
        "SF 12 must set flag bit 12; got flags={:#x}",
        app.state.flags
    );

    // CF 12 — direct.
    let (mut app2, _tmp2) = make_app();
    app2.state.flags |= 1u64 << 12; // pre-set
    app2.pending_input = Some(PendingInput::FlagPrompt {
        kind: FlagPromptKind::ClearFlag,
        ind: false,
        acc: String::new(),
    });
    app2.handle_key(key('1'));
    app2.handle_key(key('2'));
    assert!(
        app2.state.flags & (1u64 << 12) == 0,
        "CF 12 must clear flag bit 12"
    );
}

#[test]
fn test_flag_prompt_ind_dispatches_through_shift_0() {
    // Pre-set regs[5] to 12 so SF IND 05 resolves to flag 12.
    let (mut app, _tmp) = make_app();
    app.state.regs[5] = HpNum::from(12);
    app.pending_input = Some(PendingInput::FlagPrompt {
        kind: FlagPromptKind::SetFlag,
        ind: false,
        acc: String::new(),
    });
    app.handle_key(key('f'));
    app.handle_key(key('0'));
    // ind toggled to true; accumulator empty.
    match &app.pending_input {
        Some(PendingInput::FlagPrompt { ind, acc, .. }) => {
            assert!(*ind, "ind toggled");
            assert!(acc.is_empty(), "acc empty after shift-0");
        }
        other => panic!("unexpected state {other:?}"),
    }
    app.handle_key(key('0'));
    app.handle_key(key('5'));
    // SF IND 05 → flag at regs[5].int_part = 12.
    assert!(
        app.state.flags & (1u64 << 12) != 0,
        "SF IND 05 (pointer regs[5]=12) must set flag 12; got {:#x}",
        app.state.flags
    );
}

/// ClpLabel modal caps the accumulator at 7 characters (HP-41 LBL name limit).
#[test]
fn test_clp_label_capped() {
    let (mut app, _tmp) = make_app();
    app.pending_input = Some(PendingInput::ClpLabel(String::new()));

    // Push 8 alphabetic chars — only 7 must land in the accumulator.
    for c in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
        app.handle_key(key(c));
    }
    match &app.pending_input {
        Some(PendingInput::ClpLabel(acc)) => {
            assert!(
                acc.len() <= 7,
                "ClpLabel accumulator must cap at 7 chars (HP-41 LBL limit); got len {}",
                acc.len()
            );
            assert_eq!(acc, "ABCDEFG", "first 7 chars must survive");
        }
        other => panic!("expected ClpLabel still open; got {other:?}"),
    }

    // Enter dispatches Op::Clp("ABCDEFG") — succeeds only if a matching label
    // exists. Our program is empty so dispatch will return InvalidOp; we
    // assert the modal closes regardless.
    app.handle_key(raw_key(KeyCode::Enter));
    assert!(
        app.pending_input.is_none(),
        "Enter must close the ClpLabel modal"
    );
}

/// DelCount accumulates 3 digits and silently clamps to u8::MAX on overflow.
#[test]
fn test_del_count_silent_clamp() {
    let (mut app, _tmp) = make_app();
    // Seed a 5-step program so DEL has something to remove (DEL on an empty
    // program is a documented no-op — Op::Del is silent-clamped on both ends).
    app.state.program = vec![Op::Null, Op::Null, Op::Null, Op::Null, Op::Null];
    app.state.prgm_mode = true; // DEL is PRGM-mode-gated
    app.state.pc = 0;

    app.pending_input = Some(PendingInput::DelCount(String::new()));
    app.handle_key(key('9'));
    app.handle_key(key('9'));
    app.handle_key(key('9')); // 999 — exceeds u8::MAX
    assert!(
        app.pending_input.is_none(),
        "DelCount auto-closes after 3 digits"
    );
    // Op::Del(255) silently clamps to program.len() — the whole 5-step program
    // is deleted.
    assert!(
        app.state.program.is_empty(),
        "DEL 999 must silent-clamp to u8::MAX and delete the whole program (5 steps)"
    );
}

/// TonePrompt auto-dispatches on a single 0–9 digit (no Enter required).
#[test]
fn test_tone_prompt_auto_dispatch() {
    let (mut app, _tmp) = make_app();
    app.pending_input = Some(PendingInput::TonePrompt);
    app.handle_key(key('5'));
    assert!(
        app.pending_input.is_none(),
        "TonePrompt closes after a single digit"
    );
    // Op::Tone pushes a "TONE n" event into state.event_buffer.
    assert!(
        app.state.event_buffer.iter().any(|e| e.starts_with("TONE")),
        "TONE 5 must push a TONE event; got events={:?}",
        app.state.event_buffer
    );
}

/// XeqByName modal scaffold: type a name, press Enter, the existing core
/// resolver chain handles it. For an unknown label dispatch errors out, but
/// the modal MUST close.
#[test]
fn test_xeq_by_name_modal_scaffold() {
    let (mut app, _tmp) = make_app();
    app.pending_input = Some(PendingInput::XeqByName { acc: String::new(), mode: hp41_cli::app::XeqByNameMode::Normal });
    for c in ['H', 'E', 'L', 'L', 'O'] {
        app.handle_key(key(c));
    }
    match &app.pending_input {
        Some(PendingInput::XeqByName { acc, .. }) => {
            assert_eq!(acc, "HELLO", "5 chars accumulated");
        }
        other => panic!("expected XeqByName open; got {other:?}"),
    }
    app.handle_key(raw_key(KeyCode::Enter));
    assert!(
        app.pending_input.is_none(),
        "Enter must close XeqByName modal regardless of dispatch outcome"
    );
}

/// Esc cancels every new PendingInput variant uniformly — no dispatch, no
/// leaked shift_armed.
#[test]
fn test_esc_cancels_all_new_variants() {
    let variants = vec![
        PendingInput::FlagPrompt {
            kind: FlagPromptKind::SetFlag,
            ind: false,
            acc: "1".to_string(),
        },
        PendingInput::RegisterPrompt {
            op: RegisterOpKind::Rcl,
            ind: true,
            acc: "0".to_string(),
        },
        PendingInput::ClpLabel("AB".to_string()),
        PendingInput::DelCount("12".to_string()),
        PendingInput::TonePrompt,
        PendingInput::XeqByName { acc: "FOO".to_string(), mode: hp41_cli::app::XeqByNameMode::Normal },
    ];

    for v in variants {
        let (mut app, _tmp) = make_app();
        app.pending_input = Some(v);
        app.handle_key(raw_key(KeyCode::Esc));
        assert!(
            app.pending_input.is_none(),
            "Esc must cancel the modal (variant: see test trace)"
        );
        assert!(!app.shift_armed, "Esc must not leak shift_armed=true");
    }
}

// ── F-shifted modal opener wiring ────────────────────────────────────────────

/// Pressing 'f' then '7' opens FlagPrompt{SetFlag} (per the f-shifted SF
/// position on the HP-41CV keyboard).
#[test]
fn test_f_shifted_seven_opens_sf_prompt() {
    let (mut app, _tmp) = make_app();
    app.handle_key(key('f'));
    app.handle_key(key('7'));
    assert!(
        !app.shift_armed,
        "shift_armed consumed by the modal-opener key"
    );
    match &app.pending_input {
        Some(PendingInput::FlagPrompt { kind, ind, acc }) => {
            assert!(matches!(kind, FlagPromptKind::SetFlag));
            assert!(!ind, "fresh modal opens with ind=false");
            assert!(acc.is_empty(), "fresh modal opens with empty acc");
        }
        other => panic!("expected FlagPrompt{{SetFlag}}; got {other:?}"),
    }
}

/// Pressing 'f' then '8' opens FlagPrompt{ClearFlag}.
#[test]
fn test_f_shifted_eight_opens_cf_prompt() {
    let (mut app, _tmp) = make_app();
    app.handle_key(key('f'));
    app.handle_key(key('8'));
    assert!(matches!(
        app.pending_input,
        Some(PendingInput::FlagPrompt {
            kind: FlagPromptKind::ClearFlag,
            ..
        })
    ));
}

/// Pressing 'f' then '9' opens FlagPrompt{Test(IsSet)} (FS?).
#[test]
fn test_f_shifted_nine_opens_fs_prompt() {
    let (mut app, _tmp) = make_app();
    app.handle_key(key('f'));
    app.handle_key(key('9'));
    assert!(matches!(
        app.pending_input,
        Some(PendingInput::FlagPrompt {
            kind: FlagPromptKind::Test(FlagTestKind::IsSet),
            ..
        })
    ));
}
