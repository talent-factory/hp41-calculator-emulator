# Phase 25: CLI Integration & Documentation — Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 17 (10 modified, 7 new — plus shared cross-cutting patterns)
**Analogs found:** 17 / 17 (every target has a strong in-repo analog)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-cli/src/keys.rs` (rewrite + add `shifted_key_to_op`, `xeq_by_name_local_resolve`) | controller (key→Op router) | request-response | `hp41-cli/src/keys.rs` (current `key_to_op`) + `hp41-gui/src/Keyboard.tsx` KEY_DEFS (three-label structural reference) | exact (same file, role-preserving rewrite) |
| `hp41-cli/src/app.rs` — PendingInput enum (+6 variants) | model (transient UI state) | event-driven | `hp41-cli/src/app.rs:22-41` (existing enum) | exact |
| `hp41-cli/src/app.rs` — `App.shift_armed: bool` + arming logic in `handle_key` | controller (state machine) | event-driven | `hp41-gui/src/App.tsx:111-119, 161-206` (`shiftActive` one-shot — parity reference per D-25.6) | exact (cross-tier parity) |
| `hp41-cli/src/app.rs` — `handle_pending_input` arms (FlagPrompt/RegisterPrompt/Clp/Del/Tone/XeqByName) | controller (modal accumulator) | event-driven | `hp41-cli/src/app.rs:913-949` (`handle_reg_modal`) + `app.rs:746-787` (AssignKey/AssignLabel text-modal) | role-match |
| `hp41-cli/src/ui.rs` — `pending_prompt()` extension | view (status-bar formatter) | request-response | `hp41-cli/src/ui.rs:238-273` (existing exhaustive match) | exact |
| `hp41-cli/src/ui.rs` — `render_status()` `f→` indicator | view | request-response | `hp41-cli/src/ui.rs:203-220` (annunciator-bar `ann("SHIFT", false)`) | exact |
| `hp41-cli/src/help_data.rs` — migrate to JSON-loaded `help_entries()` | model (data source) | file-I/O (compile-time) | `hp41-cli/src/programs.rs:7-24` (`OnceLock<Vec<SampleProgram>>` precedent) | exact (idiom-match) |
| `hp41-cli/Cargo.toml` — confirm serde/serde_json (no new deps) | config | n/a | `hp41-cli/Cargo.toml:16-17` (already declared) | exact |
| `hp41-core/src/ops/program.rs` — `builtin_card_op` 4→12 extension | service (name resolver) | request-response | `hp41-core/src/ops/program.rs:966-974` (current 4-entry match) | exact |
| `docs/hp41cv-functions.json` (NEW canonical data) | config (canonical data) | file-I/O | `hp41-cli/src/help_data.rs:11+` (HELP_DATA tuple) — schema source | role-match |
| `docs/hp41cv-function-matrix.md` (NEW generated) | docs | file-I/O | `.planning/MILESTONES.md` (similar long-form markdown table) | role-match |
| `scripts/docs-matrix/Cargo.toml` + `src/main.rs` (NEW standalone bin) | utility (codegen) | transform | `hp41-gui/src-tauri/Cargo.toml` (nested non-workspace pattern) | role-match |
| `justfile` — `docs-matrix` + `docs-matrix-check` recipes | config (task runner) | n/a | `justfile:66-87` (gui-* recipe pattern) | role-match |
| `CLAUDE.md` — append v2.2 settled-architecture block | docs | n/a | `CLAUDE.md` (existing `### v2.1 additions` section) | exact |
| `README.md` — soft "feature-complete HP-41CV" claim + matrix link | docs | n/a | `README.md` (existing badges/intro) | exact |
| `hp41-cli/tests/phase25_keyboard.rs` + `phase25_pending_input.rs` + `phase25_xeq_by_name.rs` + `phase25_help_data.rs` | test | n/a | `hp41-cli/tests/card_io_tests.rs` (integration-test scaffold) | exact |
| `hp41-core/tests/phase25_builtin_card_op.rs` + `op_matrix_parity.rs` | test | n/a | `hp41-core/src/ops/program.rs:1495-1510` (`builtin_card_op_resolves_four_names`) | exact |

## Pattern Assignments

### `hp41-cli/src/keys.rs` (controller, request-response) — full rewrite

**Analog 1:** `hp41-cli/src/keys.rs` (current `key_to_op` shape, lines 18-87) — preserve the `fn(KeyEvent, &App) -> Option<Op>` signature and the bottom `_ => None` discipline. **Strip every v1.x letter binding per D-25.3** (`C/T/L/G/E/H/I/W/Y/q/a/c/k/s` etc.) — they coincided with v1.x crossterm convention, not HP-41CV positions.

**Analog 2 (structural reference for the three-label model):** `hp41-gui/src/Keyboard.tsx:23-94` (KEY_DEFS) — drives the `primary / shifted / alphaChar` layout that the new `KEY_REF_TABLE` must mirror, key-for-key, so CLI ↔ GUI parity (D-25.6) holds.

**Imports pattern** (keys.rs:10-13 — KEEP verbatim):
```rust
use crossterm::event::{KeyCode, KeyEvent};
use hp41_core::ops::Op;

use crate::app::App;
```

**Core pattern — `key_to_op` shape to preserve** (keys.rs:15-87 outer match; rewrite arms only):
```rust
pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        // Stack (HP-41CV row 4): ENTER↑
        KeyCode::Enter => Some(Op::Enter),
        // Bottom-row arithmetic — direct (UN-shifted):
        KeyCode::Char('+') => Some(Op::Add),
        KeyCode::Char('-') => Some(Op::Sub),
        KeyCode::Char('*') => Some(Op::Mul),
        KeyCode::Char('/') => Some(Op::Div),
        // Digits/EEX/'.' are handled in app.rs before key_to_op (existing convention).
        // All v1.x letter bindings (C/T/L/G/E/H/I/W/Y/q/a/c/k) REMOVED — reached via f-prefix only.
        _ => None,
    }
}
```

