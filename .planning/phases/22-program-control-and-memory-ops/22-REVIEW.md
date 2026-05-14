---
phase: "22"
phase_name: program-control-and-memory-ops
reviewer: gsd-code-reviewer
depth: standard
status: issues-found
critical: 0
warning: 1
info: 4
reviewed_at: 2026-05-14
---

# Phase 22 — Adversarial Code Review

## Summary

Phase 22 lands 13 HP-41CV ROM ops (Stop, Pse, GtoInd, XeqInd, Clp, Del, Ins,
Size, Cla, Clst, Pack, Catalog, Asn) plus a Wave-0 register-bounds audit and a
new `assignments: BTreeMap<u8, String>` CalcState field. The implementation is
disciplined: every new variant lands in the canonical four places, every new
helper carries `LiftEffect::Neutral` per D-22.25, the SC-4 strict grep produces
no hits in `hp41-gui/src-tauri/`, the Wave-0 audit eliminates the hardcoded
SIZE-100 register-length assumption from all 28 surveyed access sites, all 68
new Phase 22 tests pass, and `cargo clippy --workspace --all-targets -- -D
warnings` is clean. No critical correctness or security defects were
identified. One Warning surfaces a residual panic path in `op_ins` that is
unreachable through normal program operation but violates the literal reading
of the zero-panic invariant under a corrupted-save-file scenario. Four Info
items document minor code-quality observations.

## Warnings

| Location | Issue | Recommendation |
|---|---|---|
| `hp41-core/src/ops/program.rs:208` | `op_ins` performs `state.program.insert(state.pc, Op::Null)`. `Vec::insert` **panics if `index > len()`**. Under normal Phase 22 control flow, `state.pc <= state.program.len()` always holds (verified by tracing CLP cursor-reposition, DEL pc-preservation, run_loop break/Gto/Xeq arms). However, a corrupted or malicious `~/.hp41/autosave.json` could deserialize `CalcState` with `pc > program.len()` — no field-level validation exists at load time. Invoking INS in that state would panic, violating the literal reading of the D-22.23 zero-panic invariant (`#![deny(clippy::unwrap_used)]` is necessary-but-not-sufficient to enforce it). Note `op_del`/`op_clp` are immune because `saturating_sub` and `.min(...)` already neutralize the bad-pc path. | Clamp pre-insert: `let idx = state.pc.min(state.program.len()); state.program.insert(idx, Op::Null);` (1-line defense-in-depth). Alternative: validate `pc <= program.len()` once in `persistence::load_state` and clamp/reject there — but that belongs to hp41-cli / hp41-gui, not hp41-core. The in-helper clamp is the safer fix because it makes `hp41-core` panic-free regardless of where the malformed state originated. |

## Info

