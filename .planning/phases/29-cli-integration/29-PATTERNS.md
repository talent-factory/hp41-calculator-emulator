# Phase 29: CLI Integration — Pattern Map

**Mapped:** 2026-05-17
**Files analyzed:** 13 (5 new test files / 1 new JSON / 7 modified source files / 1 modified test file)
**Analogs found:** 13 / 13 (every Phase 29 file has at least one exact-shape analog in v2.2)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `docs/hp41-math1-functions.json` | config / data | static-data | `docs/hp41cv-functions.json` | exact (same schema +1 optional `xrom` object per C-28.3) |
| `hp41-cli/tests/phase29_help_data_math1.rs` | test (smoke) | static-data | `hp41-cli/tests/phase25_help_data.rs` | exact (mirror, swap accessor) |
| `hp41-cli/tests/phase29_modal_flow.rs` | test (integration) | event-driven | `hp41-cli/tests/phase25_xeq_by_name.rs` | exact (App harness + key-event injection) |
| `hp41-cli/tests/phase29_pending_prompt_modal.rs` | test (unit) | transform | `hp41-cli/tests/phase25_pending_input.rs::pending_prompt_exhaustive` | role-match |
| `hp41-cli/tests/phase29_key_ref_includes_math1.rs` | test (unit) | static-data | `hp41-cli/tests/phase25_help_data.rs` + `key_coverage.rs` | role-match |
| `hp41-cli/src/keys.rs::xeq_by_name_local_resolve` | resolver fn | transform | `hp41-core/src/ops/program.rs::op_xeq` lines 67–87 | exact (3rd call site of same pattern) |
| `hp41-cli/src/help_data.rs` (second OnceLock) | config-loader | static-data | the SAME file's first `OnceLock` (lines 60–77) | exact (verbatim mirror, swap path & panic message) |
| `hp41-cli/src/ui.rs::pending_prompt` | renderer | transform | the SAME function (lines 248–331) | self-extend (signature widen + 1 new arm) |
| `hp41-cli/src/app.rs::PendingInput::XeqByName` | enum variant | event-driven | `PendingInput::FlagPrompt {kind,ind,acc}` and `RegisterPrompt {op,ind,acc}` D-25.11 hybrid carrier-variant pattern | exact (struct-variant with allowed-values enum field) |
| `hp41-cli/src/app.rs::handle_key` (R/S + Esc interception) | event handler | event-driven | the SAME function's `S`/`R`/`F` modal openers (lines 339–388) + shift_armed Esc (438–451) | exact (same insertion shape, different guard predicate) |
| `hp41-cli/src/app.rs::call_dispatch` (post-dispatch auto-open) | event handler | event-driven | the SAME function (lines 1640–1645) | self-extend (additive trailing call) |
| `hp41-cli/tests/function_matrix_parity.rs` extension | test (parity) | transform | the SAME file's 4 existing tests (lines 200–267) | self-extend (parallel pool + 3 new tests) |
| `hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` | test (integration) | transform | the SAME function (lines 369–415) | self-extend (additive Math1 table) |
| `hp41-core/src/ops/math1/modal.rs::requires_alpha_label` | method (new) | transform | `ModalProgram::current_prompt()` lines 41–63 in same file | exact (same per-variant-match shape) |
| `hp41-core/src/ops/math1/mod.rs::submit_modal`, `cancel_modal`, `submit_modal_with_label` | service / dispatch | transform | `op_xeq` lines 67–87 in `hp41-core/src/ops/program.rs` (interactive-vs-running branch) + `op_matrix_workflow` lines 323–331 in `matrix.rs` (modal-open shape) | role-match (combine both shapes) |
| `hp41-core/src/ops/math1/{solve,integ,difeq}.rs::op_solve/op_integ/op_difeq` (Open Q1) | op (modify) | transform | `op_xeq` lines 67–87 in `hp41-core/src/ops/program.rs` | exact (literal `if !state.is_running { open modal } else { Err(InvalidOp) }` branching) |

---

## Pattern Assignments

### `docs/hp41-math1-functions.json` (config / static-data)

**Analog:** `docs/hp41cv-functions.json` (130 entries, 1395 lines)

**Entry shape pattern** (lines 2–10 of analog):

```json
{
    "op_variant": "Add",
    "display_name": "+",
    "category": "Arithmetic",
    "status": "implemented",
    "phase": "1",
    "key_path": "+",
    "description": "Add: X <- Y + X, drop stack"
}
```

**Phase 29 entry shape** (schema +1 optional `xrom` block per C-28.3):

```json
{
    "op_variant": "Sinh",
    "display_name": "SINH",
    "category": "Math1 Hyperbolics",
    "status": "implemented",
    "phase": "28",
    "key_path": "XEQ \"SINH\"",
    "description": "Hyperbolic sine: X <- sinh(X)",
    "xrom": { "module": "Math 1", "module_id": 7, "function_id": 1 }
}
```

**Schema struct** (`hp41-cli/src/help_data.rs:45–57`):

```rust
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HelpEntry {
    pub op_variant: String,
    pub display_name: String,
    pub category: String,
    pub status: String,
    pub phase: Option<String>,
    pub key_path: Option<String>,
    pub description: String,
    #[serde(default)]
    pub divergences: Vec<String>,
}
```

**Schema extension** — append `pub xrom: Option<XromEntry>` with `#[serde(default)]` so v2.2 entries (no `xrom` key) parse unchanged. `XromEntry { module, module_id, function_id }` — see RESEARCH §3.2.

