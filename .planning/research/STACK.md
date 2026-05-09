# Technology Stack

**Project:** HP-41 Calculator Emulator — v2.0 GUI Stack (Tauri v2 + React + TypeScript)
**Researched:** 2026-05-09
**Scope:** NEW dependencies needed for `hp41-gui` only. The existing v1.0/v1.1 Rust stack is validated and unchanged.

---

## Summary

`hp41-gui` is a thin Tauri v2 binary that owns the desktop window, exposes `hp41-core` operations as Tauri commands, and renders an SVG HP-41 skin in a React 19 + TypeScript + Vite 8 frontend. The Rust side is a new Cargo workspace member (`hp41-gui/src-tauri`). The frontend lives in `hp41-gui/` alongside `src-tauri/`. The entire frontend communicates with `hp41-core` exclusively through `invoke()` — no CalcState logic is duplicated in TypeScript.

All version numbers verified against npm and crates.io registries on 2026-05-09.

---

## New Dependencies

### Rust Side (hp41-gui/src-tauri/Cargo.toml)

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `tauri` | `"2.11"` | Desktop shell, webview, IPC runtime | Core Tauri runtime; v2.11.1 is current stable. |
| `tauri-build` | `"2.6"` | Build-time codegen (`build.rs`) | Required by every Tauri project — generates platform-specific glue at compile time. |
| `serde` | `{ workspace = true }` | Serialize/Deserialize for command args and return types | Already a workspace dependency; Tauri requires `serde::Serialize` on all command return types. `CalcState` already derives it. |
| `serde_json` | `{ workspace = true }` | JSON serialization | Already a workspace dependency; Tauri IPC uses JSON internally. |

`hp41-core` is added as a path dependency:

```toml
hp41-core = { path = "../../hp41-core" }
```

No additional Rust crates are needed. `std::sync::Mutex` (stdlib) wraps `CalcState` for thread-safe Tauri state management.

**tauri-specta (NOT included):** `tauri-specta = "2.0.0-rc.25"` generates TypeScript bindings from Rust command signatures. It is still in release-candidate status (latest is rc.25 as of 2026-05-09 — no stable release). For an HP-41 emulator with a small, stable command surface (~5 commands), manually maintaining TypeScript interfaces is less risky than depending on an RC crate. Revisit once tauri-specta reaches 2.0.0 stable.

### Frontend (hp41-gui/package.json)

#### Runtime Dependencies

| Package | Version | Purpose | Why |
|---------|---------|---------|-----|
| `react` | `^19.2` | UI framework | React 19 is current stable (Dec 2024). Concurrent mode, improved hooks, `use()` API. |
| `react-dom` | `^19.2` | React DOM renderer | Companion to react; required for web rendering. |
| `@tauri-apps/api` | `^2.11` | `invoke()`, `listen()`, `emit()` | Official Tauri JavaScript API for IPC. Core module provides `invoke`. Event module provides `listen` for backend-push updates. |

#### Dev Dependencies

| Package | Version | Purpose | Why |
|---------|---------|---------|-----|
| `@tauri-apps/cli` | `^2.11` | Tauri CLI (`tauri dev`, `tauri build`) | Orchestrates Vite dev server + Rust compilation. `npm run tauri dev` starts the app. |
| `vite` | `^8.0` | Frontend build tool | Tauri's recommended bundler. Instant HMR, native ESM, fast production builds. v8.0.11 is current. |
| `@vitejs/plugin-react` | `^6.0` | React HMR + JSX transform for Vite | Fast Refresh via Babel. v6.0.1 is current. Use `@vitejs/plugin-react` (Babel) rather than `@vitejs/plugin-react-swc` — SWC is faster but has less predictable TypeScript decorator handling; for this project Babel is sufficient and more battle-tested with Tauri templates. |
| `typescript` | `^6.0` | TypeScript compiler | v6.0.3 is current. Strict mode enabled. |
| `@types/react` | `^19.2` | React TypeScript types | Required for JSX type checking. |
| `@types/react-dom` | `^19.2` | React DOM TypeScript types | Required for `render`/`createRoot` type checking. |
| `tailwindcss` | `^4.3` | CSS utility framework | v4.3.0 is current. For the HP-41 skin, Tailwind handles layout, spacing, and color variables. The CSS-only approach (`@import "tailwindcss"` in index.css) is simpler than v3's PostCSS config. |
| `@tailwindcss/vite` | `^4.3` | Tailwind v4 Vite plugin | Tailwind v4 dropped PostCSS-first in favor of a Vite plugin. Required when using Tailwind v4 with Vite — no `tailwind.config.js` needed. |

---

## Integration Points with Existing Workspace

### 1. Cargo Workspace: Add hp41-gui as a Member

`Cargo.toml` (root) changes:

```toml
[workspace]
resolver = "2"
members = ["hp41-core", "hp41-cli", "hp41-gui/src-tauri"]
```

`hp41-gui/src-tauri` is a standard Cargo package. It depends on `hp41-core` via path. All workspace.dependencies (`serde`, `serde_json`, `rust_decimal`, `thiserror`) are inherited as before. Tauri's own crates (`tauri`, `tauri-build`) are NOT added to `[workspace.dependencies]` — they are local to `hp41-gui/src-tauri/Cargo.toml` because no other workspace member needs them.

