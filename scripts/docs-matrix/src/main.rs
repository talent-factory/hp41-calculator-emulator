//! Generates `docs/hp41cv-function-matrix.md` from `docs/hp41cv-functions.json`.
//!
//! Usage: `docs-matrix <input.json> <output.md>`
//!
//! Invoked by the `just docs-matrix` recipe (regenerate-and-write) and by
//! `just docs-matrix-check` (regenerate to a temp file, diff vs committed
//! — Pitfall 8 CI drift guard).
//!
//! This is a standalone (non-workspace) crate per CLAUDE.md's "Root
//! Cargo.toml members stays ["hp41-core", "hp41-cli"]" invariant. The
//! `Entry` struct mirrors `hp41_cli::help_data::HelpEntry` schema — the
//! duplication is deliberate (per RESEARCH §"Don't Hand-Roll") because
//! depending on hp41-cli would create a circular dep and pull tauri-free
//! workspace crates into a dev-only tooling build.

use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Deserialize)]
struct Entry {
    op_variant: String,
    display_name: String,
    category: String,
    status: String,
    phase: Option<String>,
    key_path: Option<String>,
    description: String,
    #[serde(default)]
    #[allow(dead_code)]
    divergences: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: docs-matrix <input.json> <output.md>");
        std::process::exit(2);
    }
    let json_path = &args[1];
    let md_path = &args[2];

    let json = fs::read_to_string(json_path).unwrap_or_else(|e| panic!("read {json_path}: {e}"));
    let entries: Vec<Entry> =
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse {json_path}: {e}"));

    let md = render_markdown(&entries);
    fs::write(md_path, md).unwrap_or_else(|e| panic!("write {md_path}: {e}"));
}

fn render_markdown(entries: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str("# HP-41CV ROM Function Matrix\n\n");
    out.push_str("> Generated from `docs/hp41cv-functions.json` via `just docs-matrix`.\n");
    out.push_str("> Edit the JSON, regenerate this file, commit both.\n\n");

    let (implemented, deferred): (Vec<&Entry>, Vec<&Entry>) =
        entries.iter().partition(|e| e.status != "deferred-v3");

    out.push_str("## Implemented (v2.x)\n\n");
    render_table(&mut out, &implemented);

    out.push_str("\n## v3.x Deferred (Module Pacs)\n\n");
    if deferred.is_empty() {
        out.push_str("_None._\n");
    } else {
        render_table(&mut out, &deferred);
    }

    out
}

fn render_table(out: &mut String, entries: &[&Entry]) {
    out.push_str("| Op | Display | Category | Status | Phase | Key Path | Description |\n");
    out.push_str("|----|---------|----------|--------|-------|----------|-------------|\n");

    let mut sorted: Vec<&&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.op_variant.cmp(&b.op_variant))
    });

    for e in sorted {
        let status = match e.status.as_str() {
            "implemented" => "✓ v2.x",
            "deferred-v3" => "⏳ v3.x module",
            "na" => "— N/A",
            other => other,
        };
        let phase = e.phase.as_deref().unwrap_or("—");
        let key = match &e.key_path {
            Some(k) => format!("`{}`", escape_pipe(k)),
            None => "—".to_string(),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            escape_pipe(&e.op_variant),
            escape_pipe(&e.display_name),
            escape_pipe(&e.category),
            status,
            escape_pipe(phase),
            key,
            escape_pipe(&e.description),
        ));
    }
}

/// Escape '|' so Markdown table rows render correctly even when a value
/// contains a pipe character (none in the v2.2 dataset, but defensive for
/// future entries).
fn escape_pipe(s: &str) -> String {
    s.replace('|', "\\|")
}
