# Phase 13: Workspace Skeleton - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Add `hp41-gui` as a standalone nested Tauri v2 Cargo workspace (its own `[workspace]`, NOT a member of the root workspace). Confirm that `just gui-dev` opens an empty Tauri window titled "HP-41 Calculator" on macOS; `just ci` (the existing CLI pipeline) remains completely unchanged and green; and `cargo build --workspace` from the repo root builds ONLY `hp41-core` and `hp41-cli`.

This phase delivers working infrastructure (tooling, workspace isolation, empty window). It does NOT wire `hp41-core` into the Tauri commands (Phase 14), does NOT render the display or keyboard (Phase 15), and does NOT implement the SVG skin (Phase 16).

</domain>

<decisions>
## Implementation Decisions

### Scaffolding Approach

- **D-01:** **Manual creation of all files** — no `cargo tauri init` or `npm create tauri-app` scaffold. Every file (`Cargo.toml`, `build.rs`, `tauri.conf.json`, `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src/index.css`) is created explicitly per the research specifications. Rationale: the research already specifies exact file contents; a scaffold would generate files that need cleanup and may have different structure assumptions.
- **D-02:** **Bundle identifier:** `ch.talent-factory.hp41` — set in `hp41-gui/src-tauri/tauri.conf.json`. This overrides the scaffold default `com.tauri.dev`, preventing Pitfall #4 (wrong bundle identifier causes silent runtime issues: no window, wrong icons, missing assets).
- **D-03:** **Window title:** `"HP-41 Calculator"` — set in `tauri.conf.json` `windows[0].title`. This is a hard success criterion (SC-1).

### Workspace Isolation

