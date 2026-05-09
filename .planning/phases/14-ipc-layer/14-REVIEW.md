---
phase: 14-ipc-layer
status: issues_found
files_reviewed: 7
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
---

# Code Review: Phase 14 — IPC Layer

## Summary

Seven files reviewed: `types.rs`, `key_map.rs`, `commands.rs`, `lib.rs`, two permission TOML files, and `capabilities/default.json`. The implementation correctly follows the project's architectural rules: `#![deny(clippy::unwrap_used)]` at the crate root, all mutex locks use `.unwrap_or_else(|e| e.into_inner())`, no hp41-core calculator logic is duplicated in the GUI crate (SC-4 clean), `print_buffer` is drained on every command path (SC-3), and unknown key IDs return `GuiError` rather than panicking or silently discarding (SC-2). Tauri capability permissions are scoped to `"main"` window with no wildcard. All test modules carry `#[allow(clippy::unwrap_used)]`.

One warning-level functional gap was found: the EEX-CHS exponent sign toggle (in-buffer operation) is absent from `handle_op`. This is a Phase 15 keyboard-wiring concern since no frontend key currently maps to this operation.

## Findings

### WR-01: EEX-CHS exponent sign toggle missing from `handle_op`

**Severity:** Warning
**Confidence:** 85%
**File:** `hp41-gui/src-tauri/src/commands.rs` (handle_op, lines ~54–107)
**CLI reference:** `hp41-cli/src/app.rs`, lines 389–404

**Description:**

The CLI's `handle_key` treats a dedicated keypress (mapped to `'n'`) as a special in-buffer operation during EEX entry: when `entry_buf` contains `'e'`, it toggles the exponent sign in-place without flushing the buffer or calling `dispatch`. This is HP-41-faithful behavior.

`handle_op` in `commands.rs` handles three special entry-buffer paths (`"0"`–`"9"`, `"."`, `"e"`) but has no equivalent path for in-place exponent sign toggling. Two failure modes depending on what key ID the frontend sends:

1. **Frontend sends `"eex_chs"`:** `key_map::resolve` returns `Err(GuiError { message: "unknown key: eex_chs" })`. Operation fails, entry buffer is left in partially-entered state.
2. **Frontend reuses `"chs"`:** `key_map::resolve` returns `Ok(Op::Chs)`, dispatch calls `flush_entry_buf` (committing the incomplete number), then negates X. Incorrect — commits partial exponent and negates the wrong value.

**Suggested fix** (Phase 15 keyboard-wiring work):

Add a dedicated entry-buffer branch in `handle_op` before the `key_map::resolve` call:

```rust
// EEX-CHS — toggle exponent sign in-place (HP-41 CHS during EEX entry)
if key_id == "eex_chs" {
    if let Some(e_pos) = calc.entry_buf.find('e') {
        let after_e = &calc.entry_buf[e_pos + 1..];
        if after_e.starts_with('-') {
            calc.entry_buf.remove(e_pos + 1);
        } else {
            calc.entry_buf.insert(e_pos + 1, '-');
        }
    }
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    return Ok(CalcStateView::from_state(calc, print_lines));
}
```

**Disposition:** Deferred to Phase 15 — no frontend key currently maps to EEX-CHS in Phase 14. The gap becomes observable only when the keyboard is wired. Document as a known gap for Phase 15 planning.

---

## Files With No Issues

- `hp41-gui/src-tauri/src/types.rs` — DTOs correctly derive only `Serialize` (not `Deserialize`). No `impl std::error::Error for GuiError`. `from_state` priority chain correct.
- `hp41-gui/src-tauri/src/key_map.rs` — `rsplit_once('_')` correctly handles `sto_arith_<op>_<reg>` (Pitfall 3 avoided). All ~50 named ops and 7 parameterized families covered.
- `hp41-gui/src-tauri/src/lib.rs` — `#![deny(clippy::unwrap_used)]` present. `mod` declarations alphabetical. `generate_handler!` registers both commands.
- `hp41-gui/src-tauri/permissions/dispatch-op.toml` — Correct Tauri v2 permission TOML format.
- `hp41-gui/src-tauri/permissions/get-state.toml` — Correct Tauri v2 permission TOML format.
- `hp41-gui/src-tauri/capabilities/default.json` — `windows: ["main"]`, no wildcard. `core:default` preserved.
