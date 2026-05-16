# Pitfalls: Adding Tauri v2 GUI to the hp41 Cargo Workspace

**Milestone:** v2.0 Tauri GUI
**Researched:** 2026-05-09
**Scope:** Mistakes specific to adding `hp41-gui` (Tauri v2 + React + TypeScript) as a new
workspace member alongside `hp41-core` and `hp41-cli`. Generic Tauri or React pitfalls are
only included when they interact with this codebase's specific architecture.

---

## Summary

Five categories of mistakes dominate Tauri-to-existing-workspace integrations. The most
dangerous ones are invisible at compile time and surface only at runtime or in CI:

1. **Workspace wiring mistakes** — `tauri-build` version mismatches, missing `build.rs`,
   wrong `resolver`, broken workspace inheritance in `[build-dependencies]`.
2. **CalcState ownership mistakes** — `std::sync::Mutex` held across `.await`, wrong `State<>` type
   causing runtime panics, `#[serde(skip)]` on `print_buffer` silently dropping state.
3. **IPC design mistakes** — returning the full serialized `CalcState` on every keypress,
   `program: Vec<Op>` bloating JSON payloads, sync command on main thread blocking the UI.
4. **SVG UI mistakes** — duplicate keyboard event listeners accumulating across React
   re-renders, missing `useEffect` cleanup causing stale invocations after hot-reload.
5. **CI mistakes** — missing Linux system libraries, npm/cargo version drift, build matrix
   width causing 20-minute CI runs on every commit.

All five areas have concrete prevention steps described below.

---

## Pitfall 1: `tauri-build` Version Drift from the `tauri` Crate

**What goes wrong:**
`tauri-build` is a `[build-dependencies]` entry; `tauri` is a `[dependencies]` entry. Both must
stay on the same minor release (e.g. both `2.5.x`). If you pin one in
`[workspace.dependencies]` and add the other manually in `hp41-gui/Cargo.toml`, they drift.
Tauri v2 CLI actively detects minor-version mismatches (since `tauri-apps/tauri#13993`) and
blocks builds. The error message is not always clear that it is a version-sync problem.

There is also a known limitation: `[build-dependencies]` entries in a member crate **cannot
inherit** from `[workspace.dependencies]` in all Cargo versions (GitHub issue #6122 was filed
against Tauri for this). The workaround is to write `tauri-build` versions explicitly in
`hp41-gui/Cargo.toml` rather than routing them through workspace inheritance.

**Warning Signs:**
- Build fails with "failed to select a version" or "links value conflict" errors.
- `cargo tauri build` reports a Tauri version mismatch between CLI and crate.
- `tauri-build` builds fine but the app refuses to start.

**Prevention:**
- Do NOT add `tauri` or `tauri-build` to `[workspace.dependencies]`. Declare both explicitly
  in `hp41-gui/Cargo.toml` with matching minor versions.
- When upgrading, run `cargo update -p tauri -p tauri-build` together and verify both resolve
  to the same `2.x.y` minor line.
- Pin `@tauri-apps/cli` in `package.json` to the same minor version as the cargo crate.
  Plugin packages (e.g. `@tauri-apps/plugin-fs`) must match their cargo counterpart exactly
  (identical patch version, not just minor).
- Add a CI step that asserts `tauri --version` matches the version in `hp41-gui/Cargo.toml`.

**Which Phase:** Phase 1 (workspace scaffolding). Get this right before any other code is written.

---

## Pitfall 2: Wrong Workspace `resolver` Breaking Tauri's Dependency Graph

**What goes wrong:**
The current `Cargo.toml` declares `resolver = "2"` correctly. Tauri v2 requires `resolver = "2"`
(edition 2021 semantics) for its dependency feature unification to work. If the resolver is
dropped or overridden by a nested workspace member, the GTK/WebKit sys-crates on Linux emit
"Only one package in the dependency graph may specify the same links value" because two
incompatible versions of `webkit2gtk-sys` (or `gdk-sys`) get resolved simultaneously.

**Warning Signs:**
- Linux-only build failure: "package `gtk-sys` links to native library `gtk-3`, but it conflicts
  with a previous package which links to `gtk-3`."
- Works on macOS, fails on Ubuntu CI.

