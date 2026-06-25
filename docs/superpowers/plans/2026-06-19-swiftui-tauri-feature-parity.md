# SwiftUI–Tauri Full Feature-Parity Implementation Plan

**Goal:** Mirror every user-visible calculator and application function in the retained React/Tauri `hp41-gui` implementation in the native SwiftUI app, while keeping `hp41-core` as the single calculator engine and preserving existing autosave compatibility.

**Definition of parity:** A Tauri function is complete only when the same input is reachable in SwiftUI, invokes the same `hp41-core` semantics, projects the same observable state, has the native platform equivalent for any OS integration, and is covered by an automated parity test. DOM/CSS/WebView implementation details are not parity requirements; their user-visible behavior, accessibility, and workflows are.

**Architecture:** Extract the framework-independent parts of `src-tauri` (key resolution, dispatch orchestration, state projection, program execution/yields, modal flow, card I/O, resets, and persistence) into a reusable Rust application façade. Make both the legacy Tauri commands and `hp41-bridge` thin adapters over that façade during migration. Extend the C ABI with a generic JSON request/result boundary rather than adding one bespoke FFI function per operation. Swift owns presentation, preferences, file panels, lifecycle, App Intents, menus, keyboard routing, haptics, and macOS window/menu-bar behavior.

**Important constraint:** Do not extend the current hand-written `hp41-bridge::resolve` match. It exposes only a small subset of `src-tauri/src/key_map.rs` and will drift. Share or move the exhaustive resolver and test its identifier set mechanically.

---

## Baseline and inventory

The existing native app currently supports basic state loading/saving, a partial key layout, basic numeric/ALPHA entry, a small set of direct operations, a simplified SST/BST/R/S path, stack display, and limited physical-keyboard input. It does not yet project or drive most of the Tauri application state.

Treat these Tauri capabilities as the authoritative migration ledger:

| Area | Tauri source of truth | Native parity required |
|---|---|---|
| Dispatch and entry | `commands.rs::dispatch_op`, `key_map.rs`, `pending_input.ts` | Every direct, parameterized, named/XEQ, ALPHA, USER-assigned, modal, and GUI-only key ID; decimal/EEX limits, exponent CHS, and contextual backspace |
| State view | `types.rs::CalcStateView` | Display precedence; X/Y/Z/T/LASTX; annunciators; EEX; program steps/PC; USER keymap; flags; overrides; print/sound events; run/modal/live-clock/yield state |
| Program control | `sst_step`, `bst_step`, `run_stop`, `run_program`, `resume_program`, `resume_program_with_key`, `request_cancel` | Stepping, continuous execution, PSE/VIEW/AVIEW timed yields, GETKEY resume, cancellation, and foreground/background behavior |
| Modal workflows | `submit_modal`, `cancel_modal`, `submit_modal_with_label`, `pending_input.ts` | STO/RCL/GTO/LBL/XEQ/ASN, flags, display formats, tests, ISG/DSE, CLP, catalog and ROM modal programs, including function-name collection |
| Time/stopwatch | `tick_time` and live fields | 100 ms refresh while needed, clock exit rules, stopwatch keyboard mode and start/stop/exit routing |
| Persistence/reset | `get_state`, `save_state`, `reset_soft`, `reset_full`, `persistence.rs` | Compatible versioned autosave, lifecycle save, soft reset, long-press ON full-reset confirmation, MEMORY LOST feedback |
| Cards/files | import/export commands and `cards.rs` | RAW program preview/selection/import/export, data-card import/export, WPRGM/RDPRGM/WDTA/RDTA storage, errors, security-scoped native file access |
| Printer/sound | `print_lines`, `event_buffer` | Persistent session print log, open/close presentation, BEEP/TONE playback, iOS haptics where supported |
| Help | `HelpOverlay.tsx`, `help_data.ts`, JSON docs | Searchable HP-41CV/Math/Stat/Time/Advantage/XMEM reference, shortcuts tab, expandable examples/notes, tap-to-run where executable |
| Settings/onboarding | `SettingsPanel`, `ShortcutRecorder`, `OnboardingWizard`, `prefs.rs` | Four themes, onboarding completion/reopen, macOS launch mode, configurable global shortcut, restart affordance |
| Platform shell | `tray.rs`, `shortcut.rs`, `app_intents.rs`, lifecycle code in `App.tsx` | Native menu-bar/window modes, global shortcut, App Intents queue/resume, menus, visibility/lifecycle saves, iOS-safe behavior if iOS remains supported |
| Input/accessibility | `App.tsx`, `Keyboard.tsx`, `BottomSheet.tsx`, overlays | Complete physical shortcut map, pointer/keyboard parity, busy gating, long press, focus/dismiss rules, labels/hints/values, Dynamic Type/VoiceOver and minimum touch targets |

