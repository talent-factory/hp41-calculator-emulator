---
phase: 25-cli-integration-and-documentation
plan: 04
type: execute
wave: 4
depends_on: [01, 02, 03]
files_modified:
  - docs/hp41cv-functions.json
  - docs/hp41cv-function-matrix.md
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/keys.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/tests/phase25_help_data.rs
  - hp41-cli/tests/function_matrix_parity.rs
  - hp41-cli/tests/key_coverage.rs
  - scripts/docs-matrix/Cargo.toml
  - scripts/docs-matrix/src/main.rs
  - justfile
  - CLAUDE.md
  - README.md
autonomous: true
requirements:
  - FN-CLI-01
  - FN-CLI-03
  - FN-DOC-01
  - FN-DOC-02
  - FN-DOC-03
  - FN-DOC-04
user_setup: []
tags:
  - cli
  - docs
  - json-pipeline
  - ci-parity

must_haves:
  truths:
    - "docs/hp41cv-functions.json exists with ≥130 entries (one per HP-41CV ROM op, including v3.x deferred Module-Pac entries) — schema per D-25.16"
    - "hp41-cli/src/help_data.rs loads docs/hp41cv-functions.json via include_str! + std::sync::OnceLock; calling help_entries() returns ≥130 HelpEntry records on first use; subsequent calls return cached slice"
    - "Malformed JSON is a build-time / startup hard-blocker via .expect() per D-25.17 — smoke tests catch empty/missing JSON at CI time (Pitfall 7)"
    - "hp41-cli/src/ui.rs render_help_overlay (?-key overlay) renders from help_entries() — every entry visible with category grouping derived from the JSON `category` field"
    - "KEY_REF_TABLE in keys.rs is regenerated from the JSON (compile-time derivation OR a regenerate-script — executor's call); contains entries only for entries with non-null `key_path`"
    - "scripts/docs-matrix/ is a standalone non-workspace Rust crate; root Cargo.toml members = [hp41-core, hp41-cli] UNCHANGED (CLAUDE.md invariant)"
    - "just docs-matrix regenerates docs/hp41cv-function-matrix.md from the canonical JSON; just docs-matrix-check diffs the regenerated output against the committed file and exits non-zero on mismatch (Pitfall 8)"
    - "hp41-cli/tests/function_matrix_parity.rs runs bidirectional drift check per D-25.15: every Op variant (minus skiplist) has a JSON entry; every implemented JSON entry has a known Op variant; matrix has ≥130 entries"
    - "CLAUDE.md gains a `### v2.2 additions` block summarizing the v2.2 milestone settled architecture (f-prefix model, Hybrid PendingInput, JSON canonical data flow, builtin_card_op 4→12 extension, soft-claim deferral)"
    - "README.md ships the soft 'feature-complete HP-41CV with documented divergences' claim with a link to docs/hp41cv-function-matrix.md per D-25.17"
    - "hp41-cli/tests/key_coverage.rs runs the FN-CLI-01 verifiable-closure parity test: every implemented JSON entry with non-null key_path dispatches via key_to_op / shifted_key_to_op / modal-opener path to a known Op:: variant — no InvalidOp, no panics; ≥80 implemented entries probed (per D-25.18)"
  artifacts:
    - path: "docs/hp41cv-functions.json"
      provides: "Canonical hand-curated data source — 130+ HelpEntry records, schema per D-25.16"
      contains: "op_variant"
    - path: "docs/hp41cv-function-matrix.md"
      provides: "Generated Markdown table of all HP-41CV ROM ops with status column; v3.x deferred section appended"
      contains: "HP-41CV ROM Function Matrix"
    - path: "hp41-cli/src/help_data.rs"
      provides: "Rewritten: HelpEntry struct + FUNCTIONS_JSON via include_str! + HELP_ENTRIES OnceLock + help_entries() accessor + backward-compat shim"
      contains: "HELP_ENTRIES"
    - path: "hp41-cli/tests/function_matrix_parity.rs"
      provides: "4 tests: test_op_inventory_count_matches_enum, test_every_rom_op_has_matrix_entry, test_every_implemented_matrix_entry_has_op, test_matrix_has_at_least_130_entries"
      contains: "ALL_OP_VARIANT_NAMES"
    - path: "scripts/docs-matrix/Cargo.toml"
      provides: "Standalone non-workspace crate (empty [workspace] stanza per CLAUDE.md), declares serde+serde_json deps"
      contains: "workspace"
    - path: "scripts/docs-matrix/src/main.rs"
      provides: "≤120 LOC Rust binary: reads JSON from arg 1, writes Markdown table to arg 2; status rendering rules per D-25.16"
      contains: "render_markdown"
    - path: "justfile"
      provides: "Two new recipes: docs-matrix (regen) + docs-matrix-check (CI diff per Pitfall 8)"
      contains: "docs-matrix"
    - path: "CLAUDE.md"
      provides: "New `### v2.2 additions` block per FN-DOC-02"
      contains: "v2.2 additions"
    - path: "README.md"
      provides: "Soft 'feature-complete HP-41CV with documented divergences' claim + matrix.md link per D-25.17"
      contains: "feature-complete HP-41CV"
  key_links:
    - from: "hp41-cli/tests/key_coverage.rs"
      to: "hp41-cli/src/keys.rs (key_to_op / shifted_key_to_op / xeq_by_name_local_resolve)"
      via: "Iterates help_entries() filtered by status=implemented && key_path.is_some(); dispatches each keystroke and asserts Some(Op) — FN-CLI-01 closure per D-25.18"
      pattern: "key_coverage|FN-CLI-01"
    - from: "hp41-cli/src/help_data.rs help_entries()"
      to: "docs/hp41cv-functions.json"
      via: "include_str! of relative path + serde_json::from_str + OnceLock cache"
      pattern: "include_str.*hp41cv-functions"
    - from: "hp41-cli/tests/function_matrix_parity.rs"
      to: "hp41-cli help_data help_entries() AND const ALL_OP_VARIANT_NAMES"
      via: "Bidirectional drift catch — every Op (minus skiplist) needs an entry; every implemented entry needs a known Op variant"
      pattern: "help_entries|ALL_OP_VARIANT_NAMES"
    - from: "justfile docs-matrix-check"
      to: "docs/hp41cv-function-matrix.md (committed) vs scripts/docs-matrix output"
      via: "diff of regenerated tmp file against committed file"
      pattern: "diff -u docs/hp41cv-function-matrix.md"
    - from: "README.md"
      to: "docs/hp41cv-function-matrix.md"
      via: "Markdown link in the project description section"
      pattern: "hp41cv-function-matrix"
