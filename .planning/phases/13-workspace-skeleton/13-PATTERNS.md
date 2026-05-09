# Phase 13: Workspace Skeleton - Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 13 new/modified files
**Analogs found:** 6 / 13 (7 are new technology with no codebase analog — covered by canonical research refs)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-gui/src-tauri/Cargo.toml` | config | batch | `hp41-cli/Cargo.toml` | role-match (same Cargo package manifest; different deps) |
| `hp41-gui/src-tauri/build.rs` | config | batch | none | no-analog (Tauri-specific codegen; see PITFALLS #4) |
| `hp41-gui/src-tauri/src/main.rs` | entrypoint | request-response | `hp41-cli/src/main.rs` | role-match (binary entry delegating to lib) |
| `hp41-gui/src-tauri/src/lib.rs` | service | request-response | `hp41-core/src/lib.rs` | partial-match (library root + re-exports; Tauri-specific setup) |
| `hp41-gui/src-tauri/tauri.conf.json` | config | batch | none | no-analog (Tauri-specific JSON config) |
| `hp41-gui/package.json` | config | batch | none | no-analog (npm manifest; no npm projects in codebase) |
| `hp41-gui/vite.config.ts` | config | batch | none | no-analog (Vite/TypeScript; no frontend tooling in codebase) |
| `hp41-gui/tsconfig.json` | config | batch | none | no-analog (TypeScript config; no TS in codebase) |
| `hp41-gui/index.html` | config | request-response | none | no-analog (HTML entry point; no web frontend in codebase) |
| `hp41-gui/src/main.tsx` | entrypoint | request-response | `hp41-cli/src/main.rs` | partial-match (frontend entry point delegating to App component) |
| `hp41-gui/src/App.tsx` | component | request-response | none | no-analog (React component; no frontend components in codebase) |
| `hp41-gui/src/index.css` | config | batch | none | no-analog (CSS; no stylesheet files in codebase) |
| `Justfile` | config | batch | `Justfile` (existing) | exact (modifying existing file — add 4 recipes) |

---

## Pattern Assignments

### `hp41-gui/src-tauri/Cargo.toml` (config, batch)

**Analog:** `hp41-cli/Cargo.toml` (lines 1–18) for package section shape.
**Also reference:** Root `Cargo.toml` (lines 1–12) for workspace declaration pattern.

**Key divergence from analogs:** This manifest declares its OWN `[workspace]` section (making it a nested/standalone workspace — decision D-04). It cannot inherit from `[workspace.package]` of the root. All fields must be explicit.

**Package pattern** from `hp41-cli/Cargo.toml` lines 1–9:
```toml
[package]
name = "hp41-cli"
version = "0.1.0"
edition = "2021"
rust-version.workspace = true

[[bin]]
name = "hp41"
path = "src/main.rs"
```

**Adapted for hp41-gui** (workspace inheritance NOT available — explicit values required per D-07 + code_context):
```toml
[workspace]
resolver = "2"

[package]
name = "hp41-gui"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"