**Adaptation notes — new `shifted_key_to_op` (Phase 25 NEW):**
- Same signature `fn(KeyEvent, &App) -> Option<Op>`; called only when `app.shift_armed == true`.
- The 4 hardware-anchored conditional tests live HERE (D-25.7):
  ```rust
  KeyCode::Char('-') => Some(Op::Test(TestKind::XEqY)),
  KeyCode::Char('+') => Some(Op::Test(TestKind::XLeY)),
  KeyCode::Char('*') => Some(Op::Test(TestKind::XGtY)),
  KeyCode::Char('/') => Some(Op::Test(TestKind::XEqZero)),
  ```
- The match can also return `None` and instead OPEN a modal (e.g. f-7 = SF prompt) — same pattern as today's `S`/`R` returning `None` because the modal is opened in `handle_key`. Document explicitly.

**KEY_REF_TABLE rewrite** (keys.rs:91-183 — preserve `&[(&str, &str)]` shape, regenerate content from `docs/hp41cv-functions.json`):
- Today's table mixes primary/shifted/letter conventions. Regenerate strictly from the HP-41CV reference card; entries become `("f -", "X=Y? (X equals Y test)")` etc.
- Recommended: derive content at compile-time from `help_data::help_entries()` rather than hand-curating a second copy. Eliminates drift between help-overlay and right-panel.

**`xeq_by_name_local_resolve` (NEW free function in keys.rs)** — per RESEARCH §"XEQ-by-Name CLI Modal":
```rust
fn xeq_by_name_local_resolve(name: &str) -> Option<Op> {
    use hp41_core::ops::{Op, TestKind};
    match name {
        "X<>Y?" | "X≠Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        "X<Y?"                    => Some(Op::Test(TestKind::XLtY)),
        "X>=Y?" | "X≥Y?"          => Some(Op::Test(TestKind::XGeY)),
        // … 5 more mnemonics — accept both ASCII-pure and Unicode-symbol forms.
        _ => None,
    }
}
```

**Why preserve `Option<Op>` returns:** every existing call site in `handle_key` already handles `None` as "modal-opens or no-op". Don't change the call-site contract.

---

### `hp41-cli/src/app.rs` — `PendingInput` enum (model, event-driven)

**Analog:** `hp41-cli/src/app.rs:22-41` (existing 11-variant enum).

**Current shape — preserve all 11 existing variants verbatim** (app.rs:22-41):
```rust
#[derive(Debug, Clone)]
pub enum PendingInput {
    StoRegister(String),      // accumulating 2-digit register number for STO [nn]
    RclRegister(String),
    StoAdd(String),
    StoSub(String),
    StoMul(String),
    StoDiv(String),
    AssignKey,
    AssignLabel(char, String),
    ConfirmLoad(usize),
    FmtDigits(hp41_core::DisplayMode),
    PrintModal,
    HexModal(String),
}
```

**Phase 25 additions** — per D-25.11 (Hybrid struct-variants for groups + specialty unique variants):
```rust
    // Phase 25: Flag operations modal (SF/CF/FS?/FC?/FS?C/FC?C × direct/IND).
    FlagPrompt {
        kind: FlagPromptKind,       // local TUI enum — see keys.rs
        ind: bool,
        acc: String,
    },

    // Phase 25: Register operations modal (STO/RCL/STO+-*/VIEW/ARCL/ASTO/ISG/DSE × direct/IND).
    RegisterPrompt {
        op: RegisterOpKind,         // local TUI enum — wraps hp41_core::StoArithKind
        ind: bool,
        acc: String,
    },

    // Specialty unique variants:
    ClpLabel(String),               // CLP "name" — text-input modal
    DelCount(String),               // DEL nnn — 3-digit numeric
    TonePrompt,                     // TONE n — single-digit 0-9
    XeqByName(String),              // XEQ "NAME" — text-input modal (Phase 25 introduces it)
```

**Adaptation notes:**
- The existing `StoRegister` / `RclRegister` / `StoAdd/Sub/Mul/Div` variants get **DEPRECATED but not yet removed** in Phase 25 Wave 0 — keep them compiling so the test suite still passes; new modal dispatches route to `RegisterPrompt { op: RegisterOpKind::Sto, … }`. Wave N removes the legacy variants once `pending_prompt()` and `handle_pending_input` are fully migrated.
- **All-new variants use struct syntax** (not tuple) for the `kind`/`ind`/`acc` triple — matches `Op::FlagTest { kind, flag }` precedent in hp41-core (D-25.13 reuse rule). Tuple form `(FlagPromptKind, bool, String)` is rejected as less self-documenting.
- **`FlagPromptKind` and `RegisterOpKind` are TUI-local enums** (defined in keys.rs or a new `app::modal_kinds` submodule). They wrap `hp41_core::FlagTestKind` and `hp41_core::StoArithKind` rather than redefining them.

---

### `hp41-cli/src/app.rs` — `App.shift_armed: bool` + arming logic (controller, event-driven)

**Analog (cross-tier parity — D-25.6 mandate):** `hp41-gui/src/App.tsx:111-119, 161-206` (the v2.1 SHIFT one-shot).

**One-shot lifecycle in GUI** (App.tsx:111-119, 161-206 — the design Phase 25 mirrors):
```typescript
// Frontend-owned SHIFT one-shot prefix (no IPC round-trip).
const [shiftActive, setShiftActive] = useState(false);
// …
const handleClick = useCallback(async (key: KeyDef) => {
    if (busyRef.current) return;
    if (key.id === 'shift') { setShiftActive(prev => !prev); return; }
    // …
    let consumesShift = false;
    if (alphaOn && key.alphaChar)         { effectiveId = `alpha_${key.alphaChar}`; }
    else if (shiftActive && key.shifted)  { effectiveId = key.shifted.id; consumesShift = true; }
    else                                   { effectiveId = key.id; }
    // …
    } finally {
        if (consumesShift) setShiftActive(false);   // one-shot consume
        busyRef.current = false;
    }
}, [calcState, shiftActive]);
```

**Adaptation for CLI** — three changes:
1. Add `pub shift_armed: bool` to `App` struct (app.rs:44-74) and initialize to `false` in `App::new` (app.rs:120-133).
2. Insert arming check in `handle_key` **AFTER** the `pending_input` route (app.rs:228-231) — same ordering rule as ALPHA mode interceptors per Pitfall 4 / 5. **BEFORE** the ALPHA mode block (app.rs:297-302) so `f` in ALPHA mode types F (D-25.5).
3. **Always** clear `shift_armed = false` at the end of the consumed-prefix branch — even if `shifted_key_to_op` returned `None` (Pitfall 5 in RESEARCH).