Before implementation, generate a machine-readable ledger containing every registered Tauri command, every `key_map::resolve` ID/pattern family, every `KEY_DEFS` primary/shifted/program key, every modal opener/result ID, every `CalcStateView` field, preference, shortcut, overlay, and platform event. Commit the generator and its checked-in snapshot so “ALL” remains enforceable as the legacy tree changes.

---

## Phase 1 — Establish a mechanical parity contract

- [x] Add `scripts/swiftui-parity` (or a small Rust test utility) that extracts the inventories above and emits `docs/swiftui-parity.json`.
- [x] Add a Swift-side capability manifest listing supported request IDs, state fields, settings, and UI workflows.
- [x] Add a drift test to `just gui-ci`; it validates every extracted item has a native disposition. During implementation `planned`/`partial` are permitted, while `--require-complete` enforces only `implemented`, `native-equivalent`, or justified `not-applicable` at cutover.
- [x] Capture golden request/result fixtures for representative basic, parameterized, modal, program-yield, time, statistics, matrix/Advantage, XMEM, card, print, and reset flows. The shared `bridge-golden-flows.json` corpus is replayed through both the Rust bridge and Swift service using stable JSON-pointer assertions; volatile clock text is intentionally excluded.
- [x] Record platform scope explicitly. The native Swift package currently declares macOS 14 only; retained Tauri iOS behavior is outside this parity claim until an iOS target is added.

**Exit criterion:** CI can report an exact parity count and name every remaining gap.

## Phase 2 — Build one reusable Rust application façade

- [x] Extract the exhaustive direct/parameterized key resolver and its 14 tests into the new UI-neutral `hp41-app` crate; make both Tauri and `hp41-bridge` consume it, deleting the bridge's hand-written resolver.
- [x] Extract canonical entry and dispatch orchestration into `hp41-app`: clock/stopwatch exit, digit/decimal/EEX rules, exponent sign, contextual backspace, resolver/core error preservation, and core dispatch. Keep card I/O in the adapters.
- [x] Extract the complete state projection, display precedence, yield/modal/live fields, transient print/event draining contract, error mapping, and exhaustive program-step formatter into `hp41-app`; decode the full payload in Swift with legacy bridge-key compatibility.
- [x] Extract SST/BST, R/S, labeled run/resume, GETKEY resume, modal submit/cancel/label, cancel signaling, and time ticks into `hp41-app`; expose them through a typed JSON C ABI and Swift model methods.
- [x] Extract versioned persistence and reset semantics into `hp41-app`; add explicit save/soft-reset/full-reset bridge requests, Swift commands, and scene-deactivation saving.
- [x] Create the UI-neutral `hp41-app` crate containing reusable resolver, request execution, state projection, program formatting/control, card/file transfer, and persistence logic formerly coupled to Tauri.
- [x] Define serializable `AppRequest`, `AppResponse`, `CalcStateView`, `GuiError`, RAW-picker metadata, and import/export/reset response shapes. The bridge now serializes these shared types directly.
- [x] Move the full dispatch pipeline into the façade: entry rules, exhaustive key resolution, native ALPHA/contextual-backspace aliases, card-operation prepare/I/O/finalize, event draining, and display priority.
- [x] Move program stepping/running/resume/GETKEY/yield/cancel and modal submit/cancel/label handling without changing `hp41-core` semantics.
- [x] Keep file-panel selection and platform preferences outside the façade; adapters provide paths while `hp41-app` owns validation, encoding, import/export, and request execution.
- [x] Convert Tauri commands to thin wrappers temporarily and run their Rust tests against the same `hp41-app` paths used by the bridge.
- [x] Replace the bridge’s duplicate request schema/executor, `StateFile`, `StateView`, resolver, entry aliases, step, and R/S logic with the façade.

**Exit criterion:** Tauri and bridge tests execute the same Rust paths; no calculator-operation resolver is duplicated.

## Phase 3 — Expand and harden the C ABI

