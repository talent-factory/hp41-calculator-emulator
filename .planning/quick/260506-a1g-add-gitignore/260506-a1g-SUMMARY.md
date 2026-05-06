---
quick_id: 260506-a1g
slug: add-gitignore
description: "Add .gitignore for Rust/Cargo workspace"
date: "2026-05-06"
status: complete
---

# Summary: Add .gitignore

## What was done

Created `/.gitignore` covering all relevant artifact categories for this Rust/Cargo workspace project.

## Result

`git status` no longer shows `target/` (Rust build output) or `.DS_Store` (macOS) as untracked.

## Sections added

| Section | Entries |
|---------|---------|
| Rust/Cargo | `/target/`, `**/*.rs.bk` |
| Coverage | `lcov.info`, `tarpaulin-report.html`, `*.profraw`, `*.profdata`, `/coverage/` |
| Insta snapshots | `**/*.pending-snap` |
| OS files | `.DS_Store`, `Thumbs.db`, `Desktop.ini` |
| Editors | `.idea/` (`.vscode/` deliberately kept — see note in file) |
| Env/secrets | `.env`, `.env.local`, `.env.*.local` |
| Tauri (future) | `/dist/`, `node_modules/`, `src-tauri/target/`, release bundles |
| Claude Code | `.claude/` |

## Note on .vscode/

`.vscode/` is intentionally NOT ignored — if the team adds shared launch configs or tasks, those can be committed. Individual devs who want to ignore it locally can add it to `~/.config/git/ignore`.