**Authoring source:** `MATH_1.ops` const at `hp41-core/src/ops/math1/xrom.rs:43–118` is the canonical pull-list (52 entries → ~47 unique-`op_variant` rows after deduplicating the 5 ASCII aliases for `C×`, `C÷`, `Z↑N`, `Z↑1/N`, `E↑Z`, `A↑Z`, `Z↑W`).

**Category convention** — recommended per-program (research §3.2 + Claude's discretion):
`Math1 Hyperbolics` (6), `Math1 Complex Arithmetic` (5), `Math1 Complex Functions` (12), `Math1 Polynomial` (2), `Math1 Matrix` (8), `Math1 Integration` (1), `Math1 Root Solver` (2), `Math1 Differential Eq` (1), `Math1 Fourier` (1), `Math1 Triangle Solvers` (5), `Math1 Coordinate Transform` (2).

---

### `hp41-cli/tests/phase29_help_data_math1.rs` (test — smoke)

**Analog:** `hp41-cli/tests/phase25_help_data.rs` (entire file, 131 lines)

**Imports + use pattern** (lines 1–13 of analog):

```rust
//! Phase 25 Plan 04 Task 1 smoke tests — `docs/hp41cv-functions.json` is the
//! canonical data source for `hp41-cli/src/help_data.rs` via include_str! +
//! OnceLock per D-25.16 / D-25.17.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries, help_overlay_rows};
```

**Phase 29 mirror:** swap `help_entries` → `help_entries_math1`; copy the same 7 test functions verbatim; adjust the count assertion from `>= 130` to `>= 47`; add 3 Math1-specific assertions for the `xrom` field.

**Smoke test pattern** (lines 14–22 of analog — the "hard-build-blocker catches malformed JSON" assertion):

```rust
#[test]
fn help_entries_loads_at_runtime() {
    let entries = help_entries();
    assert!(
        !entries.is_empty(),
        "help_entries() must return a non-empty slice — \
         docs/hp41cv-functions.json may be empty or malformed (D-25.17)"
    );
}
```

**Count-target pattern** (lines 24–36 of analog — D-25.16 / Pitfall 7 belt-and-braces):

```rust
#[test]
fn help_entries_count_meets_130_target() {
    let entries = help_entries();
    assert!(
        entries.len() >= 130,
        "help_entries().len() = {} — must be >= 130 per D-25.16 (Pitfall 7 \
         CI guard against empty/short JSON commits)",
        entries.len()
    );
}
```

**De-duplication pattern** (lines 52–66 of analog — copy verbatim, change file name in message):

```rust
#[test]
fn help_entries_has_no_duplicate_op_variants() {
    let entries = help_entries();
    let mut seen: HashSet<&str> = HashSet::with_capacity(entries.len());
    for entry in entries {
        assert!(
            seen.insert(entry.op_variant.as_str()),
            "duplicate op_variant in docs/hp41cv-functions.json: {}",
            entry.op_variant
        );
    }
}
```

**Status-enum pattern** (lines 88–101 — `matches!` macro idiom):

```rust
assert!(
    matches!(entry.status.as_str(), "implemented" | "deferred-v3" | "na"),
    "entry '{}' has invalid status '{}' — must be \
     implemented | deferred-v3 | na",
    entry.op_variant,
    entry.status
);
```

**Math1-specific additions** (per RESEARCH §5 inventory):
- `math1_help_entries_all_xrom_module_id_is_7` — assert every entry's `xrom.module_id == 7`
- `math1_help_entries_xrom_function_ids_are_dense` — `function_ids` form `1..=N` no gaps
- `math1_help_entries_categories_prefix_with_math1` — every category begins with `"Math1 "`
- `math1_help_entries_all_key_path_is_xeq_form` — every entry has `key_path == Some("XEQ \"<MNEMONIC>\"")` per D-28.6

---

### `hp41-cli/tests/phase29_modal_flow.rs` (test — integration)

**Analog:** `hp41-cli/tests/phase25_xeq_by_name.rs` (entire file, harness + 14 tests)

**Test scaffolding pattern** (lines 32–57 of analog):

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use hp41_cli::app::{App, PendingInput};
use hp41_cli::keys::xeq_by_name_local_resolve;
use hp41_core::ops::{Op, TestKind};
use hp41_core::state::CalcState;

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
```

**Phase 29 mirror:** copy `key`, `raw_key`, `make_app` verbatim (rename `state_path` filename to `phase29-modal-flow-test-state.json`).

**Drive-modal-and-assert pattern** (lines 64–87 of analog — open modal, type chars, press Enter, inspect message):

```rust
fn type_name_and_enter(name: &str) -> (App, tempfile::TempDir, Option<String>) {
    let (mut app, tmp) = make_app();
    app.pending_input = Some(PendingInput::XeqByName(String::new()));
    for c in name.chars() {
        app.handle_key(key(c));
    }
    match &app.pending_input {
        Some(PendingInput::XeqByName(acc)) => {
            assert_eq!(acc, name, "accumulator must hold the full mnemonic before Enter");
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
```

**Phase 29 modal-state-assertion shape** (per RESEARCH §5 inventory):
- After dispatch `Op::MatrixWorkflow`: assert `state.modal_program == Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt))` AND `state.modal_prompt == Some("ORDER=?".to_string())`.
- After type "3" + F5 (R/S): assert `modal_program == Matrix(ElementPrompt(0,0))` AND `modal_prompt == Some("A1,1=?")`.
- After dispatch `Op::Solve` + `call_dispatch` returns: assert `pending_input == Some(PendingInput::XeqByName { mode: CollectForModal, .. })`.

**Esc-cancellation test pattern** — new; no v2.2 analog (Phase 29 introduces top-level Esc handler in `handle_key`). Use the same harness; assert `state.modal_program.is_none() && state.modal_prompt.is_none() && app.message == Some("Cancelled")`.

---

### `hp41-cli/tests/phase29_pending_prompt_modal.rs` (test — unit)

**Analog:** `hp41-cli/tests/phase25_pending_input.rs::pending_prompt_exhaustive` lines 101–160

**Pattern** (lines 101–115 of analog):

```rust
#[test]
fn pending_prompt_exhaustive() {
    use hp41_cli::ui::pending_prompt;

    let p = PendingInput::FlagPrompt {
        kind: FlagPromptKind::SetFlag,
        ind: false,
        acc: String::new(),
    };
    assert!(
        pending_prompt(&p).starts_with("SF "),
        "got: {:?}",
        pending_prompt(&p)
    );
}
```

**Phase 29 adaptation** — after the signature widens to `pending_prompt(pending: Option<&PendingInput>, modal_prompt: Option<&str>) -> String` (recommended Option A per RESEARCH §3.3), the test shape becomes:

```rust
#[test]
fn pending_prompt_renders_modal_prompt_when_no_pending() {
    assert_eq!(
        pending_prompt(None, Some("ORDER=?")),
        "ORDER=?",
    );
}

#[test]
fn pending_prompt_renders_pending_when_no_modal() {
    let p = PendingInput::XeqByName { acc: "F".to_string(), mode: XeqByNameMode::Normal };
    assert_eq!(pending_prompt(Some(&p), None), "XEQ \"F\"_");
}

#[test]
fn pending_prompt_modal_wins_when_both_active() {
    // Discretionary precedence test (Claude's discretion §7.3 of RESEARCH).
    let p = PendingInput::XeqByName { acc: "F".to_string(), mode: XeqByNameMode::CollectForModal };
    let rendered = pending_prompt(Some(&p), Some("FUNCTION NAME?"));
    assert!(rendered.contains("FUNCTION NAME"), "modal wins: got {rendered}");
}
```

---

### `hp41-cli/tests/phase29_key_ref_includes_math1.rs` (test — unit)

**Analog:** `hp41-cli/tests/phase25_help_data.rs::help_overlay_rows_contain_category_headers` lines 103–130 (and `hp41-cli/src/keys.rs:390–405` for the function under test).

**Test shape** — single-purpose assertion that `key_ref_entries()` post-migration to `help_entries_all()` produces at least one Math1 row:

```rust
#![allow(clippy::unwrap_used)]

use hp41_cli::keys::key_ref_entries;

#[test]
fn key_ref_entries_includes_math1_sinh() {
    let entries = key_ref_entries();
    let found = entries.iter().any(|(key_path, display)| {
        key_path == "XEQ \"SINH\"" && display == "SINH"
    });
    assert!(found, "key_ref_entries() must include Math Pac I SINH row \
                     after migrating to help_entries_all() (D-29.2)");
}

#[test]
fn key_ref_entries_includes_math1_matrix() {
    let entries = key_ref_entries();
    let found = entries.iter().any(|(key_path, display)| {
        key_path == "XEQ \"MATRIX\"" && display == "MATRIX"
    });
    assert!(found, "key_ref_entries() must include MATRIX row (SC-4)");
}
```

---

### `hp41-cli/src/keys.rs::xeq_by_name_local_resolve` (resolver fn — modify)

**Analog:** `hp41-core/src/ops/program.rs::op_xeq` lines 67–87 — already calls `xrom_resolve` as the LAST step before `InvalidOp`:

```rust
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        // Built-in XEQ-by-name fallback for Card Reader ops + conditional tests.
        if let Some(card_op) = builtin_card_op(label) {
            return crate::ops::dispatch(state, card_op);
        }
        // Phase 28 (v3.0) XROM resolver — fires LAST (C-28.4 / Pitfall 1).
        // Checked after builtin_card_op so built-in names always win.
        // xeq_by_name_local_resolve (hp41-cli) is the third call site — Phase 29 / CLI-01.
        if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(label, state.xrom_modules) {
            return crate::ops::dispatch(state, xrom_op);
        }
        return Err(HpError::InvalidOp);
    }
    Err(HpError::InvalidOp)
}
```

**Current shape to modify** (`hp41-cli/src/keys.rs:347–370`):

```rust
pub fn xeq_by_name_local_resolve(name: &str) -> Option<Op> {
    match name {
        "X<>Y?" | "X\u{2260}Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        // ... 7 more conditional-test arms ...
        // The four v2.1 card-reader names + everything else: defer to
        // `hp41_core::ops::program::builtin_card_op` via the `Op::Xeq`
        // fallback chain in the modal Enter-arm.
        _ => None,  // ← Phase 29's extension point
    }
}
```

**Phase 29 change** — widen signature, replace `_ => None` with `xrom_resolve` fallback:

```rust
pub fn xeq_by_name_local_resolve(name: &str, xrom_modules: u8) -> Option<Op> {
    match name {
        // 8 conditional-test mnemonics unchanged ...
        _ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules),
    }
}
```

**Caller update** — exactly ONE production call site at `hp41-cli/src/app.rs:1423`. Current:

```rust
if let Some(op) = keys::xeq_by_name_local_resolve(&acc) {
    self.call_dispatch(op);
} else {
    self.call_dispatch(Op::Xeq(acc));
}
```

After:

```rust
if let Some(op) = keys::xeq_by_name_local_resolve(&acc, self.state.xrom_modules) {
    self.call_dispatch(op);
} else {
    self.call_dispatch(Op::Xeq(acc));
}
```

---

### `hp41-cli/src/help_data.rs` (config-loader — extend)

**Analog:** the SAME file's existing `OnceLock<Vec<HelpEntry>>` block at lines 59–77.

**Pattern to mirror** (verbatim — lines 59–77):

```rust
/// Compile-time-embedded canonical data file. The relative path is from this
/// source file (`hp41-cli/src/help_data.rs`) to `docs/hp41cv-functions.json`
/// at the repo root.
const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");

