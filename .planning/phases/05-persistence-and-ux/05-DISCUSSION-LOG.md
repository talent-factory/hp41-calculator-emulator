# Phase 5: Persistence & UX - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 5-Persistence-and-UX
**Areas discussed:** All (user delegated all decisions to Claude → best practices applied)

---

## Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| State file strategy | Save path, auto-save, named files | ✓ (Claude) |
| STO/RCL + ALPHA input UX | Modal entry deferred from Phase 4 | ✓ (Claude) |
| Help overlay design | Searchable vs scrollable, content scope | ✓ (Claude) |
| Sample program loading UI | TUI menu vs CLI arg | ✓ (Claude) |

**User's choice:** "Ich überlasse dir sämtliche Entscheidungen (-> Best Practices)" — all decisions delegated to Claude.

---

## Claude's Discretion

All areas: user explicitly delegated. Best practices applied:

- **State file location:** `~/.hp41/autosave.json` — standard hidden config dir in home; `--state-file` CLI override for named saves. Simpler than XDG spec; cross-platform via `dirs` crate.
- **STO/RCL entry:** Two-digit auto-dispatch pattern (no Enter required) — matches HP-41 hardware behavior. `pending_input` in App (not CalcState) keeps transient UI state out of serialized state.
- **ALPHA mode routing:** Global key reroute when `state.alpha_mode == true` — matches HP-41 ALPHA mode behavior exactly.
- **Help overlay:** Scrollable table (not searchable) — 130 entries is manageable with category headers; search adds significant complexity for marginal gain at v1.0.
- **Sample programs:** 10 programs as `const` Op arrays — compile-time checked, zero runtime file loading.
- **USER mode:** `BTreeMap<char, String>` for deterministic JSON serialization; Ctrl+A two-step assignment dialog via pending_input chain.

## Deferred Ideas

- Help text search — v1.1 quality-of-life
- In-TUI "Save As" dialog — v1.1; CLI arg sufficient
- ALPHA mode special characters — v1.1
- GTO label-entry dialog for R/S — v1.1 polish
