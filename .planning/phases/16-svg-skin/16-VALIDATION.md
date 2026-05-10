---
phase: 16
slug: svg-skin
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust unit tests (cargo test); frontend visual-only (no vitest/jest) |
| **Config file** | `hp41-gui/src-tauri/Cargo.toml` |
| **Quick run command** | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Full suite command** | `just ci` (CLI pipeline) + manual `just gui-dev` visual check |
| **Estimated runtime** | ~10 seconds (Rust tests); manual visual checks per SC |

**Frontend test gap:** `hp41-gui/package.json` has no test script and no vitest/jest dependency. All frontend SCs for Phase 16 require **manual visual verification** via `just gui-dev`. Consistent with Phase 15 validation approach.

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **After every plan wave:** Run `just ci` (CLI pipeline must stay green)
- **Before `/gsd-verify-work`:** Full suite green + all 5 SC manual verifications pass + `just gui-dev` opens without error
- **Max feedback latency:** ~10 seconds (Rust); manual visual per SC

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| Wave 0 | 16-01 | 0 | SKIN-01/02 | — | N/A | Rust unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |
| Keyboard.tsx | 16-02 | 1 | SKIN-01/02/03 | — | N/A | Manual visual | `just gui-dev` + visual inspect | ❌ new | ⬜ pending |
| App.tsx wiring | 16-02 | 1 | SKIN-02 | — | N/A | Manual click | `just gui-dev` + click all keys | n/a | ⬜ pending |
| tauri.conf.json | 16-02 | 1 | SC-5 | — | N/A | Manual visual | `just gui-dev` + window size check | ✅ exists | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-gui/src-tauri/src/key_map.rs` — Add `test_all_keyboard_skin_ids_are_valid` test that iterates all dispatched KEY_DEFS ids and confirms `resolve(id)` is `Ok` for named ops

Wave 0 test stub (add to key_map.rs test module):

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_all_keyboard_skin_ids_are_valid() {
        let named_ids = [
            "sigma_plus", "recip", "log", "ln", "sin", "cos", "tan",
            "rdn", "xy_swap", "enter", "div", "mul", "user_mode",
            "minus", "prgm_mode", "alpha_toggle", "chs", "plus",
            "lastx", "clreg", "clx",
        ];
        for id in named_ids {
            assert!(resolve(id).is_ok(), "key_map::resolve({id}) must succeed");
        }
    }
}
```

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| All 40 keys render with correct labels, ENTER double-wide | SKIN-01 / SC-1 | No frontend test framework | `just gui-dev` → inspect all 5 rows, count 40 keys, verify ENTER spans 2 columns |
| Color scheme matches HP-41C hardware | SKIN-01 / SC-2 | Visual judgment required | `just gui-dev` → verify dark brown body, gold f-shift labels, white primary labels, light top-row caps |
| Clicking key updates display | SKIN-02 / SC-3 | Integration click test | `just gui-dev` → click digit keys 1-9, `+`, ENTER; verify display updates correctly |
| CSS scale animation completes < 150ms | SKIN-03 / SC-4 | Visual timing judgment | `just gui-dev` → click any key, observe scale-down snap and return; should feel instantaneous |
| SVG scales correctly at 400px, no pixelation | SC-5 | Visual HiDPI check | `just gui-dev` → resize to 400×700, check SVG edges are crisp on Retina display |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s (Rust) / manual (frontend)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