static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41cv-functions.json` is
/// malformed — this is the **intentional** D-25.17 hard-build-blocker
/// behavior. The OnceLock init uses `.expect("hp41cv-functions.json is
/// malformed — fix the JSON")`; subsequent calls return the cached slice.
pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(FUNCTIONS_JSON)
            .expect("hp41cv-functions.json is malformed — fix the JSON")
    })
}
```

**Phase 29 mirror** — append after the existing block:

```rust
const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");
static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}

pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries().iter().chain(help_entries_math1().iter())
}
```

**Critical:** distinct panic-message wording per D-25.17 (so the smoke test can distinguish which file failed). `hp41cv-...` vs `hp41-math1-...` — different filenames in the literal.

**Schema extension** — add to `HelpEntry` struct at line 47:

```rust
#[serde(default)]
pub xrom: Option<XromEntry>,
```

`#[serde(default)]` means v2.2 entries (no `xrom` key) parse unchanged.

---

### `hp41-cli/src/ui.rs::pending_prompt` (renderer — modify signature)

**Analog:** the SAME function lines 248–331 (the existing 18-arm exhaustive match).

**Current signature + call site** (lines 225–246 + 258):

```rust
fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    let base: String = if let Some(ref pending) = app.pending_input {
        pending_prompt(pending)
    } else if app.state.alpha_mode {
        "ALPHA mode — Enter or A to exit".to_string()
    } else {
        app.message.as_deref().unwrap_or("Ready").to_string()
    };
    // ... shift_armed prefix logic ...
    frame.render_widget(Paragraph::new(text), area);
}

pub fn pending_prompt(pending: &crate::app::PendingInput) -> String {
    // exhaustive match over 18 arms
}
```