- **D-04:** `hp41-gui/src-tauri/Cargo.toml` declares its own `[workspace]` section, making it a **nested (standalone) Cargo workspace**. The root `Cargo.toml` `members` list stays EXACTLY as-is: `["hp41-core", "hp41-cli"]`. This is the mechanism that guarantees `cargo build --workspace` from root never touches the Tauri binary (success criterion SC-3).
- **D-05:** `tauri = "2.11"` and `tauri-build = "2.6"` are declared ONLY in `hp41-gui/src-tauri/Cargo.toml` under `[dependencies]` and `[build-dependencies]`. They do NOT enter root `[workspace.dependencies]`. (Required by Tauri issue #6122 — adding Tauri to workspace.dependencies breaks the dependency graph for other workspace members.)
- **D-06:** `hp41-core` is referenced from `hp41-gui/src-tauri` via path dependency: `hp41-core = { path = "../../hp41-core" }`. This crosses workspace boundaries and is valid in Cargo.
- **D-07:** `hp41-gui/src-tauri/Cargo.toml` must declare `resolver = "2"` (required by Tauri v2 for correct feature unification on Linux with WebKit2GTK).

### Frontend Scope (Phase 13)

- **D-08:** **Full npm scaffold installed in Phase 13** — all npm packages from the v2.0 research are installed now so Phase 14 (IPC) can immediately write React components without a separate npm-setup step.
  - Runtime: `react ^19.2`, `react-dom ^19.2`, `@tauri-apps/api ^2.11`
  - Dev: `@tauri-apps/cli ^2.11`, `vite ^8.0`, `@vitejs/plugin-react ^6.0`, `typescript ^6.0`, `@types/react ^19.2`, `@types/react-dom ^19.2`, `tailwindcss ^4.3`, `@tailwindcss/vite ^4.3`
- **D-09:** `App.tsx` renders an empty `<div className="app">` — no visible content. The Tauri window opens and shows a blank page. This satisfies success criterion SC-1 ("blank Tauri window").
- **D-10:** **Tailwind wired from day one** — `src/index.css` contains `@import "tailwindcss"` and `vite.config.ts` includes the `@tailwindcss/vite` plugin. Phase 15 (Display & Keyboard) can use Tailwind utilities immediately without a separate setup step.
- **D-11:** `main.tsx` uses `ReactDOM.createRoot(document.getElementById('root')!).render(<App />)` — standard React 19 entry point.

### just Recipes

- **D-12:** **Four new Justfile recipes added in Phase 13:**
  ```
  # GUI: install npm dependencies (run once after cloning or after package.json changes)
  gui-install:
      cd hp41-gui && npm install

  # GUI: launch development window (Rust hot-reload + Vite HMR)
  gui-dev:
      cd hp41-gui && npm run tauri dev

  # GUI: production bundle (native app)
  gui-build:
      cd hp41-gui && npm run tauri build

  # GUI: Rust type-check (fast CI check without launching dev server)
  gui-check:
      cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml
  ```
- **D-13:** `just ci` **remains completely unchanged** (`ci: lint test coverage`). No GUI recipes are referenced from `ci`. The nested workspace ensures `cargo clippy --workspace`, `cargo test --workspace`, and `cargo llvm-cov` never encounter `hp41-gui` Tauri code. CI stays green without modification (success criteria SC-2 and SC-5).

### Claude's Discretion

- `gui-check` covers **Rust only** (`cargo check`) — not TypeScript. TypeScript errors surface during `just gui-dev` (Vite) and can be added to a future CI job in Phase 18. For Phase 13, Rust type correctness is sufficient.
- Initial window dimensions: Tauri defaults (~800×600). Final proportions are set in Phase 16 when the SVG skin defines the calculator body size.
- The `[package]` `name` in `hp41-gui/src-tauri/Cargo.toml` is `"hp41-gui"`. The binary produced is `hp41-gui`.
- `tauri.conf.json` `productName`: `"HP-41 Calculator"` (matches window title).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/ROADMAP.md` — Phase 13 goal, 5 success criteria (SC-1 through SC-5), dependency on Phase 12
- `.planning/REQUIREMENTS.md` — WSPC-01, WSPC-02 acceptance criteria

### Research Files (v2.0)

- `.planning/research/STACK.md` — Exact versions for all Rust and npm dependencies; integration points with root workspace; `[workspace.dependencies]` inheritance rules
- `.planning/research/ARCHITECTURE.md` — Nested workspace pattern decision (canonical section: "Nested workspace (hp41-gui standalone)"); just recipe design; directory structure; build.rs requirement
- `.planning/research/PITFALLS.md` — Pitfall #1 (version mismatch), Pitfall #2 (resolver = "2"), Pitfall #3 (CLI broken by workspace member), Pitfall #4 (missing/wrong build.rs and bundle identifier) — all Phase 13-relevant

### Existing Workspace Files (must not regress)

- `Cargo.toml` — Root workspace (members MUST stay `["hp41-core", "hp41-cli"]` — do NOT add hp41-gui here)
- `Justfile` — Existing recipes (`ci: lint test coverage`) must be preserved exactly; new gui-* recipes added below

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `hp41-core` (path: `hp41-core/`) — the library crate hp41-gui depends on. Already derives `Serialize + Deserialize + Clone` on `CalcState`. No changes needed to hp41-core in Phase 13.
- `Justfile` — existing recipe pattern: `recipe-name:\n\tcommand`. New gui-* recipes follow the same format.

### Established Patterns

- **Nested workspace path**: `hp41-gui/src-tauri/Cargo.toml` references hp41-core as `hp41-core = { path = "../../hp41-core" }` — two levels up from `src-tauri/`.
- **`resolver = "2"`**: root workspace already uses this. The nested `hp41-gui/src-tauri/Cargo.toml` must also declare `resolver = "2"` (Tauri v2 requirement; see Pitfall #2).
- **`rust-version = "1.85"`**: the root workspace pins MSRV. The nested workspace `[package]` should also declare this for consistency.
- **`workspace.package`** inheritance: NOT applicable for the nested workspace — it cannot inherit from the root `[workspace.package]`. Version and other fields must be declared explicitly in `hp41-gui/src-tauri/Cargo.toml`.

### Integration Points

- `hp41-gui/src-tauri/` connects to root via path dependency on `hp41-core` only — no other coupling to the root workspace.
- `hp41-gui/package.json` `scripts.tauri` must point to `@tauri-apps/cli` (installed as devDependency). The `tauri dev` and `tauri build` npm scripts use this.
- `hp41-gui/vite.config.ts` must reference `hp41-gui/src-tauri/tauri.conf.json` implicitly (Tauri CLI wires them). No explicit path needed in vite.config.ts.

</code_context>

<specifics>
## Specific Ideas

- Bundle identifier: `ch.talent-factory.hp41` — explicit user choice, overrides the scaffold default.
- Window title: `"HP-41 Calculator"` — success criterion SC-1 specifies this exactly.
- Tailwind from day one: the user confirmed `@import "tailwindcss"` in `src/index.css` from the initial commit.

</specifics>

<deferred>
## Deferred Ideas

- TypeScript type-checking in `gui-check` — deferred to Phase 18 (CI/CD), where a proper frontend CI job can be structured.
- Final window dimensions — deferred to Phase 16 (SVG Skin), which defines the calculator body proportions.
- `hp41-core` dependency wired in Tauri AppState — deferred to Phase 14 (IPC Layer).
- Any React component work beyond the empty `<div>` — deferred to Phase 15 (Display & Keyboard).
- `hp41-cli` companion workflows (shared save path `~/.hp41/autosave.json`) — deferred to Phase 17.

</deferred>

---

*Phase: 13-Workspace Skeleton*
*Context gathered: 2026-05-09*
