---
phase: 17-persistence-and-print-output
reviewed: 2026-05-10T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - hp41-gui/src-tauri/Cargo.toml
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src-tauri/src/persistence.rs
  - hp41-gui/src/App.css
  - hp41-gui/src/App.tsx
findings:
  critical: 2
  warning: 4
  info: 2
  total: 8
status: issues_found
---

# Phase 17: Code Review Report

**Reviewed:** 2026-05-10T00:00:00Z
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

The phase 17 implementation covers Tauri v2 persistence (save/load CalcState) and a React print
panel that accumulates PRX/PRA/PRSTK output. The persistence module is well-structured: version
wrapping, directory creation, is_running reset, and fallback-to-fresh-state on load errors are
all correct. The auto-save thread design is sound in principle.

Two critical issues were identified: (1) the Mutex lock guard held across
`persistence::save_state()` in the auto-save thread blocks the IPC threads for the entire
duration of every disk write — this is a correctness/responsiveness defect that can freeze the
UI for hundreds of milliseconds at each 30-second interval; (2) there is no version migration
guard in `load_state`, so a future-version state file is silently deserialized into v1 fields,
which can corrupt state rather than trigger the safe "start fresh" fallback.

Four warnings cover unsafe assumptions in the key-event handler (`e.preventDefault()` on all
mapped keys without checking if the target is an input element), missing `GuiError` implementation
for the Tauri `IntoResponse` contract, an unbounded `printLog` array that grows without limit for
the lifetime of the app session, and an atomic-visibility gap with `busyRef` across React renders.

---

## Critical Issues

### CR-01: Mutex held across disk I/O in auto-save thread — UI freeze every 30 s

**File:** `hp41-gui/src-tauri/src/lib.rs:32-35`

**Issue:** The auto-save thread acquires the `AppState` Mutex lock and holds it for the entire
duration of `persistence::save_state()`, which is a synchronous `fs::File::create` + JSON
serialization write to disk. On macOS/Linux this write is typically fast, but on Windows with
antivirus scanning or on any system under I/O pressure it can block for hundreds of milliseconds.
Every IPC call from the frontend (`dispatch_op`, `get_state`) also acquires the same lock, so
any key press that arrives while the auto-save is writing will stall until the write finishes.
The user will observe a frozen UI for the duration of every 30-second save.

The fix is to clone the `CalcState` under the lock (which is fast — heap copy), release the lock,
then write the clone to disk outside the critical section.

**Fix:**
```rust
std::thread::spawn(move || {
    let thread_save_path = persistence::default_state_path();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        // Clone under lock, then release before I/O.
        let snapshot = {
            let calc = handle.state::<AppState>().lock()
                .unwrap_or_else(|e| e.into_inner());
            calc.clone()   // CalcState must implement Clone
        };  // MutexGuard dropped here — lock released before disk write
        if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
            eprintln!("auto-save failed: {e}");
        }
    }
});
```

Note: this requires `CalcState` to implement `Clone`. If it does not, an alternative is to
serialize to a `String` under the lock and write the string outside the lock.

---

### CR-02: No version guard in `load_state` — future-version files silently corrupt state

**File:** `hp41-gui/src-tauri/src/persistence.rs:56-64`

**Issue:** `load_state` deserializes the file without checking `wrapper.version`. The design
document references "D-06: version enables future migration without breaking existing saves," but
the implementation never reads `wrapper.version`. If a future version of the app saves a v2 file
(with new fields or changed semantics), this v1 code will:

- **Silently succeed** if the new fields are additive (serde ignores unknown fields by default),
  but with potentially zeroed or defaulted values for the new fields — wrong state, not an error.
- **Silently succeed** with corrupted data if field types change but names stay the same.

The safe contract is: any version other than `1` must return `Err` so the caller falls back to
`CalcState::new()`. Without this guard, the version field provides no actual protection.

**Fix:**
```rust
pub fn load_state(path: &Path) -> Result<CalcState, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let wrapper: StateFile = serde_json::from_reader(file)?;
    if wrapper.version != 1 {
        return Err(format!(
            "unsupported state file version {} (expected 1)",
            wrapper.version
        ).into());
    }
    let mut state = wrapper.state;
    state.is_running = false;
    Ok(state)
}
```

---

## Warnings

### WR-01: `e.preventDefault()` called unconditionally — may suppress browser defaults for non-calculator targets

**File:** `hp41-gui/src/App.tsx:74`

**Issue:** `e.preventDefault()` is called for every key that resolves to a non-null key ID
(digits, `.`, `e`, `Enter`, `Backspace`, `+`, `-`, `*`, `/`, and many letter keys). This is
intentional for the calculator, but the listener is attached to `window`, not a focused
calculator element. If a browser-rendered text input or button ever gets focus (e.g., the
print-panel close button, or a future settings dialog), pressing `Enter`, `Backspace`, or digits
will be consumed by the calculator handler, preventing the user from interacting with those
elements normally. The close button (`×`) at line 151 is a `<button>` — after clicking it,
focus may stay on the button, and the next `Enter` key press will re-activate the close button
(the default button click behavior) AND be consumed by `dispatch_op` via `e.preventDefault()`.

**Fix:** Check that the event target is not an interactive element before dispatching:
```tsx
const handleKey = useCallback((e: KeyboardEvent) => {
  if (e.repeat) return;
  if (busyRef.current) return;
  // Don't steal keys from focused inputs, buttons, textareas
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'BUTTON' || tag === 'SELECT') return;
  const keyId = resolveKeyId(e, calcState);
  if (keyId === null) return;
  e.preventDefault();
  // ... rest of handler
}, [calcState]);
```