**Phase 29 widening** (Option A — recommended per RESEARCH §3.3 + D-29.3):

```rust
pub fn pending_prompt(
    pending: Option<&crate::app::PendingInput>,
    modal_prompt: Option<&str>,
) -> String {
    // If modal is open (and no pending_input competes), render modal_prompt.
    if pending.is_none() {
        if let Some(prompt) = modal_prompt {
            return prompt.to_string();
        }
    }
    // Discretionary precedence (§7.3): when both Some, prefer modal_prompt for
    // CollectForModal mode; fall through to pending_prompt otherwise.
    if let (Some(p), Some(prompt)) = (pending, modal_prompt) {
        if matches!(p, crate::app::PendingInput::XeqByName { mode: crate::app::XeqByNameMode::CollectForModal, .. }) {
            return prompt.to_string();
        }
    }
    // Existing exhaustive match over PendingInput variants.
    match pending {
        None => String::new(),
        Some(p) => match p {
            PendingInput::StoRegister(acc) => format!("STO [{acc:_<2}]"),
            // ... rest unchanged ...
        }
    }
}
```

**Existing XeqByName arm** (line 329) — replace single arm with two:

```rust
PendingInput::XeqByName { acc, mode: XeqByNameMode::Normal } => format!("XEQ \"{acc}\"_"),
PendingInput::XeqByName { acc, mode: XeqByNameMode::CollectForModal } => format!("NAME: {acc}_"),
```

**Render-site update** (lines 225–234):

```rust
let base: String = if app.pending_input.is_some() || app.state.modal_prompt.is_some() {
    pending_prompt(app.pending_input.as_ref(), app.state.modal_prompt.as_deref())
} else if app.state.alpha_mode {
    "ALPHA mode — Enter or A to exit".to_string()
} else {
    app.message.as_deref().unwrap_or("Ready").to_string()
};
```

**FN-CLI-04 invariant:** no `_ =>` catch-all — every match arm explicit. Compile-time-exhaustive over PendingInput AND XeqByNameMode.

---

### `hp41-cli/src/app.rs::PendingInput::XeqByName` (enum variant — modify)

**Analog:** the SAME file's `PendingInput::FlagPrompt { kind, ind, acc }` and `RegisterPrompt { op, ind, acc }` D-25.11 hybrid carrier-variant pattern.

**Phase 25 pattern** (the proven shape — search `PendingInput::FlagPrompt` in `app.rs`):

```rust
FlagPrompt {
    kind: FlagPromptKind,
    ind: bool,
    acc: String,
},
RegisterPrompt {
    op: RegisterOpKind,
    ind: bool,
    acc: String,
},
```

Where `FlagPromptKind` and `RegisterOpKind` are CLI-local enums defined in `hp41-cli/src/keys.rs` that wrap (not duplicate) the core `FlagTestKind` / `StoArithKind` enums (D-25.13).

**Current shape to modify** (`hp41-cli/src/app.rs:113`):

```rust
XeqByName(String),
```

**Phase 29 change** (per D-29.8):

```rust
pub enum XeqByNameMode {
    /// Existing behavior — Enter resolves to Op via xeq_by_name_local_resolve.
    Normal,
    /// New — Enter calls submit_modal_with_label() instead.
    CollectForModal,
}

// In PendingInput enum:
XeqByName { acc: String, mode: XeqByNameMode },
```

**`handle_xeq_by_name` signature update** (line 1407) — add `mode` parameter, branch on it in Enter arm:

