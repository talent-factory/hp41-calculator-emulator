---
plan_id: 22-04-catalog-and-asn
phase: 22
plan: 04
type: execute
wave: 1
depends_on: [22-03-memory-ops]
files_modified:
  - hp41-core/src/state.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase22_catalog.rs
  - hp41-core/tests/phase22_asn.rs
autonomous: true
requirements: [FN-MEM-05, FN-KEY-01]
must_haves:
  truths:
    - "New CalcState field `assignments: BTreeMap<u8, String>` with `#[serde(default)]` — slot adjacent to existing `key_assignments` for grep affinity (D-22.17)"
    - "BTreeMap (not HashMap) ensures deterministic JSON serialization order (matches Phase 5 key_assignments precedent)"
    - "Save-file backward compat: v1.0–v2.1 JSON without `assignments` field deserializes to empty BTreeMap via #[serde(default)] (D-22.22)"
    - "Op::Catalog(u8) per AMENDED D-22.16 / OQ-1 (Option B hardware-faithful, 2026-05-14): n==0 OR n>=5 → InvalidOp; CAT 1 = programs (LBL listing with step counts); CAT 2/3/4 = single \"NOT AVAILABLE\" payload line (24-char padded)"
    - "Catalog output goes to state.print_buffer with 24-char width: header `-- CATALOG n --`, payload, footer `-- END --` (D-22.16)"
    - "Op::Asn { name: String, key_code: u8 } struct-variant per AMENDED D-22.18 / OQ-3: if name.is_empty() → state.assignments.remove(&key_code); else state.assignments.insert(key_code, name)"
    - "ASN serializes as JSON `{\"Asn\": {\"name\": \"SIN\", \"key_code\": 11}}` (serde default struct-variant shape — Pitfall 9)"
    - "Both new variants land in 4 places (D-22.21): Op enum + dispatch + execute_op + BOTH prgm_display.rs copies"
    - "Op::Asn target resolution (parse-as-Op OR LBL search) lives in CLI/GUI USER-mode dispatch (Phase 25/26) — hp41-core only stores the assignment as a String (D-22.19)"
  artifacts:
    - path: "hp41-core/src/state.rs"
      provides: "New field `assignments: BTreeMap<u8, String>` with #[serde(default)]"
      contains: "pub assignments: BTreeMap<u8, String>, #[serde(default)]"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Op::Catalog(u8) + Op::Asn { name, key_code } variants + dispatch arms"
      contains: "Op::Catalog, Op::Asn { name, key_code }"
    - path: "hp41-core/src/ops/program.rs"
      provides: "op_catalog helper (programs/NOT AVAILABLE branches) + op_asn helper (empty-name-removes)"
      contains: "pub fn op_catalog, pub fn op_asn, NOT AVAILABLE, state.assignments"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "op_display_name arms for CATALOG n / ASN \"name\" nn"
      contains: "\"CATALOG {n}\", \"ASN \\\"{name}\\\" {key_code:02}\""
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 2 display arms as CLI copy (SC-4)"
      contains: "\"CATALOG {n}\", \"ASN \\\"{name}\\\" {key_code:02}\""
    - path: "hp41-core/tests/phase22_catalog.rs"
      provides: "Integration tests for CATALOG 1/2/3/4 + invalid n + 24-char width"
      min_lines: 70
    - path: "hp41-core/tests/phase22_asn.rs"
      provides: "Integration tests for ASN insert + remove (OQ-3) + serde round-trip + v20 save-file load test"
      min_lines: 60
  key_links:
    - from: "hp41-core/src/state.rs CalcState.assignments"
      to: "BTreeMap<u8, String> with #[serde(default)]"
      via: "save-file backward compat for v1.x JSON files"
      pattern: "#\\[serde\\(default\\)\\]\\s*\\n\\s*pub assignments:"
    - from: "hp41-core/src/ops/program.rs::op_catalog"
      to: "state.print_buffer.push(formatted_line) — header + payload + footer"
      via: "print_buffer drain pattern from Phase 11; 24-char width"
      pattern: "state.print_buffer.push"
    - from: "hp41-core/src/ops/program.rs::op_asn"
      to: "state.assignments.insert(key_code, name) OR remove(&key_code)"
      via: "empty-name-removes branch per OQ-3"
      pattern: "state.assignments.(insert|remove)"