### 2. Tauri State: CalcState wrapped in Mutex

`hp41-gui/src-tauri/src/lib.rs`:

```rust
use hp41_core::state::CalcState;
use std::sync::Mutex;

pub struct AppState(pub Mutex<CalcState>);
```

All commands receive `state: tauri::State<'_, AppState>`, lock the mutex, call `hp41_core::ops::dispatch()`, and return the updated `CalcState` (which already derives `serde::Serialize`):

```rust
#[tauri::command]
fn dispatch_op(op_name: String, state: tauri::State<'_, AppState>) -> Result<CalcState, String> {
    let op = op_name.parse::<hp41_core::ops::Op>().map_err(|e| e.to_string())?;
    let mut s = state.0.lock().unwrap();
    hp41_core::ops::dispatch(op, &mut s).map_err(|e| e.to_string())?;
    Ok(s.clone())
}
```

`CalcState` already derives `Clone + Serialize + Deserialize` (confirmed in `hp41-core/src/state.rs` line 52). Returning the full state snapshot after every op is the correct pattern — the frontend never holds a partial view.

### 3. Command Surface (Minimal — 5 Commands)

| Command | Signature | Notes |
|---------|-----------|-------|
| `dispatch_op` | `(op_name: String) → Result<CalcState, String>` | All HP-41 ops go through this single entry point. Op enum serialized as string. |
| `get_state` | `() → CalcState` | Initial state load. |
| `load_state` | `(path: Option<String>) → Result<CalcState, String>` | Load persisted JSON (reuses hp41-core persistence logic). |
| `save_state` | `(path: Option<String>) → Result<(), String>` | Save state to JSON. |
| `get_display` | `() → String` | Convenience: formatted display string only (avoids deserializing full CalcState for display-only updates). Optional optimization. |

Keeping the command surface to 5 endpoints is deliberate: all HP-41 logic stays in `hp41-core`. The frontend never implements calculator math.

### 4. Frontend TypeScript Interfaces

Manually maintained TypeScript interfaces mirror `CalcState` struct fields. Since `CalcState` derives `Serialize`, the JSON shape is stable. No codegen needed for 5 commands.

```typescript
// src/types/CalcState.ts
export interface CalcState {
  stack: Stack;
  regs: string[];       // HpNum serializes as decimal string
  alpha_reg: string;
  alpha_mode: boolean;
  display_mode: DisplayMode;
  angle_mode: AngleMode;
  // ... (match hp41-core/src/state.rs fields exactly)
}
```

### 5. Tauri Configuration (tauri.conf.json)

Placed at `hp41-gui/src-tauri/tauri.conf.json`:

```json
{
  "productName": "HP-41",
  "identifier": "ch.talent-factory.hp41",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "title": "HP-41",
      "width": 400,
      "height": 700,
      "resizable": false,
      "decorations": true
    }]
  }
}
```

A fixed `400×700` non-resizable window matches the physical HP-41C form factor. The calculator skin is an SVG that fills this viewport.

### 6. Vite Configuration (vite.config.ts)

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || 'localhost',
  },
  build: {
    outDir: 'dist',
  },
})
```

`strictPort: true` is required because `tauri.conf.json` hardcodes `http://localhost:5173`. `TAURI_DEV_HOST` allows iOS physical device testing if that is ever needed.

### 7. just Recipes

New recipes added to the root `Justfile`:

```just
# Run hp41-gui in Tauri dev mode
gui-dev:
    cd hp41-gui && npm run tauri dev

# Build hp41-gui for release
gui-build:
    cd hp41-gui && npm run tauri build

# Install hp41-gui frontend dependencies
gui-install:
    cd hp41-gui && npm install
```

`just ci` (Rust lint + test + coverage) is unchanged — it targets `--workspace` Rust crates. The Tauri GUI build is not added to the Rust CI gate; it has its own CI job.

---

## SVG Skin Approach

The HP-41C skin is a hand-crafted inline SVG component in React. The SVG contains one `<g>` element per key with an `onClick` handler. React handles pointer events natively on SVG elements.

**Why inline SVG (not `<img src="...svg">`):** External SVG images cannot receive pointer events. Inline SVG embeds the element in the DOM, so React event delegation works.

**Why not a third-party SVG-to-React converter:** The HP-41 key layout has ~70 keys with precise positions; auto-converted SVG is unmaintainable. A handcrafted component with a typed `KeyLayout` data structure is the right abstraction.

**TypeScript pattern:**

```typescript
interface HpKey {
  op: string;          // matches Op enum serialized name
  label: string;       // visible key label
  x: number; y: number; width: number; height: number;  // SVG coords
  color: string;
}

const HP41_KEYS: HpKey[] = [ /* ~70 entries */ ];

function Calculator({ onKey }: { onKey: (op: string) => void }) {
  return (
    <svg viewBox="0 0 400 700" xmlns="http://www.w3.org/2000/svg">
      {HP41_KEYS.map(key => (
        <g key={key.op} onClick={() => onKey(key.op)} role="button" aria-label={key.label}>
          <rect x={key.x} y={key.y} width={key.width} height={key.height} fill={key.color} />
          <text ...>{key.label}</text>
        </g>
      ))}
    </svg>
  );
}
```

