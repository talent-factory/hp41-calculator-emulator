# Tauri-to-Swift test migration ledger

This ledger gives every retained legacy GUI test suite one terminal disposition.
The inventory contains 32 suites and 451 tests: 111 inline Tauri Rust tests,
28 Tauri integration tests, and 312 React/TypeScript tests.

Disposition meanings:

- **Moved** — the implementation-neutral test moved with the code into the shared Rust façade.
- **Replaced** — durable behavior is covered through the native Rust/C/Swift or rendered-app boundary.
- **Retired** — the assertion was specific to React, WebView, Tauri, CSS, or an out-of-scope iOS implementation and has no native product behavior to preserve.

No suite is deferred. Legacy tests remain only as migration evidence and must not receive new coverage.

## Inline Tauri Rust suites

| Legacy suite | Tests | Disposition | Native evidence |
|---|---:|---|---|
| `src-tauri/src/app_intents.rs` | 3 | Replaced | `AppIntentMailboxTests.swift` verifies atomic take-once, malformed/oversized rejection, and typed dispatch; `HP41GUIUITests.testExplicitSaveRestoresRenderedStateAfterRelaunch` covers cold relaunch state. |
| `src-tauri/src/cards.rs` | 7 | Moved | Card request draining and filename rules live in `hp41-app/src/cards.rs`; `CalculatorModelTests.testCalculatorCardCommandsUseConfiguredNativeCardsDirectory` and `FileTransferCoordinatorTests` cover the Swift boundary. |
| `src-tauri/src/commands.rs` | 25 | Moved | Command routing moved to `hp41-app/src/{dispatch,control,request}.rs` and exhaustive C ABI tests in `hp41-bridge/src/lib.rs`; `CalculatorContractTests` and `CalculatorModelTests` cross the native boundary. |
| `src-tauri/src/key_map.rs` | 14 | Moved | Resolver and exhaustive cases moved to `hp41-app/src/key_map.rs`; `KeyboardLayoutTests` verifies every enabled native key is accepted and identities remain canonical. |
| `src-tauri/src/persistence.rs` | 12 | Moved | Versioned load/save/reset behavior moved to `hp41-app/src/persistence.rs`; corruption, stale-running-state, explicit save, reset tiers, and relaunch are covered by `CalculatorModelTests` and XCUITest. |
| `src-tauri/src/prefs.rs` | 14 | Replaced | `AppPreferencesTests` covers defaults, corrupt/unknown values, all themes, onboarding, launch mode, shortcut validation, and legacy-compatible JSON keys. |
| `src-tauri/src/prgm_display.rs` | 5 | Moved | Formatter and exhaustive operation arms moved to `hp41-app/src/prgm_display.rs`; program listing, PC, SST/BST, and execution are rendered in XCUITest. |
| `src-tauri/src/shortcut.rs` | 3 | Replaced | `AppPreferencesTests.testShortcutCaptureFormattingAndValidationMatchesTauriGrammar`, `GlobalShortcutControllerTests`, and the rendered shortcut-recorder workflow cover parsing, formatting, registration, collision rollback, and persistence. |
| `src-tauri/src/tray_helpers.rs` | 9 | Replaced | `MacPlatformShellConfiguration` and `NativePlatformTests` cover launch/visibility policy; XCUITest covers show/hide restoration. Geometry is now native screen/window positioning rather than Tauri monitor math. |
| `src-tauri/src/types.rs` | 19 | Moved | Complete projection moved to `hp41-app/src/view.rs`; `CalculatorStateCodingTests` and `CalculatorContractTests` verify snake-case decoding, defaults, status/error invariants, yields, modal fields, and raw metadata. |

## Tauri integration suites

| Legacy suite | Tests | Disposition | Native evidence |
|---|---:|---|---|
| `src-tauri/tests/cancel_autosave_stress.rs` | 1 | Replaced | The Tauri mutex is gone. `PerformanceSoakTests.testCancellationStopsLoopingBackgroundProgram` exercises independent atomic cancellation while native state persistence is active; it passes under TSan and ASan. |
| `src-tauri/tests/cancel_command_no_deadlock.rs` | 1 | Replaced | `hp41-bridge` owns a separate atomic-only cancellation handle and tests cross-thread signaling/lifecycle; Swift background cancellation is covered by model and soak tests. |
| `src-tauri/tests/card_io_tests.rs` | 3 | Replaced | `hp41-app/src/files.rs`, bridge request tests, `FileTransferCoordinatorTests`, and rendered RAW/data-card panel workflows verify byte output, import/export, selection, and execution. |
| `src-tauri/tests/d25_6_parity.rs` | 4 | Moved | XEQ/direct-operation parity remains in `hp41-core`; shared golden fixtures replay identical requests through `hp41-app`, the C ABI, and Swift in `GoldenFlowTests`. |
| `src-tauri/tests/key_map_stub_error_arms.rs` | 3 | Retired | These locked a temporary v2.1 stub-count/source-text baseline. The completed resolver is exhaustively tested in `hp41-app/src/key_map.rs`, and the 418-item capability ledger prevents reintroducing stubs. |
| `src-tauri/tests/lcd_alternation_modal_prompt.rs` | 5 | Replaced | Shared view precedence/truncation tests live in `hp41-app/src/view.rs`; `CalculatorContractTests`, `ParameterEntryCoordinatorTests`, and rendered parameter sheets cover prompt/entry precedence. |
| `src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` | 5 | Replaced | Stat prompt projection stays in shared `hp41-app` view tests; generated catalog coverage and native function/parameter coordinators verify the same modal transitions. |
| `src-tauri/tests/lcd_alternation_modal_prompt_time.rs` | 3 | Replaced | Time prompt projection and live ticks are covered in `hp41-app/src/view.rs`, `CalculatorModelTests.testLiveTimeDriverTracksCalculatorModeAndSceneActivity`, and repeated-tick soak coverage. |
| `src-tauri/tests/prgm_display_math1_arms.rs` | 2 | Moved | Exhaustive Math/Stat formatter arms moved with `prgm_display.rs`; function-catalog generation is also checked on every `gui-ci` run. |
| `src-tauri/tests/sc4_invariant.rs` | 1 | Retired | The source-grep invariant guarded Tauri `Mutex<State>` access. That state container no longer exists; typed request exhaustiveness and boundary ownership are enforced by `CalculatorContractTests` and bridge tests. |