**Prevention:**
- `hp41-gui/Cargo.toml` must not declare its own `[workspace]` section. It is a workspace
  member, not a workspace root.
- Confirm that `Cargo.toml` (workspace root) keeps `resolver = "2"`.
- After adding `hp41-gui` to `members = [...]`, run `cargo check --all` on Ubuntu locally or
  in CI before merging.

**Which Phase:** Phase 1 (workspace scaffolding).

---

## Pitfall 3: `hp41-cli` Broken by the New Workspace Member

**What goes wrong:**
Adding `hp41-gui` as a workspace member adds Tauri's large dependency tree to the workspace.
Tauri depends on `tokio`, `hyper`, and dozens of system crates. If any of these conflict with a
crate version already used by `hp41-cli` (e.g. `serde 1.x` features, `thiserror 2.x`), `hp41-cli`
may silently acquire unexpected features or fail to compile after the workspace expansion.

The existing workspace pins `thiserror = "2.0"` and `rust_decimal = "1.42"`. Tauri 2.x uses
`thiserror` internally; if Tauri requires a version that Cargo resolves to an incompatible patch,
the existing workspace pin protects against the worst case — but verify this after adding
`hp41-gui`.

**Warning Signs:**
- `cargo test -p hp41-core` passes before adding `hp41-gui`, fails after.
- `just ci` fails on `hp41-cli` targets after the workspace member is added.

**Prevention:**
- After adding `hp41-gui` to `members`, run `just ci` (full suite) before writing any Tauri
  application code.
- Do not add Tauri-specific versions of shared crates (`serde`, `thiserror`) to
  `[workspace.dependencies]` — let Cargo unify them naturally.
- The `hp41-core` invariant (`#![deny(clippy::unwrap_used)]`) must remain enforced; check that
  Clippy still passes on `hp41-core` after the workspace expansion.

**Which Phase:** Phase 1 (workspace scaffolding). Run full CI immediately after adding the member.

---

## Pitfall 4: Missing or Wrong `build.rs` in `hp41-gui`

**What goes wrong:**
Tauri requires `src-tauri/build.rs` (or `hp41-gui/build.rs` in workspace layout) to contain
`tauri_build::build()`. Without it, the Tauri code-generation step for icons, permissions,
and platform-specific embedding does not run. The binary compiles but behaves incorrectly at
runtime (missing assets, wrong bundle identifiers).

The `build.rs` file is not inherited from the workspace; each Tauri binary crate needs its own.

**Warning Signs:**
- Binary builds but shows no window or shows a blank white screen.
- Bundle identifier is `com.tauri.dev` (the scaffold default was never overridden).
- App icon is the generic Tauri robot.

**Prevention:**
```rust
// hp41-gui/build.rs — this exact content, nothing more
fn main() {
    tauri_build::build()
}
```
- Verify `bundle.identifier` in `tauri.conf.json` is set to a real value (e.g.
  `ch.talent-factory.hp41`), not the scaffold default.
- Add `tauri_build` to `[build-dependencies]` (not `[dependencies]`) in `hp41-gui/Cargo.toml`.

**Which Phase:** Phase 1 (workspace scaffolding).

---

## Pitfall 5: Wrong `State<>` Type in Tauri Commands Causing Runtime Panics

**What goes wrong:**
Tauri's `State<T>` injection uses runtime type-lookup, not compile-time resolution. If you
register `Mutex<CalcState>` via `.manage(Mutex::new(CalcState::new()))` but accidentally write
a command parameter as `State<'_, CalcState>` (missing the `Mutex` wrapper), the application
panics at the first invocation with a runtime error rather than a compile-time error.

```rust
// WRONG — runtime panic when command is called
#[tauri::command]
fn dispatch_op(op: String, state: State<'_, CalcState>) { ... }

// CORRECT
#[tauri::command]
fn dispatch_op(op: String, state: State<'_, Mutex<CalcState>>) { ... }
```

**Warning Signs:**
- App starts, buttons do nothing, browser console shows an uncaught runtime error on the first
  `invoke()` call.
- Rust test suite passes but the running app panics.

**Prevention:**
- Define a type alias at the top of `hp41-gui/src/lib.rs`:
  `type AppState = Mutex<CalcState>;`
  and use `State<'_, AppState>` everywhere — a single wrong type becomes a compile error
  instead of scattered runtime panics.
