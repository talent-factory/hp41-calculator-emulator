---
phase: 65
slug: standalone-fidelity-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-07
---

# Phase 65 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `65-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + proptest + custom `numerical_accuracy` harness |
| **Config file** | `hp41-core/tests/numerical_accuracy.rs`, `hp41-core/tests/proptest_math.rs` (existing) |
| **Quick run command** | `cargo test -p hp41-core --lib` |
| **Full suite command** | `just test` (all workspace crates) |
| **Estimated runtime** | ~60–120 seconds (full suite; quick lib run ~10s) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core --lib`
- **After every plan wave:** Run `just test`
- **Before `/gsd:verify-work`:** `just test` green AND FACT(27..=69) golden fixtures passing
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 65-MATH-* | MATH-01 | 1 | MATH-01 | — | N/A | unit | `cargo test -p hp41-core --lib hpnum` | ❌ W0 (large-exp serde, exp-carrying arithmetic, exp>99 overflow) | ⬜ pending |
| 65-MATH-* | MATH-01 | 1 | MATH-01 | — | N/A | golden fixture | `cargo test -p hp41-core --test numerical_accuracy fact` | ✅ (extend) | ⬜ pending |
| 65-MATH-* | MATH-01 | 1 | MATH-01 | — | N/A | regression | `cargo test -p hp41-core --test proptest_math` | ✅ (recalibrate X≤26 wall) | ⬜ pending |
| 65-MATH-* | MATH-01 | 1 | MATH-01 | — | N/A | unit (legacy serde) | `cargo test -p hp41-core --lib hpnum serde` | ✅ (extend `test_hpnum_serde_is_string`) | ⬜ pending |
| 65-DISP01-* | DISP-01 | 2 | DISP-01 | — | N/A | integration | `cargo test -p hp41-cli` | ❌ W0 | ⬜ pending |
| 65-DISP02-* | DISP-02 | 2 | DISP-02 | — | N/A | integration | `cargo test -p hp41-cli` | ❌ W0 (mantissa toggle); ✅ existing EEX-CHS regression (~line 2683) | ⬜ pending |
| 65-DISP03-* | DISP-03 | 2 | DISP-03 | — | N/A | unit + integration | `cargo test -p hp41-core --lib types` + `cargo test -p hp41-cli` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Exact task IDs assigned by the planner; rows above map by requirement + plan.*

---

## Wave 0 Requirements

- [ ] HpNum large-exponent serde round-trip test (new `{ "m": "...", "e": N }` form)
- [ ] HpNum arithmetic tests for exponent-carrying ops (`checked_add`/`checked_mul` with exponent > 0)
- [ ] HpNum overflow-at-exp-99 test (exp > 99 → `HpError::Overflow`)
- [ ] CLI DISP-01 integration test (`display_override` rendered in `get_display_string()`)
- [ ] CLI DISP-02 integration test (CHS mantissa toggle `"123"` → `"-123"` → `"123"`, no flush/lift)
- [ ] DISP-03 unit test for the AON flag-48 branch in BOTH `hp41-core` GUI `from_state` (types.rs) and CLI `get_display_string`

*Existing infrastructure covers: FACT golden harness, proptest_math regression, legacy HpNum serde test, EEX-CHS regression.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual CLI display of VIEW/AVIEW/PROMPT override in the TUI | DISP-01 | TUI rendering is exercised by `get_display_string()` unit/integration tests; the rendered terminal frame itself is visual | After implementation, run `hp41` (CLI), execute a VIEW, confirm the main display line shows the register name + value matching the GUI |
| GUI AON parity at rest | DISP-03 | Cross-frontend visual parity (D-25.6) confirmed by automated `from_state` test, but final look is visual | In GUI, set flag 48 (AON), confirm ALPHA register auto-displays after an op; AOFF reverts to X |

*Most behaviors have automated verification; the two rows above are visual confirmations layered on top of automated coverage.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