```rust
fn handle_xeq_by_name(&mut self, key: KeyEvent, acc: String, mode: XeqByNameMode) {
    match key.code {
        KeyCode::Esc => { self.pending_input = None; }
        KeyCode::Enter => {
            if !acc.is_empty() {
                match mode {
                    XeqByNameMode::Normal => {
                        if let Some(op) = keys::xeq_by_name_local_resolve(&acc, self.state.xrom_modules) {
                            self.call_dispatch(op);
                        } else {
                            self.call_dispatch(Op::Xeq(acc));
                        }
                    }
                    XeqByNameMode::CollectForModal => {
                        match hp41_core::ops::math1::submit_modal_with_label(&mut self.state, &acc) {
                            Ok(()) => self.message = None,
                            Err(e) => self.message = Some(format!("{e}")),
                        }
                    }
                }
            }
            self.pending_input = None;
        }
        KeyCode::Backspace => {
            let mut new_acc = acc;
            new_acc.pop();
            self.pending_input = Some(PendingInput::XeqByName { acc: new_acc, mode });
        }
        KeyCode::Char(ch) => {
            let mut new_acc = acc;
            if new_acc.len() < Self::XEQ_NAME_CAP { new_acc.push(ch); }
            self.pending_input = Some(PendingInput::XeqByName { acc: new_acc, mode });
        }
        _ => {
            self.pending_input = Some(PendingInput::XeqByName { acc, mode });
        }
    }
}
```

**20 match-arm sites total** — see RESEARCH §4 for full enumerated list (10 production + 10 test sites). Compile-time exhaustive match means any missed site fails the build.

---

### `hp41-cli/src/app.rs::handle_key` — R/S + Esc interception + post-dispatch hook

**Analog (S/R/F modal-opener pattern):** `hp41-cli/src/app.rs` lines 339–388 — the existing v2.2 `S`/`R`/`F`/`P`/`X` modal-opener block:

```rust
// Phase 5: route to pending_input handler if modal is active — MUST come before
// the modal-opening interceptors below (CR-02). If any modal is active, 'S', 'R',
// and Ctrl+A must be handled by the active modal, not silently replaced.
if self.pending_input.is_some() {
    self.handle_pending_input(key);
    return;
}

// Only open new modals when no modal is currently active (D-08, Pitfall 5).
if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::RegisterPrompt {
        op: RegisterOpKind::Sto,
        ind: false,
        acc: String::new(),
    });
    self.message = None;
    return;
}
```

**Phase 29 R/S insertion** (per RESEARCH §3.4 — between help/programs-overlay block and F5 fallthrough; D-07 ordering invariant requires this go AFTER the `pending_input.is_some()` route at line 327):

```rust
// Phase 29 (D-29.5): R/S submits modal numeric input when modal_program is active.
// MUST be ABOVE the existing F5 (run_program("A")) handler — modal flow takes
// precedence over the v1.0 run-A binding when a math1 modal is open.
if key.code == KeyCode::F(5) && self.state.modal_program.is_some() {
    match hp41_core::ops::math1::submit_modal(&mut self.state) {
        Ok(()) => {
            self.message = None;
            self.drain_and_show_print_output(None);
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
    return;
}
```

**Analog (Esc handler pattern):** lines 436–451 — the existing v2.1 `shift_armed` Esc handler:

```rust
if self.shift_armed {
    if key.code == KeyCode::Esc {
        self.shift_armed = false;
        return;
    }
    if let Some(op) = keys::shifted_key_to_op(key, self) {
        self.call_dispatch(op);
    }
    self.shift_armed = false;
    return;
}
```

**Phase 29 Esc insertion** (per RESEARCH §3.5 — ordering: pending_input route returns first, shift_armed returns second, then this fires):

```rust
// Phase 29 (D-29.6): Esc cancels an open math1 modal (no pending_input active).
// If pending_input is active (XeqByName / RegisterPrompt / etc.), Esc is handled
// by handle_pending_input via existing per-arm handlers (D-07 — never override).
if key.code == KeyCode::Esc
    && self.state.modal_program.is_some()
    && self.pending_input.is_none()
{
    hp41_core::ops::math1::cancel_modal(&mut self.state);
    self.message = Some("Cancelled".to_string());
    return;
}
```

**Analog (post-dispatch hook insertion point):** `hp41-cli/src/app.rs::call_dispatch` lines 1640–1645:

```rust
fn call_dispatch(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => self.message = None,
        Err(e) => self.message = Some(format!("{e}")),
    }
}
```

**Phase 29 auto-open extension** (per RESEARCH §3.7 — insert trailing call to a new helper):

```rust
fn call_dispatch(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => self.message = None,
        Err(e) => self.message = Some(format!("{e}")),
    }
    self.maybe_auto_open_collect_for_modal();  // ← NEW Phase 29 hook
}

fn maybe_auto_open_collect_for_modal(&mut self) {
    use hp41_core::ops::math1::ModalProgram;
    if self.pending_input.is_some() { return; }
    let Some(ref mp) = self.state.modal_program else { return; };
    if mp.requires_alpha_label() {
        self.pending_input = Some(PendingInput::XeqByName {
            acc: String::new(),
            mode: XeqByNameMode::CollectForModal,
        });
    }
}
```

**Also wire `call_dispatch_and_drain`** (line 1651) — same trailing-call insertion. Both helpers route ops; both must auto-open.

---

### `hp41-cli/tests/function_matrix_parity.rs` extension (test — parity)

**Analog:** the SAME file's 4 existing tests (lines 200–267).

**Test-shape pattern** (lines 211–231 — forward parity):