- Add a smoke test that calls every registered command once as part of integration testing.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring).

---

## Pitfall 6: `std::sync::Mutex` Guard Held Across `.await` Deadlocks

**What goes wrong:**
If a Tauri command is declared `async` and holds a `std::sync::Mutex` guard across an `.await`
point, the future becomes `!Send` — the compiler may reject it, or (worse, if the guard happens
to implement `Send`) it silently deadlocks.

For `CalcState`, the natural pattern is:

```rust
#[tauri::command]
async fn dispatch_op(state: State<'_, Mutex<CalcState>>) -> Result<DisplayUpdate, AppError> {
    let mut s = state.lock().unwrap(); // guard held here
    s.dispatch(op)?;                   // synchronous, fast — OK
    Ok(DisplayUpdate::from(&*s))
    // guard dropped here before return — OK
}
```

This is safe because `dispatch()` is synchronous and the guard is not held across any await.
The problem arises if someone adds `tokio::time::sleep().await` or a file I/O call inside the
lock. Using `tokio::sync::Mutex` instead of `std::sync::Mutex` allows holding guards across
`.await` — but introduces its own risk: the Tokio mutex is slower and can deadlock differently
if misused.

**Warning Signs:**
- App freezes after a burst of rapid keypresses.
- Async command never resolves (Promise never settles in the frontend).
- Clippy emits "future is not Send" for the command function.

**Prevention:**
- Keep `CalcState` dispatch synchronous. Use `std::sync::Mutex<CalcState>`. The HP-41
  emulation is microseconds per op; there is no reason to go async inside the lock.
- Never hold the `MutexGuard` across any `.await` point.
- Mark commands `#[tauri::command]` (sync) rather than `async` unless there is a concrete
  reason. Sync commands run on the main thread — acceptable for sub-microsecond HP-41 ops.
- If a command must be async (e.g. file save), acquire the lock, copy out the data needed,
  drop the lock, then do the async work with the copy.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring).

---

## Pitfall 7: Mutex Poisoning from `#![deny(clippy::unwrap_used)]` Interaction

**What goes wrong:**
`hp41-core` enforces `#![deny(clippy::unwrap_used)]` — all panics must be explicit `.expect()`.
In `hp41-gui`, `state.lock().unwrap()` on a poisoned `Mutex` will be flagged. Developers
sometimes work around this by replacing `.unwrap()` with `.expect("poisoned")`, which still
panics. More subtly: if any earlier command handler panics (even in a test), the mutex is
poisoned and all subsequent commands fail with "mutex poisoned" instead of the original error.

**Warning Signs:**
- A burst of errors all report "poisoned" after one unrelated panic.
- `cargo clippy -p hp41-gui -- -D clippy::unwrap_used` fails.

**Prevention:**
- Extend the `#![deny(clippy::unwrap_used)]` lint to `hp41-gui` from the start.
- Handle the `PoisonError` explicitly in the command layer:
  ```rust
  let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
  ```
  This recovers the inner data and continues. For a single-user desktop app where the
  state is non-critical, recovering from poison is safe — the emulator state is at worst
  in an inconsistent position, which the user can fix by reloading.
- Alternatively, use `parking_lot::Mutex` which never poisons — but adds a dependency.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring).

---

## Pitfall 8: Returning the Full `CalcState` JSON on Every Keypress

**What goes wrong:**
`CalcState` serializes to roughly 3–6 KB of JSON at rest (100 registers × HpNum string
representation, `program: Vec<Op>` for any loaded program, `key_assignments` BTreeMap).
If a command returns the full state after every op dispatch, each keypress triggers:
1. `serde_json::to_string(&*state)` on the Rust side.
2. JSON parse in the WebView JavaScript engine.
3. React state diffing against the previous full object.
4. Re-render of all display components.

With IPC serialization overhead (~0.5–2ms per round-trip on macOS, higher on Windows), this
is imperceptible for the HP-41's 1-key-at-a-time input. However, when running a program loop
(`XEQ`) that executes thousands of ops, returning full state on every op saturates the IPC
channel. The `Vec<Op> program` field is the most likely bloat source — a loaded program can
be hundreds of ops, each serializing to a JSON object.

