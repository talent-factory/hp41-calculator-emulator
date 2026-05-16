---
phase: 22
slug: program-control-and-memory-ops
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Sources: `22-RESEARCH.md` §5, `22-CONTEXT.md` decisions D-22.1..D-22.25
> (incl. OQ-1/-2/-3 resolutions, 2026-05-14).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (built-in) + `proptest 1.x` (existing in workspace) |
| **Config file** | `Cargo.toml` workspace + `hp41-core/tests/` integration suite |
| **Quick run command** | `just test-core` |
| **Full suite command** | `just ci` (clippy + fmt + workspace test) |
| **Estimated runtime** | ~5 s quick / ~35 s full |
| **Coverage gate** | `just coverage` ≥ 80 % on `hp41-core` (matches CLAUDE.md target) |

---

## Sampling Rate

- **After every task commit:** Run `just test-core` (≤ 5 s)
- **After every plan wave:** Run `just ci` (≤ 35 s; clippy + fmt + workspace test)
- **Before `/gsd-verify-work`:** `just ci` + `just coverage` must be green
- **Max feedback latency:** 12 s

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 22-01-01 | 01 | 1 | FN-PROG-01 | — | STOP halts run_loop; resume_program continues from next step | integration | `cargo test --package hp41-core --test phase22_program_control test_stop_then_resume` | ❌ W0 | ⬜ pending |
| 22-01-02 | 01 | 1 | FN-PROG-02 | — | PSE writes display_override AND event_buffer `"PAUSE 1000"`, run_loop continues | integration | `cargo test --package hp41-core --test phase22_program_control test_pse_writes_both_channels` | ❌ W0 | ⬜ pending |
| 22-01-03 | 01 | 1 | FN-PROG-06 | — | GTO IND nn happy path + non-integer reject + reg-out-of-range reject | integration | `cargo test --package hp41-core --test phase22_program_control test_gto_ind` | ❌ W0 | ⬜ pending |
| 22-01-04 | 01 | 1 | FN-PROG-07 | — | XEQ IND nn happy + 4-deep call-stack + reg-out-of-range reject | integration | `cargo test --package hp41-core --test phase22_program_control test_xeq_ind` | ❌ W0 | ⬜ pending |
| 22-02-01 | 02 | 1 | FN-PROG-03 | — | CLP drains LBL..next-LBL (or to end-of-Vec); missing label → InvalidOp; prgm_mode == false → InvalidOp | integration | `cargo test --package hp41-core --test phase22_program_edit test_clp_boundary` | ❌ W0 | ⬜ pending |
| 22-02-02 | 02 | 1 | FN-PROG-04 | — | DEL nnn clamps to remaining; nnn == 0 → no-op; prgm_mode == false → InvalidOp | integration | `cargo test --package hp41-core --test phase22_program_edit test_del_clamping` | ❌ W0 | ⬜ pending |
| 22-02-03 | 02 | 1 | FN-PROG-05 | — | INS inserts `Op::Null` at state.pc; pc unchanged; prgm_mode == false → InvalidOp | integration | `cargo test --package hp41-core --test phase22_program_edit test_ins_inserts_null_at_pc` | ❌ W0 | ⬜ pending |
| 22-03-00 | 03 | 1 | (D-22.11.1) | — | Wave-0 bounds-audit: every `regs[i]` access uses `.get(i).ok_or(InvalidOp)?` — no panics under SIZE shrink | unit + integration | `cargo test --package hp41-core` (existing tests stay green, no panic regressions) | ❌ W0 | ⬜ pending |
| 22-03-01 | 03 | 1 | FN-MEM-01 | — | SIZE nnn resizes `state.regs`; nnn==0 → clamp to 1 (OQ-2); nnn > 319 → InvalidOp; out-of-range register on STO/RCL after shrink → InvalidOp | integration | `cargo test --package hp41-core --test phase22_memory_ops test_size` | ❌ W0 | ⬜ pending |
| 22-03-02 | 03 | 1 | FN-MEM-02 | — | CLA clears `alpha_reg` (delegates to existing `op_alpha_clear`) | unit | `cargo test --package hp41-core --lib ops::program::tests::test_cla` (or chosen location) | ❌ W0 | ⬜ pending |
| 22-03-03 | 03 | 1 | FN-MEM-03 | — | CLST zeros X/Y/Z/T; preserves `lastx` and `lift_enabled` | integration | `cargo test --package hp41-core --test phase22_memory_ops test_clst_preserves_lastx_and_lift` | ❌ W0 | ⬜ pending |
| 22-03-04 | 03 | 1 | FN-MEM-04 | — | PACK no-op + Neutral lift; returns Ok | unit | `cargo test --package hp41-core --lib ops::program::tests::test_pack` | ❌ W0 | ⬜ pending |
| 22-04-01 | 04 | 1 | FN-MEM-05 | — | CATALOG 1 = programs (LBL listing, hardware-faithful, OQ-1); CATALOG 2/3/4 = "NOT AVAILABLE"; header + footer 24-char width | integration | `cargo test --package hp41-core --test phase22_catalog test_catalog_1234` | ❌ W0 | ⬜ pending |
| 22-04-02 | 04 | 1 | FN-KEY-01 | — | ASN inserts into `state.assignments`; ASN with empty `name` REMOVES (OQ-3); JSON save/load round-trip preserves map | integration | `cargo test --package hp41-core --test phase22_asn test_asn_roundtrip` | ❌ W0 | ⬜ pending |

*Status legend: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky · W0 = Wave-0 file does not yet exist*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/phase22_program_control.rs` — STOP / resume / PSE / GTO IND / XEQ IND
- [ ] `hp41-core/tests/phase22_program_edit.rs` — CLP / DEL / INS
- [ ] `hp41-core/tests/phase22_memory_ops.rs` — SIZE / CLA / CLST / PACK + bounds-audit sentinels
- [ ] `hp41-core/tests/phase22_catalog.rs` — CATALOG 1/2/3/4 output
- [ ] `hp41-core/tests/phase22_asn.rs` — ASN insert / remove / serde round-trip
- No framework installs needed — `cargo test`, `proptest`, `serde_json` already in workspace dev-deps.
- No new fixtures required — existing v1.x save files transparently exercise `#[serde(default)]` on the new `CalcState.assignments` field.

---

## Proptest Opportunities (deferred to Phase 27)

Per RESEARCH.md §5, defer to Phase 27 the following property-based tests; Phase 22
plans must not block on these:

- STOP-resume idempotence (n resumes between STOPs is deterministic)
- DEL clamping invariant: `program.len() == max(0, original_len - min(nnn, original_len - pc))`
- CATALOG empty-body: zero LBLs in `state.program` → CAT 1 emits only header + footer

---

## Manual-Only Verifications

*None in Phase 22.* All behavior is `hp41-core`-only and asserts via `cargo test`.
Phase 25 (CLI) and Phase 26 (GUI) will introduce manual verification for the
keyboard modal flows (ASN name/key prompts, CLP label prompt, DEL nnn prompt).

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or explicit Wave-0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all "❌ W0" references above
- [ ] No watch-mode flags in CI invocation
- [ ] Feedback latency < 12 s for `just test-core`
- [ ] `nyquist_compliant: true` set in frontmatter after sign-off

**Approval:** pending