```rust
#[test]
fn test_every_rom_op_has_matrix_entry() {
    let entries = help_entries();
    let json_variants: HashSet<&str> = entries.iter().map(|e| e.op_variant.as_str()).collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in ALL_OP_VARIANT_NAMES {
        if INTERNAL_OP_VARIANTS.contains(name) {
            continue;
        }
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Op::* variants missing from docs/hp41cv-functions.json: {missing:?}"
    );
}
```

**Phase 29 extension** — add a parallel `MATH1_OP_VARIANT_NAMES` const + 3 new tests reusing the SAME HashSet idiom (per RESEARCH §5):

```rust
const MATH1_OP_VARIANT_NAMES: &[&str] = &[
    "Sinh", "Cosh", "Tanh", "Asinh", "Acosh", "Atanh",
    "CPlus", "CMinus", "CTimes", "CDiv", "Real",
    "Magz", "Cinv", "ZpowN", "Zpow1N", "ExpZ", "LnZ", "SinZ", "CosZ", "TanZ", "ApowZ", "LogZ", "ZpowW",
    "PolyWorkflow", "Roots",
    "MatrixWorkflow", "MatSize", "MatVmat", "MatEdit", "MatDet", "MatInv", "MatSimeq", "MatVcol",
    "Integ", "Solve", "Sol", "Difeq",
    "Four", "TriSss", "TriAsa", "TriSaa", "TriSas", "TriSsa",
    "Trans2d", "Trans3d",
];

#[test]
fn test_math1_op_inventory_count() {
    assert_eq!(MATH1_OP_VARIANT_NAMES.len(), 47, "MATH1 inventory drift");
}

#[test]
fn test_every_math1_rom_op_has_math1_json_entry() {
    let entries = help_entries_math1();  // narrow accessor — Math1 pool only
    let json_variants: HashSet<&str> = entries.iter().map(|e| e.op_variant.as_str()).collect();
    let mut missing: Vec<&str> = Vec::new();
    for name in MATH1_OP_VARIANT_NAMES {
        if !json_variants.contains(name) { missing.push(name); }
    }
    assert!(missing.is_empty(), "Math1 Op variants missing from JSON: {missing:?}");
}

#[test]
fn test_every_math1_json_entry_has_xrom_resolver_match() {
    for entry in help_entries_math1() {
        let resolved = hp41_core::ops::math1::xrom::xrom_resolve(
            entry.display_name.as_str(), 0b0000_0001,
        );
        assert!(resolved.is_some(),
            "Math1 JSON '{}' (display: '{}') not resolvable via xrom_resolve",
            entry.op_variant, entry.display_name);
    }
}
```

**Critical:** existing 4 tests survive unchanged — they continue using narrow `help_entries()` for the v2.2 130-row pool. The 2 pools are partitioned, never mixed (CONTEXT D-29.2).

---

### `hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` extension

**Analog:** the SAME function lines 369–415.

**Existing canonical-table pattern** (lines 370–397):

```rust
let canonical: &[(&str, TestKind)] = &[
    ("X<>Y?", TestKind::XNeY),
    ("X\u{2260}Y?", TestKind::XNeY),
    // ... 12 more conditional-test entries ...
];
for (name, kind) in canonical {
    assert_eq!(
        xeq_by_name_local_resolve(name),
        Some(Op::Test(kind.clone())),
        "CLI-local resolver disagreed with the canonical table for {name:?}"
    );
}
```

**Phase 29 extension** — append a parallel Math Pac I table after the existing canonical table (per RESEARCH §3.1 + §5):

```rust
// Phase 29 / CLI-01: extend cli_resolver_matches_core_resolver with Math Pac I names.
let math1_cases: &[(&str, Op)] = &[
    ("SINH", Op::Sinh),
    ("ASINH", Op::Asinh),
    ("MATRIX", Op::MatrixWorkflow),
    ("DET", Op::MatDet),
    ("INV", Op::MatInv),
    ("C+", Op::CPlus),
    ("REAL", Op::Real),
    ("INTG", Op::Integ),
    ("SOLVE", Op::Solve),
    ("DIFEQ", Op::Difeq),
];
for (name, expected_op) in math1_cases {
    assert_eq!(
        xeq_by_name_local_resolve(name, 0b0000_0001),
        Some(expected_op.clone()),
        "CLI-local resolver must resolve Math Pac I '{name}' via xrom_resolve fallback",
    );
    // Negative: with XROM unloaded, CLI resolver must NOT resolve Math Pac I names
    assert_eq!(
        xeq_by_name_local_resolve(name, 0b0000_0000),
        None,
        "CLI-local resolver MUST return None for Math Pac I '{name}' with module unloaded",
    );
}
```

**Signature widening rippling** — every call to `xeq_by_name_local_resolve(...)` in this file (lines 94, 99, 105, 108, 113, 122, 133, 200, 214, 225, 243, 322, 344, 376 per RESEARCH §3.1) gains a `, 0b0000_0001` argument. Trivial sweep.

---

### `hp41-core/src/ops/math1/modal.rs::requires_alpha_label` (new method)

**Analog:** the SAME file's `ModalProgram::current_prompt()` method at lines 41–63 — the canonical "per-program-variant exhaustive match" shape:

```rust
impl ModalProgram {
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
}
```

**Phase 29 mirror — `requires_alpha_label` shape** (per RESEARCH §3.7 + Specific Idea):