[[bin]]
name = "hp41-gui"
path = "src/main.rs"
```

**Path dependency pattern** from `hp41-cli/Cargo.toml` line 12:
```toml
hp41-core = { path = "../hp41-core" }
```
Adapted for the nested location (two levels up from `src-tauri/`):
```toml
hp41-core = { path = "../../hp41-core" }
```

**Explicit (non-inherited) dependency pattern** — tauri and tauri-build are NOT in root `[workspace.dependencies]` (decision D-05; PITFALLS #1):
```toml
[dependencies]
tauri = { version = "2.11", features = ["macos-private-api"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hp41-core = { path = "../../hp41-core" }

[build-dependencies]
tauri-build = { version = "2.6", features = [] }
```

---

### `hp41-gui/src-tauri/build.rs` (config, batch)

**Analog:** None in codebase. Pattern comes directly from PITFALLS.md #4 (canonical reference).

**Exact content — do not add anything else** (PITFALLS.md lines 133-136):
```rust
fn main() {
    tauri_build::build()
}
```

---

### `hp41-gui/src-tauri/src/main.rs` (entrypoint, request-response)

**Analog:** `hp41-cli/src/main.rs` (lines 1–80)

**Pattern:** Binary entry point that delegates all work to lib. The CLI analog does arg parsing + state loading before delegating to `App`. The Tauri equivalent is simpler — just call `lib::run()`.

**Module declaration pattern** from `hp41-cli/src/main.rs` lines 6-14:
```rust
#![deny(clippy::unwrap_used)]

mod app;
mod help_data;
mod keys;
// ...

use app::App;
```

**Adapted Tauri pattern** (standard Tauri v2 main.rs — prevents console window on Windows):
```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hp41_gui_lib::run()
}
```

Note: No `#![deny(clippy::unwrap_used)]` in `main.rs` itself — it belongs in `lib.rs` where the production code lives. The `main.rs` is a 3-line stub.

---

### `hp41-gui/src-tauri/src/lib.rs` (service, request-response)

**Analog:** `hp41-core/src/lib.rs` (lines 1–23) for the crate-root pattern (deny lint at top, pub mod declarations, re-exports). The Tauri-specific `Builder` pattern has no analog in this codebase.

**Crate-root pattern** from `hp41-core/src/lib.rs` lines 1–4:
```rust
#![deny(clippy::unwrap_used)]
//! hp41-core — HP-41 calculator behavioral emulation library.
//!
//! Zero UI/CLI dependencies. All state is in [`state::CalcState`].
```

**State management pattern** from ARCHITECTURE.md lines 259-276 (canonical reference):
```rust
use std::sync::Mutex;

pub struct AppState(pub Mutex<hp41_core::CalcState>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(hp41_core::CalcState::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // commands registered here in Phase 14
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
```

**Phase 13 scope:** `lib.rs` contains only the empty shell above — no `#[tauri::command]` functions yet (those come in Phase 14 per the deferred decisions). The `invoke_handler` list is empty.

---

### `hp41-gui/src-tauri/tauri.conf.json` (config, batch)

**Analog:** None in codebase. Pattern comes from STACK.md (canonical reference) with overrides from decisions D-02 and D-03.

**From STACK.md lines 135-154, adapted with D-02 (bundle identifier) and D-03 (window title):**
```json
{
  "productName": "HP-41 Calculator",
  "identifier": "ch.talent-factory.hp41",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "HP-41 Calculator",
        "width": 800,
        "height": 600,
        "resizable": true,
        "decorations": true
      }
    ]
  }
}
```

**Critical values (must match exactly):**
- `identifier`: `"ch.talent-factory.hp41"` — decision D-02; overrides scaffold default `com.tauri.dev`
- `windows[0].title`: `"HP-41 Calculator"` — decision D-03; success criterion SC-1
- `productName`: `"HP-41 Calculator"` — Claude's Discretion section
- `devUrl`: `"http://localhost:5173"` — must match `vite.config.ts` `server.port`
- `frontendDist`: `"../dist"` — Vite outputs to `hp41-gui/dist/`; `tauri.conf.json` is in `src-tauri/` so `../dist` is correct

**Window dimensions:** Tauri defaults (~800×600) per Claude's Discretion — final size deferred to Phase 16.

---

### `hp41-gui/package.json` (config, batch)

**Analog:** None in codebase. Pattern from decisions D-08 and STACK.md lines 38-59.

**Standard Tauri + React npm manifest:**
```json
{
  "name": "hp41-gui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.11",
    "react": "^19.2",
    "react-dom": "^19.2"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.11",
    "@tailwindcss/vite": "^4.3",
    "@types/react": "^19.2",
    "@types/react-dom": "^19.2",
    "@vitejs/plugin-react": "^6.0",
    "tailwindcss": "^4.3",
    "typescript": "^6.0",
    "vite": "^8.0"
  }
}
```

**Key constraint from code_context:** `scripts.tauri` must point to `@tauri-apps/cli` (installed as devDependency). The `"tauri": "tauri"` script resolves via `node_modules/.bin/tauri` when run as `npm run tauri dev`.

---

### `hp41-gui/vite.config.ts` (config, batch)

**Analog:** None in codebase. Pattern from STACK.md lines 159-177 (canonical reference).

**Exact content from STACK.md:**
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

**Critical constraint:** `strictPort: true` is required because `tauri.conf.json` hardcodes `http://localhost:5173`. If port 5173 is busy, the dev server must fail rather than silently switching ports.

---

### `hp41-gui/tsconfig.json` (config, batch)

**Analog:** None in codebase. Standard strict TypeScript config for a Vite + React project.

**Standard pattern for Vite + React + TypeScript 6:**
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

**Note:** `"moduleResolution": "bundler"` is the correct setting for TypeScript 5+ with Vite — not `"node"` or `"node16"`.

---

### `hp41-gui/index.html` (config, request-response)

**Analog:** None in codebase. Standard Vite HTML entry point.

**Standard Vite + React HTML entry:**
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>HP-41 Calculator</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

**Key points:**
- `<div id="root">` — React mounts here; matches `document.getElementById('root')` in `main.tsx`
- `<title>` — "HP-41 Calculator" for consistency with `tauri.conf.json` window title
- `type="module"` — required for ESM imports in Vite

---

### `hp41-gui/src/main.tsx` (entrypoint, request-response)

**Analog:** `hp41-cli/src/main.rs` (role-match — both are the entry point that bootstraps the UI). The React 19 API pattern has no Rust analog.

**Pattern from decision D-11:**
```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

**Note:** `React.StrictMode` is included — it double-invokes effects in development to expose cleanup failures (directly relevant to PITFALLS #11 on duplicate event listeners).

---

### `hp41-gui/src/App.tsx` (component, request-response)

**Analog:** None in codebase. Pattern from decision D-09.

**Minimal content per D-09 (empty div, no visible content):**
```tsx
function App() {
  return (
    <div className="app">
    </div>
  )
}

export default App
```

**No imports needed** at Phase 13 — no state, no Tauri IPC, no child components. `className="app"` gives Phase 15 a CSS hook.

---

### `hp41-gui/src/index.css` (config, batch)

**Analog:** None in codebase. Pattern from decision D-10.

**Exact content per D-10 (Tailwind v4 CSS-only approach):**
```css
@import "tailwindcss";
```

**Note:** Tailwind v4 dropped PostCSS-first in favor of a Vite plugin (`@tailwindcss/vite`). No `tailwind.config.js` is needed. The single `@import "tailwindcss"` line in CSS is the complete setup when the Vite plugin is registered in `vite.config.ts`.

---

### `Justfile` (config, batch) — MODIFY EXISTING

**Analog:** `Justfile` (exact — adding recipes to the existing file)

**Existing recipe pattern** from `Justfile` lines 1-55 (established format):
```just
# Comment explaining the recipe
recipe-name:
	command
```

Tab-indented (NOT spaces). Comment line immediately above the recipe name. No blank line between comment and recipe.

**Four new recipes from decision D-12** (add after line 55, before end of file):
```just
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

**Critical constraint (D-13):** The existing `ci: lint test coverage` recipe on line 31 must NOT be modified. No `gui-*` recipe is referenced from `ci`.

**`gui-check` uses `--manifest-path`** (not `-p hp41-gui`) because `hp41-gui/src-tauri` is a nested workspace, not a member of the root workspace. `-p hp41-gui` would fail because `cargo` cannot find that package name in the root workspace.

---

## Shared Patterns

### Workspace Isolation (applies to ALL new `hp41-gui/src-tauri/` Rust files)

**Source:** Root `Cargo.toml` lines 1-3 + CONTEXT.md decisions D-04 through D-07
**Apply to:** `hp41-gui/src-tauri/Cargo.toml`, `hp41-gui/src-tauri/build.rs`, all `src/` Rust files

The nested workspace pattern means:
1. `hp41-gui/src-tauri/Cargo.toml` declares `[workspace]` with `resolver = "2"`
2. Root `Cargo.toml` `members` stays as `["hp41-core", "hp41-cli"]` — unchanged
3. `cargo build --workspace` from root NEVER touches Tauri code
4. `tauri` and `tauri-build` crates are declared ONLY in `hp41-gui/src-tauri/Cargo.toml`

### Deny Unwrap Lint (applies to all new Rust files)

**Source:** `hp41-core/src/lib.rs` line 1, `hp41-cli/src/main.rs` line 6
**Apply to:** `hp41-gui/src-tauri/src/lib.rs`

```rust
#![deny(clippy::unwrap_used)]
```

The project enforces zero panics in production Rust code. New Tauri code follows the same convention (PITFALLS #7 — mutex poisoning interacts badly with unwrap). Use `.unwrap_or_else(|e| e.into_inner())` for mutex lock in command handlers.

### Version Pinning (applies to all dependency declarations)

**Source:** PITFALLS.md #1 + CONTEXT.md D-05
**Apply to:** `hp41-gui/src-tauri/Cargo.toml`, `hp41-gui/package.json`

- `tauri = "2.11"` and `tauri-build = "2.6"` in Cargo — same minor line
- `@tauri-apps/cli = "^2.11"` and `@tauri-apps/api = "^2.11"` in npm — same minor line
- Never route `tauri` or `tauri-build` through `[workspace.dependencies]` (Tauri issue #6122)

---

## No Analog Found

Files with no close match in the codebase — planner must use RESEARCH.md / canonical research refs instead:

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `hp41-gui/src-tauri/build.rs` | config | batch | Tauri-specific build codegen; no build.rs files exist in this codebase |
| `hp41-gui/src-tauri/tauri.conf.json` | config | batch | Tauri-specific JSON; no equivalent config format in codebase |
| `hp41-gui/package.json` | config | batch | No npm packages in this codebase |
| `hp41-gui/vite.config.ts` | config | batch | No TypeScript/Vite in this codebase |
| `hp41-gui/tsconfig.json` | config | batch | No TypeScript in this codebase |
| `hp41-gui/index.html` | config | request-response | No HTML files in this codebase |
| `hp41-gui/src/App.tsx` | component | request-response | No React components in this codebase |
| `hp41-gui/src/index.css` | config | batch | No CSS files in this codebase |

**For all no-analog files:** Use the exact content specified in STACK.md, ARCHITECTURE.md, and the decisions in CONTEXT.md. The research files contain verified, working patterns for each.

---

## Metadata

**Analog search scope:** `hp41-core/`, `hp41-cli/`, root `Cargo.toml`, root `Justfile`
**Files scanned:** 8 (Cargo.toml x3, Justfile, main.rs x2, lib.rs x2)
**Research files consumed:** STACK.md, ARCHITECTURE.md, PITFALLS.md
**Pattern extraction date:** 2026-05-09