---

<objective>
Land the canonical JSON-driven documentation pipeline per D-25.16 + D-25.17, close FN-CLI-03 + FN-DOC-01..04, and seal the v2.2 milestone documentation. `docs/hp41cv-functions.json` becomes the single source of truth for help_data.rs (compile-time embedded via include_str! + OnceLock per project precedent in programs.rs). `scripts/docs-matrix/` regenerates `docs/hp41cv-function-matrix.md` on demand via a standalone non-workspace Rust crate. `just docs-matrix-check` diffs against committed Markdown for CI drift catch (Pitfall 8). Bidirectional CI parity test asserts JSON Op-enum drift catch (Pitfall 6). CLAUDE.md and README.md ship the v2.2 settled-architecture block and the soft 'feature-complete HP-41CV' claim.

Purpose: This is the final plan in Phase 25 — it depends on all prior plans (01: f-prefix + KEY_REF_TABLE deprecation note; 02: pending_input variants which determine the key_path field of JSON entries; 03: 12-name builtin_card_op + xeq_by_name_local_resolve which determine key_path values like XEQ X<>Y? for the 8 XEQ-only conditional tests). Per D-25.16, no build.rs codegen; only include_str! + OnceLock + a developer-side just recipe + CI-side check recipe. Per D-25.17, the README soft-claim defers the hard claim to Phase 27 (subject to coverage gate).

Output: 130+ JSON entries; help_data.rs rewritten with backward-compat shim; ≥4 parity tests; standalone scripts/docs-matrix crate; 2 new justfile recipes; CLAUDE.md v2.2 additions block; README soft-claim + matrix link.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md
@.planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md
@.planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md
@.planning/phases/25-cli-integration-and-documentation/25-01-SUMMARY.md
@.planning/phases/25-cli-integration-and-documentation/25-02-SUMMARY.md
@.planning/phases/25-cli-integration-and-documentation/25-03-SUMMARY.md
@CLAUDE.md

<interfaces>
Key types and contracts for this plan.

From hp41-cli/src/programs.rs (lines 7–24 — OnceLock precedent — PROJECT IDIOM to MIRROR):
- `use std::sync::OnceLock;`
- `static PROGRAMS_CACHE: OnceLock<Vec<SampleProgram>> = OnceLock::new();`
- `pub fn sample_programs() -> &'static [SampleProgram] { PROGRAMS_CACHE.get_or_init(build_all_programs) }`

From hp41-cli/src/help_data.rs (lines 1–342 — CURRENT shape — REPLACE):
- `pub const HELP_DATA: &[(&str, &str, &str)] = &[ /* 33 entries */ ];`
- Consumed in hp41-cli/src/ui.rs:285 — preserve consumer signature via a derivation shim OR rewrite the consumer.

From hp41-cli/Cargo.toml (lines 11–17 — deps EXIST, no changes needed):
hp41-core / ratatui 0.30 / crossterm 0.29 / clap 4 / serde (workspace) / serde_json (workspace) / dirs 6
Dev-deps (lines 20–22): tempfile 3 / sha2 0.10.

JSON schema per D-25.16 (canonical — DO NOT DEVIATE):
- `op_variant`: hp41-core Op:: PascalCase name (e.g. "Pi")
- `display_name`: HP-41 mnemonic (e.g. "PI")
- `category`: one of Stack / Arithmetic / Math / Trig / Registers / Alpha / Programming / Flags / Display / Print / Sound / Catalog / Synthetic / CardReader / Indirect / Conversion / MathPac / StatPac / TimePac / AdvantagePac
- `status`: "implemented" | "deferred-v3" | "na"
- `phase`: GSD phase ID string ("20".."24") or null
- `key_path`: CLI keystroke (e.g. "f--", "S", "XEQ \"X<>Y?\"") or null for internal/programmatic-only variants
- `description`: ≤80 chars
- `divergences`: optional list of strings (defaults to empty via `#[serde(default)]`)

ALL_OP_VARIANT_NAMES skeleton: see RESEARCH.md lines 742–780. Plan executor copies this verbatim into the parity test file. 130 entries.

INTERNAL_OP_VARIANTS skiplist (from RESEARCH same section):
- "PushNum" — numeric-literal entry, not a named ROM op
- "SyntheticByte" — hex-modal insertion, internal
Executor may extend the skiplist if `grep -n 'pub.* fn op_' hp41-core/src/ops/mod.rs` reveals additional internal Op variants — document additions in 25-04-SUMMARY.md.

Existing justfile recipes (lines 66–87 — REFERENCE for indentation/style — tabs not spaces):
gui-install / gui-dev / gui-build / gui-check / gui-ci