```rust
impl ModalProgram {
    /// Returns true for steps that require the user to type a user-program LBL name.
    /// Currently: only the FunctionNamePrompt step of Integ / Solve / Difeq.
    /// Used by CLI/GUI to auto-open the XeqByName modal in CollectForModal mode
    /// (D-29.7 / D-29.9). Mirrored verbatim in `hp41-gui` per CLI ↔ GUI parity (D-25.6).
    pub fn requires_alpha_label(&self) -> bool {
        matches!(
            self,
            ModalProgram::Integ(IntegInputStep::FunctionNamePrompt)
            | ModalProgram::Solve(SolveInputStep::FunctionNamePrompt)
            | ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt)
        )
    }
}
```

**Verified variant names** via grep on this file:
- `IntegInputStep::FunctionNamePrompt` (line 268)
- `SolveInputStep::FunctionNamePrompt` (line 121)
- `DifeqInputStep::FunctionNamePrompt` (line 220)

Positive-form `matches!` macro: no `_ =>` arm needed; future per-program ALPHA-step additions extend the matcher explicitly (FN-CLI-04 spirit preserved).

---

### `hp41-core/src/ops/math1/mod.rs::submit_modal`, `cancel_modal`, `submit_modal_with_label`

**Analogs:**
1. **Modal-open shape** — `op_matrix_workflow` at `hp41-core/src/ops/math1/matrix.rs:323–331`:

```rust
pub fn op_matrix_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG);
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    state.modal_prompt = Some("ORDER=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

2. **Free-function dispatch hub shape** — `op_xeq` at `hp41-core/src/ops/program.rs:67–87`:

```rust
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running { /* ... interactive branch ... */ }
    Err(HpError::InvalidOp)
}
```

**Phase 29 shape** (per RESEARCH §3.4 + §3.5 + Open Q 3) — three new free functions in `hp41-core/src/ops/math1/mod.rs`:

```rust
pub use modal::ModalProgram;

/// Submit numeric input to the open modal — flush entry_buf, advance step.
/// Called from CLI/GUI on R/S key press when state.modal_program.is_some().
/// Mirrors hp41-gui via shared core surface (D-25.6 parity).
pub fn submit_modal(state: &mut CalcState) -> Result<(), HpError> {
    crate::ops::flush_entry_buf(state)?;
    let Some(modal) = state.modal_program.clone() else {
        return Err(HpError::InvalidOp);
    };
    match modal {
        ModalProgram::Matrix(step) => matrix::submit_step(state, step),
        ModalProgram::Solve(step)  => solve::submit_step(state, step),
        ModalProgram::Poly(step)   => poly::submit_step(state, step),
        ModalProgram::Integ(step)  => integ::submit_step(state, step),
        ModalProgram::Difeq(step)  => difeq::submit_step(state, step),
        ModalProgram::Four(step)   => four::submit_step(state, step),
        ModalProgram::Trans(step)  => trans::submit_step(state, step),
    }
}

/// Cancel an open modal — clear modal_program + modal_prompt + entry_buf.
/// Called from CLI/GUI on Esc key press.
pub fn cancel_modal(state: &mut CalcState) {
    state.modal_program = None;
    state.modal_prompt = None;
    state.entry_buf.clear();
    // Stack untouched. matrix_dim / matrix_active_reg outlive the modal.
}