- [x] Add the typed `hp41_request_json` boundary and Swift request helpers for state, stepping, run/resume/GETKEY, modal actions, cancellation signaling, and time ticks; retain `hp41_press` and `hp41_state_json` compatibility shims.
- [x] Extend `hp41_request_json` with dispatch, explicit save/reset, and RAW/data-card request variants plus the final status/error envelope.
- [x] Add explicit `ok`/`error` response status semantics, null/UTF-8 validation, panic containment, and documented ownership/thread rules. Invalid requests return a full error-state envelope when a live calculator is available; invalid creation inputs and null calculator handles return null.
- [x] Provide façade requests for dispatch, get/tick/step/run/resume/GETKEY/cancel, modal actions, save/reset, RAW/data encode/decode/import/export, and program selection.
- [x] Ensure long-running execution does not block Swift’s main actor. Use single-flight calculator execution and an independent atomic cancellation path.
- [x] Add Rust FFI tests for every request variant plus malformed JSON, unknown IDs, cancellation, repeated create/destroy, and state compatibility. Tests call the exported C functions, free returned strings, and load the retained v4.0 state fixture.
- [ ] Generate or validate the C header from the Rust ABI to prevent signature drift.

**Exit criterion:** Every non-platform Tauri command has a façade/FFI equivalent and round-trips through JSON tests.

## Phase 4 — Model the complete state and service boundary in Swift

- [x] Replace the reduced `CalculatorState` with Codable, Sendable types matching the complete façade `StateView`, including yields, modal fields, flags, assignments, print/events, and live time fields.
- [x] Split `CalculatorModel` into a main-actor observable view model and a serial `CalculatorService` that owns the opaque Rust pointer. The service serializes normal requests with a lock while cancellation retains its independent atomic path.
- [x] Implement an exhaustive typed/async Swift request boundary for every façade request and surface encoding, transport, decoding, and calculator errors consistently without losing the last valid state.
- [x] Add return-driven timed-yield scheduling, GETKEY capture/cancel sentinel routing, scene-aware 100 ms live ticking, alarm/event consumption, print accumulation, and lifecycle saves.
- [x] Move opaque calculator ownership off the main actor so cancellation can interrupt a no-yield compute segment concurrently.
- [x] Preserve display precedence exactly: pending yield and display override are layered over the shared Rust state display, whose tested priority remains clock/stopwatch/modal prompt/entry/program/ALPHA/AON/X.
- [x] Add decoder compatibility tests and replay the Phase 1 golden fixtures through both Rust and Swift. `CalculatorStateCodingTests` covers current/legacy decoding and `GoldenFlowTests` exercises the native service against the same corpus as the Rust bridge.

**Exit criterion:** Swift can observe and drive the entire calculator lifecycle without UI-specific shortcuts in the bridge.

## Phase 5 — Reach keyboard and operation parity

- [x] Replace `KeyboardLayout` with one canonical data definition covering all Tauri `KEY_DEFS`: top row, primary IDs, shifted IDs, PRGM-shift variants, ALPHA characters, key codes, spans, labels, and USER relabeling.
- [ ] Implement contextual routing for SHIFT, ALPHA, PRGM, stopwatch, GETKEY, USER assignments, backspace/CL X/A, ON tap/long press, and busy/running states.
- [ ] Port every modal opener and `pending_input.ts` transition as a tested Swift state machine; do not encode modal rules directly in view bodies.
- [ ] Port the complete physical-keyboard shortcut map, including Esc cancellation/reset escape behavior and save/help/program shortcuts.
- [ ] Make help “Run” actions use the same dispatch path as keys.
- [ ] Add parameterized tests asserting every canonical key and modal result maps to a façade-accepted ID; compare the set with `swiftui-parity.json`.

**Exit criterion:** No operation reachable from Tauri keys, shortcuts, help, USER mode, or modal entry is unreachable in SwiftUI.

## Phase 6 — Mirror calculator-facing UI workflows

- [x] Implement native STO/RCL/VIEW/ISG direct and indirect register entry, GTO/LBL label entry, ASN hardware-key assignment, FIX/SCI/ENG, SF/CF/FS?, TONE/CATALOG, and conditional tests using canonical dispatch IDs.
- [x] Implement CLP/DEL/SIZE, DSE/ARCL/ASTO, STO arithmetic, flag test-and-clear variants, and indirect GTO/XEQ entry with program-mode CLP key behavior.

