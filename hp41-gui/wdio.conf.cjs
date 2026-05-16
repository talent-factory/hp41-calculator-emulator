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

const net = require('net');
const os = require('os');
const path = require('path');
const { spawn } = require('child_process');

let tauriDriver;

// Probe TCP port 4444 in a loop until tauri-driver is accepting connections,
// or timeout. Without this, the first WDIO POST /session may race the driver's
// bind() and surface as "WebDriverError: Failed to match capabilities" /
// ECONNREFUSED — which mochaOpts.retries: 1 would then mask (silent flake).
function waitForPort(host, port, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  return new Promise((resolve, reject) => {
    const attempt = () => {
      const sock = net
        .createConnection({ host, port })
        .on('connect', () => {
          sock.end();
          resolve();
        })
        .on('error', () => {
          sock.destroy();
          if (Date.now() > deadline) {
            reject(
              new Error(
                `tauri-driver did not bind ${host}:${port} within ${timeoutMs}ms`,
              ),
            );
          } else {
            setTimeout(attempt, 100);
          }
        });
    };
    attempt();
  });
}

exports.config = {
  // Connect to the local tauri-driver spawned by beforeSession (line ~60).
  // Without these, WDIO 9 errors: "No browserName defined in capabilities
  // nor hostname or port found!" — it cannot infer where to send WebDriver
  // commands from `tauri:options` alone. Match the port tauri-driver
  // listens on (default 4444); localhost-only so no inbound exposure.
  hostname: '127.0.0.1',
  port: 4444,
  // Spec glob — `.spec.js` only. The smoke is small (~60 lines) and uses
  // ambient `declare` globals for Mocha + WDIO at runtime, so type-checking
  // adds no value. Sticking to `.js` removes the WDIO 9 → tsx auto-detection
  // dependency (no `tsx` devDep needed) and the matching tsconfig include
  // rule. If a future spec genuinely needs TS, add `tsx` to devDeps AND
  // restore `./e2e/**/*.spec.ts` here AND update tsconfig.json `include`.
  specs: ['./e2e/**/*.spec.js'],
  maxInstances: 1,
  capabilities: [{
    maxInstances: 1,
    // No `browserName` — tauri-driver routes via the `tauri:options`
    // extension; setting browserName='wry' or other made-up values causes
    // the wrapped webkit2gtk-driver to reject the session with
    // "Failed to match capabilities". The hostname/port above already
    // satisfy WDIO 9's capability validation (it requires EITHER a
    // recognized browserName OR explicit hostname/port; we have the latter).
    // tauri-driver passes this through to webkitwebdriver as the application
    // to spawn. Path is interpreted relative to the WDIO process cwd, which
    // is `hp41-gui/` (per `just gui-e2e: cd hp41-gui && npx wdio ...`). The
    // binary lands at `hp41-gui/src-tauri/target/release/hp41-gui` per
    // Cargo.toml `[[bin]] name = "hp41-gui"`, so the relative path from
    // `hp41-gui/` is `src-tauri/...` — NOT `../src-tauri/...` (that would
    // escape to repo-root where no `src-tauri/` exists, surfacing as
    // "Failed to execute child process ... (No such file or directory)").
    'tauri:options': {
      application: 'src-tauri/target/release/hp41-gui',
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
  //
  // Spawn hardening (against silent-flake masking by mochaOpts.retries: 1):
  // - `.on('error', ...)` catches ENOENT (tauri-driver binary missing) and
  //   similar process-creation failures that spawn() does NOT throw on.
  // - `.on('exit', ...)` catches early termination (panic, EADDRINUSE) so a
  //   driver that died before WDIO's first POST /session doesn't surface
  //   only as a connection error.
  // - `waitForPort(...)` blocks until tauri-driver is actually accepting
  //   connections — otherwise the first WDIO POST races the bind() and a
  //   transient ECONNREFUSED could be masked by the 1-retry budget.
  beforeSession: async () => {
    const driverPath = path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver');
    tauriDriver = spawn(driverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });
    tauriDriver.on('error', (err) => {
      console.error(`tauri-driver spawn error (${driverPath}):`, err);
      process.exitCode = 1;
    });
    tauriDriver.on('exit', (code, signal) => {
      if (code !== 0 && code !== null) {
        console.error(
          `tauri-driver exited early (code=${code}, signal=${signal}) before session completed`,
        );
        process.exitCode = code;
      }
    });
    await waitForPort('127.0.0.1', 4444, 10000);
  },
  afterSession: () => {
    if (tauriDriver) {
      const killed = tauriDriver.kill();
      if (!killed) {
        console.warn(
          'tauri-driver.kill() returned false — process may be orphaned (rerun risks port 4444 collision)',
        );
      }
    }
  },
};