---

<objective>
Land the two final Phase 22 ops in `hp41-core` per D-22.16 (AMENDED OQ-1) and D-22.17/D-22.18/D-22.19 (AMENDED OQ-3) — `Op::Catalog(u8)` (hardware-faithful: CAT 1 = programs, CAT 2/3/4 = "NOT AVAILABLE") and `Op::Asn { name: String, key_code: u8 }` (empty-name removes, non-empty inserts). Add the new `assignments: BTreeMap<u8, String>` field to CalcState with `#[serde(default)]` for v1.x save-file backward compat.

Purpose: CATALOG provides program listings via print_buffer (matches HP-41 hardware Cat-1 with programs). ASN unlocks key-assignment programming — a hallmark HP-41CV feature. The new `assignments` field coexists with the Phase 5 `key_assignments: BTreeMap<char, String>` (char-keyed for hp41-cli USER mode); Phase 25/26 will reconcile the two maps when CLI/GUI keyboard wiring lands.

Output: 1 new CalcState field + 2 new Op variants (Catalog + Asn) + 2 dispatch arms + 2 execute_op arms + 2 helper functions (op_catalog, op_asn) + both prgm_display.rs copies updated + 2 integration test files (phase22_catalog.rs, phase22_asn.rs) with serde round-trip and OQ-3 empty-name-removes coverage.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md
@.planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md
@.planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md
@.planning/phases/22-program-control-and-memory-ops/22-VALIDATION.md
@CLAUDE.md

<interfaces>
<!-- State after plans 22-01/02/03: Op enum has gained Stop, Pse, Clp, Del, Ins, GtoInd, XeqInd, Size, Cla, Clst, Pack. -->
<!-- This plan adds the final 2: Op::Catalog(u8) and Op::Asn { name: String, key_code: u8 }. -->

From hp41-core/src/state.rs (existing precedent for new field placement):
```rust
// :86–88 — existing Phase 5 field (analog for the new assignments field)
pub user_mode: bool,
pub key_assignments: BTreeMap<char, String>,

// :97–112 — established #[serde(default)] precedent for new fields
#[serde(default)]
pub last_key_code: u8,

#[serde(default)]
pub reg_m: HpNum,
// ... etc ...

// CalcState::new() at :145–171 — initialize new field with BTreeMap::new()
```