- [ ] Reproduce the authentic 12-cell display behavior, all annunciators, overrides/yields, errors/toasts, and expandable X/Y/Z/T/LASTX panel.
- [x] Add calculator-key controls for SST, BST, state-routed R/S, timed yield continuation, and hardware-key-code GETKEY continuation.
- [x] Add expandable program listing/PC presentation and dedicated desktop program controls.
- [x] Add the session print log with a native desktop sheet; preserve drained-line ordering, repeated identical lines, clear/open controls, and auto-scroll.
- [x] Consume BEEP/TONE and alarm message/XEQ/missing events exactly once, play native macOS alert audio, and surface transient notices.
- [ ] Add compact/mobile printer presentation and supported touch haptics.
- [x] Add the soft-reset command and long-press ON full-reset confirmation with cancel/confirm and MEMORY LOST result.
- [ ] Match overlay focus and mutual-exclusion rules for modal input, help, settings, onboarding, RAW selection, shortcut recording, and reset confirmation.

**Exit criterion:** Golden UI flows produce equivalent visible state and can be completed using mouse/touch, keyboard, and accessibility actions.

## Phase 7 — Native file, card, and program exchange

- [x] Extract byte/path RAW archive inspection, atomic selected-program import, RAW export, and data-card encode/decode into `hp41-app`; expose typed bridge and Swift model requests.
- [x] Implement native SwiftUI/AppKit open/save panels for `.raw` and data-card files, including security-scoped access.
- [x] Decode RAW archives through Rust, show all discovered programs, allow multi-selection, and insert selected programs with the same PC/state semantics.
- [x] Export the current program and calculator data with the same format and native completion/error messages.
- [x] Route calculator `WPRGM`, `RDPRGM`, `WDTA`, and `RDTA` operations to the compatible cards directory, preserving filename sanitization and error behavior with a sandbox/test path override.
- [ ] Handle cancellation, malformed files, empty program sets, duplicate names, large files, sandbox/security-scoped URLs, and atomic writes.
- [x] Add RAW and data-card round-trip integration tests using temporary directories, including empty and multi-program archives.

**Exit criterion:** Files written by either app can be read by the other and produce identical calculator state.

## Phase 8 — Help, settings, onboarding, and themes

- [x] Generate a native searchable XEQ catalog from the canonical help JSON, including aliases, categories, descriptions, arbitrary program labels, and automatic ALPHA-label continuation for modal functions.
- [x] Package the existing function JSON/help data as native resources and implement search aliases, section filters, expandable examples/notes, shortcuts, keyboard-only indicators, and tap-to-run.
- [x] Implement four native themes (`dark`, `light`, `classic-beige`, `high-contrast`) with persisted selection and accessibility contrast tests.
- [x] Implement the five-step first-run onboarding flow, non-dismissible first-run rules, persisted completion, and “Show Guide” reopening.
- [x] Implement native settings for theme, onboarding, macOS launch mode, and global shortcut; migrate legacy preference values where useful.
- [x] Implement shortcut capture/format/validation, collision/error reporting, re-registration, and persistence.

**Exit criterion:** All Tauri settings, guide, and reference workflows exist and survive relaunch.

## Phase 9 — Native platform shell parity

- [x] Implement macOS window mode and `MenuBarExtra`/status-item mode with launch preference, show/hide toggle, blur behavior, positioning, and restart-now affordance.
- [x] Register the configurable global shortcut using an appropriate native mechanism and test unregister/re-register behavior.
- [x] Replace Tauri App Intent mailbox/events with native App Intents that queue atomic requests until the calculator model is ready and open the app when run.
- [ ] Add native application menus and commands for help, settings, save, import/export, reset, and calculator shortcuts.
- [ ] Save on scene/background/termination transitions and restore without resuming a stale running program.
- [ ] If iOS is in scope, add safe-area layout, minimum 44 pt targets, compact stack/print presentation, haptics/audio-session recovery, scene handling, and iOS App Intents.

**Exit criterion:** Every Tauri OS integration has a working native equivalent on each declared SwiftUI platform.

## Phase 10 — Accessibility, regression, and cutover

