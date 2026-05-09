---
phase: 13-workspace-skeleton
plan: "02"
subsystem: ui
tags: [react, vite, typescript, tailwindcss, tauri, npm]

# Dependency graph
requires:
  - phase: 12-synthetic-programming
    provides: "Completed v1.1 CLI — hp41-core unchanged and ready for GUI reuse"
provides:
  - "hp41-gui/package.json — npm manifest with react 19.2, @tauri-apps/api 2.11, vite 8.0, tailwindcss 4.3"
  - "hp41-gui/vite.config.ts — Vite config with strictPort: true on port 5173 and @tailwindcss/vite plugin"
  - "hp41-gui/tsconfig.json — strict TypeScript config with moduleResolution bundler and react-jsx transform"
  - "hp41-gui/index.html — HTML entry point with div#root and /src/main.tsx module script"
  - "hp41-gui/src/main.tsx — React 19 entry point with ReactDOM.createRoot and React.StrictMode"
  - "hp41-gui/src/App.tsx — minimal App component with empty div.app (CSS hook for Phase 15)"
  - "hp41-gui/src/index.css — Tailwind v4 import (@import tailwindcss)"
affects: [14-ipc-layer, 15-display-keyboard, 16-svg-skin, 18-gui-ci]

# Tech tracking
tech-stack:
  added:
    - "react 19.2 + react-dom 19.2 — React 19 stable, concurrent mode, improved hooks"
    - "@tauri-apps/api 2.11 — Tauri JS API for invoke/listen/emit IPC"
    - "@tauri-apps/cli 2.11 — Tauri CLI (npm run tauri dev/build)"
    - "vite 8.0 — frontend build tool with HMR"
    - "@vitejs/plugin-react 6.0 — React HMR and JSX transform via Babel"
    - "typescript 6.0 — strict TypeScript compiler"
    - "tailwindcss 4.3 + @tailwindcss/vite 4.3 — Tailwind v4 with Vite plugin (no tailwind.config.js needed)"
    - "@types/react 19.2 + @types/react-dom 19.2 — TypeScript types"
  patterns:
    - "Tailwind v4 CSS-only setup: single @import tailwindcss in index.css + @tailwindcss/vite plugin"
    - "React 19 entry: ReactDOM.createRoot with React.StrictMode wrapping"
    - "Vite strictPort: true to fail-fast when port 5173 conflicts with tauri.conf.json devUrl"
    - "tsconfig moduleResolution: bundler for TypeScript 6 + Vite (not node or node16)"

key-files:
  created:
    - "hp41-gui/package.json"
    - "hp41-gui/vite.config.ts"
    - "hp41-gui/tsconfig.json"
    - "hp41-gui/index.html"
    - "hp41-gui/src/main.tsx"
    - "hp41-gui/src/App.tsx"
    - "hp41-gui/src/index.css"
  modified: []

key-decisions:
  - "D-08: Full npm scaffold installed in Phase 13 so Phase 14 can write React components without a separate npm setup step"
  - "D-09: App.tsx renders empty div.app — blank window satisfies SC-1; className=app is CSS hook for Phase 15"
  - "D-10: Tailwind v4 wired from day one via @import tailwindcss in index.css + @tailwindcss/vite plugin — no tailwind.config.js needed"
  - "D-11: React.StrictMode enabled in main.tsx — double-invokes effects to prevent duplicate IPC listeners (Pitfall #11)"
  - "strictPort: true prevents silent port-switching when port 5173 is busy (T-13-02-02 mitigation)"
  - "@tauri-apps/api and @tauri-apps/cli both pinned to ^2.11 — same minor version as Rust crate (T-13-02-03 mitigation, Pitfall #1)"

patterns-established:
  - "Frontend entry chain: index.html → /src/main.tsx → App → index.css"
  - "React.StrictMode: Phase 15 keyboard event handlers MUST include cleanup functions in useEffect (Pitfall #11)"
  - "All .tsx files: no explicit React import needed (tsconfig jsx: react-jsx handles JSX transform)"
  - "TypeScript strict mode: noUnusedLocals, noUnusedParameters, noFallthroughCasesInSwitch all enabled"

requirements-completed: [WSPC-01]

# Metrics
duration: 8min
completed: "2026-05-09"
---

# Phase 13 Plan 02: Frontend Scaffold Summary

**Vite 8 + React 19 + TypeScript 6 + Tailwind v4 frontend scaffold with strictPort:5173, StrictMode, and CSS-only Tailwind setup**

## Performance