## React and TypeScript suites

| Legacy suite | Tests | Disposition | Native evidence |
|---|---:|---|---|
| `src/App.test.tsx` | 52 | Replaced | `CalculatorModelTests`, coordinator tests, `AppIntentMailboxTests`, golden replays, and 29 rendered XCUITest workflows cover ASN, overlays, display/event precedence, USER/PRGM/ALPHA, modal routing, keyboard input, yields, GETKEY, reset tiers, and App Intents. |
| `src/BottomSheet.test.tsx` | 3 | Retired | React portal visibility and CSS `expanded` classes do not exist. Native sheets/disclosures are covered by stable accessibility state plus open/close/Escape XCUITest workflows. |
| `src/Display14Seg.test.tsx` | 24 | Retired | SVG path counts, segment opacity, and WebView glyph-cell mechanics were intentionally replaced by native monospaced text. Display value, truncation/precedence, accessibility value, and program rendering remain covered in Rust, XCTest, and XCUITest. |
| `src/HelpOverlay.test.tsx` | 58 | Replaced | Catalog generation checks all six modules and runnable status; `FunctionEntryCoordinatorTests` covers search/aliases/catalog integrity; XCUITest covers filtering, expansion state, keyboard dismissal, and canonical tap-to-run. DOM class/order assertions are retired. |
| `src/Keyboard.test.tsx` | 14 | Replaced | `KeyboardLayoutTests` locks canonical IDs/key codes, reachability, USER relabeling/truncation, shifted behavior, physical shortcuts, and unique accessibility identifiers. React touch-overlay node tests are retired because the declared target is macOS. |
| `src/OnboardingWizard.test.tsx` | 11 | Replaced | `AppPreferencesTests.testQuickStartHasExactlyFiveStableSteps` and full first-run/reopen XCUITest workflows cover page order, progress, Back/Next, non-dismissible first run, completion persistence, and reset to page one. |
| `src/SettingsPanel.test.tsx` | 15 | Replaced | `AppPreferencesTests`, `NativePlatformTests`, `GlobalShortcutControllerTests`, and rendered Settings workflows cover themes, guide reopening, launch mode, restart affordance, shortcut display, accessibility values, and persistence. |
| `src/ShortcutRecorder.test.tsx` | 11 | Replaced | Shortcut grammar/format tests moved to `AppPreferencesTests`; registration/collision rollback moved to `GlobalShortcutControllerTests`; XCUITest covers capture, enabled Save, persistence, and Escape cancellation. |
| `src/haptics.test.ts` | 23 | Retired | Haptic plugin tiers and WebView double-fire guards are iOS/Tauri mechanics. iOS is not a declared Swift package target; the plan keeps iOS haptics explicitly out of scope until an iOS target exists. |
| `src/help_data.test.ts` | 31 | Replaced | `generate-function-catalog.rb --check`, `FunctionEntryCoordinatorTests`, and the parity ledger cover six-pool completeness, hidden aliases, runnable filtering, search fields, ordering, and shortcut counts. React-specific fuzzy ranking was intentionally replaced by native localized matching. |
| `src/pending_input.test.ts` | 56 | Replaced | `ParameterEntryCoordinatorTests` covers register/flag/IND, formatting, ASN, label, delete/size, conditional, arithmetic, error, and modal dispatch families; XCUITest covers focus, submission, cancellation, and keyboard dismissal. Preview-string DOM tests are retired. |
| `src/scale.test.ts` | 14 | Retired | CSS transform/design-box scaling and bottom-sheet reserved height do not exist. Native content sizing, window positioning, and platform target policy are covered by `NativePlatformTests`, `MacPlatformShellConfiguration`, and rendered window tests. |

## Native verification commands

```sh
just gui-ci                 # Rust façade/bridge, Swift, parity, rendered smoke
just gui-ui-test            # all rendered macOS workflows
just gui-soak               # bounded stress suite
just gui-sanitize-thread    # full Swift/FFI suite under TSan
just gui-sanitize-address   # full Swift/FFI suite under ASan
```

The Tauri and React suites may be removed only after the unsandboxed macOS CI
run is green and the final inventory/cutover audit is archived.