- [x] Establish the native accessibility/XCUITest baseline: stable identifiers for every calculator key plus primary displays, stack registers, annunciators, program controls, errors, settings, help, onboarding, printer, parameter/function entry, shortcut recording, reset confirmation, and RAW selection; add rendered macOS workflows for calculator input, programs, overlays, and keyboard dismissal.
- [x] Extend the rendered accessibility workflows with deterministic Help search and canonical Help Run dispatch, explicit expanded/collapsed entry state, logical calculator section ordering, and semantic printer-line output plus clear-tape coverage.
- [x] Cover native settings and menu interaction paths: expose theme, launch mode, and global-shortcut values and hints; verify theme persistence, modified-key shortcut capture, and Calculator menu commands in the rendered XCUITest suite.
- [x] Cover interruptible program execution and native RAW import: expose PSE/GETKEY status semantically, verify automatic PSE resume, import a hardware-compatible GETKEY fixture through the macOS open panel, and resume it with the canonical on-screen key code.
- [x] Cover rendered state continuity and the complete first-run path: verify explicit Save State survives process termination/relaunch, expose stable onboarding title/instruction/progress semantics, and verify completion persists without presenting the guide again.
- [x] Cover native export and data-card panel boundaries: verify canonical RAW bytes written through NSSavePanel, validate the exported data-card format tag, overwrite calculator memory, then restore and render the original register value through NSOpenPanel import.
- [x] Make keyboard dismissal deterministic across settings, Help, printer, and nested shortcut-recorder sheets; verify Escape returns focus to the calculator and cancels a GETKEY suspension with the hardware sentinel value zero.
- [x] Give every control stable accessibility identifiers, labels, hints, values, selected/expanded state, logical focus order, and keyboard dismissal behavior.
- [x] Expand XCTest for state/FFI/modal/input/parity fixtures and XCUITest for all end-to-end workflows, themes, menus, overlays, file operations, reset, program run/yield/GETKEY, onboarding, and menu-bar mode.
- [x] Add performance/soak tests for rapid key input, long programs, repeated tick cycles, cancellation, print/event bursts, and file imports; run with Thread Sanitizer and Address Sanitizer where supported.
- [x] Make `just gui-ci` run Rust façade/FFI tests, Swift tests, the inventory parity gate, and key XCUITest smoke flows.
- [x] Complete `TEST-MIGRATION.md` with a one-to-one disposition for every legacy test suite. Retire legacy tests only after their durable behavior is covered natively.
- [x] Run a final inventory diff. Require zero unexplained gaps before removing the Tauri/React source and Tauri-only dependencies. The reproducible 418/418 review is archived in `docs/swiftui-cutover-audit.md`; `just gui-ci` now enforces `--require-complete`.
- [x] Update architecture, build/release, sandbox entitlement, privacy, and user documentation; archive the final `swiftui-parity.json` as migration evidence. Native macOS distribution, the deliberate unsandboxed shared-state contract, and the bundled privacy manifest are documented in `docs/release-setup.md` and the GUI README.

**Exit criterion:** The mechanical ledger is 100%, native CI is green, manual platform smoke tests pass, state/file compatibility is verified, and Tauri can be removed without losing user-visible behavior.

---

## Recommended implementation order and pull-request boundaries

1. Inventory/parity gate only.
2. Shared Rust façade extraction with Tauri behavior unchanged.
3. Generic C ABI plus exhaustive bridge tests.
4. Complete Swift state/service layer.
5. Canonical keyboard and modal state machine.
6. Program/time/print/sound/reset UI.
7. RAW/data/card file workflows.
8. Help/settings/themes/onboarding.
9. Menu-bar/global-shortcut/App-Intent platform shell.
10. Accessibility, full regression, documentation, and Tauri deletion.

Keep each PR vertically testable and avoid mixing façade extraction with visual redesign. Until cutover, any newly added Tauri capability must update the parity inventory and either land with its SwiftUI equivalent or create an explicit failing parity item.

## Final acceptance checklist

- [ ] Every registered Tauri command has a native-equivalent request or justified platform disposition.
- [ ] Every Tauri key ID/pattern, modal transition, shortcut, and help-run action is accepted and reachable.
- [ ] Every `CalcStateView` behavior is represented and rendered with identical precedence.
- [ ] Every preference, overlay, file workflow, lifecycle event, and platform integration is mirrored.
- [ ] Cross-app autosave, RAW, and data-card round trips pass.
- [ ] Automated inventory reports zero gaps; XCTest/XCUITest/Rust CI and manual macOS (and iOS, if declared) smoke tests pass.
