// WebdriverIO + tauri-driver E2E config (D-27.15 AMENDED 2026-05-15).
//
// Drives the production Tauri binary on Ubuntu (WebKitGTK) for the
// FN-QUAL-05 literal ROADMAP smoke (Plan 27-04, Phase 27).
//
// Preconditions for `just gui-e2e`:
//   1. `just gui-build` has produced hp41-gui/src-tauri/target/release/hp41-gui
//   2. `cargo install tauri-driver --locked --version 2.0.6` is on PATH
//      (typically ~/.cargo/bin/tauri-driver)
//   3. On a headless Ubuntu runner, wrap with `xvfb-run -a`.
//
// D-27.15 AMENDED replaces the original Playwright wording: tauri-driver 2.0.6
// speaks WebDriver classic which Playwright does NOT (CDP/native only). WebdriverIO
// is the protocol-compatible Tauri-recommended client. The CI workflow
// (.github/workflows/ci-gui.yml::e2e-linux, added in Plan 27-04 Task 4) runs
// `just gui-build` BEFORE `just gui-e2e`, so this config does NOT re-build the
// binary (no onPrepare step — keeps local + CI usage symmetric).

const os = require('os');
const path = require('path');
const { spawn } = require('child_process');

let tauriDriver;

exports.config = {
  // Spec glob — accept both .ts and .js so the executor can pick either
  // depending on whether the WDIO TS loader plays nicely with the current
  // tsconfig. The smoke is small (~20 lines); types are not load-bearing.
  specs: ['./e2e/**/*.spec.ts', './e2e/**/*.spec.js'],
  maxInstances: 1,
  capabilities: [{
    maxInstances: 1,
    // tauri-driver passes this through to webkitwebdriver as the application
    // to spawn. Binary name resolved during Plan 27-04 read_first:
    //   hp41-gui/src-tauri/Cargo.toml [[bin]] name = "hp41-gui"
    // -> hp41-gui/src-tauri/target/release/hp41-gui
    'tauri:options': {
      application: '../src-tauri/target/release/hp41-gui',
    },
  }],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
    // D-27.16: 1 retry tolerates transient WebKitGTK / Xvfb startup hiccups
    // without masking real regressions (a retry would also fail on a true
    // dispatch-chain breakage).
    retries: 1,
  },

  // Spawn tauri-driver on 127.0.0.1:4444 before each WDIO session; kill after.
  // Localhost-only; not exposed to the public internet (threat T-27-04-02).
  beforeSession: () => {
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    );
  },
  afterSession: () => {
    if (tauriDriver) tauriDriver.kill();
  },
};