**Imports pattern** — no new imports needed; `KeyCode::Char('f')` already in scope via `use crossterm::event::{KeyCode, …}` (app.rs:11).

**Anti-pattern to avoid:** Don't put `shift_armed` on `CalcState` — it's frontend-only per D-25.5 (never crosses IPC). This is the exact same boundary as GUI v2.1's `shiftActive` (which lives in React state, not `CalcStateView`).

---

### `hp41-cli/src/app.rs` — `handle_pending_input` arms (controller, event-driven modal accumulator)

**Primary analog:** `hp41-cli/src/app.rs:913-949` (`handle_reg_modal` — generic 2-digit accumulator).

**Reusable scaffold pattern** (app.rs:913-949):
```rust
fn handle_reg_modal(
    &mut self,
    key: KeyEvent,
    acc: String,
    op_fn: impl Fn(u8) -> Op,
    pending_fn: impl Fn(String) -> PendingInput,
) {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let mut new_acc = acc;
            new_acc.push(c);
            if new_acc.len() == 2 {
                let reg: u8 = new_acc.parse()
                    .expect("two ASCII digit chars always parse as u8 ≤ 99");
                self.call_dispatch(op_fn(reg));
                self.pending_input = None;
            } else {
                self.pending_input = Some(pending_fn(new_acc));
            }
        }
        KeyCode::Backspace => self.pending_input = Some(pending_fn(String::new())),
        KeyCode::Esc => self.pending_input = None,
        _ => self.pending_input = Some(pending_fn(acc)),
    }
}
```

**Phase 25 extension — `handle_reg_modal_with_ind`** (NEW) — adapt the scaffold to support `ind: bool` toggling:
- Detect IND-toggle keystroke FIRST (`KeyCode::Char('I') | KeyCode::Char('i')` for simple flavor; OR `f`-while-in-modal for hardware-faithful per RESEARCH p.14 verbatim). Open question: pick one or implement both — see RESEARCH Open Q 3.
- On IND-toggle, mutate `ind` field of the struct-variant; preserve `acc`.
- On 2nd digit fill, dispatch resolved op: `if ind { Op::StoInd(n) } else { Op::StoReg(n) }` — single dispatch decision point per D-25.12.
- Same `Backspace` / `Esc` / fallthrough discipline as existing scaffold.

**Secondary analog — text-input modal (for `ClpLabel`/`XeqByName`):** `hp41-cli/src/app.rs:746-787` (`AssignKey` → `AssignLabel` text accumulator):
```rust
Some(PendingInput::AssignLabel(c, ref acc)) => {
    match key.code {
        KeyCode::Esc => self.pending_input = None,
        KeyCode::Enter => {
            if !acc.is_empty() {
                self.state.key_assignments.insert(c, acc.clone());
                self.message = Some(format!("Assigned '{c}' \u{2192} LBL:{acc}"));
            }
            self.pending_input = None;
        }
        KeyCode::Backspace => {
            let mut new_acc = acc.clone();
            new_acc.pop();
            self.pending_input = Some(PendingInput::AssignLabel(c, new_acc));
        }
        KeyCode::Char(ch) => {
            let mut new_acc = acc.clone();
            new_acc.push(ch);
            self.pending_input = Some(PendingInput::AssignLabel(c, new_acc));
        }
        _ => self.pending_input = Some(PendingInput::AssignLabel(c, acc.clone())),
    }
}
```