**Warning Signs:**
- Program execution (`XEQ`) feels sluggish in the GUI compared to the CLI.
- DevTools Network tab shows hundreds of IPC payloads per second during program run.
- IPC latency jumps when a long program is loaded.

**Prevention:**
- Define a purpose-built `DisplayUpdate` struct for what the frontend actually needs after
  each keypress: display string, annunciators, stack X/Y/Z/T, entry mode. This is ~200 bytes
  of JSON, not 6 KB.
  ```rust
  #[derive(Serialize)]
  pub struct DisplayUpdate {
      pub display: String,
      pub stack_x: String,
      pub stack_y: String,
      pub stack_z: String,
      pub stack_t: String,
      pub prgm_mode: bool,
      pub alpha_mode: bool,
      pub user_mode: bool,
      pub is_running: bool,
      pub annunciators: Vec<String>,
  }
  ```
- Provide a separate `get_full_state` command that the frontend calls only on startup or
  after load/save operations — never after a keypress.
- The `program: Vec<Op>` field should be gated behind a "show program listing" command only.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring). Design `DisplayUpdate` before
writing any frontend code.

---

## Pitfall 9: `#[serde(skip)]` on `print_buffer` Dropped by `DisplayUpdate`

**What goes wrong:**
`CalcState.print_buffer` is marked `#[serde(default, skip)]` — it never serializes into JSON.
In `hp41-cli`, `hp41-cli` drains `print_buffer` after every dispatch via
`call_dispatch_and_drain()`. In `hp41-gui`, if the command handler does not explicitly drain
and forward the print buffer, all `PRX`/`PRA`/`PRSTK` output is silently discarded. No error,
no panic — it just disappears.

**Warning Signs:**
- `PRX` or `PRSTK` keys do nothing visible in the GUI.
- Print output visible in CLI but absent in GUI.

**Prevention:**
- After calling `dispatch()` in the Tauri command handler, check and drain `print_buffer`
  before dropping the lock:
  ```rust
  let lines: Vec<String> = s.print_buffer.drain(..).collect();
  ```
- Include `print_lines: Vec<String>` in `DisplayUpdate`. The frontend displays them in a
  print-log panel.
- Write an integration test: dispatch `Op::PRX`, assert `DisplayUpdate.print_lines` is
  non-empty.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring).

---

## Pitfall 10: `anyhow::Error` Not Usable as a Tauri Command Return Type

**What goes wrong:**
Tauri commands must return `Result<T, E>` where `E: serde::Serialize`. Neither
`anyhow::Error` nor `Box<dyn std::error::Error>` implements `Serialize`. The naïve fix of
`.map_err(|e| e.to_string())` works but loses type information, making frontend error handling
impossible. `hp41-core` exposes `CalcError` via `thiserror`; wrapping it in a new GUI-layer
error enum that derives both `thiserror::Error` and `serde::Serialize` is the correct path.

```rust
// WRONG — anyhow::Error does not implement Serialize
#[tauri::command]
fn dispatch_op(...) -> anyhow::Result<DisplayUpdate> { ... }

// CORRECT
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum GuiError {
    #[error("Calculator error: {0}")]
    Calc(#[from] hp41_core::CalcError),
    #[error("State lock poisoned")]
    Poisoned,
}
```

**Warning Signs:**
- Compile error: "the trait `serde::Serialize` is not implemented for `anyhow::Error`."
- All errors appear as opaque string messages in the frontend.

**Prevention:**
- Create `hp41-gui/src/error.rs` with a `GuiError` enum as the single error type for all
  commands. Derive `thiserror::Error` and `serde::Serialize` on it.
- Use `#[from] hp41_core::CalcError` so the `?` operator converts automatically.
- Never use `anyhow` in `hp41-gui` command signatures.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring). Define `GuiError` before
writing any command bodies.

---

## Pitfall 11: Duplicate Keyboard Event Listeners from React `useEffect` Leaks

**What goes wrong:**
The HP-41 GUI uses keyboard input as a primary interaction mode (same key bindings as the CLI).
In React, the natural pattern is:

```ts
useEffect(() => {
  window.addEventListener('keydown', handleKey);
  return () => window.removeEventListener('keydown', handleKey);
}, [handleKey]);
```

If `handleKey` is an inline arrow function or a function that captures a stale closure,
`handleKey` changes identity on every render, causing the `useEffect` to re-register a new
listener on every render without removing the old one. After 10 renders, 10 independent
listeners fire for every keypress, issuing 10 IPC calls for a single key.