- **Duration:** 8 min
- **Started:** 2026-05-09T12:00:00Z
- **Completed:** 2026-05-09T12:08:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Complete npm manifest with all 11 packages from D-08 at correct version ranges (react 19.2, tauri api/cli 2.11, vite 8.0, tailwindcss 4.3)
- Vite config with strictPort: true on port 5173 (matches tauri.conf.json devUrl) and both @vitejs/plugin-react and @tailwindcss/vite plugins
- React 19 entry point with StrictMode wrapping to prevent duplicate IPC event listeners in Phase 14+
- Tailwind v4 wired via single @import "tailwindcss" in index.css — no tailwind.config.js needed, Phase 15 can use utilities immediately

## npm Package Versions (for Phase 14/15 reference)

| Package | Version Range | Resolved |
|---------|--------------|---------|
| react | ^19.2 | 19.2.x |
| react-dom | ^19.2 | 19.2.x |
| @tauri-apps/api | ^2.11 | 2.11.x |
| @tauri-apps/cli | ^2.11 | 2.11.x |
| vite | ^8.0 | 8.0.x |
| @vitejs/plugin-react | ^6.0 | 6.0.x |
| typescript | ^6.0 | 6.0.x |
| @types/react | ^19.2 | 19.2.x |
| @types/react-dom | ^19.2 | 19.2.x |
| tailwindcss | ^4.3 | 4.3.x |
| @tailwindcss/vite | ^4.3 | 4.3.x |

## vite.config.ts Key Configuration

- `server.port: 5173` — matches `devUrl: http://localhost:5173` in tauri.conf.json
- `server.strictPort: true` — fail-fast when port 5173 is busy (not silent switch)
- `server.host: process.env.TAURI_DEV_HOST || 'localhost'` — allows iOS device testing
- `build.outDir: 'dist'` — matches `frontendDist: "../dist"` in tauri.conf.json
- `plugins: [react(), tailwindcss()]` — both required for React HMR and Tailwind v4

## tsconfig.json Key compilerOptions

- `moduleResolution: "bundler"` — correct for TypeScript 6 + Vite (NOT node or node16)
- `jsx: "react-jsx"` — React 19 JSX transform; no `import React` needed in components
- `strict: true` — full strict mode matching Rust lint philosophy
- `noEmit: true` — TypeScript type-checks only; Vite handles transpilation
- `include: ["src"]` — scope to frontend sources only

## Task Commits

Each task was committed atomically:

1. **Task 1: Create npm manifest and build tooling config** - `cbe9162` (chore)
2. **Task 2: Create HTML entry point and React source files** - `03dea81` (feat)

**Plan metadata:** (committed with SUMMARY.md)

## Files Created/Modified

- `hp41-gui/package.json` — npm manifest with 3 runtime + 8 dev dependencies at pinned version ranges
- `hp41-gui/vite.config.ts` — Vite config with strictPort: true, port 5173, react() and tailwindcss() plugins
- `hp41-gui/tsconfig.json` — strict TypeScript config for Vite + React 19, moduleResolution bundler
- `hp41-gui/index.html` — HTML entry: div#root, title "HP-41 Calculator", module script /src/main.tsx
- `hp41-gui/src/main.tsx` — React 19 entry: ReactDOM.createRoot with React.StrictMode, imports index.css
- `hp41-gui/src/App.tsx` — minimal App: empty div.app (blank window for Phase 13; CSS hook for Phase 15)
- `hp41-gui/src/index.css` — Tailwind v4: @import "tailwindcss" (single line, no tailwind.config.js)

## Decisions Made

- React.StrictMode enabled — Phase 15 keyboard event handlers MUST include cleanup functions in useEffect to prevent duplicate event listener bug (Pitfall #11 direct mitigation)
- strictPort: true — tauri.conf.json hardcodes http://localhost:5173; if port busy, dev server must fail-fast not silently switch
- @tauri-apps/api and @tauri-apps/cli both pinned to ^2.11 — same minor version as Rust tauri = "2.11" (Pitfall #1 prevention)
- Tailwind v4 with @tailwindcss/vite: no tailwind.config.js needed; CSS-only approach via @import "tailwindcss"
- moduleResolution: "bundler" not "node16" — correct for TypeScript 6 with Vite bundler

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required. Dependencies will be installed via `just gui-install` (runs `npm install` in hp41-gui/).

## Next Phase Readiness

- Frontend scaffold complete — Phase 14 (IPC Layer) can immediately write React components and invoke() calls
- All 11 npm packages declared; `just gui-install` installs them before first `just gui-dev` run
- Tailwind v4 ready — Phase 15 (Display & Keyboard) can use utility classes immediately
- React.StrictMode active — Phase 15 useEffect hooks MUST return cleanup functions (keyboard event listeners)
- Port 5173 locked — tauri.conf.json (created in Plan 01) must set devUrl: http://localhost:5173

---
*Phase: 13-workspace-skeleton*
*Completed: 2026-05-09*