| Location | Observation | Suggestion |
|---|---|---|
| `hp41-core/src/ops/program.rs:286, 309-312` | `op_catalog` uses the double-format pattern `format!("{:<24}", format!("-- CATALOG {n} --"))` and the same shape for LBL lines. Allocates two `String` per emitted line; clippy `format_in_format_args` is allow-by-default in stable so this currently passes `-D warnings`. | Optional micro-tightening: `format!("{:<24}", format_args!("-- CATALOG {n} --"))` would avoid the inner String allocation. Behavior is identical; no test changes. Skip if churn-cost not justified. |
| `hp41-core/src/ops/program.rs:322` | Defensive match arm `_ => return Err(HpError::InvalidOp)` is unreachable: the early-return at `program.rs:281` already guards `n == 0 \|\| n >= 5`, leaving only `n in 1..=4` to reach the inner match, and the arms `1` and `2..=4` exhaust that range. Reads as either belt-and-braces or dead code depending on taste. | Either annotate `// unreachable, defensive — exhaustive guard above` (already done) or remove the arm and let exhaustiveness fall out. Current state is acceptable; flagged only because adversarial review notes it. |
| `hp41-core/src/ops/registers.rs:22, 58-64`; `hp41-core/src/ops/program.rs:110-125`; `hp41-core/src/ops/stats.rs:32-37` | After explicit `if idx >= state.regs.len() { return Err(InvalidOp); }` guards or `if state.regs.len() < 7 { return Err(InvalidOp); }`, the bodies use raw `state.regs[idx]` indexing. The check-then-index pattern is correct and bounds-safe, but newer idiomatic Rust would prefer the `.get(idx).ok_or(InvalidOp)?` style consistently with `op_rcl` (registers.rs:31-35) and the new `Op::GtoInd`/`Op::XeqInd` inline resolver (program.rs:463-467). The mixed style — `.get().ok_or()?` for new code, raw `[idx]` for older bodies — is functional but inconsistent. | No action required for v2.2. If touched in a future refactor (e.g., Phase 24's `resolve_indirect()` extraction), align on the `.get().ok_or()?` idiom consistently. |
| `hp41-core/src/ops/registers.rs:4` | Module doc-comment says "Phase 22 D-22.11.1 — bound is dynamic; default SIZE is 100". The default-SIZE-is-100 phrasing is technically true (set at `state.rs:154`), but the comment could be read as encouraging callers to assume 100. The op_sto inline comment on the same file (`registers.rs:17`) calls out the audit explicitly. | Minor wording polish (optional): "bound is `state.regs.len()` (dynamic; CalcState::new() initializes 100 slots, FN-MEM-01 `Op::Size` resizes)". Not a bug — just a clarity nudge. |

## Verified invariants

| # | Invariant | Status | Note |
|---|---|---|---|
| 1 | Zero-panic in hp41-core (no `.unwrap()`/`panic!()`/raw indexing of unbounded vecs in production code) | OK | All Phase 22 helpers use `.get().ok_or()?` for `regs` access (GtoInd/XeqInd) or have explicit upstream bounds checks. All `.unwrap()`s located are inside `#[cfg(test)]` modules carrying `#[allow(clippy::unwrap_used)]`. Caveat: see Warning above for `op_ins` `Vec::insert` corner case. |
| 2 | Wave-0 bounds audit complete: `registers.rs` + `stats.rs` + `display_ops.rs` use `state.regs.len()` dynamically (no hardcoded 100/7 constants) | OK | `grep -n "reg >= 100\|>= 100\|< 100\|vec!\[.*; 100\]"` on `hp41-core/src/ops/*.rs` yields zero production hits. `state.rs:154` retains `vec![HpNum::zero(); 100]` as the cold-start default — correct (HP-41 hardware default SIZE = 100). Σ-family entry guard `if state.regs.len() < 7` present in all 8 functions (`op_sigma_plus`, `op_sigma_minus`, `op_mean`, `op_sdev`, `op_lr`, `op_yhat`, `op_corr`, `op_cl_sigma_stat`). |
| 3 | LiftEffect correctness: every new Phase 22 Op declares `LiftEffect::Neutral` | OK | Inline grep confirms 13/13 Phase 22 helpers and inline arms call `apply_lift_effect(state, LiftEffect::Neutral)` (or no-call on Err paths). No mistaken `Enable`/`Disable` entries. |
| 4 | `#[serde(default)]` on `CalcState.assignments` (D-22.22 / backward compat) | OK | `state.rs:93` carries `#[serde(default)]`. Sentinel test `test_load_v20_save_no_assignments_field` (`tests/phase22_asn.rs:26-37`) loads `v20-autosave.json` (which lacks the field) and asserts the resulting map is empty. |
| 5 | SC-4 invariant: no `op_*`/`flush_entry_*`/`format_hpnum` duplication in `hp41-gui/src-tauri/` | OK | Strict grep `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` returns zero hits. `hp41-gui/src-tauri/src/prgm_display.rs` only contains the documented display formatter `op_display_name` (D-22.24 acknowledged exception). |
| 6 | `Op::Stop` no-display-write invariant (D-22.1) | OK | `run_loop` arm `Op::Stop => break` (`program.rs:575`) has no `display_override` write. Dispatch arm at `mod.rs:712-715` is a Neutral no-op. Sentinel test `test_stop_does_not_write_display_override` (`tests/phase22_program_control.rs:53-74`) enforces. |
| 7 | `Op::Pse` two-channel write (D-22.2 / D-22.4) | OK | Both the dispatch arm (`mod.rs:720-726`) and the execute_op arm (`program.rs:770-776`) write `state.display_override = Some(format_hpnum(...))` AND `state.event_buffer.push("PAUSE 1000".to_string())`. Sentinel test `test_pse_writes_both_channels` asserts both channels populated. Pitfall 3 test confirms display_override survives subsequent run_loop iterations and is cleared by the next interactive dispatch. |
| 8 | PRGM-mode record gate (D-22.10): Clp/Del/Ins are NOT auto-recorded | OK | Special-case at `mod.rs:533-541`: when `prgm_mode == true`, ops other than `PrgmMode \| Clp(_) \| Del(_) \| Ins` are appended to `state.program`; the trio falls through to dispatch the helpers directly. Tested by `test_ins_is_not_self_recorded_in_prgm_mode` and confirmed indirectly by `test_clp_boundary` / `test_del_clamping` (which would gain spurious entries if the gate were broken). |
| 9 | Indirect-resolver bounds (D-22.4..D-22.6): `Op::GtoInd(u8)` / `Op::XeqInd(u8)` use `.get().ok_or(InvalidOp)?` for register read and `trunc_int + equality` for non-integer rejection | OK | Both arms at `program.rs:462-475` and `488-505` follow the 6-step pattern. Reg out-of-range → InvalidOp (not panic) verified by `test_gto_ind_reg_out_of_range_rejects` and `test_xeq_ind_reg_out_of_range_rejects`. Non-integer pointer → InvalidOp verified by `test_gto_ind_non_integer_rejects` and `test_xeq_ind_non_integer_rejects`. `Decimal::trunc()` resets the scale to 0 (verified against rust_decimal 1.42 source), so `int_part.inner().to_string()` produces the canonical integer string form ("42", "-7") regardless of the original `Decimal` scale. |
| 10 | XEQ stack-depth guard pre-mutation (`Op::XeqInd`) | OK | `program.rs:489-491`: `if state.call_stack.len() >= 4 { return Err(HpError::CallDepth); }` fires BEFORE the register read and `call_stack.push`. Sentinel test `test_xeq_ind_4_deep_call_stack_rejects` pre-fills call_stack with 4 frames, asserts CallDepth Err AND `call_stack.len() == 4` post-call (no spurious push). Mirrors `Op::Xeq` precedent at `program.rs:507-509`. |
| 11 | Backward-compat: v1.x save files load with `assignments` defaulting to empty | OK | `state.rs:93` `#[serde(default)]`. `tests/fixtures/v20-autosave.json` confirmed to lack the `"assignments"` key. `test_load_v20_save_no_assignments_field` exercises the load path. |
| 12 | Idiomatic Rust: no unused mut, no wildcards that mask bugs, `?`-propagation in error paths | OK | Spot-checked all 13 new helpers + inline arms. `resume_program` correctly uses the `let result = ... ; reset is_running; return result;` pattern (Pitfall 2 — must NOT use `?` to short-circuit before the `is_running = false` cleanup). No `.unwrap()` in production. Helper functions are tight and focused. The catch-all `Op::Lbl(_) \| Op::Gto(_) \| ... \| Op::Ins => Err(HpError::InvalidOp)` at `program.rs:800-816` correctly enumerates programming ops + Phase 22 additions in the exhaustive match arm (compiler-checked). |
| 13 | Clippy cleanliness: `cargo clippy --workspace --all-targets -- -D warnings` exits 0 | OK | Confirmed at review time. |
| 14 | Test coverage gaps for op_clp/op_del/op_ins/op_size critical paths | OK (with one observation) | `tests/phase22_program_edit.rs` covers: CLP middle-block, CLP last-block (Pitfall 4), CLP pc-reposition (Pitfall 6), CLP pc-clamp edge case, CLP missing label, CLP prgm_mode=false rejection, DEL clamping (nnn > available), DEL nnn==0 no-op, DEL pc==len no-op, DEL prgm_mode=false rejection, INS basic, INS prgm_mode=false rejection, INS not-self-recorded. `tests/phase22_memory_ops.rs` covers SIZE basic / SIZE 0 clamp / SIZE > 319 reject / SIZE 319 boundary / SIZE shrink truncate / SIZE grow zero-fill / STO+RCL post-shrink Pitfall 4 / Σ+ post-shrink Pitfall 5 / Σ+ size-7 boundary / CLREG honors SIZE / CLA / CLA == AlphaClear / CLST + LASTX preservation / CLST + lift_enabled preservation (true and false) / PACK no-op. **Observation**: no regression test exercises `op_ins` with `state.pc == state.program.len()` (the legitimate "append" edge case after a STOP at end-of-program). The Warning above also has no regression test (pc > len corruption is a corner case). Not blocking. |

## Quick remediation list

1. **(Warning, low priority)** Clamp `state.pc` in `op_ins` (`hp41-core/src/ops/program.rs:208`) before `Vec::insert` to make hp41-core panic-free under any deserialized state. 1-line change: replace `state.program.insert(state.pc, Op::Null);` with `let idx = state.pc.min(state.program.len()); state.program.insert(idx, Op::Null);`. Add a regression test `test_ins_at_pc_past_len_does_not_panic` in `tests/phase22_program_edit.rs`.

(No other Critical or Warning items require remediation. Info items are optional polish.)

---

_Reviewed: 2026-05-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
