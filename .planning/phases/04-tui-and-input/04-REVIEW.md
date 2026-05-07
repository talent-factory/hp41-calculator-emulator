---
phase: 04-tui-and-input
status: findings
files_reviewed: 9
findings:
  critical: 2
  warning: 2
  info: 2
  total: 6
---

# Code Review — Phase 4: TUI & Input

**Depth:** standard | **Files reviewed:** 9

---

### CR-001: EEX entry (`'e'` in `entry_buf`) always fails to parse — EEX is non-functional (Critical)

**File:** `hp41-cli/src/app.rs` | `hp41-core/src/ops/mod.rs`

**Issue:** `app.handle_key()` appends `'e'` to `entry_buf` for EEX (scientific-notation) input. When the user presses a non-digit key next, `flush_entry_buf()` parses the buffer with `Decimal::from_str(&s)`. `rust_decimal`'s `FromStr` does **not** support scientific-notation strings — that requires `Decimal::from_scientific()`. Strings like `"1.5e3"` or `"e"` typed alone all fail `from_str`, returning `HpError::InvalidOp` and silently discarding the entered digits.

EEX entry is **completely non-functional**: no scientific-notation number can be committed to the stack. The Phase 4 smoke-test did not include an EEX test case.

**Recommendation:**
```rust
// In flush_entry_buf (hp41-core/src/ops/mod.rs)
let d = Decimal::from_str(&s)
    .or_else(|_| Decimal::from_scientific(&s))
    .map_err(|_| HpError::InvalidOp)?;
```
Also guard the `'e'` append in `app.handle_key` — reject EEX when `entry_buf` is empty (HP-41 hardware ignores EEX with no preceding mantissa).

---

### CR-002: `entry_buf` accepts structurally invalid sequences — multiple `'.'` and `'e'` silently corrupt input (Critical)

**File:** `hp41-cli/src/app.rs` (handle_key, entry_buf append)

**Issue:** The append condition `c.is_ascii_digit() || c == '.' || c == 'e'` has no state tracking. Users can produce `"1.2.3"`, `"1e2e3"`, `"1e2.3"`, or `"e"` — all of which fail `flush_entry_buf` parse, discarding entered digits. HP-41 hardware physically blocks a second decimal point and ignores EEX when no mantissa is present.

**Recommendation:**
```rust
let buf = &self.state.entry_buf;
let ok = c.is_ascii_digit()
    || (c == '.' && !buf.contains('.') && !buf.contains('e'))
    || (c == 'e' && !buf.is_empty() && !buf.contains('e'));
if ok {
    self.state.entry_buf.push(c);
    self.message = None;
    return;
}
```

---

### WR-001: `'q'` quit intercept is unconditional — will break ALPHA-mode text entry in Phase 5 (Warning)

**File:** `hp41-cli/src/app.rs` (handle_key, quit guard)

**Issue:** The `'q'` quit check fires unconditionally before any mode-context check. Phase 5 adds ALPHA text entry — when `alpha_mode` is active, `'q'` should append to the alpha register, not exit the app. The same ordering issue exists for `'d'` and `'f'`, but only `'q'` causes termination.

**Recommendation:** Guard the quit check with `!self.state.alpha_mode`, or add an alpha-mode early-return block at the top of `handle_key` that routes printable characters into the alpha register before all other key handling.

---

### WR-002: `format_step` displays 0-based `pc` as step number — diverges from HP-41 1-based convention (Warning)

**File:** `hp41-cli/src/prgm_display.rs` | `hp41-cli/src/tests/prgm_display_tests.rs`

**Issue:** `format_step` formats the step number as the raw `state.pc` (0-indexed). On HP-41 hardware, the first recorded instruction is "001", not "000". The test `program_step_at_add` asserts `"000 + "` at `pc = 0`, enshrining the discrepancy. SST from "000 END" should advance to "001 first_op"; under current code it shows "000 first_op".

**Recommendation:**
```rust
pub fn format_step(state: &CalcState) -> String {
    if state.pc >= state.program.len() {
        "000 END".to_string()
    } else {
        let op_name = op_display_name(&state.program[state.pc]);
        format!("{:03} {op_name}", state.pc + 1)
    }
}
```
Update test assertions accordingly.

---

### IN-001: `App` does not implement `Default` trait (Info)

**File:** `hp41-cli/src/app.rs`

**Issue:** `App::new()` exists but `impl Default for App` is absent, while `CalcState` and `Stack` both implement `Default`. Minor style inconsistency.

**Recommendation:** `impl Default for App { fn default() -> Self { Self::new() } }`

---

### IN-002: `KEY_REF_TABLE` test checks count but not content (Info)

**File:** `hp41-cli/src/tests/keys_tests.rs`

**Issue:** `assert_eq!(KEY_REF_TABLE.len(), 33)` gives an opaque count mismatch if entries change and does not verify that listed keys correspond to ops in `key_to_op`.

**Recommendation:** Add content-based assertions or a round-trip test asserting every non-None `key_to_op` result has a corresponding `KEY_REF_TABLE` entry. The count check can remain as a secondary guard.
