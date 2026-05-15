/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || 'localhost',
  },
  build: {
    outDir: 'dist',
  },
  // Phase 26 Plan 02 — vitest needs jsdom for the Display14Seg.test.tsx
  // render tests (@testing-library/react requires a DOM). The existing
  // pending_input.test.ts is a pure-function test that runs equally well
  // under jsdom — no behavior change there.
  test: {
    environment: 'jsdom',
    globals: false,
  },
})
