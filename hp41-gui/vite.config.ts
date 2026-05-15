/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || 'localhost',
    // Phase 26 Plan 03 W8 — the JSON import in src/help_data.ts uses
    // `../../docs/hp41cv-functions.json`, which climbs outside the default
    // vite project root (`hp41-gui/`). Extend `fs.allow` to include the
    // repo root so the import resolves at build time.
    // `path.resolve(__dirname, '..')` evaluates to the repo root
    // (`<repo>/hp41-calculator-emulator/`).
    fs: {
      allow: [path.resolve(__dirname, '..')],
    },
  },
  build: {
    outDir: 'dist',
  },
  // Phase 26 Plan 02 — vitest needs jsdom for the Display14Seg.test.tsx
  // render tests (@testing-library/react requires a DOM). The existing
  // pending_input.test.ts is a pure-function test that runs equally well
  // under jsdom — no behavior change there.
  // Phase 26 Plan 04 — setupFiles enables the React 19 act() environment
  // for App.test.tsx integration tests; pure-function tests are unaffected.
  test: {
    environment: 'jsdom',
    globals: false,
    setupFiles: ['./src/test_setup.ts'],
  },
})
