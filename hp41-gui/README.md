# HP-41 native GUI

`hp41-gui` is a native macOS SwiftUI application. Swift owns the window, input,
accessibility, and calculator presentation; `hp41-bridge` exposes the existing
Rust `hp41-core` through a small C ABI and is linked statically into the app.
There is no WebView or JavaScript runtime in the native executable.

Platform scope is explicitly macOS 14 and later. The retained Tauri iOS product
is not included in the native parity claim unless a Swift package/Xcode iOS
target is added in a future slice.

```bash
just gui-dev      # build the bridge and run the app
just gui-check    # type-check Rust and Swift
just gui-ci       # bridge, Swift, parity, and focused rendered smoke tests
just gui-ui-test  # launch and drive the full app workflow suite with XCUITest
just gui-soak     # bounded input/program/tick/cancel/output/file stress suite
just gui-build    # produce .build/HP-41 Calculator.app
```

The old test-suite disposition and native XCTest mapping are documented in
[`TEST-MIGRATION.md`](TEST-MIGRATION.md).

The app continues to use `~/.hp41/autosave.json`, so calculator state remains
compatible with `hp41-cli` and earlier GUI releases.

The Developer ID distribution is intentionally not App-Sandboxed: sharing
`~/.hp41` with the CLI is a product contract that an App Sandbox container
would break. User-selected RAW and data-card files still flow exclusively
through `NSOpenPanel`/`NSSavePanel` with scoped access. The app makes no network
requests, performs no tracking, and collects no user data; its bundled
`Resources/PrivacyInfo.xcprivacy` records that posture and the file-timestamp
API reason used for app-managed files.

The former Tauri/React source directories are temporarily retained as migration
reference. They are no longer used by the `gui-*` build, test, or CI commands.
The native GUI release is macOS 14+ on Apple Silicon; Windows and Linux continue
to receive the CLI only.