---

## What NOT to Add

| Package | Reason to Exclude |
|---------|-------------------|
| `zustand` / `redux` / `jotai` | Global state stores are unnecessary. All state lives in `hp41-core`'s `CalcState` on the Rust side. The frontend receives a full `CalcState` snapshot after every `invoke()` call — React's `useState` with the returned snapshot is sufficient. Adding a client-side store would duplicate state and create sync bugs. |
| `react-router` | Single-page app with one view (the calculator). No routing needed. |
| `axios` / `fetch` wrappers | No HTTP calls. All communication is Tauri IPC via `invoke()`. |
| `tauri-specta` | Still RC (2.0.0-rc.25). Command surface is small enough (5 commands) that manually written TypeScript interfaces are less risky. Revisit at stable release. |
| `@tauri-apps/plugin-fs` | hp41-core handles persistence internally. The save/load Tauri commands call hp41-core functions directly — no frontend filesystem access needed. |
| `vitest` / `@testing-library/react` | Not added in the initial GUI phase. The core arithmetic is tested at 94% coverage in hp41-core. GUI integration testing can be added in a follow-on phase once the skin is stable. |
| `electron` | Not Tauri — noted only because it is sometimes suggested as an alternative. Tauri produces binaries 10–50x smaller and has a well-audited security model (capability system). |
| `shadcn/ui` / `radix-ui` | Component libraries optimized for standard web UIs (forms, dialogs, tables). The HP-41 skin is a custom SVG widget — no generic UI component fits. These would add ~100 kB of unused components. |
| `next.js` / `remix` | Server-side rendering frameworks — incompatible with Tauri's static-file webview model. Vite + React is the correct choice for a local desktop app with no server. |
| Custom BCD arithmetic in TypeScript | hp41-core already implements all HP-41 arithmetic in Rust. Duplicating any of it in TypeScript violates the core invariant (GUI-05) and introduces divergence risk. |

---

## Directory Structure After Adding hp41-gui

```
hp41-calculator-emulator/
├── Cargo.toml              (workspace — add "hp41-gui/src-tauri" to members)
├── Justfile                (add gui-dev, gui-build, gui-install recipes)
├── hp41-core/              (unchanged)
├── hp41-cli/               (unchanged)
└── hp41-gui/               (NEW — frontend root)
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── index.html
    ├── src/
    │   ├── main.tsx
    │   ├── App.tsx
    │   ├── components/
    │   │   ├── Calculator.tsx    (SVG skin component)
    │   │   └── Display.tsx       (12-char dot-matrix display)
    │   ├── types/
    │   │   └── CalcState.ts      (TypeScript mirror of hp41-core CalcState)
    │   └── hooks/
    │       └── useCalc.ts        (invoke() wrapper hook)
    └── src-tauri/          (NEW — Rust Tauri binary)
        ├── Cargo.toml
        ├── tauri.conf.json
        ├── build.rs
        ├── capabilities/
        │   └── default.json
        └── src/
            ├── main.rs
            └── lib.rs            (AppState, #[tauri::command] handlers)
```

---

## Sources

- `@tauri-apps/cli` version: npm registry — 2.11.1 (verified 2026-05-09)
- `@tauri-apps/api` version: npm registry — 2.11.0 (verified 2026-05-09)
- `create-tauri-app` version: npm registry — 4.6.2 (verified 2026-05-09)
- `tauri` crate version: crates.io — 2.11.1 (verified 2026-05-09)
- `tauri-build` crate version: crates.io — 2.6.1 (verified 2026-05-09)
- `tauri-specta` crate version: crates.io — 2.0.0-rc.25, RC status confirmed (verified 2026-05-09)
- `react` version: npm registry — 19.2.6 (verified 2026-05-09)
- `vite` version: npm registry — 8.0.11 (verified 2026-05-09)
- `@vitejs/plugin-react` version: npm registry — 6.0.1 (verified 2026-05-09)
- `typescript` version: npm registry — 6.0.3 (verified 2026-05-09)
- `tailwindcss` / `@tailwindcss/vite` version: npm registry — 4.3.0 (verified 2026-05-09)
- Tauri v2 State Management: https://v2.tauri.app/develop/state-management/ (Context7 /tauri-apps/tauri-docs)
- Tauri v2 Commands: https://v2.tauri.app/develop/calling-rust/ (Context7 /tauri-apps/tauri-docs)
- Tauri v2 Project Structure: https://v2.tauri.app/start/project-structure/
- Tauri v2 Vite Frontend: https://v2.tauri.app/start/frontend/vite/
- Tailwind v4 + Vite setup: https://tailwindcss.com/docs (verified via npm package docs)
- tauri-specta releases: https://github.com/specta-rs/tauri-specta/releases
- CalcState derives: hp41-core/src/state.rs line 52 — `#[derive(Debug, Clone, Serialize, Deserialize)]`