This is a React problem, but it is particularly harmful here because each `invoke()` call
mutates shared `Mutex<CalcState>` — 10 concurrent `dispatch` invocations fight for the
same lock, producing incorrect calculator state.

**Warning Signs:**
- Single keypress triggers multiple op dispatches (display shows X incremented multiple times).
- Console shows multiple identical `invoke('dispatch_op', ...)` calls per keypress.
- Problem worsens with hot-module reload during development.

**Prevention:**
- Wrap `handleKey` in `useCallback` with a stable dependency array.
- Never put the `addEventListener` call outside a `useEffect` — it will re-register on
  every render.
- During development, enable React StrictMode; it double-invokes effects intentionally to
  expose cleanup failures.
- Add an integration test: render the calculator, simulate 3 key events, assert exactly 3
  IPC calls were made (use `vi.mock` on the `invoke` function).

**Which Phase:** Phase 3 (SVG UI and keyboard wiring).

---

## Pitfall 12: SVG Click Targets Too Small for Desktop Use

**What goes wrong:**
The HP-41C hardware keys are small. A direct 1:1 SVG recreation with pixel-accurate key sizes
will have click targets of 8–12px on a standard display. Desktop users can click them, but it
is frustrating; the experience regresses significantly from the keyboard-driven CLI.

The deeper problem: SVG elements do not automatically have a "hover" state or focus ring
unless explicitly styled. Without visual feedback on hover/active, users cannot tell which
key their pointer is over.

**Warning Signs:**
- Testers report frequent mis-clicks on adjacent keys.
- No visual feedback when hovering or clicking.

**Prevention:**
- Make the SVG responsive — it should fill its container and scale with the window. Use
  `viewBox` (not fixed `width`/`height`) on the root `<svg>`.
- Apply a minimum effective touch/click target of 24px using an invisible `<rect>` overlay
  per key (standard for desktop at 100% DPI), distinct from the visible key path.
- Add CSS `:hover` and `:active` states on the key `<g>` elements.
- Keyboard input (inherited from CLI key bindings) remains the primary interaction mode;
  the SVG skin is a supplement, not a replacement.

**Which Phase:** Phase 3 (SVG UI and keyboard wiring).

---

## Pitfall 13: Linux CI Failing Due to Missing `webkit2gtk-4.1` System Libraries

**What goes wrong:**
Tauri v2 links against `libwebkit2gtk-4.1` (not `4.0` as in v1). The GitHub Actions workflow
must install this library explicitly on the Ubuntu runner before `cargo build`. The list of
required packages changes between Ubuntu 22.04 and 24.04. Tauri v1 docs and many blog posts
list the wrong package names for v2. Using a stale list causes "webkit2gtk-4.1 required by
crate `webkit2gtk-sys` was not found" during the build step.

The existing CI (`just ci`) runs `cargo test` and coverage for `hp41-core` and `hp41-cli` only.
Adding `hp41-gui` requires a separate CI matrix that:
1. Installs system libraries on Linux.
2. Runs `cargo tauri build` (not `cargo build`) because `tauri-build` generates assets.
3. Handles macOS universal binary (two targets).
4. Signs on macOS/Windows for distribution builds.

**Warning Signs:**
- Linux CI step fails with "pkg-config: No package 'webkit2gtk-4.1' found."
- macOS build succeeds but Linux/Windows fail silently.

**Prevention:**
The official Tauri v2 GitHub Actions guide lists the required packages for `ubuntu-22.04`:
```yaml
- name: Install Linux dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y libwebkit2gtk-4.1-dev \
      libappindicator3-dev librsvg2-dev patchelf
```
For `ubuntu-24.04` (LTS as of 2024), the package names are identical. Verify against the
official Tauri v2 prerequisites page at https://v2.tauri.app/start/prerequisites/ when
setting up CI.

- Keep the existing `just ci` (hp41-core + hp41-cli) job unchanged. Add a separate
  `gui-build` job that uses `tauri-apps/tauri-action` and runs only when files under
  `hp41-gui/` or `hp41-core/` change.