---

### WR-02: `printLog` array grows without bound — potential memory exhaustion in long sessions

**File:** `hp41-gui/src/App.tsx:56,102`

**Issue:** `printLog` is a `useState<string[]>` that is only ever appended to (`[...prev, ...calcState.print_lines]`) and never trimmed. On an HP-41 with PRSTK (prints 4 lines) or a running program with repeated PRX calls, a user can accumulate thousands of entries within minutes. The CSS panel shows a fixed 130 px scroll area, so old entries are invisible but remain in the React virtual DOM, consuming memory and slowing reconciliation.

**Fix:** Cap the log at a reasonable maximum (e.g., 500 lines), discarding oldest entries:
```tsx
const MAX_PRINT_LINES = 500;

useEffect(() => {
  if (calcState && calcState.print_lines.length > 0) {
    setPrintLog(prev => {
      const combined = [...prev, ...calcState.print_lines];
      return combined.length > MAX_PRINT_LINES
        ? combined.slice(combined.length - MAX_PRINT_LINES)
        : combined;
    });
    setPrintPanelOpen(true);
  }
}, [calcState]);
```

---

### WR-03: `busyRef` does not prevent key-repeat through `handleKey` re-registration races

**File:** `hp41-gui/src/App.tsx:68-80`

**Issue:** `busyRef.current` is set to `true` synchronously before `invoke()`, and reset to
`false` in `.finally()`. However, because `handleKey` is recreated on every `calcState` change
(the `useCallback` dependency at line 80), and because the `useEffect` at line 92 removes and
re-adds the listener whenever `handleKey` changes, there is a brief window during the
listener-replacement microtask in which the old handler is removed and the new one is added.
This is not atomic; the `busyRef.current = false` in `.finally()` executes in a Promise
microtask after `calcState` is set (which triggers re-registration). In practice the `busyRef`
ref object itself is stable (same ref across renders), so the guard mostly works, but the
dependency chain `invoke → setCalcState → handleKey recreated → listener replaced` creates
unnecessary DOM event listener churn on every key press. This is also the cause of the
`calcState` dependency on `handleKey` — needed only for the `'n'` key EEX routing.

**Fix:** Extract only the EEX-mode flag needed by `resolveKeyId` into a separate ref, making
`handleClick` and `handleKey` dependency-free on `calcState`:
```tsx
const inEexModeRef = useRef(false);

useEffect(() => {
  inEexModeRef.current = calcState?.in_eex_mode ?? false;
}, [calcState]);

const handleKey = useCallback((e: KeyboardEvent) => {
  if (e.repeat) return;
  if (busyRef.current) return;
  const keyId = resolveKeyId(e, inEexModeRef.current);
  // ...
}, []); // stable — no calcState dependency
```

---

### WR-04: `macos-private-api` Tauri feature enabled without documentation or justification

**File:** `hp41-gui/src-tauri/Cargo.toml:19`

**Issue:** The `tauri` dependency is declared with `features = ["macos-private-api"]`. This
feature enables macOS-specific private APIs (notably the `transparent_titlebar` / vibrancy
APIs that require `NSWindow` private selectors). Enabling it on a crate that does not otherwise
use those APIs submits the app for App Store review under stricter scrutiny and may cause
rejection if Apple's static analysis detects private API usage with no purpose. Additionally,
this feature requires the `tauri.conf.json` entry `"macOSPrivateApi": true`, and if that is
absent the feature has no effect but still increases the attack surface at review time.

More practically: if the app is distributed outside the App Store this is harmless, but it
signals an unreviewed configuration that could cause subtle build failures or capability
differences on non-macOS targets.

**Fix:** Remove the feature unless it is explicitly needed and documented:
```toml
tauri = { version = "2.11" }
```
If the private API IS needed (e.g., for a transparent title bar), add a comment explaining which
specific capability it enables and confirm the matching `tauri.conf.json` setting is present.

---

## Info

### IN-01: `key_to_op` mapping in `resolveKeyId` is a partial duplicate of `key_map::resolve`

**File:** `hp41-gui/src/App.tsx:36-50`

**Issue:** The `MAP` constant in `resolveKeyId` duplicates the string-to-op mapping that already
exists authoritatively in `hp41-gui/src-tauri/src/key_map.rs`. If a new op is added to
`key_map.rs` (e.g., a new HMS function), the developer must also update the TypeScript `MAP`
table in `App.tsx`. The comment at line 35 ("authoritative source: hp41-cli/src/keys.rs") is
also inaccurate — the authoritative source for the GUI is `key_map.rs`, not the CLI keys file.
Divergence between the two tables will produce silent no-ops (unknown key → `null` → no invoke)
rather than any error.

**Suggestion:** Document this dual-maintenance requirement explicitly with a code comment that
references `key_map.rs` as the Rust counterpart, and add a note to `CLAUDE.md` or the phase
plan that both files must be updated together when adding new ops.

---

### IN-02: `print-panel-close` button has no keyboard-accessible label

**File:** `hp41-gui/src/App.tsx:151`, `hp41-gui/src/App.css:114-122`

**Issue:** The close button renders `×` as its text content with no `aria-label` attribute.
Screen readers will announce "times" or "multiplication sign" rather than "close print panel."
This is a minor accessibility gap.

**Suggestion:**
```tsx
<button
  className="print-panel-close"
  onClick={() => setPrintPanelOpen(false)}
  aria-label="Close print panel"
>×</button>
```

---

_Reviewed: 2026-05-10T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