/// Submit a user-LBL name (typed via the XeqByName CollectForModal modal) to
/// the open modal. Trims + uppercases per HP-41 ALPHA convention.
pub fn submit_modal_with_label(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    let upper = label.trim().to_ascii_uppercase();
    state.alpha_reg = upper.clone();
    let Some(modal) = state.modal_program.clone() else {
        return Err(HpError::InvalidOp);
    };
    // Per-program advancement reads alpha_reg and advances modal_program / modal_prompt.
    match modal {
        ModalProgram::Integ(IntegInputStep::FunctionNamePrompt)  => integ::submit_label_step(state, &upper),
        ModalProgram::Solve(SolveInputStep::FunctionNamePrompt)  => solve::submit_label_step(state, &upper),
        ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt)  => difeq::submit_label_step(state, &upper),
        _ => Err(HpError::InvalidOp),  // unreachable in well-formed flow per D-29.9 gate
    }
}
```

**Per-program `submit_step` / `submit_label_step`** — new free functions added to each of `solve.rs`, `integ.rs`, `difeq.rs`, `matrix.rs`, `poly.rs`, `four.rs`, `trans.rs`. RESEARCH §8 Open Q 2 confirms "additive" interpretation acceptable.

---

### `hp41-core/src/ops/math1/{solve,integ,difeq}.rs::op_solve/op_integ/op_difeq` (Open Q1)

**Analog:** `op_xeq` at `hp41-core/src/ops/program.rs:67–87` — the canonical "interactive-vs-running branch" pattern:

```rust
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        // ... interactive branch — open modal / dispatch built-in / etc. ...
    }
    // ... programmatic branch — run_loop owns this ...
    Err(HpError::InvalidOp)
}
```

**Current shape to modify** (`solve.rs:117–120`):

```rust
pub fn op_solve(_state: &mut CalcState) -> Result<(), HpError> {
    // Op::Solve can only run inside run_loop; real implementation in op_solve_run_loop.
    Err(HpError::InvalidOp)
}
```

**Phase 29 modification** (RESEARCH §7.4 / Open Q1 — per the existing source comment at solve.rs:188 that Phase 28 anticipated):

```rust
pub fn op_solve(state: &mut CalcState) -> Result<(), HpError> {
    if !state.is_running {
        // Interactive entry — open the SOLVE modal at FunctionNamePrompt.
        // Phase 28's run_loop implementation reads alpha_reg + R00/R01 staged by this modal.
        state.modal_program = Some(ModalProgram::Solve(SolveInputStep::FunctionNamePrompt));
        state.modal_prompt = Some("FUNCTION NAME?".to_string());
        return Ok(());
    }
    // run_loop arm — never reached here directly (run_loop matches Op::Solve before calling).
    Err(HpError::InvalidOp)
}
```

Same shape for `op_integ` (`integ.rs:150`) and `op_difeq` (`difeq.rs:126`) — distinct prompt text per program. **Note:** this is a small additive `hp41-core` change. RESEARCH calls this out as needing discuss-phase / user confirmation since it brushes against the "hp41-core frozen since Plan 25-01" invariant. The matching shape to `op_xeq` and the explicit Phase 28 cross-reference comment at `solve.rs:188` strongly suggest Phase 28 anticipated it.

---

## Shared Patterns

### Resolver-chain ordering (C-28.4)

**Source:** `hp41-core/src/ops/program.rs:67–87` (`op_xeq`) — the canonical ordering pattern repeated across 3 call sites.

**Apply to:** `hp41-cli/src/keys.rs::xeq_by_name_local_resolve` (the 3rd call site Phase 29 closes).

**Invariant:** `builtin_card_op` / local-table wins → `xrom_resolve` LAST → `InvalidOp` / `None`.

```rust
if let Some(builtin) = local_table_lookup(name) { return Some(builtin); }
if let Some(xrom_op) = xrom_resolve(name, modules) { return Some(xrom_op); }
None
```

### JSON canonical pipeline (D-25.16 / D-25.17 / D-25.18)

**Source:** `hp41-cli/src/help_data.rs` lines 59–77 — `include_str!` + `OnceLock<Vec<HelpEntry>>` + `.expect("<filename> is malformed — fix the JSON")`.

**Apply to:** the second OnceLock for `hp41-math1-functions.json`. Per-file distinct panic-message wording is non-negotiable (CONTEXT D-29.2 / RESEARCH §7.5).

### Compile-time exhaustive match (FN-CLI-04 invariant)

**Sources:**
- `hp41-cli/src/ui.rs::pending_prompt` lines 248–331 (18-arm match, no `_ =>`)
- `hp41-core/src/ops/math1/modal.rs::ModalProgram::current_prompt` lines 41–63
- `hp41-core/src/ops/math1/modal.rs` per-step `current_prompt` impls (each per-program enum has its own exhaustive match)

**Apply to:** every match Phase 29 adds (`XeqByNameMode` in `pending_prompt`, `requires_alpha_label`, `submit_modal`, `submit_modal_with_label`). Positive-form `matches!` is acceptable when the predicate is naturally Boolean (`requires_alpha_label`).

### D-25.11 hybrid struct-variant carrier (allowed-values field)

**Source:** `hp41-cli/src/app.rs::PendingInput::FlagPrompt { kind: FlagPromptKind, ind: bool, acc: String }` and the parallel `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }`.

**Apply to:** `PendingInput::XeqByName { acc: String, mode: XeqByNameMode }`. The `XeqByNameMode` enum is declared in `hp41-cli/src/app.rs` (CLI-local, mirrors the FlagPromptKind/RegisterOpKind placement).

### D-25.6 CLI ↔ GUI parity (shared-core surface)

**Source:** every behavior the CLI gains routes through a `pub fn` in `hp41-core` — `xrom_resolve`, `dispatch`, `flush_entry_buf` are the v2.2 examples.

**Apply to:** `submit_modal`, `cancel_modal`, `submit_modal_with_label`, `requires_alpha_label`. Phase 31's GUI calls the SAME functions identically. Zero parallel implementations.

### D-07 never-discard ordering

**Source:** `hp41-cli/src/app.rs::handle_key` lines 327–330 — the `pending_input.is_some()` route MUST stay above all modal-opening interceptors.

**Apply to:** Phase 29's R/S and Esc interceptors. They MUST go below line 327 (pending_input wins), with explicit `&& self.pending_input.is_none()` guards for self-documenting safety (RESEARCH §7.1).

### Single `call_dispatch` chokepoint (DRY)

**Source:** `hp41-cli/src/app.rs::call_dispatch` lines 1640–1645 + `call_dispatch_and_drain` lines 1651+ — ALL `hp41-core` dispatch goes through one of these two helpers.

**Apply to:** Phase 29's `maybe_auto_open_collect_for_modal` post-hook — insert in BOTH helpers (single insertion-point principle for the auto-open behavior).

---

## No Analog Found

Every Phase 29 file/function has at least one structurally close analog. No file falls back to abstract-research-pattern guidance.

| File | Closest Analog | Notes |
|------|----------------|-------|
| (none) | — | All 13 files map to concrete excerpts above. |

The Esc-cancellation top-level handler is the only behavior with no preexisting v2.2 implementation — but it follows the shape of the shift_armed Esc handler (lines 436–451) verbatim with a different guard predicate.

---

## Metadata

**Analog search scope:**
- `hp41-cli/src/` — keys.rs, help_data.rs, ui.rs, app.rs
- `hp41-cli/tests/` — phase25_help_data.rs, phase25_xeq_by_name.rs, phase25_pending_input.rs, function_matrix_parity.rs
- `hp41-core/src/ops/` — program.rs (op_xeq), math1/xrom.rs, math1/modal.rs, math1/matrix.rs, math1/solve.rs, math1/mod.rs
- `docs/` — hp41cv-functions.json (schema source)

**Files scanned:** 14 source files + 4 test files + 1 JSON data file = 19 files inspected
**Pattern extraction date:** 2026-05-17
**Phase:** 29-cli-integration
**Build sequence:** Phase 28 (XROM framework + Math Pac I core ops) → Phase 29 (this) → Phase 30 (docs + matrix regen) → Phase 31 (GUI mirror) → Phase 32 (E2E + accuracy gates)
