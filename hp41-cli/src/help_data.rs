//! JSON-loaded HP-41CV function help data (D-25.16 canonical pipeline).
//!
//! `docs/hp41cv-functions.json` is the **single source of truth** for both
//! the CLI `?` help overlay AND the generated function matrix
//! (`docs/hp41cv-function-matrix.md` produced by `scripts/docs-matrix/`).
//! This module embeds the JSON at compile time via `include_str!` and
//! lazy-parses it once via `std::sync::OnceLock` per the project precedent
//! in `hp41-cli/src/programs.rs:19,22`.
//!
//! **Hard-build-blocker semantics (D-25.17):** a malformed JSON file fails
//! the OnceLock init with `.expect("hp41cv-functions.json is malformed")`.
//! This is intentional — canonical data files must not be empty / malformed.
//! The smoke test `phase25_help_data::help_entries_count_meets_130_target`
//! also catches empty-file commits at CI time (RESEARCH Pitfall 7).
//!
//! **D-25.18:** Both the right-panel discoverability listing (`KEY_REF_TABLE`
//! consumers) and the `?` overlay derive from the SAME `help_entries()`
//! call — no parallel hand-curated tables permitted.

use std::sync::OnceLock;

use serde::Deserialize;

/// XROM module descriptor embedded in a [`HelpEntry`] row (C-28.3).
///
/// Present for Math Pac I entries; absent (`None`) for v2.2 built-in entries.
/// `#[serde(default)]` on the `xrom` field of `HelpEntry` means v2.2 JSON
/// (no `xrom` key) parses unchanged — schema extension is additive.
///
/// - `module` — human-readable module name (e.g. `"Math 1"`).
/// - `module_id` — HP-41C hardware XROM module ID (`7` for Math Pac I).
/// - `function_id` — 1-indexed position of this entry in `MATH_1.ops`.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XromEntry {
    pub module: String,
    pub module_id: u8,
    pub function_id: u16,
}

/// One row in the canonical HP-41CV function table.
///
/// Schema per D-25.16:
/// - `op_variant` — hp41-core `Op::` PascalCase name (e.g. `"Pi"`). For the
///   8 XEQ-by-Name-only conditional tests, this is an `_XEQ`-suffixed alias
///   (e.g. `"XNeY_XEQ"`) that resolves to `Op::Test(TestKind::XNeY)` via
///   `keys::xeq_by_name_local_resolve` and `builtin_card_op`. For v3.x-deferred
///   Module-Pac entries, this is a placeholder ID (no real Op variant exists).
/// - `display_name` — HP-41 mnemonic as shown on the display (e.g. `"PI"`).
/// - `category` — one of the 20 enumerated categories (see CONTEXT D-25.16).
/// - `status` — `"implemented"`, `"deferred-v3"`, or `"na"`.
/// - `phase` — GSD phase ID string (e.g. `"21"`) or `None` for v3.x.
/// - `key_path` — CLI keystroke (e.g. `"f-7"`, `"S"`, `"XEQ \"X<>Y?\""`) or
///   `None` for internal / programmatic-only variants.
/// - `description` — <= 80 chars, suitable for the `?` overlay row.
/// - `divergences` — optional free-form notes about HP-41 hardware divergences.
/// - `xrom` — optional XROM descriptor (present for Math Pac I entries; `None`
///   for v2.2 built-in entries). `#[serde(default)]` ensures backward compat.
// `op_variant`, `status`, `phase`, `divergences` are not read inside src/ —
// only by integration tests under `tests/` (cross-crate, opaque to dead-code
// analysis) and by the `scripts/docs-matrix/` bin (deliberate JSON-schema
// duplication per RESEARCH §"Don't Hand-Roll"). They MUST be deserialized so
// the schema remains the single source of truth for both consumers.
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
    /// XROM descriptor (C-28.3). `None` for v2.2 built-ins; `Some(_)` for
    /// Math Pac I entries. `#[serde(default)]` keeps v2.2 JSON parsing clean.
    #[serde(default)]
    pub xrom: Option<XromEntry>,
}

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

/// Compile-time-embedded canonical data file for Math Pac I (D-29.1 / D-29.2).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-math1-functions.json` at the repo root.
const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");

static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed Math Pac I help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-math1-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior. Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the Math Pac I pool. Use [`help_entries_all`]
/// for the merged pool (v2.2 + Math Pac I) in UI rendering paths.
pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}

/// Merged accessor: chains both JSON pools (v2.2 built-ins + Math Pac I)
/// in order. This is the **single source of truth** for:
/// - The `?` help overlay (`ui::render_help_overlay` via `help_overlay_rows`)
/// - The right-panel discoverability listing (`keys::key_ref_entries`)
/// - The `function_matrix_parity.rs` full-pool sweep
///
/// The narrow accessors [`help_entries`] and [`help_entries_math1`] are
/// retained for per-pool surgical tests (130-target smoke test, 45-target
/// smoke test) and MUST NOT be removed.
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries().iter().chain(help_entries_math1().iter())
}

/// Render a list of `(key, op, desc)` 3-tuples in the legacy `HELP_DATA`
/// shape with category-header rows interleaved. Used by
/// `ui::render_help_overlay` so the existing table-rendering code keeps
/// working over the new JSON-derived data source.
///
/// Entries are sorted by category (in their first-appearance order in the
/// JSON) so the overlay groups them naturally; within a category, entries
/// keep the JSON's declared order.
///
/// **Static-lifetime trade-off:** the legacy `HELP_DATA` const used
/// `&'static str` directly. The JSON-loaded entries are `String`-owned, so
/// the consumer (`ui::render_help_overlay`) now receives a borrowed
/// `&HelpEntry` slice via [`help_entries`] and reads `&str` slices directly
/// from those strings. Callers that want the legacy 3-tuple shape should use
/// the [`help_overlay_rows`] helper which returns owned `String`s grouped by
/// category with synthetic `=== {category} ===` header rows interleaved.
pub fn help_overlay_rows() -> Vec<HelpRow> {
    let entries = help_entries();
    let mut categories: Vec<&str> = Vec::new();
    for entry in entries {
        if !categories.iter().any(|c| *c == entry.category) {
            categories.push(&entry.category);
        }
    }

    let mut rows: Vec<HelpRow> = Vec::with_capacity(entries.len() + categories.len());
    for cat in categories {
        rows.push(HelpRow {
            key: String::new(),
            op: String::new(),
            desc: format!("=== {cat} ==="),
        });
        for entry in entries.iter().filter(|e| e.category == cat) {
            let key = entry.key_path.clone().unwrap_or_default();
            rows.push(HelpRow {
                key,
                op: entry.display_name.clone(),
                desc: entry.description.clone(),
            });
        }
    }
    rows
}

/// One row of the help overlay table, produced by [`help_overlay_rows`].
/// Category headers carry `desc == "=== <name> ==="` with empty `key`/`op`.
#[derive(Debug, Clone)]
pub struct HelpRow {
    pub key: String,
    pub op: String,
    pub desc: String,
}
