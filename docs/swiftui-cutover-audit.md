# SwiftUI cutover inventory audit

Audit date: 2026-06-20

## Decision

The mechanical React/Tauri-to-SwiftUI inventory has **zero unexplained capability gaps**: 418 of 418 extracted items have a complete native disposition, with 0 remaining. This clears the inventory-diff gate; it does not by itself authorize deletion of the retained React/Tauri implementation. The documentation/release audit is complete; removal remains contingent on green unsandboxed CI, including the rendered XCUITest smoke workflows, and the final manual platform smoke.

The parity claim covers the declared native target, macOS 14 and later. The retained Tauri iOS product is outside this claim unless a native iOS target is added.

## Reproducible evidence

- Source inventory: registered Tauri commands and frontend invocations, direct and parameterized key identities, keyboard identities, modal flows, projected state fields, preferences, physical shortcuts, UI components, and platform events.
- Extractor: `scripts/swiftui-parity.py`.
- Native disposition manifest: `hp41-gui/swiftui-capabilities.json`.
- Archived generated result: `docs/swiftui-parity.json`.
- Cutover gate: `python3 scripts/swiftui-parity.py --check --require-complete`, also enforced by `just gui-ci`.
- Test migration evidence: `hp41-gui/TEST-MIGRATION.md` maps all 32 legacy suites and 451 legacy tests to moved, replaced, or retired native coverage.
- Native declarations at audit time: 115 Swift unit/integration/soak tests and 29 rendered XCUITest workflows, plus the Rust façade and bridge suites run by `just gui-ci`.

## Inventory result

| Disposition | Count | Meaning |
|---|---:|---|
| `implemented` | 388 | The capability is directly implemented by the native application or shared façade. |
| `native-equivalent` | 25 | The same user-visible behavior is provided through a native SwiftUI/AppKit mechanism rather than the legacy Tauri shape. |
| `not-applicable` | 5 | The item is outside the declared platform or is unreachable legacy scaffold, with a recorded justification. |
| `partial` | 0 | No incomplete implementation is accepted at cutover. |
| `planned` | 0 | No unimplemented inventory item is accepted at cutover. |
| **Total complete** | **418/418** | **0 remaining.** |

## Reviewed non-literal dispositions

The 25 native equivalents fall into explainable implementation-shape changes:

- IPC-free native services: preferences, platform identity, restart, and the App Intent mailbox.
- Native input normalization: contextual backspace, one-shot shift, and direct typed modal submission.
- Native presentation: keyboard, Help, onboarding, settings, shortcut recording, and RAW program selection.
- Native lifecycle/events: SwiftUI scene phase and `NotificationCenter` replace Tauri focus/event plumbing.
- Native modal consolidation: the two-stage legacy ASN kinds are one native assignment sheet.

The five justified exclusions are:

- Two `is_ios` inventory entries: the declared Swift package target is macOS 14+, so runtime iOS probing has no native role.
- `confirm_load`, `hex`, and `print` modal kinds: these are unreachable scaffolds retained in `pending_input.ts`; no legacy opener constructs them and their transitions intentionally dispatch nothing. Native printer output is covered by the implemented tape sheet.

Each individual disposition and its precise note remains machine-readable in `docs/swiftui-parity.json`.

## Removal boundary

The inventory gate now fails CI if the retained legacy surface changes, if the generated snapshot drifts, or if any item returns to `partial`/`planned`. Architecture, build/release, sandbox/privacy, and user documentation now describe the native product. React/Tauri source and Tauri-only dependencies should be removed only after external CI and the final manual platform smoke are green.

## Local cutover-gate execution

The native gate was executed on 2026-06-20 after the documentation/release audit:

- parity inventory: 418/418 complete, 0 remaining;
- `hp41-app`: 61 tests passed;
- `hp41-bridge`: 17 tests passed;
- Swift native suite: 115 tests passed, including golden replay, background R/S regression, and soak coverage;
- XCUITest build-for-testing: succeeded and produced the native app plus `.xctestrun` bundle;
- rendered smoke execution initially exposed inherited SwiftUI container identifiers and incorrect background R/S routing; both native defects were fixed and the R/S regression is covered in the 115-test Swift suite. A subsequent initialized smoke run passed 2 of 6 workflows before surfacing the remaining fixes.
- rendered revalidation is currently blocked by the local macOS 27/Xcode beta LaunchServices runner, which repeatedly reports the test app as `Running Background` and cannot activate it. This is a host-runner failure before individual workflow assertions, not a new calculator failure.

Consequently the retained React/Tauri tree must not yet be deleted. The six-workflow smoke tier and then the full 29-workflow suite need a clean, UI-testing-authorized macOS host (or the GitHub Actions runner) before the removal boundary is crossed.