- Use `swatinem/rust-cache@v2` with a cache key that includes the `hp41-gui` Cargo.lock
  hash. Tauri's large dependency tree makes cold builds 8–15 minutes; caching reduces this
  to 2–4 minutes.
- Gate release builds (signing, bundling) on `push: tags` events, not on every PR.

**Which Phase:** Phase 4 (CI/CD). But read this before phase 1 to avoid painting yourself
into a corner with CI assumptions.

---

## Pitfall 14: macOS Universal Binary Requires Both Targets Before Bundling

**What goes wrong:**
A macOS `.app` bundle for "Apple Silicon + Intel" requires compiling for both
`aarch64-apple-darwin` and `x86_64-apple-darwin` and combining with `lipo`. Using
`cargo tauri build --target universal-apple-darwin` handles this, but both targets must be
installed in the Rust toolchain first. If one target is missing, the build silently produces
a single-arch binary without warning.

GitHub's `macos-latest` runners are now ARM64 (`macos-14`) by default. Running on `macos-14`
without installing `x86_64-apple-darwin` produces an ARM-only binary, which fails on Intel Macs.

**Warning Signs:**
- Users on Intel Macs report "app is damaged or incompatible."
- `file hp41.app/Contents/MacOS/hp41` shows only one architecture.

**Prevention:**
```yaml
- name: Add macOS targets
  run: |
    rustup target add aarch64-apple-darwin
    rustup target add x86_64-apple-darwin
- name: Build universal binary
  run: cargo tauri build --target universal-apple-darwin
```

**Which Phase:** Phase 4 (CI/CD).

---

## Pitfall 15: GitHub Actions Token Write Permission Required for Release Assets

**What goes wrong:**
The `tauri-apps/tauri-action` GitHub Action creates release assets (`.dmg`, `.msi`, `.AppImage`)
and attaches them to a GitHub Release. By default, the `GITHUB_TOKEN` has read-only permissions
for actions triggered by PRs from forks. Creating release assets requires `contents: write`
permission. Without it, the action silently skips attaching assets and emits a
"Resource not accessible by integration" error in the Actions log.

**Warning Signs:**
- Release is created but has no attached binary assets.
- Action log shows "Resource not accessible by integration."

**Prevention:**
```yaml
permissions:
  contents: write
jobs:
  release:
    ...
```
Add `permissions: contents: write` at the job level (not workflow level) for the release job.

**Which Phase:** Phase 4 (CI/CD).

---

## Pitfall 16: Forgetting to Register Commands in `generate_handler!`

**What goes wrong:**
Tauri commands are only reachable from the frontend if they are listed in
`tauri::Builder::invoke_handler(tauri::generate_handler![...])`. Forgetting to add a new
command to the macro does not produce a compile error — it compiles fine. The frontend
receives a "Command not found" error at runtime.

Additionally, only one `invoke_handler` call is permitted per `Builder`. A common mistake when
incrementally adding commands is calling `invoke_handler` twice, where only the second
registration takes effect (the first is silently overwritten).

**Warning Signs:**
- Frontend `invoke('new_command', ...)` rejects with "Command 'new_command' not found."
- Only some commands work; others silently fail.

**Prevention:**
- Keep all commands in a single `generate_handler![cmd1, cmd2, cmd3]` call.
- Use a Rust integration test that calls `tauri::test::mock_app` and asserts all expected
  commands are registered.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring).

---

## Pitfall 17: `last_key_code` Update Architecture Between CLI and GUI

**What goes wrong:**
`CalcState.last_key_code` is written by `hp41-cli`'s `handle_key()` — it is updated in the
CLI event loop, not inside `dispatch()`. This is correct for the CLI because the CLI controls
both the physical key event and the dispatch.

In `hp41-gui`, the frontend sends a key name (e.g. `"enter"`, `"sqrt"`) to the Rust command.
If the command handler calls `dispatch()` without first writing `last_key_code`, the `GETKEY`
instruction in keystroke programs will always read 0. The GUI must replicate the CLI's
`handle_key()` logic: convert the key identifier to the HP-41 row-column key code and write
it to `state.last_key_code` before calling `dispatch()`.

**Warning Signs:**
- `GETKEY` programs that work in the CLI return 0 in the GUI.
- Synthetic programs behave differently across the two frontends.