From hp41-core/src/ops/print.rs (Phase 11 print_buffer push analog):
```rust
// :13–19 — op_prx pattern (24-char width)
pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

OQ-1 resolution (CONTEXT.md line 6): Option B = hardware-faithful CATALOG.
- CAT 1 = programs (LBL listing)
- CAT 2 = XROM modules ("NOT AVAILABLE" — no plug-in modules in v2.x)
- CAT 3 = HP-IL devices ("NOT AVAILABLE")
- CAT 4 = peripherals ("NOT AVAILABLE")

OQ-3 resolution (CONTEXT.md line 10): Option A = empty-name-removes.
- Op::Asn { name: "", key_code: 11 } → state.assignments.remove(&11)
- Op::Asn { name: "SIN", key_code: 11 } → state.assignments.insert(11, "SIN")
</interfaces>
</context>

<tasks>

<task id="22-04-01" type="auto" tdd="true">
  <name>Task 22-04-01: Add `assignments: BTreeMap<u8, String>` field to CalcState with #[serde(default)] (slot adjacent to key_assignments at state.rs:88) + initialize in CalcState::new()</name>
  <files>
    hp41-core/src/state.rs
  </files>
  <read_first>
    - hp41-core/src/state.rs (full file, especially lines 53–171 — struct + new())
    - hp41-core/src/state.rs lines 86–88 (key_assignments — analog for placement)
    - hp41-core/src/state.rs lines 97–134 (#[serde(default)] precedents)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"hp41-core/src/state.rs — new `assignments` field" (lines 513–549)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.17, D-22.22
  </read_first>
  <behavior>
    - Add field declaration AFTER key_assignments (state.rs:88):
      ```rust
      /// HP-41 ASN key assignments: maps hardware key code (row×10+col, 1-indexed)
      /// → assigned target name. Phase 22 (FN-KEY-01). Coexists with key_assignments
      /// (Phase 5, char-keyed) — Phase 25/26 reconciles the two maps.
      /// `#[serde(default)]` keeps v1.0–v2.1 save files loadable (default → empty map).
      #[serde(default)]
      pub assignments: BTreeMap<u8, String>,
      ```
    - Update CalcState::new() to initialize: `assignments: BTreeMap::new(),` (placed next to the existing `key_assignments: BTreeMap::new(),` line at state.rs:160 for grep affinity).
    - Verify `use std::collections::BTreeMap;` is already at the file head (it should be — key_assignments uses it).
  </behavior>
  <action>
    1. Open hp41-core/src/state.rs.
    2. Find the existing `pub key_assignments: BTreeMap<char, String>,` line (around :88).
    3. Insert IMMEDIATELY AFTER it (in the struct body) the new field declaration per <behavior>. Include the full doc-comment.
    4. Find CalcState::new() (around :145). Find the existing `key_assignments: BTreeMap::new(),` initialization (around :160).
    5. Insert IMMEDIATELY AFTER it: `assignments: BTreeMap::new(),`.
    6. Confirm `use std::collections::BTreeMap;` is at file head; if missing add it.
    7. Run `cargo check --workspace` — must compile.
    8. Run `cargo test --package hp41-core` — existing tests stay green (new field defaults to empty in deserialization).
    9. Run `cargo clippy --workspace --all-targets -- -D warnings` — clean.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo test --package hp41-core` exits 0 (existing tests, including any phase21_*.rs serde round-trips on v20-autosave.json, still pass — proves #[serde(default)] works)
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -nE 'pub assignments: BTreeMap<u8, String>' hp41-core/src/state.rs` shows exactly 1 hit
    - `grep -B 1 'pub assignments:' hp41-core/src/state.rs | grep -c '#\[serde(default)\]'` returns 1 (serde-default attribute is on the new field)
    - `grep -c 'assignments: BTreeMap::new()' hp41-core/src/state.rs` returns 1 (CalcState::new initializes it)
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-04-01.log; cargo test --package hp41-core 2>&1 | tail -5; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-04-01.log; grep -nB 1 'pub assignments: BTreeMap<u8, String>' hp41-core/src/state.rs</automated>
  </verify>
  <done>CalcState.assignments field landed with #[serde(default)] + BTreeMap::new() initializer; existing tests green; clippy clean.</done>
</task>

<task id="22-04-02" type="auto" tdd="true">
  <name>Task 22-04-02: Add Op::Catalog(u8) variant + dispatch + execute_op arm + op_catalog helper (per AMENDED D-22.16 / OQ-1: CAT 1 = programs, CAT 2/3/4 = "NOT AVAILABLE") + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/print.rs lines 13–60 (op_prx / op_pra / op_prstk — 24-char width + print_buffer.push pattern)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Catalog(u8) arm" (lines 323–367) — full sketch
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.16 + D-22.16.1 + D-22.16.2 + D-22.16.3 (AMENDED 2026-05-14 per OQ-1 Option B)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §6 OQ-1 + §7 CATALOG sketch (lines 642–661)
  </read_first>
  <behavior>
    - Op::Catalog(u8) variant appended to Op enum
    - dispatch arm calls op_catalog(state, n)
    - op_catalog helper:
      ```
      pub fn op_catalog(state: &mut CalcState, n: u8) -> Result<(), HpError> {
          if n == 0 || n >= 5 {
              return Err(HpError::InvalidOp);
          }
          state.print_buffer.push(format!("{:<24}", format!("-- CATALOG {n} --")));
          match n {
              1 => {
                  // CATALOG 1: programs (hardware-faithful, OQ-1 Option B)
                  let labels: Vec<(usize, String)> = state.program.iter().enumerate()
                      .filter_map(|(i, op)| match op {
                          Op::Lbl(nm) => Some((i, nm.clone())),
                          _ => None,
                      })
                      .collect();
                  for (idx, (pos, name)) in labels.iter().enumerate() {
                      let end = labels.get(idx + 1).map(|(p, _)| *p).unwrap_or(state.program.len());
                      let steps = end - pos;
                      // Truncate long names to 9 chars so total line width stays within 24
                      // ("LBL " = 4, name :9, two spaces, steps :5 = 20 → padded to 24).
                      let display_name: String = name.chars().take(9).collect();
                      state.print_buffer.push(format!("{:<24}", format!("LBL {display_name:9}  {steps:5}")));
                  }
              }
              2 | 3 | 4 => {
                  // CATALOG 2 (XROM) / 3 (HP-IL) / 4 (peripherals) — none in this emulator
                  state.print_buffer.push(format!("{:<24}", "NOT AVAILABLE"));
              }
              _ => unreachable!(),  // guarded above
          }
          state.print_buffer.push(format!("{:<24}", "-- END --"));
          apply_lift_effect(state, LiftEffect::Neutral);
          Ok(())
      }
      ```
    - execute_op arm: `Op::Catalog(n) => op_catalog(state, n)`
    - prgm_display in BOTH copies: `Op::Catalog(n) => format!("CATALOG {n}"),`
    - LiftEffect: Neutral
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Catalog(u8),` to the Op enum.
    2. Add a doc-comment on the variant: `/// Phase 22 D-22.16 (AMENDED OQ-1 Option B, FN-MEM-05). Hardware-faithful CATALOG. /// CAT 1 = programs (LBL listing); CAT 2 = XROM, CAT 3 = HP-IL, CAT 4 = peripherals — none in this emulator → "NOT AVAILABLE". /// Output goes to state.print_buffer (Phase 11 drain pattern).`
    3. In hp41-core/src/ops/mod.rs dispatch(): add `Op::Catalog(n) => crate::ops::program::op_catalog(state, n),`.
    4. In hp41-core/src/ops/program.rs: add `pub fn op_catalog(state: &mut CalcState, n: u8) -> Result<(), HpError>` adjacent to the other Phase 22 helpers. Use the EXACT body from <behavior> (or PATTERNS.md sketch lines 337–366). Important: `unreachable!()` is permitted ONLY inside a match arm that is guaranteed unreachable by a prior guard — here the `n == 0 || n >= 5` check guards it. clippy may permit this; if it complains, replace with `_ => Err(HpError::InvalidOp)` instead (defensive, never triggers).
    5. In hp41-core/src/ops/program.rs execute_op: add `Op::Catalog(n) => op_catalog(state, n),`.
    6. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Catalog(n) => format!("CATALOG {n}"),`.
    7. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    8. Run cargo check + clippy. If clippy complains about `unreachable!()`, swap to `_ => return Err(HpError::InvalidOp)`.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -n 'pub fn op_catalog' hp41-core/src/ops/program.rs` shows 1 hit
    - `grep -A 25 'pub fn op_catalog' hp41-core/src/ops/program.rs | grep -c 'NOT AVAILABLE'` returns ≥1 (CAT 2/3/4 payload)
    - `grep -A 25 'pub fn op_catalog' hp41-core/src/ops/program.rs | grep -c '-- CATALOG'` returns ≥1 (header)
    - `grep -A 25 'pub fn op_catalog' hp41-core/src/ops/program.rs | grep -c '-- END --'` returns ≥1 (footer)
    - `grep -A 25 'pub fn op_catalog' hp41-core/src/ops/program.rs | grep -E 'Op::Lbl'` shows the LBL-iteration code for CAT 1
    - `grep -nE '"CATALOG \{n\}"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-04-02.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-04-02.log; awk '/pub fn op_catalog/,/^}/' hp41-core/src/ops/program.rs | head -30</automated>
  </verify>
  <done>Op::Catalog(u8) lands in all 4 places + helper writes header/body/footer to print_buffer per OQ-1; clippy clean.</done>
</task>

<task id="22-04-03" type="auto" tdd="true">
  <name>Task 22-04-03: Add Op::Asn { name: String, key_code: u8 } struct-variant + dispatch + execute_op arm + op_asn helper (per AMENDED D-22.18 / OQ-3: empty-name removes, non-empty inserts) + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/mod.rs (full file — find existing struct-variant precedent Op::FlagTest { kind, flag })
    - hp41-cli/src/prgm_display.rs (existing struct-variant display arm for Op::FlagTest as the analog for the new Asn arm)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Asn { name, key_code } arm" (lines 369–389)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.18 (AMENDED OQ-3), D-22.19 (resolution at USER-mode dispatch — Phase 25/26 concern, NOT this plan), D-22.21
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §6 OQ-3 + §2 Pitfall 9 (struct-variant JSON shape)
  </read_first>
  <behavior>
    - Op::Asn { name: String, key_code: u8 } struct-variant appended to Op enum
    - dispatch arm: `Op::Asn { name, key_code } => crate::ops::program::op_asn(state, name, key_code)`
    - op_asn helper (OQ-3 Option A — empty-name removes):
      ```
      pub fn op_asn(state: &mut CalcState, name: String, key_code: u8) -> Result<(), HpError> {
          if name.is_empty() {
              state.assignments.remove(&key_code);   // OQ-3: empty name removes
          } else {
              state.assignments.insert(key_code, name);
          }
          apply_lift_effect(state, LiftEffect::Neutral);
          Ok(())
      }
      ```
    - execute_op arm: `Op::Asn { name, key_code } => op_asn(state, name, key_code)`
    - prgm_display in BOTH copies: `Op::Asn { name, key_code } => format!("ASN \"{name}\" {key_code:02}"),` (note: the inner `\"` escapes the double-quote in the format-string — match the existing Op::FlagTest struct-variant pattern)
    - LiftEffect: Neutral
    - LATE-BINDING NOTE: Op::Asn STORES the assignment as a String; resolution (parse-as-Op vs LBL search) is a Phase 25/26 concern per D-22.19 — hp41-core layer is NOT responsible for resolving.
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append the struct-variant `Asn { name: String, key_code: u8 },` to the Op enum.
    2. Add doc-comment: `/// Phase 22 D-22.18 (AMENDED OQ-3 Option A, FN-KEY-01). HP-41 ASN key assignment. /// If `name` is empty: removes assignment for `key_code`. Otherwise inserts. /// key_code is row×10+col (1-indexed; same encoding as last_key_code and keycode_to_hp41_code). /// Late-binding resolution (parse-as-Op vs LBL search) happens at USER-mode dispatch in Phase 25/26.`
    3. In hp41-core/src/ops/mod.rs dispatch(): add `Op::Asn { name, key_code } => crate::ops::program::op_asn(state, name, key_code),`. Verify that the Op enum's destructuring pattern `Op::Asn { name, key_code }` correctly binds the owned String — if the dispatch function takes Op by value (not by reference), the binding moves the String into the helper call; otherwise use `.clone()` or borrow appropriately.
    4. In hp41-core/src/ops/program.rs: add `pub fn op_asn(state: &mut CalcState, name: String, key_code: u8) -> Result<(), HpError>` adjacent to the other Phase 22 helpers. Body per PATTERNS.md sketch lines 375–384.
    5. In hp41-core/src/ops/program.rs execute_op: add `Op::Asn { name, key_code } => op_asn(state, name, key_code),`.
    6. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Asn { name, key_code } => format!("ASN \"{name}\" {key_code:02}"),`.
    7. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    8. Run cargo check + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -n 'pub fn op_asn' hp41-core/src/ops/program.rs` shows 1 hit
    - `grep -A 8 'pub fn op_asn' hp41-core/src/ops/program.rs | grep -E 'name.is_empty\(\)'` shows OQ-3 branch
    - `grep -A 8 'pub fn op_asn' hp41-core/src/ops/program.rs | grep -E 'state.assignments.remove'` shows remove branch
    - `grep -A 8 'pub fn op_asn' hp41-core/src/ops/program.rs | grep -E 'state.assignments.insert'` shows insert branch
    - `grep -nE 'Op::Asn \{ name, key_code \}' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows ≥5 hits (1 enum + 1 dispatch + 1 execute_op + 2 prgm_display)
    - `grep -nE 'ASN' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows the new arms in both copies
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-04-03.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-04-03.log; awk '/pub fn op_asn/,/^}/' hp41-core/src/ops/program.rs | head -12</automated>
  </verify>
  <done>Op::Asn struct-variant lands in all 4 places with OQ-3 empty-name-removes semantic; clippy clean.</done>
</task>

<task id="22-04-04" type="auto" tdd="true">
  <name>Task 22-04-04: Create hp41-core/tests/phase22_catalog.rs (CATALOG 1 programs + CAT 2/3/4 NOT AVAILABLE + invalid n) AND hp41-core/tests/phase22_asn.rs (ASN insert/remove + serde round-trip + #[serde(default)] sentinel via v20 fixture)</name>
  <files>
    hp41-core/tests/phase22_catalog.rs,
    hp41-core/tests/phase22_asn.rs
  </files>
  <read_first>
    - hp41-core/tests/phase21_sound.rs (event_buffer + print_buffer assertion analogs)
    - hp41-core/tests/phase21_flags.rs lines 27–46 (serde round-trip + v20-autosave.json fixture load pattern)
    - hp41-core/tests/fixtures/v20-autosave.json (existing fixture — proves #[serde(default)] works for new fields)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §5 (Validation map FN-MEM-05 + FN-KEY-01)
    - .planning/phases/22-program-control-and-memory-ops/22-VALIDATION.md per-task verification (lines 56–57)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"phase22_catalog" + §"phase22_asn" (lines 768–769)
  </read_first>
  <behavior>
    `tests/phase22_catalog.rs`:
    - `test_catalog_1_lists_programs` (FN-MEM-05 + OQ-1): build state with program `[LBL "A", PushNum(1), LBL "B", PushNum(2), PushNum(3)]`; dispatch Op::Catalog(1); assert print_buffer contains header `-- CATALOG 1 --`, two LBL lines (each 24-char wide), and footer `-- END --`.
    - `test_catalog_1_empty_program_emits_header_footer_only`: empty state.program; Op::Catalog(1); assert print_buffer has exactly 2 entries (header + footer, no payload).
    - `test_catalog_2_not_available`: Op::Catalog(2); assert print_buffer has header, single "NOT AVAILABLE" line (24-char padded), footer.
    - `test_catalog_3_not_available`: same shape with n=3.
    - `test_catalog_4_not_available`: same shape with n=4.
    - `test_catalog_0_rejects`: Op::Catalog(0); assert Err(InvalidOp); print_buffer unchanged.
    - `test_catalog_5_rejects`: Op::Catalog(5); assert Err(InvalidOp); print_buffer unchanged.
    - `test_catalog_lines_are_24_chars_wide`: assert each line in print_buffer has `line.chars().count() == 24`. Fixture programs use short labels (≤9 chars: "A", "B"). To pin the long-name truncation invariant, add a second `test_catalog_long_label_truncated_to_9_chars` test with a program containing `Op::Lbl("VERYLONGLABEL".to_string())` (13 chars), dispatch Op::Catalog(1), assert the corresponding LBL line still has exactly 24 chars AND the line contains `"VERYLONGL"` (9 chars, no truncation marker).

    `tests/phase22_asn.rs`:
    - `test_assignments_field_defaults_to_empty` (D-22.22 sentinel): `CalcState::new().assignments.is_empty()` → true.
    - `test_load_v20_save_no_assignments_field` (#[serde(default)] sentinel): load `tests/fixtures/v20-autosave.json`; assert deserialized.assignments.is_empty().
    - `test_asn_inserts` (FN-KEY-01): dispatch Op::Asn { name: "SIN".to_string(), key_code: 11 }; assert state.assignments.get(&11) == Some(&"SIN".to_string()).
    - `test_asn_overwrites`: ASN insert "SIN" at 11; ASN insert "COS" at 11; assert get(&11) == Some(&"COS".to_string()).
    - `test_asn_empty_name_removes` (OQ-3 sentinel): ASN insert "SIN" at 11; dispatch Op::Asn { name: "".to_string(), key_code: 11 }; assert state.assignments.get(&11) == None.
    - `test_asn_remove_nonexistent_is_noop`: with assignments empty, dispatch Op::Asn { name: "", key_code: 99 }; assert returns Ok AND state.assignments.is_empty().
    - `test_asn_roundtrip_through_json` (FN-KEY-01 SC#5): populate state.assignments with multiple entries; serialize to JSON; deserialize; assert the round-trip preserves all entries with deterministic order (BTreeMap guarantees sorted-by-key).
    - `test_asn_json_struct_variant_shape` (Pitfall 9): serialize a single Op::Asn variant directly; assert JSON exactly matches `{"Asn":{"name":"SIN","key_code":11}}` (or whatever serde's default struct-variant shape produces — the test pins the shape so future schema changes are caught immediately).

    Module headers `#![allow(clippy::unwrap_used)]`.
  </behavior>
  <action>
    1. Create `hp41-core/tests/phase22_catalog.rs` with the 8 tests listed in <behavior>. Use the standard imports `use hp41_core::ops::{dispatch, Op}; use hp41_core::{CalcState, HpError, HpNum};`.
    2. For `test_catalog_lines_are_24_chars_wide`: iterate over `state.print_buffer` after the dispatch and assert `line.chars().count() == 24` for each entry (use `.chars().count()` to handle UTF-8 correctly, though all CATALOG output is ASCII).
    3. Create `hp41-core/tests/phase22_asn.rs` with the 8 tests listed in <behavior>.
    4. For `test_load_v20_save_no_assignments_field`: copy the existing `phase21_flags.rs:27–37` pattern verbatim — `let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap(); let s: CalcState = serde_json::from_str(&json).unwrap(); assert!(s.assignments.is_empty());`.
    5. For `test_asn_roundtrip_through_json`: dispatch 3–4 ASN ops with different key codes; serialize state to JSON via `serde_json::to_string(&s).unwrap()`; deserialize back via `serde_json::from_str(&json).unwrap()`; assert the recovered `state.assignments` equals the pre-serialization map (use `assert_eq!`).
    6. For `test_asn_json_struct_variant_shape`: serialize `Op::Asn { name: "SIN".to_string(), key_code: 11 }` via `serde_json::to_string(&op).unwrap()`; assert the result is exactly the expected shape (Pitfall 9 — pins the shape, no need to enforce a particular layout other than "stable"). If the exact shape is uncertain, run the test once, observe the output, and pin the result. Acceptable shapes: `{"Asn":{"name":"SIN","key_code":11}}` (serde's default tagged-union format for struct variants).
    7. Run `cargo test --package hp41-core --test phase22_catalog` and `cargo test --package hp41-core --test phase22_asn` — all tests pass.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core --test phase22_catalog` exits 0 with ≥8 tests passing
    - `cargo test --package hp41-core --test phase22_asn` exits 0 with ≥8 tests passing
    - File `hp41-core/tests/phase22_catalog.rs` exists and is ≥70 lines
    - File `hp41-core/tests/phase22_asn.rs` exists and is ≥60 lines
    - FN-MEM-05 covered with: CAT 1 = programs (positive), CAT 2/3/4 = NOT AVAILABLE (3 tests), invalid n (2 tests), 24-char width sentinel
    - FN-KEY-01 covered with: insert, overwrite, empty-name-removes (OQ-3 sentinel), roundtrip through JSON, serde-default-empty (v20 fixture sentinel)
    - `just ci` exits 0 (full workspace test + clippy + fmt green)
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core --test phase22_catalog 2>&1 | tee /tmp/22-04-04a.log; cargo test --package hp41-core --test phase22_asn 2>&1 | tee /tmp/22-04-04b.log; tail -5 /tmp/22-04-04a.log; tail -5 /tmp/22-04-04b.log; wc -l hp41-core/tests/phase22_catalog.rs hp41-core/tests/phase22_asn.rs; just ci 2>&1 | tail -10</automated>
  </verify>
  <done>Both test files exist, ≥16 tests pass total, FN-MEM-05 and FN-KEY-01 covered with all locked decisions (OQ-1 / OQ-3 / Pitfall 9 / D-22.22) verified; just ci green; PHASE 22 COMPLETE.</done>
</task>

</tasks>

<verification>
- `cargo check --workspace` exits 0 after every task.
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- `cargo test --package hp41-core --test phase22_catalog` exits 0.
- `cargo test --package hp41-core --test phase22_asn` exits 0.
- `cargo test --package hp41-core` exits 0 (all Phase 22 tests + all pre-Phase-22 tests stay green).
- `just ci` exits 0 (full CI: clippy + fmt + workspace test).
- `just coverage` reports `hp41-core` line coverage ≥ 80% (Phase 22 should not regress from 92.68% baseline — target ≥ 92.5%).
- All 13 Phase 22 Op variants visible in `grep -nE '^\s*(Stop|Pse|Clp|Del|Ins|GtoInd|XeqInd|Size|Cla|Clst|Pack|Catalog|Asn)' hp41-core/src/ops/mod.rs` (or equivalent).
- All 13 display strings × 2 copies = 26 hits in `grep -cE '"STOP"|"PSE"|"CLP|"DEL |"INS"|"GTO IND|"XEQ IND|"SIZE |"CLA"|"CLST"|"PACK"|"CATALOG |ASN \\"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` (rough sanity check — exact count may vary due to format-string vs literal).
- v20-autosave.json still loads correctly (`tests/fixtures/v20-autosave.json` deserializes into CalcState with assignments empty per #[serde(default)]).
</verification>

<success_criteria>
1. `state.assignments: BTreeMap<u8, String>` field added with #[serde(default)] (D-22.17, D-22.22); v1.0–v2.1 save files deserialize with assignments empty.
2. `Op::Catalog(u8)` emits structured output to print_buffer (D-22.16 AMENDED / OQ-1 Option B): CAT 1 = programs (LBL listing); CAT 2/3/4 = "NOT AVAILABLE" (24-char-padded); n==0 or n>=5 → InvalidOp.
3. `Op::Asn { name: String, key_code: u8 }` per D-22.18 AMENDED / OQ-3: empty `name` removes assignment, non-empty inserts. JSON struct-variant shape pinned by sentinel test (Pitfall 9).
4. ASN survives JSON save/load round-trip (FN-KEY-01 SC#5) — verified by dedicated test.
5. All Phase 22 ops (13 total across 4 plans) land in 4 places (D-22.21); both prgm_display.rs copies stay synchronized; SC-4 invariant preserved (no op_* / flush_entry_* / format_hpnum in hp41-gui/src-tauri/).
6. `just ci` green; coverage ≥ 92.5% on hp41-core (no regression from Phase 21 baseline).
</success_criteria>

<output>
After completion, create `.planning/phases/22-program-control-and-memory-ops/22-04-catalog-and-asn-SUMMARY.md` AND a phase-level rollup `.planning/phases/22-program-control-and-memory-ops/22-SUMMARY.md` summarizing all 4 plans, the 13 new Op variants, the new `assignments` field, OQ-1/-2/-3 resolutions, and Phase 22 metrics (test count, coverage, just ci status).
</output>