**Adaptation for `XeqByName(String)`:**
- Copy this exact scaffold structure.
- On `KeyCode::Enter`: first try `xeq_by_name_local_resolve(&acc)`; if `Some(op)`, dispatch directly via `self.call_dispatch(op)`. Else fall through to `self.call_dispatch(Op::Xeq(acc.clone()))` (which uses hp41-core's existing user-LBL → builtin_card_op chain).
- Cap accumulator length at 24 chars (HP-41 ALPHA register width) per RESEARCH Security V5.

**Adaptation for `ClpLabel(String)`:** identical to AssignLabel scaffold; on Enter dispatches `Op::Clp(acc.clone())`. Cap at 7 chars (HP-41 LBL name limit).

**Adaptation for `DelCount(String)`:** identical to `handle_reg_modal` scaffold BUT 3-digit accumulator (terminate-on-3, not terminate-on-2); silent-clamp at u8::MAX (255) per RESEARCH on Op::Del semantics.

**Adaptation for `TonePrompt` (unit variant, no acc):** auto-dispatch on first digit 0-9; Esc cancels. Even simpler than the 2-digit scaffold.

---

### `hp41-cli/src/ui.rs` — `pending_prompt()` extension (view, request-response)

**Analog:** `hp41-cli/src/ui.rs:238-273` (existing exhaustive match).

**Core pattern — preserve exhaustive-match discipline** (ui.rs:238-273):
```rust
fn pending_prompt(pending: &crate::app::PendingInput) -> String {
    use crate::app::PendingInput;
    match pending {
        PendingInput::StoRegister(acc) => format!("STO [{acc:_<2}]"),
        PendingInput::RclRegister(acc) => format!("RCL [{acc:_<2}]"),
        PendingInput::StoAdd(acc) => format!("STO+ [{acc:_<2}]"),
        // … 8 more arms …
        PendingInput::HexModal(acc) => {
            if acc.is_empty() {
                "HEX: __".to_string()
            } else {
                format!("HEX: {acc}_")
            }
        }
    }
}
```

**Adaptation notes — 6 new arms (D-25.14 — exhaustiveness is the FN-CLI-04 guarantee):**

```rust
PendingInput::FlagPrompt { kind, ind, acc } => {
    let mnemonic = match kind {
        FlagPromptKind::SetFlag => "SF",
        FlagPromptKind::ClearFlag => "CF",
        FlagPromptKind::Test(FlagTestKind::IsSet) => "FS?",
        FlagPromptKind::Test(FlagTestKind::IsClear) => "FC?",
        FlagPromptKind::Test(FlagTestKind::IsSetThenClear) => "FS?C",
        FlagPromptKind::Test(FlagTestKind::IsClearThenClear) => "FC?C",
    };
    let ind_str = if *ind { " IND" } else { "" };
    format!("{mnemonic}{ind_str} [{acc:_<2}]")
}
PendingInput::RegisterPrompt { op, ind, acc } => {
    let mnemonic = match op {
        RegisterOpKind::Sto => "STO",
        RegisterOpKind::Rcl => "RCL",
        RegisterOpKind::StoArith(StoArithKind::Add) => "STO+",
        RegisterOpKind::StoArith(StoArithKind::Sub) => "STO-",
        RegisterOpKind::StoArith(StoArithKind::Mul) => "STO\u{00D7}",
        RegisterOpKind::StoArith(StoArithKind::Div) => "STO\u{00F7}",
        RegisterOpKind::View => "VIEW",
        RegisterOpKind::Arcl => "ARCL",
        RegisterOpKind::Asto => "ASTO",
        RegisterOpKind::Isg => "ISG",
        RegisterOpKind::Dse => "DSE",
    };
    let ind_str = if *ind { " IND" } else { "" };
    format!("{mnemonic}{ind_str} [{acc:_<2}]")
}
PendingInput::ClpLabel(acc) => format!("CLP [{acc}]_"),
PendingInput::DelCount(acc) => format!("DEL [{acc:_<3}]"),
PendingInput::TonePrompt => "TONE [_]".to_string(),
PendingInput::XeqByName(acc) => format!("XEQ \"{acc}\"_"),
```

**Hard rule (FN-CLI-04):** No `_ =>` catch-all. No `unreachable!()`. The compiler is the runtime guarantee.

---

### `hp41-cli/src/ui.rs` — `render_status` `f→` indicator (view, request-response)

**Analog:** `hp41-cli/src/ui.rs:203-220` (annunciator bar) AND `ui.rs:223-234` (`render_status` body).

**Current shape — annunciator with hardcoded false** (ui.rs:206-219, including line 212):
```rust
let line = Line::from(vec![
    ann("USER", st.user_mode),
    Span::raw(" "),
    ann("PRGM", st.prgm_mode),
    Span::raw(" "),
    ann("ALPHA", st.alpha_mode),
    Span::raw(" "),
    ann("SHIFT", false),     // ← Phase 25 changes this to `app.shift_armed`
    // … RAD/DEG/GRAD …
]);
```

**Adaptation note:** Single-line edit at ui.rs:212 — replace `false` with `app.shift_armed`. The `ann()` helper already handles the bold/dim styling toggle, so no other view changes needed.

**Optional second change** (RESEARCH Open Q 5 — recommended yes):  
In `render_status` body (ui.rs:226-232), prepend `"f\u{2192} "` to the status text when `app.shift_armed && app.pending_input.is_none()` so users see the armed-prefix state in the status bar too.

---

### `hp41-cli/src/help_data.rs` — JSON-loaded migration (model, file-I/O)

**Analog (project precedent for `include_str!` + `OnceLock`):** `hp41-cli/src/programs.rs:7-24`.

**Reference imports + cache pattern** (programs.rs:1-24):
```rust
use std::sync::OnceLock;

use hp41_core::ops::{Op, StoArithKind, TestKind};
use hp41_core::HpNum;

pub struct SampleProgram {
    pub name: &'static str,
    pub description: &'static str,
    pub ops: Vec<Op>,
}

static PROGRAMS_CACHE: OnceLock<Vec<SampleProgram>> = OnceLock::new();

pub fn sample_programs() -> &'static [SampleProgram] {
    PROGRAMS_CACHE.get_or_init(build_all_programs)
}
```

**Adaptation for help_data.rs** — same shape, JSON parse instead of build_all_programs:
```rust
use std::sync::OnceLock;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HelpEntry {
    pub op_variant: String,
    pub display_name: String,
    pub category: String,
    pub status: String,           // "implemented" | "deferred-v3" | "na"
    pub phase: Option<String>,
    pub key_path: Option<String>,
    pub description: String,
    #[serde(default)]
    pub divergences: Vec<String>,
}

const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");

static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(FUNCTIONS_JSON)
            .expect("hp41cv-functions.json is malformed — fix the JSON")
    })
}
```

**Adaptation notes:**
- **Existing `HELP_DATA` const** (help_data.rs:11+) — leave as transitional shim if `ui.rs:285` still references it: define `pub fn help_data() -> Vec<(&'static str, &'static str, &'static str)>` that derives 3-tuples from `help_entries()` so `render_help_overlay` (ui.rs:280-311) keeps compiling. Wave N rewrites `render_help_overlay` to consume `HelpEntry` directly.
- **`.expect("…")` on JSON parse is intentional** per D-25.17 — malformed JSON is a build-time blocker. Mitigated by `tests/phase25_help_data.rs` smoke test asserting `help_entries().len() >= 130` (Pitfall 7).

**Anti-pattern to avoid:**
- Do NOT use `lazy_static` or external `once_cell` — stdlib `OnceLock` is the project precedent (MSRV 1.88 has it).
- Do NOT introduce `build.rs` for codegen — `include_str!` is sufficient per D-25.16.

---

### `hp41-cli/Cargo.toml` — dependency check (config)

**Analog:** `hp41-cli/Cargo.toml:11-19` (current dependency block).

**Current state** (Cargo.toml:11-19):
```toml
[dependencies]
hp41-core = { path = "../hp41-core" }
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = "0.29"
clap = { version = "4", features = ["derive"] }
serde = { workspace = true }
serde_json = { workspace = true }
dirs = "6"
```

**Adaptation notes:** **NO changes required.** `serde` + `serde_json` already present (lines 16-17). `OnceLock` is stdlib (MSRV 1.88). The original mandate "add serde + serde_json runtime deps" is satisfied transitively.

---

### `hp41-core/src/ops/program.rs` — `builtin_card_op` 4→12 extension (service, request-response)

**Analog:** `hp41-core/src/ops/program.rs:966-974` (current 4-entry match).

**Current shape** (program.rs:966-974):
```rust
pub(super) fn builtin_card_op(name: &str) -> Option<Op> {
    match name {
        "WPRGM" => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA" => Some(Op::Wdta),
        "RDTA" => Some(Op::Rdta),
        _ => None,
    }
}
```

**Adaptation — extend to 12 entries** (per RESEARCH Recommendation Path 2, user-cleared per Discussion log):
```rust
pub(super) fn builtin_card_op(name: &str) -> Option<Op> {
    use crate::ops::TestKind;
    match name {
        // v2.1 card-reader builtins (unchanged):
        "WPRGM" => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA"  => Some(Op::Wdta),
        "RDTA"  => Some(Op::Rdta),
        // Phase 25: 8 non-keyboard conditional tests (D-25.8).
        // Accept BOTH ASCII-pure and Unicode-symbol spellings per RESEARCH p.8.
        "X<>Y?" | "X≠Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        "X<Y?"                    => Some(Op::Test(TestKind::XLtY)),
        "X>=Y?" | "X≥Y?"          => Some(Op::Test(TestKind::XGeY)),
        "X#0?" | "X≠0?"           => Some(Op::Test(TestKind::XNeZero)),
        "X<0?"                    => Some(Op::Test(TestKind::XLtZero)),
        "X>0?"                    => Some(Op::Test(TestKind::XGtZero)),
        "X<=0?" | "X≤0?"          => Some(Op::Test(TestKind::XLeZero)),
        "X>=0?" | "X≥0?"          => Some(Op::Test(TestKind::XGeZero)),
        _ => None,
    }
}
```

**Adaptation notes:**
- **Function rename consideration:** RESEARCH suggested renaming to `builtin_op_by_name` — defer this rename (out of scope for hp41-core FROZEN constraint). The 4→12 extension is the minimal, surgical change cleared by the user.
- Existing call sites at program.rs:73, 389, 506 require **NO changes** — they call `builtin_card_op(label)` and dispatch the returned `Op`. Test dispatch route already works for any Op variant including `Op::Test(TestKind::…)`.
- Existing test `builtin_card_op_resolves_four_names` (program.rs:1495-1510) **continues to pass** unchanged. The new test file `hp41-core/tests/phase25_builtin_card_op.rs` adds 8 mnemonic-resolution tests + drift catch.

**Existing test pattern to mirror** (program.rs:1495-1510):
```rust
#[test]
fn builtin_card_op_resolves_four_names() {
    use crate::ops::program::builtin_card_op;
    use crate::ops::Op;
    assert_eq!(builtin_card_op("WPRGM"), Some(Op::Wprgm));
    assert_eq!(builtin_card_op("RDPRGM"), Some(Op::Rdprgm));
    assert_eq!(builtin_card_op("WDTA"), Some(Op::Wdta));
    assert_eq!(builtin_card_op("RDTA"), Some(Op::Rdta));
    assert_eq!(builtin_card_op("wprgm"), None, "case-sensitive — HP-41 names are uppercase");
    assert_eq!(builtin_card_op("UNKNOWN"), None);
    assert_eq!(builtin_card_op(""), None);
}
```

---

### `docs/hp41cv-functions.json` (NEW, config)

**Analog:** `hp41-cli/src/help_data.rs:11+` (current `HELP_DATA` tuple shape — informs the JSON schema).

**Schema per entry** (LOCKED per D-25.16):
```json
{
  "op_variant": "Pi",
  "display_name": "PI",
  "category": "Math",
  "status": "implemented",
  "phase": "20",
  "key_path": "f-1/x",
  "description": "Push π onto X (3.141592654, 10-digit rounded HP-41 hardware value)",
  "divergences": ["10-digit precision per Phase 20 D-09"]
}
```

**Adaptation notes:**
- **Wave 0 seeding strategy:** Start with `[]` and let `tests/phase25_help_data.rs::test_matrix_has_at_least_130_entries` initially fail. Subsequent waves add entries category-by-category (Stack → Arithmetic → Math → Trig → Registers → Alpha → Programming → Flags → Display → Print → Sound → Catalog → Synthetic → CardReader → Indirect → Conversion → MathPac → StatPac → TimePac → AdvantagePac).
- **`op_variant` is the PascalCase Rust identifier** — `"StoReg"` not `"sto_reg"` and not `"STO"`. This is what the CI parity test (`hp41-core/tests/op_matrix_parity.rs`) matches against.
- **`status: "deferred-v3"`** rows have `phase: null` and a placeholder `op_variant` like `"MAT_PLUS_MATHPAC"` (NOT a real Op variant — the parity test skips them on the JSON→Op direction).
- **`description` ≤ 80 chars** for `render_help_overlay` table width compatibility.

---

### `docs/hp41cv-function-matrix.md` (NEW generated, docs)

**Analog:** Generated output — no in-repo Markdown table at this scale yet. Closest analog: `CLAUDE.md` Quality Gates table.

**Required output structure** (per RESEARCH §"Function Matrix Schema" — `just docs-matrix` writes this):
```markdown
# HP-41CV ROM Function Matrix

> Generated from `docs/hp41cv-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

| Op | Display | Category | Status | Phase | Key Path | Description |
|----|---------|----------|--------|-------|----------|-------------|
| Pi | PI | Math | ✓ v2.x | 20 | `f-1/x` | Push π onto X |
| Add | + | Arithmetic | ✓ v2.x | 1 | `+` | Add: X ← Y + X, drop stack |
…

## v3.x Deferred (Module Pacs)
…
```

**Status rendering rules:** `implemented` → `✓ v2.x`; `deferred-v3` → `⏳ v3.x module`; `na` → `— N/A`.

**Adaptation notes:**
- File is **committed** AND **generated** — `just docs-matrix-check` recipe diffs the regenerated version against the committed copy in CI (Pitfall 8).
- README links to this file for the soft-claim per D-25.17.

---

### `scripts/docs-matrix/` (NEW standalone bin, utility)

**Analog (nested non-workspace crate pattern):** `hp41-gui/src-tauri/Cargo.toml` — already demonstrates the "this crate is NOT in workspace members" pattern that protects the root `Cargo.toml members = ["hp41-core", "hp41-cli"]` invariant.

**Cargo.toml shape** (mirrors the nested-workspace exclusion idiom):
```toml
# scripts/docs-matrix/Cargo.toml
[workspace]
# Empty stanza — this is a standalone crate, NOT part of the root workspace.

[package]
name = "docs-matrix"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**main.rs shape** (≤80 LOC — reads JSON, writes Markdown):
```rust
use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Deserialize)]
struct Entry { /* mirrors HelpEntry — same shape */ }

fn main() {
    let args: Vec<String> = env::args().collect();
    let json_path = &args[1];
    let md_path = &args[2];
    let json = fs::read_to_string(json_path).expect("read JSON");
    let entries: Vec<Entry> = serde_json::from_str(&json).expect("parse JSON");
    let md = render_markdown(&entries);
    fs::write(md_path, md).expect("write MD");
}

fn render_markdown(entries: &[Entry]) -> String { /* table-rendering loop */ }
```

**Adaptation notes:**
- **CRITICAL invariant** (CLAUDE.md): Root `Cargo.toml members` MUST remain `["hp41-core", "hp41-cli"]`. The standalone `[workspace]` stanza in `scripts/docs-matrix/Cargo.toml` excludes it from the parent workspace.
- The `Entry` struct mirrors `HelpEntry` but is **deliberately duplicated** here so the bin doesn't need to depend on `hp41-cli` (which would create a circular dep when help_data.rs is part of the binary). Acceptable single-source-of-truth violation per RESEARCH §"Don't Hand-Roll".

---

### `justfile` — `docs-matrix` + `docs-matrix-check` recipes (config, task runner)

**Analog:** `justfile:66-87` (existing `gui-*` recipe family).

**Existing recipe pattern** (justfile:66-87):
```just
# GUI: install npm dependencies (run once after cloning or after package.json changes)
gui-install:
	cd hp41-gui && npm install

# GUI: launch development window
gui-dev:
	cd hp41-gui && npm run tauri dev

# gui-ci: CI gate — TypeScript type-check, Rust tests, and release build
gui-ci:
	cd hp41-gui && npm install
	cd hp41-gui && npx tsc --noEmit
	cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
	cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
```

**Adaptation for Phase 25** (append to justfile):
```just
# Regenerate the HP-41CV function matrix from canonical JSON.
docs-matrix:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
	    docs/hp41cv-functions.json docs/hp41cv-function-matrix.md

# CI-friendly: regenerate to a temp file and diff against the committed version.
docs-matrix-check:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
	    docs/hp41cv-functions.json /tmp/hp41cv-function-matrix-check.md
	diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md
```

**Adaptation notes:**
- Tab indentation (justfile uses tabs, per existing pattern).
- Use `--manifest-path` rather than `cd` so the recipe is reproducible from any cwd.
- `just docs-matrix-check` is what `ci.yml` calls; `just docs-matrix` is the developer-side regenerate-and-commit recipe (Pitfall 8 splits these).

---

### `CLAUDE.md` — v2.2 settled-architecture block (docs)

**Analog:** `CLAUDE.md` `### v2.1 additions (Keyboard authenticity, Phase 19)` section — existing convention for milestone-summary blocks.

**Reference structure to mirror:**
```markdown
### v2.1 additions (Keyboard authenticity, Phase 19)

- **Authentic 5×8 layout**: …
- **Three-label `KeyDef`**: …
- **One-shot SHIFT is frontend-only**: …
- [bullet form, 80-140 chars per bullet]
```

**Adaptation for Phase 25 — append `### v2.2 additions` section with bullets for:**
- v2.2 milestone scope (Phases 20-24 ROM ops + Phase 25 CLI wiring + Phase 25 docs)
- f-prefix one-shot CLI/GUI parity (D-25.1, D-25.6) — references hp41-cli/src/app.rs::App::shift_armed
- Hybrid PendingInput struct-variants (D-25.11) — references the FlagPrompt/RegisterPrompt collapse
- JSON-canonical data flow (D-25.16) — `docs/hp41cv-functions.json` → include_str! → help_data.rs → CI parity
- `builtin_card_op` 4→12 extension exception to "hp41-core FROZEN" rule (cleared per Discussion log)
- README soft-claim, hard-claim deferred to Phase 27 (D-25.17)

**Adaptation notes:**
- Follow existing 80-char-wide bullet style.
- Cross-link to phase docs: `Phase 25 GSD plan` and `docs/hp41cv-function-matrix.md`.
- This section is the SINGLE source-of-truth for Phase 26 (GUI parity) downstream agents.

---

### `README.md` — soft "feature-complete HP-41CV" claim (docs)

**Analog:** Existing project README structure.

**Adaptation per D-25.17 (verbatim wording template):**
> "Implements the full HP-41CV ROM built-in function set (~130 ops) with documented divergences. See [HP-41CV function matrix](docs/hp41cv-function-matrix.md) for status per op and known hardware divergences."

**Adaptation notes:**
- Soft claim only — keep the "with documented divergences" caveat. Hard claim deferred to Phase 27 (FN-DOC-03).
- Divergences enumerated in matrix.md per-row, NOT in README body. README links out.
- Smoke test `grep -q "feature-complete HP-41CV" README.md && grep -q "hp41cv-function-matrix.md" README.md` per RESEARCH Validation Architecture.

---

### `hp41-cli/tests/phase25_*.rs` test suite (test scaffold)

**Analog:** `hp41-cli/tests/card_io_tests.rs:1-36` (integration-test scaffold).

**Imports + scaffold pattern** (card_io_tests.rs:1-36):
```rust
//! End-to-end Card Reader integration tests for hp41-cli.
//! Spec: docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md

#![allow(clippy::unwrap_used)]

use std::fs;

use hp41_cli::cards::drain_pending_card_op;
use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::{dispatch, Op};
use hp41_core::run_program;
use hp41_core::state::CalcState;

#[test]
fn roundtrip_program_via_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = make_state_with_simple_program();
    // …
}
```

**Adaptation per file:**

- **`hp41-cli/tests/phase25_keyboard.rs`** — exercises `key_to_op` + `shifted_key_to_op` for the 4 hardware-anchored conditional tests:
  ```rust
  #[test]
  fn f_minus_dispatches_x_eq_y() {
      let mut state = CalcState::new();
      state.stack.y = HpNum::from(5);
      state.stack.x = HpNum::from(5);
      // Construct KeyEvent for '-' with shift_armed=true via shifted_key_to_op
      let op = hp41_cli::keys::shifted_key_to_op(/* key='-' */, /* &app */).unwrap();
      assert!(matches!(op, Op::Test(TestKind::XEqY)));
  }
  ```

- **`hp41-cli/tests/phase25_pending_input.rs`** — exercises the 6 new variants' state transitions (IND toggle, digit accumulation, Esc cancel). Each test constructs an `App`, sets `pending_input = Some(…)`, simulates a key event via `handle_pending_input`, asserts the resulting state.

- **`hp41-cli/tests/phase25_xeq_by_name.rs`** — 8 tests, one per non-keyboard conditional test, asserting `xeq_by_name_local_resolve("X<>Y?")` returns the right `Op::Test(TestKind::*)` variant.

- **`hp41-cli/tests/phase25_help_data.rs`** — smoke tests:
  ```rust
  #[test]
  fn help_entries_loads_at_runtime() {
      let entries = hp41_cli::help_data::help_entries();
      assert!(!entries.is_empty(), "JSON load returned 0 entries — check FUNCTIONS_JSON");
  }

  #[test]
  fn help_entries_count_meets_130_target() {
      let entries = hp41_cli::help_data::help_entries();
      assert!(entries.len() >= 130,
          "function matrix should list ≥130 HP-41CV ROM ops; got {}", entries.len());
  }
  ```

**Adaptation notes:**
- `#![allow(clippy::unwrap_used)]` at module head — test files exempt from the global lint per CLAUDE.md.
- Avoid raw `KeyEvent` construction — use a helper `fn key(c: char) -> KeyEvent` to wrap `KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())` per existing test pattern.

---

### `hp41-core/tests/phase25_builtin_card_op.rs` + `op_matrix_parity.rs` (test)

**Analog:** `hp41-core/src/ops/program.rs:1495-1510` (existing builtin_card_op test).

**Test 1: `phase25_builtin_card_op.rs`** — mirrors the existing 4-names test, expanded to 12 mnemonics:
```rust
#![allow(clippy::unwrap_used)]

use hp41_core::ops::{program::builtin_card_op, Op, TestKind};

#[test]
fn resolves_8_conditional_test_mnemonics() {
    assert_eq!(builtin_card_op("X<>Y?"),  Some(Op::Test(TestKind::XNeY)));
    assert_eq!(builtin_card_op("X≠Y?"),   Some(Op::Test(TestKind::XNeY)));
    assert_eq!(builtin_card_op("X<Y?"),   Some(Op::Test(TestKind::XLtY)));
    // … 6 more …
}

#[test]
fn preserves_4_card_reader_names() {
    // Regression: extension must not break existing names.
    assert_eq!(builtin_card_op("WPRGM"), Some(Op::Wprgm));
    // … 3 more …
}

#[test]
fn unknown_name_returns_none() {
    assert_eq!(builtin_card_op("foobar"), None);
}
```

**Test 2: `op_matrix_parity.rs`** — bidirectional drift catch (per D-25.15):

**Analog:** RESEARCH §"CI parity test" sketch (lines 729-835 of 25-RESEARCH.md).

Adapt by hand-curating `ALL_OP_VARIANT_NAMES: &[&str]` enumeration (130 entries) and asserting:
1. Every entry in `ALL_OP_VARIANT_NAMES` (minus INTERNAL_OP_VARIANTS skip-list) has a matching JSON entry by `op_variant` name.
2. Every JSON entry with `status: "implemented"` has a matching name in `ALL_OP_VARIANT_NAMES`.
3. `help_entries().len() >= 130`.

**Adaptation notes:**
- File location: choose `hp41-core/tests/` (closer to Op enum) OR `hp41-cli/tests/` (closer to JSON consumer). RESEARCH recommends `hp41-cli/tests/` because it can `use hp41_cli::help_data::help_entries`. Either works; planner decides.
- The hand-curated `ALL_OP_VARIANT_NAMES` list is a maintenance burden — every future Op variant Phase 26+ must be appended. The constant test `test_op_inventory_count_matches_enum` (assert .len() == 130) catches forgotten additions.

---

## Shared Patterns

### One-shot prefix state machine (cross-tier parity reference — D-25.6)
**Source:** `hp41-gui/src/App.tsx:111-119, 161-206`
**Apply to:** `hp41-cli/src/app.rs` (`App.shift_armed`, `handle_key`, `render_status`)

```typescript
const [shiftActive, setShiftActive] = useState(false);

// Resolution priority: shift > alpha-override > primary
if (key.id === 'shift') { setShiftActive(prev => !prev); return; }
if (alphaOn && key.alphaChar)       { /* alpha override */ }
else if (shiftActive && key.shifted) { effectiveId = key.shifted.id; consumesShift = true; }
else                                  { effectiveId = key.id; }

// Always consume on dispatch (one-shot lifetime, regardless of success/failure)
finally { if (consumesShift) setShiftActive(false); }
```

**CLI mirror invariants:**
- ALPHA mode OVERRIDES f-prefix in v2.2 (D-25.5) — both tiers defer this to v3.x identically.
- `shift_armed` is frontend-only — NEVER on `CalcState`, NEVER crosses IPC.
- Always clear in finally / end-of-branch (Pitfall 5) regardless of whether key was recognized.
- Esc cancels armed prefix (both tiers).

### Generic 2-digit accumulator scaffold
**Source:** `hp41-cli/src/app.rs:913-949` (`handle_reg_modal`)
**Apply to:** new `handle_reg_modal_with_ind` AND new 3-digit `handle_del_modal` (DelCount) AND new 1-digit `handle_tone_modal` (TonePrompt).

```rust
match key.code {
    KeyCode::Char(c) if c.is_ascii_digit() => {
        let mut new_acc = acc;
        new_acc.push(c);
        if new_acc.len() == 2 {       // ← change to 1 (Tone) or 3 (Del)
            let reg: u8 = new_acc.parse().expect("two ASCII digits ≤ 99");
            self.call_dispatch(op_fn(reg));
            self.pending_input = None;
        } else {
            self.pending_input = Some(pending_fn(new_acc));
        }
    }
    KeyCode::Backspace => self.pending_input = Some(pending_fn(String::new())),
    KeyCode::Esc => self.pending_input = None,
    _ => self.pending_input = Some(pending_fn(acc)),  // restore modal silently
}
```

### Text-input modal scaffold
**Source:** `hp41-cli/src/app.rs:760-787` (`AssignLabel` arm — Enter to dispatch, Backspace pop, Esc cancel)
**Apply to:** new `XeqByName(String)` AND new `ClpLabel(String)` arms.

Common shape:
```rust
match key.code {
    KeyCode::Esc => self.pending_input = None,
    KeyCode::Enter => {
        if !acc.is_empty() {
            self.call_dispatch(/* Op derived from acc */);
        }
        self.pending_input = None;
    }
    KeyCode::Backspace => { /* pop last char, store back */ }
    KeyCode::Char(ch) => { /* push char with length cap */ }
    _ => { /* restore modal silently */ }
}
```

**Length cap rule:** 7 chars for LBL names (HP-41 hardware limit), 24 chars for XEQ name (ALPHA register width) — per RESEARCH §"Security Domain" V5.

### Exhaustive match discipline (FN-CLI-04 compile-time guarantee)
**Source:** `hp41-cli/src/ui.rs:238-273` (current `pending_prompt()`)
**Apply to:** All match sites on `PendingInput` — `pending_prompt`, `handle_pending_input`.

**Hard rule:** No `_ =>` catch-all. No `unreachable!()`. Adding a new variant forces the compiler to flag every match site at build time. This is THE runtime guarantee that no PendingInput slips through silently.

### Compile-time JSON embedding via `include_str!` + `OnceLock`
**Source:** `hp41-cli/src/programs.rs:7-24`
**Apply to:** `hp41-cli/src/help_data.rs` (new shape).

```rust
use std::sync::OnceLock;
const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");
static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();
pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| serde_json::from_str(FUNCTIONS_JSON).expect("malformed JSON"))
}
```

**Why stdlib over `once_cell` external crate:** MSRV 1.88 — `OnceLock` is in std since 1.70. Project precedent (`programs.rs`) sets the rule. No new deps.

### `pending_input` route ABOVE modal-opening interceptors
**Source:** `hp41-cli/src/app.rs:225-231` + CLAUDE.md "Settled Architecture Decisions"
**Apply to:** All new modal openers in Phase 25 (Flag/Reg/Tone/Del/CLP/XeqByName).

```rust
// Phase 5: route to pending_input handler if modal is active — MUST come before
// the modal-opening interceptors below (CR-02). If any modal is active, 'S', 'R',
// and Ctrl+A must be handled by the active modal, not silently replaced.
if self.pending_input.is_some() {
    self.handle_pending_input(key);
    return;
}
```

**Hard rule (per CLAUDE.md):** Phase 25's new `shift_armed` arming logic ALSO goes AFTER this route (Pitfall 4 — modals already have a defined Esc/Enter/digit interface; f-prefix arming inside an active modal is meaningless).

### Crossterm KeyEventKind::Release filter
**Source:** `hp41-cli/src/app.rs:181-185` + CLAUDE.md Windows trap
**Apply to:** No change required — Phase 25 inherits the existing first-check filter. Critically: the `shift_armed` arming logic MUST run after the Release filter so Windows double-events don't arm+consume in one cycle.

### Settled-architecture milestone summary
**Source:** `CLAUDE.md` `### v2.1 additions (Keyboard authenticity, Phase 19)` block
**Apply to:** New `### v2.2 additions` block in CLAUDE.md.

Bullet-style summary of 6-12 settled invariants per milestone. ~80-140 chars per bullet. Cross-link to phase docs.

### Nested non-workspace crate isolation
**Source:** `hp41-gui/src-tauri/Cargo.toml` (existing pattern keeping `hp41-gui` out of root members)
**Apply to:** `scripts/docs-matrix/Cargo.toml`.

```toml
[workspace]
# Empty stanza — standalone crate, NOT part of root workspace.
```

**Hard rule (CLAUDE.md):** Root `Cargo.toml members = ["hp41-core", "hp41-cli"]`. Never add `scripts/docs-matrix` here.

## No Analog Found

All target files have a strong in-repo analog. The closest "no-analog" candidate was the generated Markdown function matrix (`docs/hp41cv-function-matrix.md`), but the long-form table style of `CLAUDE.md` Quality Gates serves as a sufficient structural reference.

## Metadata

**Analog search scope:**
- `hp41-cli/src/` (app.rs, keys.rs, ui.rs, help_data.rs, programs.rs, cards.rs)
- `hp41-cli/tests/` (card_io_tests.rs)
- `hp41-cli/Cargo.toml`
- `hp41-core/src/ops/program.rs` (builtin_card_op + tests)
- `hp41-gui/src/App.tsx` + `Keyboard.tsx` (cross-tier parity reference for D-25.6)
- `hp41-gui/src-tauri/Cargo.toml` (nested workspace exclusion idiom)
- `justfile`, `CLAUDE.md`, `README.md`

**Files scanned:** ~15 codebase files + 2 phase docs (25-CONTEXT.md, 25-RESEARCH.md)

**Pattern extraction date:** 2026-05-14

**Key invariants surfaced for the planner:**
1. **CLI ↔ GUI parity (D-25.6) is exact** — GUI v2.1's `shiftActive` is the de-facto reference design. Mirror it bullet-for-bullet.
2. **`pending_input` route is sacred** — every new modal opener (incl. shift_armed arming) goes AFTER the pending_input route per CR-02 + Pitfall 4.
3. **ALPHA mode overrides shift_armed (D-25.5)** — must guard the arming logic with `if !self.state.alpha_mode` OR place it AFTER the ALPHA-mode routing block (app.rs:299-302).
4. **`builtin_card_op` 4→12 extension is the surgical exception** to "hp41-core FROZEN" — cleared per Discussion log; no new Ops, no new state, no new errors, just an enlarged match arm.
5. **JSON parse `.expect()` is intentional** per D-25.17 — malformed canonical data is a hard build-blocker by design. Smoke test catches it at CI.
6. **`scripts/docs-matrix` is standalone** — must NOT enter root workspace `members` array (CLAUDE.md invariant).