**Prevention:**
- Extract the key-code mapping from `hp41-cli/src/keys.rs` into a function in `hp41-core`
  (or a shared helper crate), so both `hp41-cli` and `hp41-gui` use the same mapping.
- Alternatively, accept the HP-41 key code directly as an argument to the Tauri command
  (`dispatch_op(op: String, key_code: u8, state: ...)`).
- Add a test: dispatch `Op::GetKey` from the GUI path, assert `last_key_code` is non-zero.

**Which Phase:** Phase 2 (Tauri commands / CalcState wiring). Discovered during synthetic
programming research; the v1.1 Phase 12 implementation sets `last_key_code` in `app.rs`
`handle_key()` before `dispatch()`.

---

## Phase-Specific Warning Matrix

| Phase | Topic | Highest Risk Pitfall | Mitigation |
|-------|-------|---------------------|------------|
| 1: Workspace scaffold | Cargo.toml wiring | `tauri-build` version drift (P1) | Pin versions explicitly; run `just ci` immediately |
| 1: Workspace scaffold | resolver | `resolver = "2"` dropped by member (P2) | Confirm root Cargo.toml unchanged |
| 1: Workspace scaffold | CLI regression | hp41-cli broken by new workspace member (P3) | Run `just ci` before any Tauri code |
| 2: Commands / state | State type | Wrong `State<>` type, runtime panic (P5) | `type AppState = Mutex<CalcState>` alias |
| 2: Commands / state | Concurrency | Mutex guard across `.await` (P6) | Keep dispatch sync; no await inside lock |
| 2: Commands / state | IPC payload | Returning full `CalcState` JSON (P8) | Design `DisplayUpdate` struct first |
| 2: Commands / state | Print buffer | `print_buffer` silently dropped (P9) | Drain explicitly; include in `DisplayUpdate` |
| 2: Commands / state | Error types | `anyhow` not serializable (P10) | `GuiError` with `serde::Serialize` |
| 2: Commands / state | Command registration | Missing from `generate_handler!` (P16) | Single `generate_handler!` call |
| 2: Commands / state | `last_key_code` | Not updated before `dispatch()` (P17) | Replicate CLI key-code mapping |
| 3: SVG + keyboard | React lifecycle | Duplicate event listeners (P11) | `useCallback` + cleanup in `useEffect` |
| 3: SVG + keyboard | Click targets | Keys too small (P12) | `viewBox` + invisible overlay rects |
| 4: CI/CD | Linux | Missing webkit2gtk-4.1 (P13) | Use official Tauri v2 apt package list |
| 4: CI/CD | macOS | Single-arch binary (P14) | Install both Rust targets; `universal-apple-darwin` |
| 4: CI/CD | GitHub token | Release assets not attached (P15) | `permissions: contents: write` |

---

## Sources

- Tauri v2 Project Structure: https://v2.tauri.app/start/project-structure/
- Tauri v2 State Management: https://v2.tauri.app/develop/state-management/
- Tauri v2 Calling Rust from Frontend: https://v2.tauri.app/develop/calling-rust/
- Tauri v2 Updating Dependencies: https://v2.tauri.app/develop/updating-dependencies/
- Tauri v2 GitHub Actions CI: https://v2.tauri.app/distribute/pipelines/github/
- Tauri v2 Permissions/Capabilities: https://v2.tauri.app/security/capabilities/
- tauri-build crate: https://crates.io/crates/tauri-build
- Tauri workspace inheritance bug #6122: https://github.com/tauri-apps/tauri/issues/6122
- Tauri workspace override build bug #6252: https://github.com/tauri-apps/tauri/issues/6252
- Tauri version mismatch detection #13993: referenced in https://github.com/Martichou/rquickshare/issues/407
- libwebkit2gtk-4.0 unavailable in Ubuntu 24: https://github.com/tauri-apps/tauri/issues/9662
- Tokio Mutex vs std Mutex (deadlock if Send implemented): https://tokio.rs/tokio/tutorial/shared-state
- Tauri error handling best practices: https://tauritutorials.com/blog/handling-errors-in-tauri
- IPC performance / large JSON: https://github.com/tauri-apps/tauri/issues/5641
- Tauri macOS universal binary discussion: https://github.com/orgs/tauri-apps/discussions/9419
- Tauri IPC performance vs React Native JSI: https://github.com/orgs/tauri-apps/discussions/11915