</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Create docs/hp41cv-functions.json (≥130 entries), rewrite help_data.rs with include_str!+OnceLock, smoke + parity tests</name>
  <files>docs/hp41cv-functions.json, hp41-cli/src/help_data.rs, hp41-cli/tests/phase25_help_data.rs, hp41-cli/tests/function_matrix_parity.rs</files>
  <read_first>
    - hp41-cli/src/programs.rs (lines 1–24 — OnceLock precedent to MIRROR verbatim for the help_data rewrite)
    - hp41-cli/src/help_data.rs (full file — current HELP_DATA structure, 33+ entries, 13 categories; understand the (&str, &str, &str) tuple shape consumed at ui.rs:285)
    - hp41-cli/src/ui.rs (lines 280–311 — render_help_overlay consumer; this plan either keeps it working via a derivation shim OR rewrites it to consume HelpEntry directly — executor's choice; if rewriting, update tests for render_help_overlay too)
    - hp41-core/src/ops/mod.rs (full file — enumerate the Op variants to populate the JSON content + ALL_OP_VARIANT_NAMES list)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md sections "Pattern 3: include_str! + OnceLock", "Function Matrix Schema", "CI parity test" (lines 742–835)
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (help_data.rs JSON-loaded migration section + function_matrix_parity.rs)
    - CLAUDE.md (D-25.17 says malformed-JSON .expect() is intentional; D-18 single-source-of-truth invariant)
  </read_first>
  <behavior>
    - docs/hp41cv-functions.json is a JSON array at the top level. Every entry is an object matching the schema in &lt;interfaces&gt;. Minimum 130 entries.
    - Status rules: "implemented" for every Op variant landed in Phases 1–24 (use ROADMAP + REQUIREMENTS for phase-ID assignment); "deferred-v3" for Module-Pac entries (MAT_PLUS / MAT_INV / DATE / TIME / CMPLX / ITG / SOLVE — ~15–20 entries with placeholder op_variant strings since no hp41-core variant exists); "na" reserved for future use.
    - Category must be one of the 20 enumerated strings in &lt;interfaces&gt;. Use the 13 existing categories from current help_data.rs as a starting point; add Flags / Display / Sound / Indirect / Conversion for Phases 21–24 ops; add MathPac / StatPac / TimePac / AdvantagePac for v3.x-deferred.
    - hp41-cli/src/help_data.rs is REWRITTEN: replace the existing `pub const HELP_DATA` with the JSON pipeline. Keep a backward-compat shim `pub fn help_data() -> Vec` of 3-tuples that derives from help_entries() if ui.rs:285 still consumes the legacy shape. If rewriting render_help_overlay to consume HelpEntry directly, update the ui.rs consumer too.
    - The `.expect("hp41cv-functions.json is malformed — fix the JSON")` panic at OnceLock init is INTENTIONAL per D-25.17 — malformed canonical data is a hard-build-blocker by design.
    - hp41-cli/tests/phase25_help_data.rs is a smoke-test file: assert help_entries() loads without panic; assert help_entries().len() ≥ 130 (Pitfall 7 mitigation); assert at least one entry per category appears; assert no duplicate op_variant strings.
    - hp41-cli/tests/function_matrix_parity.rs is the bidirectional drift catch per D-25.15. Implements the 4 tests from RESEARCH "CI parity test" section.
  </behavior>
  <action>
    Step 1 — Seed docs/hp41cv-functions.json: produce the canonical data file with one JSON object per HP-41CV ROM op. Iterate hp41-core/src/ops/mod.rs and produce an entry for each Op variant per the schema. For parameterized variants like Op::StoReg(u8), produce ONE entry with display_name "STO" and key_path describing the modal flow. For Op::Test(TestKind), produce 12 entries (one per TestKind variant) with display_name "X=Y?" etc. and key_path "f--" (etc.) for the 4 keyboard-reachable and `XEQ "X<>Y?"` for the 8 XEQ-only. For internal primitives (Op::PushNum, Op::SyntheticByte) — DO NOT add entries; they live in INTERNAL_OP_VARIANTS. Append v3.x-deferred Module-Pac entries (~15–20) with status "deferred-v3" and phase null. Total ≥ 130 entries. Pretty-printed JSON (4-space indent) to ease diff review.

    Step 2 — Rewrite hp41-cli/src/help_data.rs:
    - DELETE the legacy `pub const HELP_DATA` constant.
    - ADD `use std::sync::OnceLock;` and `use serde::Deserialize;` imports.
    - DEFINE `pub struct HelpEntry` with fields op_variant: String, display_name: String, category: String, status: String, phase: Option of String, key_path: Option of String, description: String, `#[serde(default)] divergences: Vec<String>`; derive Debug, Clone, Deserialize.
    - ADD `const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");`
    - ADD `static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();`
    - ADD `pub fn help_entries() -> &'static [HelpEntry]` accessor that uses `HELP_ENTRIES.get_or_init(|| serde_json::from_str(FUNCTIONS_JSON).expect("hp41cv-functions.json is malformed — fix the JSON"))`.
    - For backward compat with ui.rs:285, ADD `pub fn help_data() -> Vec<(&'static str, &'static str, &'static str)>` shim that maps help_entries() into 3-tuples (key_path-or-empty, display_name, description) — to get static-lifetime strings, leak the strings into &'static via a second OnceLock-cached structure. Document the trade-off in the function doc-comment. Alternative: rewrite ui.rs:285 (render_help_overlay) to consume HelpEntry directly. EXECUTOR PICKS the path with less code churn; document the choice in 25-04-SUMMARY.md.

    Step 3 — Update hp41-cli/src/ui.rs render_help_overlay (lines 280–311) to consume help_entries() (if direct rewrite chosen) OR keep using HELP_DATA via the help_data() shim. Either way: category-header rows (the existing `desc.starts_with("===")` check) should be derived from JSON `category` field grouping — sort entries by category, insert synthetic header rows between groups.

    Step 4 — Create hp41-cli/tests/phase25_help_data.rs with `#![allow(clippy::unwrap_used)]`. Tests:
    a. help_entries_loads_at_runtime — call help_entries(), assert returned slice is non-empty
    b. help_entries_count_meets_130_target — assert help_entries().len() ≥ 130 (Pitfall 7)
    c. help_entries_has_at_least_one_per_category — collect distinct categories; assert at least 13 distinct categories appear
    d. help_entries_has_no_duplicate_op_variants — collect op_variant strings; assert HashSet length equals Vec length
    e. help_entries_all_have_non_empty_description — assert every entry's description.len() > 0

    Step 5 — Create hp41-cli/tests/function_matrix_parity.rs with `#![allow(clippy::unwrap_used)]`. Implement the 4 tests from RESEARCH "CI parity test" (lines 742–835):
    a. test_op_inventory_count_matches_enum — assert ALL_OP_VARIANT_NAMES.len() == 130 (hand-curated, copy from RESEARCH lines 742–780)
    b. test_every_rom_op_has_matrix_entry — iterate ALL_OP_VARIANT_NAMES minus INTERNAL_OP_VARIANTS; assert each has a matching JSON entry by op_variant
    c. test_every_implemented_matrix_entry_has_op — iterate help_entries() filtered by status "implemented"; assert each op_variant is in ALL_OP_VARIANT_NAMES
    d. test_matrix_has_at_least_130_entries — assert help_entries().len() ≥ 130 with a descriptive message

    Both ALL_OP_VARIANT_NAMES and INTERNAL_OP_VARIANTS are `const &[&str]` defined at the top of the test file (copy from RESEARCH lines 742–788; document any executor-added internal variants in the SUMMARY).

    Use `.expect("reason")` outside test bodies. Tests are allowed `.unwrap()` via the `#![allow(clippy::unwrap_used)]` attribute.
  </action>
  <verify>
    <automated>cargo build -p hp41-cli && cargo test -p hp41-cli --test phase25_help_data && cargo test -p hp41-cli --test function_matrix_parity</automated>
  </verify>
  <acceptance_criteria>
    - File docs/hp41cv-functions.json exists and parses as a JSON array; entry count ≥ 130 (verify via `python3 -c 'import json; print(len(json.load(open("docs/hp41cv-functions.json"))))'`)
    - hp41-cli/src/help_data.rs no longer contains the legacy const: `grep -n "pub const HELP_DATA: &\\[(&str" hp41-cli/src/help_data.rs | grep -v '^[^:]*:[[:space:]]*//' | wc -l` returns 0
    - help_data.rs contains the OnceLock+include_str! idiom: `grep -n "include_str!.*hp41cv-functions.json" hp41-cli/src/help_data.rs` returns 1 line AND `grep -n "OnceLock<Vec<HelpEntry>>" hp41-cli/src/help_data.rs` returns 1 line
    - `grep -n "HELP_ENTRIES.get_or_init" hp41-cli/src/help_data.rs` returns 1 line in the help_entries() accessor
    - hp41-cli/tests/phase25_help_data.rs has ≥5 `#[test]` functions; `cargo test -p hp41-cli --test phase25_help_data` exits 0
    - hp41-cli/tests/function_matrix_parity.rs has 4 `#[test]` functions; `cargo test -p hp41-cli --test function_matrix_parity` exits 0
    - `cargo build -p hp41-cli` succeeds; include_str! correctly embeds the JSON
    - Manual sanity: `just run-cli`, press `?` to open help overlay; verify entries from the JSON render with category groupings
  </acceptance_criteria>
  <done>
    JSON canonical data exists (≥130 entries); help_data.rs uses OnceLock+include_str! per project precedent; smoke and parity tests GREEN; render_help_overlay continues to display entries with category grouping (either via shim or direct HelpEntry consumption); FN-CLI-03 closed; FN-DOC-01 (JSON existence + parity) progress.
  </done>
</task>

<task type="auto">
  <name>Task 2: Build scripts/docs-matrix/ standalone crate, just docs-matrix recipes, generated docs/hp41cv-function-matrix.md</name>
  <files>scripts/docs-matrix/Cargo.toml, scripts/docs-matrix/src/main.rs, docs/hp41cv-function-matrix.md, justfile</files>
  <read_first>
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md sections "just docs-matrix recipe" + "Function Matrix Schema → Generated Markdown shape"
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (scripts/docs-matrix section + justfile section)
    - justfile (lines 60–87 — existing recipe style; tab indentation; --manifest-path pattern in gui-ci)
    - hp41-gui/src-tauri/Cargo.toml (the nested-non-workspace pattern reference per CLAUDE.md "Root Cargo.toml members stay")
    - docs/hp41cv-functions.json (Task 1 output — the canonical data the bin reads)
    - CLAUDE.md (root `Cargo.toml members` invariant — NEVER add scripts/docs-matrix to root members)
  </read_first>
  <behavior>
    - scripts/docs-matrix/Cargo.toml is a STANDALONE crate: opens with an empty `[workspace]` stanza so it is excluded from the root workspace. Package name `docs-matrix`. Dependencies: serde 1 with derive feature, and serde_json 1. Edition 2021.
    - scripts/docs-matrix/src/main.rs is ≤120 LOC: defines an Entry struct mirroring HelpEntry (intentional duplication per RESEARCH "Don't Hand-Roll" — single-source-of-truth violation accepted because crossing the crate boundary would require depending on hp41-cli, which would loop). Reads JSON path from argv[1], writes Markdown path to argv[2]. render_markdown() produces the table per RESEARCH "Generated Markdown shape"; status rendering rules `implemented` → `✓ v2.x`, `deferred-v3` → `⏳ v3.x module`, `na` → `— N/A`; splits implemented entries from deferred-v3 entries into two sections per the schema.
    - justfile gains TWO new recipes per RESEARCH "justfile" (tab-indented):
      * `docs-matrix:` — runs `cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- docs/hp41cv-functions.json docs/hp41cv-function-matrix.md` (writes to the committed location)
      * `docs-matrix-check:` — runs the bin to a tmp file then `diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md` (Pitfall 8 — exit non-zero on mismatch)
    - docs/hp41cv-function-matrix.md is the generator's initial committed output. Run `just docs-matrix` once, commit the result. Verify it matches the schema in RESEARCH "Generated Markdown shape" (Op | Display | Category | Status | Phase | Key Path | Description columns; v3.x Deferred section).
    - Root Cargo.toml `members = ["hp41-core", "hp41-cli"]` REMAINS UNCHANGED. No additions to root workspace members.
  </behavior>
  <action>
    Step 1 — Create scripts/docs-matrix/Cargo.toml: place `[workspace]` empty stanza FIRST (excludes from root workspace), then `[package]` with name "docs-matrix", version "0.1.0", edition "2021", and `[dependencies]` with serde and serde_json per &lt;behavior&gt;. Do NOT add a `rust-version` field — this script is dev-only tooling.

    Step 2 — Create scripts/docs-matrix/src/main.rs:
    - `use serde::Deserialize;` and `use std::{env, fs};`
    - Entry struct mirrors HelpEntry from help_data.rs (op_variant, display_name, category, status, phase as Option of String, key_path as Option of String, description, `#[serde(default)]` divergences as Vec of String) with `#[derive(Debug, Deserialize)]`.
    - fn main(): collect args; read JSON file; parse via `serde_json::from_str::<Vec<Entry>>`; call render_markdown; write file.
    - fn render_markdown(entries) -> String: build the document with the header (# HP-41CV ROM Function Matrix + the > Generated from… blurb); two tables — one for implemented entries, one for deferred-v3. For each row: `| {op_variant} | {display_name} | {category} | {status_symbol} | {phase_or_dash} | {key_path_or_dash} | {description} |`. Sort entries by (category, op_variant) for stable diffs.
    - Use `.expect("read JSON")` / `.expect("parse JSON")` / `.expect("write MD")` per project's `.expect("reason")` style — but main is in a dev-tooling crate without `#![deny(clippy::unwrap_used)]`, so .unwrap() is acceptable here; prefer .expect() with brief reason strings for diagnosability.
    - Total ≤120 LOC.

    Step 3 — Generate docs/hp41cv-function-matrix.md by running `just docs-matrix` once (or `cargo run --manifest-path scripts/docs-matrix/Cargo.toml -- docs/hp41cv-functions.json docs/hp41cv-function-matrix.md`). Commit the result. The Markdown file MUST be present at this exact path so README links to it work.

    Step 4 — Update justfile: append (with a blank line above for separation) two new recipes per RESEARCH "just docs-matrix recipe". Tab indentation per existing style. Names: `docs-matrix` and `docs-matrix-check`. The docs-matrix-check recipe must exit non-zero on `diff` mismatch (the default behavior of `diff -u`).

    Step 5 — Verify the standalone crate isolation: run `grep -n "docs-matrix" Cargo.toml` (the ROOT Cargo.toml) — MUST return 0 lines (no member entry). Run `cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; m=json.load(sys.stdin); print([p["name"] for p in m["packages"]])'` and verify `docs-matrix` does NOT appear in the list.
  </action>
  <verify>
    <automated>just docs-matrix && just docs-matrix-check && cargo build --manifest-path scripts/docs-matrix/Cargo.toml</automated>
  </verify>
  <acceptance_criteria>
    - File scripts/docs-matrix/Cargo.toml exists with `[workspace]` empty stanza appearing before `[package]`: `head -5 scripts/docs-matrix/Cargo.toml` shows `[workspace]` ahead of `[package]`
    - File scripts/docs-matrix/src/main.rs exists and is ≤120 LOC: `wc -l scripts/docs-matrix/src/main.rs` returns a number ≤ 120
    - File docs/hp41cv-function-matrix.md exists and starts with `# HP-41CV ROM Function Matrix`: `head -1 docs/hp41cv-function-matrix.md` shows the title
    - The generated matrix lists ≥130 rows: `grep -c "^| " docs/hp41cv-function-matrix.md` returns ≥ 132 (header row + separator + 130 data rows)
    - Root Cargo.toml `members` UNCHANGED: `grep -A2 "^members" Cargo.toml` shows exactly `["hp41-core", "hp41-cli"]`
    - `just docs-matrix` exits 0 and re-writes docs/hp41cv-function-matrix.md to the same content (idempotent)
    - `just docs-matrix-check` exits 0 (no drift between committed and regenerated)
    - `cargo build --manifest-path scripts/docs-matrix/Cargo.toml` exits 0
    - Two new justfile recipes present: `grep -c "^docs-matrix" justfile` ≥ 2
  </acceptance_criteria>
  <done>
    Standalone non-workspace scripts/docs-matrix crate exists and builds; just docs-matrix regenerates the Markdown matrix idempotently; just docs-matrix-check is the CI drift-catch recipe; the committed docs/hp41cv-function-matrix.md mirrors the canonical JSON; root Cargo.toml unchanged per CLAUDE.md invariant; FN-DOC-01 (matrix generation pipeline) closed.
  </done>
</task>

<task type="auto">
  <name>Task 3: CLAUDE.md v2.2 additions block, README soft-claim, KEY_REF_TABLE regeneration, rustdoc cross-refs</name>
  <files>CLAUDE.md, README.md, hp41-cli/src/keys.rs, hp41-cli/src/ui.rs</files>
  <read_first>
    - CLAUDE.md (full file — locate the `### v2.1 additions (Keyboard authenticity, Phase 19)` block which is the structural reference per FN-DOC-02; append the new `### v2.2 additions` block immediately after)
    - README.md (full file — identify the project description / feature section where the soft claim goes)
    - hp41-cli/src/keys.rs (Plan 01–03 output — current KEY_REF_TABLE at lines 91+; this task either deletes the table OR regenerates it from JSON-derived data; executor's choice — document in 25-04-SUMMARY.md)
    - hp41-cli/src/ui.rs (consumer of KEY_REF_TABLE in render_right_panel)
    - docs/hp41cv-function-matrix.md (Task 2 output — README links to this path)
    - hp41-cli/src/help_data.rs (Task 1 output — help_entries() is the data source for KEY_REF_TABLE regeneration if executor chooses derivation)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md sections "CLAUDE.md Update Plan" + "README Soft-Claim Wording" + "Don't Hand-Roll" (KEY_REF_TABLE derivation recommended)
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (CLAUDE.md v2.2 additions section + README section + keys.rs KEY_REF_TABLE rewrite)
  </read_first>
  <behavior>
    - CLAUDE.md gains a new `### v2.2 additions (HP-41CV Feature Completeness, Phases 20–25)` block IMMEDIATELY after the existing `### v2.1 additions` block. Bullets cover (per RESEARCH "CLAUDE.md Update Plan"): v2.2 milestone scope; f-prefix one-shot CLI/GUI parity (D-25.1, D-25.6); Hybrid PendingInput struct-variants (D-25.11); JSON-canonical data flow (D-25.16); builtin_card_op 4→12 extension as the documented surgical exception to the "hp41-core FROZEN" rule (cleared per Discussion log); CLI/GUI invariants for IND-toggle via shift-0 (Pitfall 10); 12-conditional-tests keyboard reachability (FN-TEST-01, D-25.9). 80–140 chars per bullet; mirror the prose style of v2.1 additions.
    - README.md gains a "feature-complete" claim — soft per D-25.17. Suggested wording: "Implements the full HP-41CV ROM built-in function set (~130 ops) with documented divergences. See [HP-41CV function matrix](docs/hp41cv-function-matrix.md) for status per op and known hardware divergences." Place near the project description / features section.
    - KEY_REF_TABLE in hp41-cli/src/keys.rs (currently lines 91+) is either DELETED (and ui.rs's right-panel consumer is rewritten to read from help_entries() with key_path filter) OR regenerated as a runtime-derived function `pub fn key_ref_entries() -> impl Iterator<Item = (&'static str, &'static str)>` that lazily derives from help_entries() (matches OnceLock idiom from help_data.rs). Per D-25.18 (added 2026-05-14 in CONTEXT.md), a parallel hand-curated KEY_REF_TABLE is FORBIDDEN — the JSON canonical source is the single source of truth; this task closes the v1.x letter-binding cleanup by removing the parallel table entirely (or by reducing it to a thin JSON-derived view).
    - Rustdoc cross-refs in hp41-core public items remain accurate. Run `cargo doc --no-deps -p hp41-core` and verify it builds without broken intra-doc-link warnings; repair any that may have decayed since Phase 24.
  </behavior>
  <action>
    Step 1 — CLAUDE.md: append a new `### v2.2 additions (HP-41CV Feature Completeness, Phases 20–25)` section IMMEDIATELY after the existing v2.1 additions block. Use bullet-point style mirroring v2.1 — each bullet 80–140 chars. Cover these settled invariants:
    - Phase 20–24 ROM ops landed in hp41-core (10 math/stack + flags+display+sound + program-control+memory + ALPHA + indirect) — 90+ new variants
    - f-prefix one-shot model on hp41-cli (App.shift_armed); ALPHA overrides prefix per D-25.5; one-shot lifetime per D-25.4; CLI/GUI parity per D-25.6
    - Hybrid PendingInput struct-variants (D-25.11): FlagPrompt/RegisterPrompt collapse 18 logical modal arms via FlagTestKind + StoArithKind reuse; pending_prompt() exhaustive per FN-CLI-04
    - IND-toggle via shift-0 inside an open RegisterPrompt/FlagPrompt modal — hardware-faithful per QRG p.14 / Pitfall 10
    - JSON-canonical data flow (D-25.16): docs/hp41cv-functions.json → help_data.rs via include_str! + OnceLock; just docs-matrix regenerates docs/hp41cv-function-matrix.md
    - hp41-core builtin_card_op extended 4→12 entries as the documented surgical exception to the "hp41-core FROZEN" Phase 25 rule — cleared by user
    - All 12 HP-41CV conditional tests now keyboard-reachable per D-25.9 (4 via f-arith + 8 via XEQ-by-Name modal)
    - README soft-claim "feature-complete HP-41CV with documented divergences"; hard claim deferred to Phase 27 (coverage gate ≥95%)
    - SC-4 invariant unchanged: Phase 25 does NOT touch hp41-gui
    - CalcState save-file backward compat: NO new CalcState fields in Phase 25; v1.0–v2.1 save files load without migration

    Step 2 — README.md: insert the soft-claim sentence per &lt;behavior&gt; near the top-of-readme features list. Add the matrix link as a Markdown link. Verify the rendered link target is `docs/hp41cv-function-matrix.md` (relative path from repo root).

    Step 3 — KEY_REF_TABLE: open hp41-cli/src/keys.rs at lines 91+ (the table from Plan 01 still carries v1.x letter conventions). Per D-25.18 (added 2026-05-14 — the JSON canonical source is the single source of truth; a parallel hand-curated table is forbidden), execute ONE of:
    (a) DELETE the table and rewrite hp41-cli/src/ui.rs render_right_panel (or wherever it consumes KEY_REF_TABLE) to derive the right-panel content from `help_entries()` filtered by non-null `key_path`; OR
    (b) Re-implement KEY_REF_TABLE as a function `pub fn key_ref_entries() -> impl Iterator<Item = (&'static str, &'static str)>` that lazily derives from help_entries() (matches OnceLock idiom from help_data.rs).
    Choose ONE path and execute it cleanly per D-25.18. The right-panel discoverability behavior remains visually identical (key | description rows for every keyboard-reachable op). Cite D-25.18 explicitly in the function's doc-comment (the function is the auditable trace of the reinterpretation).

    Step 4 — Rustdoc: run `cargo doc --no-deps -p hp41-core` and capture any warnings. Fix obvious broken intra-doc links (e.g. `[Op::FlagTest]` should resolve; `[CalcState]` should resolve). Do NOT do a full doc-rewrite pass — only repair existing broken links so FN-DOC-04 (rustdoc cross-refs stay accurate) is met. If no broken links surface, record the no-op in the SUMMARY.
  </action>
  <verify>
    <automated>cargo build -p hp41-cli && cargo doc --no-deps -p hp41-core && grep -q "v2.2 additions" CLAUDE.md && grep -q "feature-complete HP-41CV" README.md && grep -q "hp41cv-function-matrix.md" README.md</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "### v2.2 additions" CLAUDE.md` returns 1 (exactly one new section header)
    - The new section appears AFTER the v2.1 additions block: verify by checking that `grep -n "### v2.1 additions" CLAUDE.md` line number is LESS than `grep -n "### v2.2 additions" CLAUDE.md` line number
    - New CLAUDE.md section has ≥8 bullet points: verify by counting `^- ` lines in the block (use `sed` to extract the v2.2 block between the two headers, then count)
    - `grep -q "feature-complete HP-41CV" README.md` succeeds
    - `grep -q "hp41cv-function-matrix.md" README.md` succeeds (the matrix link is present)
    - hp41-cli/src/keys.rs no longer has the old v1.x KEY_REF_TABLE entries — verify by `grep -c "Cosine\\|Tangent\\|Natural log" hp41-cli/src/keys.rs | grep -v '^[^:]*:[[:space:]]*//' | wc -l` returns 0 (description strings that referenced v1.x letter conventions are gone). Either path (a) or (b) from &lt;behavior&gt; produces this outcome.
    - `cargo build -p hp41-cli` and `cargo doc --no-deps -p hp41-core` BOTH exit 0
    - `cargo doc --no-deps -p hp41-core 2>&1 | grep -cE "warning:.*broken|warning:.*unresolved"` returns 0 (no broken intra-doc links)
    - `cargo clippy -p hp41-cli -- -D warnings` passes
  </acceptance_criteria>
  <done>
    CLAUDE.md ships the v2.2 additions block (≥8 bullets, mirrors v2.1 style); README.md ships the soft "feature-complete HP-41CV with documented divergences" claim + matrix.md link; KEY_REF_TABLE is regenerated (or deleted+consumer rewritten) so the right-panel reads from the JSON canonical source; rustdoc compiles clean; FN-DOC-02 + FN-DOC-03 + FN-DOC-04 CLOSED.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 4: Key-coverage parity test — verifiable closure of FN-CLI-01 from the JSON pipeline</name>
  <files>hp41-cli/tests/key_coverage.rs</files>
  <read_first>
    - docs/hp41cv-functions.json (Task 1 output — the canonical data with key_path strings)
    - hp41-cli/src/keys.rs (Plan 01–03 output — `key_to_op`, `shifted_key_to_op`, `xeq_by_name_local_resolve` signatures and call shapes)
    - hp41-cli/src/app.rs (Plan 01–02 output — `App::new` constructor + modal-opener paths for the parameterized ops)
    - hp41-cli/tests/phase25_keyboard.rs (Plan 01 output — `key(c)` helper + App-construction scaffold to mirror)
    - hp41-cli/tests/function_matrix_parity.rs (Task 1 output — `help_entries()` accessor to iterate)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"CI parity test" + §"Wave 0 Gaps" (`key_coverage.rs` listed)
    - .planning/phases/25-cli-integration-and-documentation/25-VALIDATION.md row FN-CLI-01 `cargo test -p hp41-cli --test key_coverage`
    - .planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md D-25.18 (KEY_REF_TABLE is JSON-derived; key_coverage is the parity guard)
  </read_first>
  <behavior>
    - For every entry in `docs/hp41cv-functions.json` with `status == "implemented"` AND `key_path != null`, the test parses the `key_path` keystroke sequence and asserts dispatch reaches a known `Op::` variant.
    - Keystroke parsing supports the three keyboard-path classes already shipped by Plans 01–03:
      (a) Primary positions (e.g. `"+"`, `"-"`, `"ENTER"`, `"BACKSPACE"`, `"n"`, `"r"`, `"x"`, `"l"`, `"p"`, `"u"`, `"%"`) — drives `keys::key_to_op` and asserts `Some(Op::*)` returned (no `None`, no panic).
      (b) f-shifted positions (e.g. `"f--"`, `"f-+"`, `"f-*"`, `"f-/"`, `"f-7"` for SF opener, etc.) — drives `keys::shifted_key_to_op` (note the &mut App signature from Plan 02) and asserts EITHER `Some(Op::*)` OR a populated `app.pending_input` (modal opener).
      (c) Modal-opener primaries (e.g. `"S"`, `"R"`, `"X"`) — drives the full `App::handle_key` path and asserts `app.pending_input.is_some()` after the keystroke.
      (d) XEQ-by-Name (e.g. `"XEQ \"X<>Y?\""`) — drives `xeq_by_name_local_resolve(name)` (Plan 03) and asserts `Some(Op::Test(TestKind::*))` for the 8 conditional-test mnemonics; for the 4 v2.1 card-reader names (`WPRGM`/`RDPRGM`/`WDTA`/`RDTA`) falls through to `hp41_core::ops::program::builtin_card_op` and asserts the matching `Op::Wprgm/Rdprgm/Wdta/Rdta`.
    - Total implemented-and-keypath entries probed: ≥80 (the lower bound — actual count depends on Task 1's JSON authoring, but the test asserts a minimum so an empty/short JSON cannot pass silently — Pitfall 7 belt-and-braces).
    - No `Op::InvalidOp` accepted as a passing dispatch (it is not a real variant in this codebase; explicit `HpError::InvalidOp` is the corresponding error path and is a TEST FAILURE if reached).
    - No `.unwrap()`-induced panic during the probe loop — every dispatch site uses `.expect("…")` with a per-key diagnostic message that names the offending `op_variant` + `key_path` for fast failure triage.
    - Per D-25.18 (added 2026-05-14 in CONTEXT.md), this test IS the verifiable closure of FN-CLI-01 — no parallel hand-curated KEY_REF_TABLE; `help_data.rs::help_entries()` is the discovery surface; `key_coverage.rs` is the binding-correctness guard.
  </behavior>
  <action>
    Step 1 — Create `hp41-cli/tests/key_coverage.rs` with `#![allow(clippy::unwrap_used)]` at module head (test files exempt per CLAUDE.md). Imports mirror `phase25_keyboard.rs` (Plan 01): `crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers}`, `hp41_cli::app::App`, `hp41_cli::help_data::help_entries`, `hp41_cli::keys::{self, xeq_by_name_local_resolve}`, `hp41_core::ops::{Op, TestKind}`, plus the per-test helper `fn key(c: char) -> KeyEvent` from Plan 01.

    Step 2 — Implement a `parse_key_path(&str) -> KeyPath` enum-or-tagged-struct helper (TUI-local to the test file) that returns one of `Primary(KeyEvent)`, `FShifted(KeyEvent)`, `ModalOpener(KeyEvent)`, or `XeqByName(String)`. Parsing rules: `"XEQ \"NAME\""` → `XeqByName("NAME")`; strings starting with `"f-"` → `FShifted(KeyEvent for the char after "f-")`; single-char strings `"S"`/`"R"`/`"X"` → `ModalOpener(key(c))`; everything else with a single visible char → `Primary(key(c))`; special tokens `"ENTER"`/`"BACKSPACE"`/`"ESC"` → `Primary(KeyEvent::new(KeyCode::Enter|Backspace|Esc, KeyModifiers::empty()))`. Unknown shapes → return `None` from the helper and the test SKIPS the entry with a logged-once warning (the JSON may later contain key_path values the test doesn't yet parse — Phase 26 territory).

    Step 3 — Implement the main `#[test] fn key_coverage_implemented_entries_dispatch()` test. Loop over `help_entries()`; filter to `status == "implemented" && key_path.is_some()`; for each entry call `parse_key_path` and dispatch per class:
    - `Primary(k)` → `assert!(keys::key_to_op(k, &app).is_some(), "{} via {}: key_to_op returned None", entry.op_variant, key_path)`.
    - `FShifted(k)` → set `app.shift_armed = true`; call `keys::shifted_key_to_op(k, &mut app)`; assert EITHER returned `Some(Op::*)` OR `app.pending_input.is_some()` (modal opener path) — fail with `"{} via f-{}: neither dispatched nor opened modal"`.
    - `ModalOpener(k)` → call `App::handle_key` end-to-end; assert `app.pending_input.is_some()` after the call.
    - `XeqByName(name)` → first try `xeq_by_name_local_resolve(&name)`; if `Some(op)` accept; else try `hp41_core::ops::program::builtin_card_op(&name)`; if `Some(op)` accept; else fail with `"{} via XEQ-by-Name: no resolver matched"`.

    Step 4 — Add a count-bound assertion AFTER the loop: `assert!(probed >= 80, "key_coverage probed only {} entries — JSON is empty or filter is wrong", probed);` — Pitfall 7 belt-and-braces.

    Step 5 — Tag the test file with a top-of-file doc-comment citing FN-CLI-01 + D-25.18: `//! key_coverage.rs — FN-CLI-01 verifiable-closure test per D-25.18. Every implemented JSON entry with non-null key_path dispatches to a known Op variant via key_to_op / shifted_key_to_op / modal-opener / xeq_by_name path. No InvalidOp, no panics.`

    Use `.expect("reason")` outside test bodies; `.unwrap()` permitted inside `#[test]` bodies (module has `#![allow(clippy::unwrap_used)]`).
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test key_coverage</automated>
  </verify>
  <acceptance_criteria>
    - File `hp41-cli/tests/key_coverage.rs` exists with `#![allow(clippy::unwrap_used)]` at top
    - File starts with a top-of-file doc-comment citing FN-CLI-01 and D-25.18: `grep -n "FN-CLI-01" hp41-cli/tests/key_coverage.rs | head -1` returns 1 line
    - At least one `#[test]` function present: `grep -c "^#\[test\]" hp41-cli/tests/key_coverage.rs` ≥ 1
    - `cargo test -p hp41-cli --test key_coverage` exits 0 with the `probed >= 80` assertion satisfied
    - `cargo clippy -p hp41-cli --tests -- -D warnings` passes
    - Test runs AFTER the JSON authoring task (Task 1) — depends on `docs/hp41cv-functions.json` being populated; no separate plan-level dependency change (Plan 04 already depends on Plans 01–03; Task 4 only depends on Task 1 within this plan)
  </acceptance_criteria>
  <done>
    `hp41-cli/tests/key_coverage.rs` is the FN-CLI-01 verifiable-closure test per D-25.18; every implemented JSON entry with `key_path != null` dispatches through Plan 01–03's keyboard / modal / XEQ-by-Name infrastructure to a known `Op::` variant; ≥80 entries probed; no InvalidOp, no panics; closure is verifiable from `cargo test -p hp41-cli --test key_coverage` on every CI run.
  </done>
</task>

</tasks>

<verification>
- `cargo test -p hp41-cli --test phase25_help_data` exits 0 with ≥5 tests GREEN.
- `cargo test -p hp41-cli --test function_matrix_parity` exits 0 with 4 tests GREEN.
- `cargo test -p hp41-cli --test key_coverage` exits 0 with ≥80 entries probed (FN-CLI-01 verifiable closure per D-25.18).
- `cargo test -p hp41-cli --test phase25_keyboard` (Plan 01) AND `--test phase25_pending_input` (Plan 02) AND `--test phase25_xeq_by_name` (Plan 03) ALL exit 0 — no regressions.
- `just docs-matrix-check` exits 0 — no drift between canonical JSON and committed Markdown.
- `cargo build --manifest-path scripts/docs-matrix/Cargo.toml` exits 0.
- `cargo doc --no-deps -p hp41-core` exits 0 with no broken intra-doc link warnings.
- `just ci` (full workspace) GREEN.
- `cargo clippy --workspace -- -D warnings` passes.
- `grep -c "### v2.2 additions" CLAUDE.md` returns 1; `grep -q "feature-complete HP-41CV" README.md && grep -q "hp41cv-function-matrix.md" README.md`.
- Root Cargo.toml `members` UNCHANGED: `grep -A1 "^members" Cargo.toml` shows `["hp41-core", "hp41-cli"]`.
- Manual smoke: `just run-cli`, press `?` — help overlay reads from JSON; press `S`/`R`/etc. — modals open with right key_path strings; navigate to right-panel — discoverability list shows current bindings.
</verification>

<success_criteria>
- docs/hp41cv-functions.json exists with ≥130 entries per D-25.16 schema; categories cover all 20 enumerated strings.
- hp41-cli/src/help_data.rs uses include_str! + OnceLock per project precedent; legacy HELP_DATA const is gone; backward-compat shim OR ui.rs rewrite preserves the help-overlay functionality.
- scripts/docs-matrix/ standalone crate generates the Markdown matrix; root Cargo.toml unchanged.
- justfile gains 2 new recipes (docs-matrix + docs-matrix-check); CI catches drift via diff per Pitfall 8.
- function_matrix_parity.rs runs 4 bidirectional drift tests (Pitfall 6 mitigation); ALL GREEN.
- CLAUDE.md ships the v2.2 additions block per FN-DOC-02.
- README.md ships the soft "feature-complete HP-41CV with documented divergences" claim + matrix link per D-25.17.
- KEY_REF_TABLE regeneration OR deletion+derivation completes the v1.x letter-binding cleanup started in Plan 01.
- Rustdoc cross-refs in hp41-core are accurate per FN-DOC-04.
- FN-CLI-01 (verifiable closure via key_coverage.rs) + FN-CLI-03 + FN-DOC-01 + FN-DOC-02 + FN-DOC-03 + FN-DOC-04 CLOSED.
- All Wave-0 tests + Plan-01/02/03 regression tests GREEN.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| docs/hp41cv-functions.json → include_str! → help_data.rs OnceLock parse | Committed-in-repo JSON is trusted; malformed JSON is a build-time/startup hard-blocker by design per D-25.17 |
| scripts/docs-matrix bin → docs/hp41cv-function-matrix.md | Dev-tooling, executed by `just docs-matrix`; output diffed in CI |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-25-13 | Tampering | Hand-curated JSON drifts from Op enum reality | mitigate | function_matrix_parity.rs runs 4 bidirectional drift tests on every CI build (Pitfall 6) |
| T-25-14 | Denial of Service | Malformed JSON crashes CLI startup with `.expect()` panic | accept | INTENTIONAL per D-25.17 — malformed canonical data is a hard-build-blocker by design; smoke test phase25_help_data catches empty/short JSON at CI time |
| T-25-15 | Tampering | Generated docs/hp41cv-function-matrix.md drifts from JSON (commit gap) | mitigate | just docs-matrix-check CI recipe diffs regenerated vs committed; non-zero exit on drift per Pitfall 8 |
| T-25-16 | Information Disclosure | scripts/docs-matrix accidentally added to root workspace members | mitigate | scripts/docs-matrix/Cargo.toml's empty `[workspace]` stanza excludes it; acceptance criterion verifies `cargo metadata` does NOT list `docs-matrix` |
| T-25-17 | Information Disclosure | Future Op variant added without ALL_OP_VARIANT_NAMES update | mitigate | test_op_inventory_count_matches_enum asserts exact count of 130; any new variant forces planner+developer to update both lists in the same commit |
</threat_model>

<output>
After completion, create `.planning/phases/25-cli-integration-and-documentation/25-04-SUMMARY.md` per execute-plan template. Record: the executor's choice of help_data shim path (3-tuple shim vs direct HelpEntry consumer rewrite); KEY_REF_TABLE handling (delete vs derive); the final entry count of docs/hp41cv-functions.json (must be ≥130); the list of any internal Op variants ADDED to INTERNAL_OP_VARIANTS skiplist beyond the original 2 (PushNum, SyntheticByte); the v2.2 milestone status — Phase 25 CLOSED with all 9 requirement IDs (FN-TEST-01, FN-CLI-01..04, FN-DOC-01..04) covered.
</output>
