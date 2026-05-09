# Phase 13: Workspace Skeleton - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 13-Workspace Skeleton
**Areas discussed:** Scaffolding approach, Phase 13 frontend scope, just recipe additions

---

## Scaffolding Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Manual creation | Write every file by hand: Cargo.toml, tauri.conf.json, index.html, vite.config.ts, package.json, etc. Full control, no cleanup needed, exactly what the planner specifies. | ✓ (Claude) |
| cargo tauri init | Official Tauri CLI scaffold, then modify/delete boilerplate. Faster start but requires cleanup pass. | |
| npm create tauri-app | npm-based scaffolding tool — generates the full project. Also requires cleanup. | |

**User's choice:** "you decide" — Claude selected manual creation.
**Notes:** The v2.0 research already specifies exact file contents; a scaffold would generate files requiring cleanup and may make different structural assumptions.

---

### Bundle Identifier

| Option | Description | Selected |
|--------|-------------|----------|
| ch.talent-factory.hp41 | Derived from user domain (talent-factory.ch). Correct reverse-DNS format. | ✓ |
| io.github.hp41-emulator | Open-source style tied to GitHub repo. | |

**User's choice:** `ch.talent-factory.hp41`
**Notes:** Avoids Pitfall #4 — default scaffold `com.tauri.dev` causes silent runtime issues (wrong icons, missing assets).

---

## Phase 13 Frontend Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full npm scaffold | Install all npm packages now (React 19, TypeScript, Vite, Tailwind 4, @tauri-apps/api). App.tsx renders empty div. Phase 14 can immediately focus on Tauri commands. | ✓ |
| Bare HTML only | index.html with `<h1>HP-41 Calculator</h1>`, no npm packages. Phase 14 adds React/Tailwind. | |

**User's choice:** Full npm scaffold
**Notes:** Phase 14 (IPC Layer) benefits from having the React environment ready immediately.

---

### Tailwind Integration Timing

| Option | Description | Selected |
|--------|-------------|----------|
| @import tailwindcss from day one | Wire Tailwind in index.css and vite.config.ts in Phase 13. Phase 15 uses utilities immediately. | ✓ |
| Bare CSS, Tailwind in Phase 15 | Keep Phase 13 minimal; Tailwind added when display components need it. | |

**User's choice:** Tailwind wired from day one
**Notes:** Since Tailwind is installed anyway (as a devDependency), wiring it costs nothing and removes a step from Phase 15.

---

## just Recipe Additions

| Option | Description | Selected |
|--------|-------------|----------|
| gui-install | cd hp41-gui && npm install — first-time setup recipe. | ✓ |
| gui-dev | cd hp41-gui && npm run tauri dev — development window (required by success criteria). | ✓ (required) |
| gui-build | cd hp41-gui && npm run tauri build — production bundle. | ✓ |
| gui-check | cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml — Rust type-check. | ✓ |

**User's choice:** All four recipes (gui-install, gui-dev, gui-build, gui-check)
**Notes:** `just ci` remains completely unchanged — nested workspace isolation ensures `cargo clippy/test/llvm-cov --workspace` never touches hp41-gui.

---

### gui-check Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Rust only (cargo check) | Fast, catches Rust type errors. TypeScript checked by Vite dev server. | ✓ (Claude) |
| Both Rust + TypeScript | cargo check AND npx tsc --noEmit. Full static analysis, slower. | |

**User's choice:** "you decide" — Claude selected Rust only.
**Notes:** TypeScript validation deferred to Phase 18 CI/CD job where a proper frontend CI structure can be defined.

---

## Claude's Discretion

- **Scaffolding approach:** Manual creation (user delegated; Claude chose for control and cleanliness)
- **gui-check scope:** Rust-only cargo check (user delegated; Claude chose for speed; TypeScript deferred to Phase 18)
- **Window dimensions:** Tauri defaults (~800×600); final dimensions deferred to Phase 16 (SVG Skin)
- **Package name:** `hp41-gui` in Cargo.toml; `productName: "HP-41 Calculator"` in tauri.conf.json

## Deferred Ideas

- TypeScript type-checking in `gui-check` — noted for Phase 18 (CI/CD) where a frontend CI job can be structured properly
- Final window dimensions — deferred to Phase 16 (SVG Skin) which defines calculator body proportions
- `hp41-core` integration in AppState — Phase 14 (IPC Layer)
- Any React component work beyond empty div — Phase 15 (Display & Keyboard)
- Shared save path `~/.hp41/autosave.json` — Phase 17 (Persistence & Print Output)
