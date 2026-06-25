# macOS Menu-Bar Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On macOS, run the HP-41 Tauri GUI as a pure menu-bar (status-item) app — no Dock icon, no window on launch, a left-click popover and a right-click menu — while Windows/Linux keep today's windowed behavior.

**Architecture:** macOS-only behavior is applied at runtime in `setup()` (Accessory activation policy, decorations off, always-on-top, hidden start, tray creation), gated with `#[cfg(target_os = "macos")]`, so the cross-platform `tauri.conf.json` window stays normal for other OSes. Pure decision/geometry logic is extracted into testable helper functions; the React frontend is wrapped in a CSS-scale container so the fixed 440×1020 layout fits any screen. Auto-hide-on-blur is suppressed only around native file dialogs via a backend flag (in-app React modals live in the same webview and do not trigger window blur).

**Tech Stack:** Tauri v2.11 (Rust), `tauri-plugin-autostart` v2, React 19 + TypeScript + Vite, Vitest, WebdriverIO + tauri-driver (E2E).

---

## Background notes for the implementer

- **Why runtime-gated, not config:** `tauri.conf.json` window settings apply to *all* platforms. We must NOT make Windows/Linux fenster-less. So the only config change is `visible: false` (avoids a launch flash on macOS); everything else (decorations off, always-on-top, hide, Accessory policy, tray) is done at runtime under `#[cfg(target_os = "macos")]`. On non-macOS we simply `show()` the window in `setup()`.
- **In-app modals are NOT a blur problem.** The HP-41 modals (`PendingInput` in `src/pending_input.ts` / `App.tsx`) are DOM overlays inside the same webview window — opening them does not move OS focus, so `WindowEvent::Focused(false)` does not fire. Only the **native** file dialogs (`.raw`/`.card.json` open/save panels in `commands.rs`) take OS focus away. Therefore auto-hide suppression is needed ONLY around those four dialog commands, and can be a pure backend flag — no new IPC command, no new permission TOML.
- **Confirmed facts (verified against the codebase):**
  - `main.tsx` (React 19) uses `ReactDOM.createRoot(...).render(...)` and imports BOTH `./index.css` and `./themes.css` (themes.css MUST stay after index.css — D-48.11 specificity). The scale wrapper must preserve both imports and keep using `createRoot`.
  - The four dialog commands in `commands.rs` are `#[tauri::command]` fns; `commands.rs` already `use tauri::AppHandle;`. The dialog commands that perform native open/save take an `app`/`AppHandle` handle (verify the exact param name when editing — see Task 5 Step 3).
  - E2E files are `hp41-gui/wdio.conf.cjs` (CommonJS) and `hp41-gui/e2e/smoke.spec.js`. The CI E2E job is **Ubuntu-only** (`ci-gui.yml` → `e2e-linux`), run via `xvfb-run -a just gui-e2e`. Because the E2E runs on Linux, the macOS menu-bar code path is `#[cfg(target_os = "macos")]`-gated and never executes there — **the existing E2E is unaffected by default.** The `HP41_SHOW_ON_START` hook in Task 7 is a safety net for anyone running E2E on macOS locally; it is optional for CI.
- **Tauri API version sensitivity:** the tray/menu/activation-policy APIs are version-specific to tauri 2.11. Run `cargo check` after Task 4's first edit to confirm exact signatures (`tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState}`, `tauri::menu::{MenuBuilder, MenuItemBuilder, CheckMenuItemBuilder}`, `tauri::ActivationPolicy`). If a signature differs, adapt the call but keep the extracted pure helpers unchanged.
- **Frozen invariants respected:** no `hp41-core` change, no new `Op`, SC-4 untouched, bundle ID `ch.talent-factory.hp41` unchanged, IPC contract unchanged.
- **Commits:** the project mandates `/git-workflow:commit --with-skills` with English messages. Each task's final commit step shows the message; create it via the commit workflow (not bare `git commit`). Do NOT add Co-Authored-By/Generated-with suffixes. Keep the unrelated working-tree change to `Justfile` OUT of every commit (stage only the files each task touches).
- **Working dir:** all paths are relative to repo root `/Users/daniel/GitRepository/hp41-calculator-emulator`. The GUI is the nested workspace `hp41-gui/`.

---

## File Structure

**Created:**
- `hp41-gui/src/scale.ts` — pure `computeScale()` for the frontend fit-to-screen wrapper.
- `hp41-gui/src/scale.test.ts` — Vitest for `computeScale()`.
- `hp41-gui/src-tauri/src/tray_helpers.rs` — pure, unit-tested geometry/decision helpers (`compute_popover_position`, `should_show_after_hide`, `fit_inner_height`).
- `hp41-gui/src-tauri/src/tray.rs` — macOS tray icon, popover toggle, right-click menu, and the macOS setup routine.
- `hp41-gui/src-tauri/icons/tray-template.png` + `tray-template@2x.png` — monochrome menu-bar template icon.
- `docs/adr/v4.1-001-macos-menu-bar-mode.md` — ADR recording the menu-bar decision.

**Modified:**
- `hp41-gui/src/main.tsx` — wrap `<App/>` in the scale container (preserve `createRoot` + both CSS imports).
- `hp41-gui/src-tauri/Cargo.toml` — add `tauri-plugin-autostart = "2"`.
- `hp41-gui/src-tauri/src/lib.rs` — register autostart plugin, manage `PopoverState`, call macOS setup, wire `on_window_event` blur→hide, declare new modules.
- `hp41-gui/src-tauri/src/commands.rs` — set/clear the suppress-hide flag around the four native dialog commands.
- `hp41-gui/src-tauri/tauri.conf.json` — window `visible: false`.
- `hp41-gui/wdio.conf.cjs` — launch the E2E app with `HP41_SHOW_ON_START=1` (macOS-local safety net).
- `CLAUDE.md` — add a "macOS menu-bar mode" bullet under GUI specifics.

---

## Task 1: Frontend fit-to-screen scaling

**Files:**
- Create: `hp41-gui/src/scale.ts`
- Test: `hp41-gui/src/scale.test.ts`
- Modify: `hp41-gui/src/main.tsx`

- [ ] **Step 1: Write the failing test**

Create `hp41-gui/src/scale.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { computeScale, DESIGN_WIDTH, DESIGN_HEIGHT } from "./scale";

describe("computeScale", () => {
    it("returns 1 when the viewport is at least the design size", () => {
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT)).toBe(1);
        expect(computeScale(2000, 2000)).toBe(1);
    });

    it("never scales above 1 (no upscaling)", () => {
        expect(computeScale(10000, 10000)).toBe(1);
    });

    it("scales down by the height-limiting dimension", () => {
        // half the design height, ample width -> 0.5
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT / 2)).toBeCloseTo(0.5, 5);
    });

    it("scales down by the width-limiting dimension", () => {
        expect(computeScale(DESIGN_WIDTH / 4, DESIGN_HEIGHT)).toBeCloseTo(0.25, 5);
    });

    it("uses the smaller of the two ratios", () => {
        // width ratio 0.5, height ratio 0.25 -> 0.25 wins
        expect(computeScale(DESIGN_WIDTH / 2, DESIGN_HEIGHT / 4)).toBeCloseTo(0.25, 5);
    });

    it("guards against zero/negative viewport values", () => {
        expect(computeScale(0, 0)).toBe(1);
        expect(computeScale(-100, -100)).toBe(1);
    });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd hp41-gui && npx vitest run src/scale.test.ts`
Expected: FAIL — `Failed to resolve import "./scale"` (module does not exist yet).

- [ ] **Step 3: Write the minimal implementation**

Create `hp41-gui/src/scale.ts`:

```ts
// Fixed design size of the HP-41 calculator layout. The whole UI is authored
// against this size; the menu-bar popover (and any small screen) scales it down
// uniformly so nothing is clipped.
// See docs/superpowers/specs/2026-05-29-macos-menu-bar-mode-design.md.
export const DESIGN_WIDTH = 440;
export const DESIGN_HEIGHT = 1020;

/**
 * Uniform scale factor to fit the DESIGN_WIDTH x DESIGN_HEIGHT layout inside the
 * given viewport. Never upscales (capped at 1). Falls back to 1 for non-positive
 * inputs (e.g. a zero-size viewport during first paint).
 */
export function computeScale(
    viewportWidth: number,
    viewportHeight: number,
    designWidth: number = DESIGN_WIDTH,
    designHeight: number = DESIGN_HEIGHT,
): number {
    if (viewportWidth <= 0 || viewportHeight <= 0) {
        return 1;
    }
    return Math.min(1, viewportWidth / designWidth, viewportHeight / designHeight);
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd hp41-gui && npx vitest run src/scale.test.ts`
Expected: PASS (6 tests).

- [ ] **Step 5: Wire the scale wrapper into `main.tsx`**

The current `main.tsx` is:

```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'
// D-48.11: themes.css must come after index.css so [data-theme] selector specificity wins.
import './themes.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

Replace its entire contents with (keep `createRoot`, keep BOTH CSS imports in the same order, add the scale wrapper):

```tsx
import React, { useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { computeScale, DESIGN_WIDTH, DESIGN_HEIGHT } from './scale'
import './index.css'
// D-48.11: themes.css must come after index.css so [data-theme] selector specificity wins.
import './themes.css'

/**
 * Wraps <App/> in a fixed-size (DESIGN_WIDTH x DESIGN_HEIGHT) box and applies a
 * uniform CSS transform so the layout fits the current window without scrolling.
 * Used by the macOS menu-bar popover (which may be shorter than 1020px) and any
 * small display. On a full-height window the scale is 1 (no visual change).
 */
function ScaledApp(): React.ReactElement {
  const [scale, setScale] = useState(() =>
    computeScale(window.innerWidth, window.innerHeight),
  )

  useEffect(() => {
    const update = () => setScale(computeScale(window.innerWidth, window.innerHeight))
    update()
    window.addEventListener('resize', update)
    return () => window.removeEventListener('resize', update)
  }, [])

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        overflow: 'hidden',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'flex-start',
      }}
    >
      <div
        style={{
          width: DESIGN_WIDTH,
          height: DESIGN_HEIGHT,
          transform: `scale(${scale})`,
          transformOrigin: 'top center',
          flex: '0 0 auto',
        }}
      >
        <App />
      </div>
    </div>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ScaledApp />
  </React.StrictMode>,
)
```

- [ ] **Step 6: Verify the whole frontend still builds and all unit tests pass**

Run: `cd hp41-gui && npm run build && npm test`
Expected: `tsc` succeeds, Vite build succeeds, all Vitest suites pass (existing `App.test.tsx` etc. are unaffected — they mount `<App/>` directly, not `ScaledApp`).

- [ ] **Step 7: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
✨ feat(gui): add fit-to-screen scaling wrapper for the calculator layout

Introduce computeScale() and a ScaledApp wrapper in main.tsx so the fixed
440x1020 layout scales uniformly to fit any window height. Prepares the UI
for the macOS menu-bar popover, where available height can be < 1020px.
Preserves createRoot and the index.css/themes.css import order (D-48.11);
no change to App.tsx or the component tree.
```

---

## Task 2: Add the autostart plugin dependency and permission

**Files:**
- Modify: `hp41-gui/src-tauri/Cargo.toml`
- Modify: `hp41-gui/src-tauri/capabilities/default.json`

- [ ] **Step 1: Add the dependency**

In `hp41-gui/src-tauri/Cargo.toml`, under `[dependencies]` (after the `tauri-plugin-dialog = "2"` line), add:

```toml
tauri-plugin-autostart = "2"
```

- [ ] **Step 2: Add the autostart permission to the capability set**

In `hp41-gui/src-tauri/capabilities/default.json`, add `"autostart:default"` to the `permissions` array (after the `"dialog:allow-save"` entry). The end of the array becomes:

```json
    "dialog:allow-open",
    "dialog:allow-save",
    "autostart:default"
```

(The autostart toggle is driven from Rust, but the plugin registers its permission
namespace; including `autostart:default` keeps the capability set consistent.)

- [ ] **Step 3: Verify it resolves and compiles**

Run: `cd hp41-gui/src-tauri && cargo check`
Expected: dependency downloads and the crate compiles (no usage yet — plugin is wired in Task 4). If `cargo check` errors that `autostart:default` is unknown (the plugin is not initialised until Task 4), temporarily remove the `autostart:default` line, finish Task 4, then re-add it and re-run `cargo check`. Note which path you took.

- [ ] **Step 4: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
🔧 chore(gui): add tauri-plugin-autostart dependency and capability

Add tauri-plugin-autostart v2 and its default permission to the GUI
capability set, in preparation for the menu-bar "Start at Login" menu item.
```

---

## Task 3: Pure tray helper functions (geometry + debounce)

**Files:**
- Create: `hp41-gui/src-tauri/src/tray_helpers.rs`
- Modify: `hp41-gui/src-tauri/src/lib.rs` (declare the module)

These are the testable units of the tray behavior. They take plain numbers, not
Tauri types, so they unit-test without a running app.

- [ ] **Step 1: Write the failing tests + module skeleton**

Create `hp41-gui/src-tauri/src/tray_helpers.rs`:

```rust
//! Pure helper logic for the macOS menu-bar popover (Task 3 of the menu-bar plan).
//!
//! These functions take plain numbers (physical pixels / durations) instead of
//! Tauri types so they can be unit-tested without a running application. The
//! tray event handlers in `tray.rs` call them.

use std::time::{Duration, Instant};

/// Horizontal/vertical position (physical pixels) for the popover window so it
/// sits centered under the tray icon, just below the menu bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopoverPos {
    pub x: i32,
    pub y: i32,
}

/// Center the window horizontally under the icon and place its top at the icon's
/// bottom edge. `icon_x` is the icon rect's left; `icon_w` its width; `icon_bottom`
/// the icon rect's bottom edge; `window_w` the window width. All physical pixels.
pub fn compute_popover_position(
    icon_x: i32,
    icon_w: i32,
    icon_bottom: i32,
    window_w: i32,
) -> PopoverPos {
    let icon_center_x = icon_x + icon_w / 2;
    PopoverPos {
        x: icon_center_x - window_w / 2,
        y: icon_bottom,
    }
}

/// Decide whether a tray left-click on a currently-hidden window should SHOW it.
///
/// When the popover is open and the user clicks the tray icon, macOS fires a
/// window blur (which hides the popover and records `last_hidden`) *before* the
/// tray click event. Without a guard, the click would immediately re-show the
/// just-hidden window ("flicker"). If the window was hidden within `debounce` of
/// now, we treat this click as the closing click and do NOT re-show.
pub fn should_show_after_hide(
    last_hidden: Option<Instant>,
    now: Instant,
    debounce: Duration,
) -> bool {
    match last_hidden {
        Some(t) => now.duration_since(t) >= debounce,
        None => true,
    }
}

/// Clamp the popover's inner height to a fraction of the screen height so it never
/// runs off the bottom of the display. Returns the height to apply (logical px).
pub fn fit_inner_height(design_h: f64, screen_h: f64, max_fraction: f64) -> f64 {
    let cap = screen_h * max_fraction;
    design_h.min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_window_under_icon() {
        // icon at x=100, width=24 -> center 112; window 440 wide -> x = 112-220 = -108
        let pos = compute_popover_position(100, 24, 30, 440);
        assert_eq!(pos, PopoverPos { x: -108, y: 30 });
    }

    #[test]
    fn places_top_at_icon_bottom() {
        let pos = compute_popover_position(500, 24, 38, 440);
        assert_eq!(pos.y, 38);
    }

    #[test]
    fn show_allowed_when_never_hidden() {
        assert!(should_show_after_hide(None, Instant::now(), Duration::from_millis(250)));
    }

    #[test]
    fn show_suppressed_within_debounce() {
        let now = Instant::now();
        assert!(!should_show_after_hide(Some(now), now, Duration::from_millis(250)));
    }

    #[test]
    fn show_allowed_after_debounce_elapsed() {
        let now = Instant::now();
        let long_ago = now - Duration::from_millis(500);
        assert!(should_show_after_hide(Some(long_ago), now, Duration::from_millis(250)));
    }

    #[test]
    fn height_uncapped_on_tall_screen() {
        // 1020 design fits within 92% of a 1440 screen (1324) -> stays 1020
        assert_eq!(fit_inner_height(1020.0, 1440.0, 0.92), 1020.0);
    }

    #[test]
    fn height_capped_on_short_screen() {
        // 92% of 800 = 736 -> capped
        assert_eq!(fit_inner_height(1020.0, 800.0, 0.92), 736.0);
    }
}
```

- [ ] **Step 2: Declare the module so tests can run**

In `hp41-gui/src-tauri/src/lib.rs`, add to the module declarations block (after the `mod prgm_display;` line, around line 11):

```rust
mod tray_helpers; // pure geometry/debounce helpers for the macOS menu-bar popover
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cd hp41-gui/src-tauri && cargo test tray_helpers`
Expected: PASS (7 tests in `tray_helpers::tests`).

- [ ] **Step 4: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
✨ feat(gui): add pure helpers for menu-bar popover geometry

Add tray_helpers.rs with compute_popover_position (center under icon),
should_show_after_hide (flicker-guard debounce for the open/blur race), and
fit_inner_height (clamp popover height to a fraction of screen height).
Unit-tested in isolation; consumed by tray.rs next.
```

---

## Task 4: Tray icon, popover toggle, right-click menu, macOS setup

**Files:**
- Create: `hp41-gui/src-tauri/src/tray.rs`
- Create: `hp41-gui/src-tauri/icons/tray-template.png`, `hp41-gui/src-tauri/icons/tray-template@2x.png`
- Modify: `hp41-gui/src-tauri/src/lib.rs`

This task is integration-level Tauri wiring; it is verified by `cargo check`/build
and manual run (the pure logic it relies on is already tested in Task 3). There is
no unit test for the event closures (they need a running app).

- [ ] **Step 1: Create the menu-bar template icon**

Run this from the repo root to generate a monochrome template icon (black glyph on
transparent background — macOS recolors template images for light/dark menu bars):

```bash
cd hp41-gui/src-tauri/icons
python3 - <<'PY'
from PIL import Image, ImageDraw
def make(path, size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pad = max(1, size // 8)
    d.rounded_rectangle([pad, pad, size - pad, size - pad],
                        radius=max(2, size // 6), outline=(0, 0, 0, 255),
                        width=max(1, size // 12))
    sx0, sy0 = pad + size // 8, pad + size // 8
    sx1, sy1 = size - pad - size // 8, pad + size // 4
    d.rectangle([sx0, sy0, sx1, sy1], fill=(0, 0, 0, 255))
    img.save(path)
make("tray-template.png", 22)
make("tray-template@2x.png", 44)
print("wrote tray-template.png (22) and tray-template@2x.png (44)")
PY
```

If Pillow is missing: `pip3 install pillow --break-system-packages` then re-run.
Last-resort fallback: `cp icon.png tray-template.png && cp 128x128@2x.png tray-template@2x.png` and in Step 2 change `icon_as_template(true)` to `icon_as_template(false)` (a colored icon works but looks less native). Note which path you took.

Verify:

Run: `ls -la hp41-gui/src-tauri/icons/tray-template*.png`
Expected: both files present, non-zero size.

- [ ] **Step 2: Create `tray.rs`**

Create `hp41-gui/src-tauri/src/tray.rs`:

```rust
//! macOS menu-bar (status-item) mode for the HP-41 GUI.
//!
//! Everything here is macOS-only. On other platforms the app keeps its normal
//! decorated window (see `apply_menu_bar_mode` callers in lib.rs). The frontend
//! and IPC contract are unchanged; this module only manages the window's
//! presentation (Accessory policy, decorations off, always-on-top, hidden start)
//! and a tray icon whose left-click toggles a borderless popover and whose
//! right-click opens a small menu.

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;

use crate::tray_helpers::{compute_popover_position, fit_inner_height, should_show_after_hide};

/// Shared state for the popover toggle. Managed via `app.manage`.
/// - `last_hidden`: when the window was last hidden (flicker-guard, see tray_helpers).
/// - `suppress_hide`: true while a native file dialog is open, so the blur it
///   causes does not hide the popover (set/cleared in commands.rs).
#[derive(Default)]
pub struct PopoverState {
    pub last_hidden: Mutex<Option<Instant>>,
    pub suppress_hide: AtomicBool,
}

const DEBOUNCE: Duration = Duration::from_millis(250);
const DESIGN_HEIGHT: f64 = 1020.0;
const DESIGN_WIDTH: f64 = 440.0;
const MAX_SCREEN_FRACTION: f64 = 0.92;

/// Resize the popover to fit the current monitor height, position it centered
/// under the tray icon, then show + focus it.
fn show_popover(window: &WebviewWindow, icon_x: i32, icon_w: i32, icon_bottom: i32) {
    // Clamp height to the monitor so a 1020px layout fits short laptop screens.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let screen_h_logical = monitor.size().height as f64 / scale;
        let h = fit_inner_height(DESIGN_HEIGHT, screen_h_logical, MAX_SCREEN_FRACTION);
        let _ = window.set_size(LogicalSize::new(DESIGN_WIDTH, h));
    }

    // Position using physical pixels (icon rect is physical).
    let win_w_physical = window.outer_size().map(|s| s.width as i32).unwrap_or(440);
    let pos = compute_popover_position(icon_x, icon_w, icon_bottom, win_w_physical);
    let _ = window.set_position(PhysicalPosition::new(pos.x, pos.y));

    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_popover(app: &AppHandle, window: &WebviewWindow) {
    let _ = window.hide();
    if let Some(state) = app.try_state::<PopoverState>() {
        if let Ok(mut g) = state.last_hidden.lock() {
            *g = Some(Instant::now());
        }
    }
}

/// Build the tray icon, its menu, and event handlers. macOS only.
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let handle = app.handle();

    // ---- right-click menu ----
    let about_i = MenuItemBuilder::with_id("about", "About HP-41 Calculator").build(app)?;
    let start_login_i = CheckMenuItemBuilder::with_id("start_login", "Start at Login")
        .checked(handle.autolaunch().is_enabled().unwrap_or(false))
        .build(app)?;
    let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&about_i)
        .item(&start_login_i)
        .separator()
        .item(&quit_i)
        .build()?;

    let icon_path = app
        .path()
        .resolve("icons/tray-template.png", tauri::path::BaseDirectory::Resource)
        .unwrap_or_else(|_| "icons/tray-template.png".into());

    TrayIconBuilder::with_id("hp41-tray")
        .icon(tauri::image::Image::from_path(icon_path)?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false) // left click is the popover toggle, not the menu
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "about" => {
                let version = env!("CARGO_PKG_VERSION");
                app.dialog()
                    .message(format!("HP-41 Calculator\nVersion {version}"))
                    .title("About")
                    .blocking_show();
            }
            "start_login" => {
                let mgr = app.autolaunch();
                let now_enabled = mgr.is_enabled().unwrap_or(false);
                let _ = if now_enabled { mgr.disable() } else { mgr.enable() };
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                let Some(window) = app.get_webview_window("main") else {
                    return;
                };
                let visible = window.is_visible().unwrap_or(false);
                if visible {
                    hide_popover(app, &window);
                    return;
                }
                // Hidden: apply flicker-guard before re-showing.
                let last_hidden = app
                    .try_state::<PopoverState>()
                    .and_then(|s| s.last_hidden.lock().ok().map(|g| *g))
                    .flatten();
                if !should_show_after_hide(last_hidden, Instant::now(), DEBOUNCE) {
                    return; // this click is the one that just closed the popover
                }
                let icon_x = rect.position.x as i32;
                let icon_w = rect.size.width as i32;
                let icon_bottom = rect.position.y as i32 + rect.size.height as i32;
                show_popover(&window, icon_x, icon_w, icon_bottom);
            }
        })
        .build(app)?;

    Ok(())
}

/// Apply pure menu-bar presentation to the main window: no Dock icon, no
/// decorations, always on top, hidden until the tray toggles it.
pub fn apply_menu_bar_mode(app: &App) -> tauri::Result<()> {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_decorations(false);
        let _ = win.set_always_on_top(true);
        let _ = win.hide();
    }
    setup_tray(app)
}
```

> **Implementer note:** the builder/type names above target tauri 2.x. If `cargo check` reports a mismatch for this 2.11 point release, adjust the call (e.g. the field name on the `TrayIconEvent::Click` variant, the `Image::from_path` path, or `DialogExt` import) but keep the calls into `tray_helpers` unchanged. The `blocking_show()` About dialog is delivered on a menu-event worker thread (not the main thread), matching the existing file-dialog usage in `commands.rs`.

- [ ] **Step 3: Wire it into `lib.rs` — declare module, register plugin, manage state, call setup**

In `hp41-gui/src-tauri/src/lib.rs`:

(a) Add the module declaration after the `mod tray_helpers;` line from Task 3:

```rust
#[cfg(target_os = "macos")]
mod tray; // macOS menu-bar mode (tray icon + popover + Accessory policy)
```

(b) Register the autostart plugin. Change the builder head (currently lines 36-37) from:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init()) // Phase 50 — file dialog plugin for .raw/.card.json import/export
```

to:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init()) // Phase 50 — file dialog plugin for .raw/.card.json import/export
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
```

(c) Apply menu-bar mode at the END of the `.setup(|app| { ... })` closure, immediately before the final `Ok(())` (currently around line 99). Insert:

```rust
            // ── macOS menu-bar mode (Task 4 of the menu-bar plan) ──
            // Set HP41_SHOW_ON_START to opt out (normal visible window) — used by
            // anyone running the E2E suite on macOS locally.
            #[cfg(target_os = "macos")]
            {
                app.manage(crate::tray::PopoverState::default());
                if std::env::var_os("HP41_SHOW_ON_START").is_some() {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                } else if let Err(e) = crate::tray::apply_menu_bar_mode(app) {
                    eprintln!("hp41-gui: failed to enter menu-bar mode: {e}; showing window");
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                    }
                }
            }
            // Non-macOS: the window stays a normal decorated window. Because the
            // bundle config now starts hidden (visible:false, Task 6), show it.
            #[cfg(not(target_os = "macos"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                }
            }
```

- [ ] **Step 4: Compile**

Run: `cd hp41-gui/src-tauri && cargo check`
Expected: compiles. Resolve any 2.11 signature mismatches per the implementer note in Step 2. If you removed `autostart:default` in Task 2 Step 3, re-add it to `capabilities/default.json` now and re-run `cargo check`.

- [ ] **Step 5: Run all backend tests**

Run: `cd hp41-gui/src-tauri && cargo test --all-features`
Expected: all existing tests + the Task 3 `tray_helpers` tests pass.

- [ ] **Step 6: Manual smoke (macOS) — interactive verification**

Run: `cd hp41-gui && unset HP41_SHOW_ON_START && npm run tauri dev`
Verify:
1. No Dock icon appears; a calculator glyph appears in the menu bar.
2. Left-click the menu-bar icon → borderless popover drops down under the icon, focused.
3. Click another app → popover hides.
4. Left-click the icon again → reopens (no flicker; clicking the icon while open closes it, and an immediate re-click does not "bounce" it back open within ~250ms).
5. Right-click the icon → menu with "About HP-41 Calculator", "Start at Login" (checkbox), "Quit".
6. "About" shows version; "Quit" exits; toggle "Start at Login", reopen the menu, confirm the checkmark persists.

Document any deviation. (No automated assertion; this is the acceptance gate for the tray wiring.)

- [ ] **Step 7: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
✨ feat(gui): macOS menu-bar mode with tray popover and menu

On macOS, run as a status-item app: Accessory activation policy (no Dock
icon), borderless always-on-top window hidden on launch, left-click tray
toggles a popover positioned under the icon (with flicker-guard), right-click
menu offers About / Start at Login / Quit. Register tauri-plugin-autostart.
HP41_SHOW_ON_START env var opts into a normal visible window. Windows/Linux
behavior is unchanged.
```

---

## Task 5: Auto-hide on blur + suppress around native dialogs

**Files:**
- Modify: `hp41-gui/src-tauri/src/lib.rs` (add `on_window_event` blur→hide)
- Modify: `hp41-gui/src-tauri/src/commands.rs` (guard the four dialog commands)

- [ ] **Step 1: Add the blur→hide handler in `lib.rs`**

In `hp41-gui/src-tauri/src/lib.rs`, add an `.on_window_event(...)` to the builder chain, immediately AFTER the `.invoke_handler(tauri::generate_handler![ ... ])` block and BEFORE `.run(tauri::generate_context!())`:

```rust
        .on_window_event(|window, event| {
            #[cfg(target_os = "macos")]
            {
                if let tauri::WindowEvent::Focused(false) = event {
                    if window.label() != "main" {
                        return;
                    }
                    let app = window.app_handle();
                    // Don't hide while a native file dialog is open (it steals
                    // focus and would otherwise dismiss the popover mid-operation).
                    if let Some(state) = app.try_state::<crate::tray::PopoverState>() {
                        if state.suppress_hide.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                        if let Ok(mut g) = state.last_hidden.lock() {
                            *g = Some(std::time::Instant::now());
                        }
                    }
                    let _ = window.hide();
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (window, event); // silence unused warnings off-macOS
            }
        })
```

- [ ] **Step 2: Add a reusable suppress guard in `commands.rs`**

In `hp41-gui/src-tauri/src/commands.rs`, add after the existing `use` lines (around line 36, after the `#[cfg(test)] use std::path::Path;` line) an RAII guard:

```rust
/// RAII guard: while alive, sets PopoverState.suppress_hide so the macOS
/// auto-hide-on-blur handler does not dismiss the popover while a native file
/// dialog is open. Clearing on drop is panic-safe and covers early returns / `?`.
#[cfg(target_os = "macos")]
struct SuppressHideGuard<'a> {
    flag: Option<tauri::State<'a, crate::tray::PopoverState>>,
}

#[cfg(target_os = "macos")]
impl<'a> SuppressHideGuard<'a> {
    fn new(app: &'a AppHandle) -> Self {
        let flag = app.try_state::<crate::tray::PopoverState>();
        if let Some(s) = &flag {
            s.suppress_hide.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Self { flag }
    }
}

#[cfg(target_os = "macos")]
impl Drop for SuppressHideGuard<'_> {
    fn drop(&mut self) {
        if let Some(s) = &self.flag {
            s.suppress_hide.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// No-op on non-macOS so call sites stay identical across platforms.
#[cfg(not(target_os = "macos"))]
struct SuppressHideGuard;
#[cfg(not(target_os = "macos"))]
impl SuppressHideGuard {
    fn new(_app: &AppHandle) -> Self {
        Self
    }
}
```

- [ ] **Step 3: Hold the guard in each of the four dialog commands**

In `hp41-gui/src-tauri/src/commands.rs`, locate the four `#[tauri::command]` functions
`import_raw_dialog` (line ~541), `export_raw_dialog` (line ~690), `import_data_dialog`
(line ~736), `export_data_dialog` (line ~787). All four already take `app: AppHandle`
as their first parameter (verified), so `&app` is in scope. As the FIRST line of each
function body, insert:

```rust
    let _suppress = SuppressHideGuard::new(&app);
```

The guard lives for the whole command body — including the
`blocking_pick_file`/`blocking_save_file` call — and clears the flag when the
function returns (including on `?` early returns, since clearing happens in `Drop`).

- [ ] **Step 4: Compile**

Run: `cd hp41-gui/src-tauri && cargo check`
Expected: compiles on macOS. (Off-macOS the guard is a zero-field no-op; `_suppress` is intentionally unused — the leading underscore silences the warning.)

- [ ] **Step 5: Run backend tests**

Run: `cd hp41-gui/src-tauri && cargo test --all-features`
Expected: all pass (including `cancel_autosave_stress`, `card_io_tests`, `d25_6_parity`, etc.).

- [ ] **Step 6: Manual verification (macOS)**

Run: `cd hp41-gui && unset HP41_SHOW_ON_START && npm run tauri dev`
1. Open the popover, then trigger a `.raw` import (the in-app control calling `import_raw_dialog`). The native open panel appears and the popover does NOT vanish behind it.
2. Cancel/confirm the dialog → focus returns to the popover, which is still visible.
3. Open an in-app modal (e.g. STO prompt) → popover stays visible (modals are in-webview; confirms no regression).
Document any deviation.

- [ ] **Step 7: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
✨ feat(gui): auto-hide popover on blur, suppressed during file dialogs

On macOS, hide the menu-bar popover when the window loses focus, so clicking
away dismisses it. Guard the four native file-dialog commands with an RAII
suppress flag so the focus loss they cause does not dismiss the popover
mid-operation. In-app modals are unaffected (same webview, no window blur).
```

---

## Task 6: Window config — start hidden

**Files:**
- Modify: `hp41-gui/src-tauri/tauri.conf.json`

- [ ] **Step 1: Set the window to start hidden**

In `hp41-gui/src-tauri/tauri.conf.json`, in the single entry of `app.windows`, add `"visible": false` after the `"decorations": true` line:

```json
      {
        "title": "HP-41 Calculator",
        "width": 440,
        "height": 1020,
        "resizable": false,
        "decorations": true,
        "visible": false
      }
```

(`decorations` stays `true` — the cross-platform default for Windows/Linux. macOS
turns decorations off at runtime in `apply_menu_bar_mode`. `visible:false` prevents
a launch flash on macOS; Windows/Linux and the macOS `HP41_SHOW_ON_START` path
explicitly `show()` the window in `setup()` from Task 4.)

- [ ] **Step 2: Verify**

If a Linux/Windows runner is available: `cd hp41-gui && npm run tauri dev` → a normal decorated window appears (the Task 4 `#[cfg(not(target_os = "macos"))]` show path). On macOS, confirm the hidden-start path:

Run: `cd hp41-gui && unset HP41_SHOW_ON_START && npm run tauri dev`
Expected: no window on launch (only the menu-bar icon), confirming `visible:false` took effect.

- [ ] **Step 3: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
🔧 chore(gui): start the window hidden (visible:false)

Start the GUI window hidden to avoid a launch flash before macOS enters
menu-bar mode. Non-macOS and the HP41_SHOW_ON_START path explicitly show
the window in setup(), so behavior there is unchanged.
```

---

## Task 7: Keep the E2E smoke green

**Files:**
- Modify: `hp41-gui/wdio.conf.cjs`

The CI E2E job runs on **Ubuntu only** (`ci-gui.yml` → `e2e-linux`), where the
macOS code path never executes — so CI is green without any change. This task is a
safety net for running the E2E on macOS locally: launch the app with
`HP41_SHOW_ON_START=1` so it shows a normal window instead of the hidden popover.

- [ ] **Step 1: Confirm the launcher shape**

The config (verified) spawns `tauri-driver` in `beforeSession` via
`spawn(driverPath, [], { stdio: [...] })` with NO explicit `env`, so the child —
and the app `tauri-driver` launches from `tauri:options.application` — inherits
`process.env`. The cleanest injection point is therefore setting the variable on
`process.env` at the top of the config module.

Run: `grep -n "spawn(driverPath" hp41-gui/wdio.conf.cjs`
Expected: shows the `beforeSession` spawn (around line 118), confirming no `env` override.

- [ ] **Step 2: Set the env var at the top of the config module**

In `hp41-gui/wdio.conf.cjs`, immediately after the `const { spawn } = require('child_process');`
line (around line 22) and before `let tauriDriver;`, add:

```js
// macOS menu-bar mode: force a normal visible window so the smoke spec (which
// asserts the LCD renders) works if the suite is ever run on macOS. The spawned
// tauri-driver (and the app it launches) inherits process.env. No effect on the
// Ubuntu CI job, where the menu-bar code path is macOS-gated and never runs.
process.env.HP41_SHOW_ON_START = '1';
```

- [ ] **Step 3: Verify the change is syntactically valid**

Run: `cd hp41-gui && node -e "require('./wdio.conf.cjs'); console.log('wdio config loads OK')"`
Expected: prints `wdio config loads OK` (config parses without error).

- [ ] **Step 4: (Optional, macOS-local) Run the E2E**

Run: `cd hp41-gui && just gui-e2e` (requires `tauri-driver` installed: `cargo install tauri-driver --locked --version 2.0.6`).
Expected: `e2e/smoke.spec.js` passes — the window is shown (via `HP41_SHOW_ON_START`) and the LCD element is found. If `tauri-driver` is unavailable locally, rely on the Ubuntu CI job (which is unaffected by the macOS path).

- [ ] **Step 5: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
🧪 test(gui): keep macOS-local E2E visible under menu-bar mode

Launch the E2E app with HP41_SHOW_ON_START=1 so the smoke spec sees a normal
visible window if run on macOS. CI E2E is Ubuntu-only and unaffected (the
menu-bar path is macOS-gated). No production behavior change.
```

---

## Task 8: Documentation — ADR + CLAUDE.md

**Files:**
- Create: `docs/adr/v4.1-001-macos-menu-bar-mode.md`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Write the ADR**

Create `docs/adr/v4.1-001-macos-menu-bar-mode.md`:

```markdown
# ADR v4.1-001 — macOS Menu-Bar Mode

**Status:** Accepted
**Date:** 2026-05-29
**Context:** v4.1 iOS Foundation milestone (desktop "always at hand" request)

## Context

A user asked to turn the macOS `.app` into a desktop widget. True macOS desktop
widgets require WidgetKit + SwiftUI in an Xcode project and are effectively
non-interactive (glanceable, App-Intent-only) — unworkable for a ~40-key
interactive RPN calculator. The closest "always at hand" experience that reuses
the entire existing React UI and `hp41-core` backend is a macOS menu-bar
(status-item) app with a toggling popover.

## Decision

On macOS only, run as a pure menu-bar app:

- **Accessory activation policy** at runtime (`set_activation_policy`) — no Dock
  icon, no window on launch.
- Applied at **runtime under `#[cfg(target_os = "macos")]`**, not in
  `tauri.conf.json`, so the cross-platform window config keeps Windows/Linux as
  normal decorated, visible windows. The only config change is `visible:false`
  (avoids a launch flash); non-macOS and the E2E path explicitly `show()`.
- **Left-click** the tray icon toggles a borderless, always-on-top popover
  positioned under the icon, with a 250ms flicker-guard for the click/blur race.
- **Right-click** opens a menu: About / Start at Login (autostart) / Quit.
- **Auto-hide on blur**, suppressed only around the four native file-dialog
  commands via a backend `AtomicBool` (in-app modals share the webview and never
  trigger window blur, so they need no handling).
- **`HP41_SHOW_ON_START`** env var forces a normal visible window (used by E2E on macOS).

## Consequences

- Pure geometry/debounce logic lives in `tray_helpers.rs` and is unit-tested;
  the Tauri event wiring in `tray.rs` is verified by build + manual smoke + E2E.
- New runtime dependency: `tauri-plugin-autostart` (GUI workspace only — does
  not affect the "zero new runtime deps in hp41-core" invariant).
- No `hp41-core`, IPC-contract, or `Op` changes; SC-4 and bundle ID unchanged.
- A real WidgetKit status widget remains possible later as a separate,
  display-only glance; explicitly out of scope here.

## Alternatives considered

- **WidgetKit widget** — rejected: non-interactive, requires native Swift/Xcode
  wrapper and App-Group state sharing; cannot host the keyboard.
- **Übersicht (HTML desktop widgets)** — rejected: third-party, separate install,
  not Apple's native system.
- **Config-level fenster-less mode** — rejected: `tauri.conf.json` is
  cross-platform and would break Windows/Linux.
```

- [ ] **Step 2: Add a CLAUDE.md bullet under GUI specifics**

In `CLAUDE.md`, in the `### GUI specifics` section, add a new bullet after the
"Persistence sharing" bullet:

```markdown
- **macOS menu-bar mode (ADR-v4.1-001):** On macOS only, the GUI runs as a
  status-item app — Accessory activation policy, no Dock icon, window starts
  hidden (`visible:false`), left-click tray toggles a borderless always-on-top
  popover, right-click menu = About / Start at Login / Quit. Applied at runtime
  under `#[cfg(target_os = "macos")]` in `tray.rs` (NOT in `tauri.conf.json`, so
  Windows/Linux keep a normal decorated window). Pure logic in `tray_helpers.rs`
  (geometry + flicker-guard) is unit-tested. Auto-hide-on-blur is suppressed
  around native file dialogs via `PopoverState.suppress_hide`; in-app modals
  don't trigger window blur. `HP41_SHOW_ON_START=1` forces a normal window (E2E).
```

- [ ] **Step 3: Verify docs reference real paths**

Run: `ls docs/adr/v4.1-001-macos-menu-bar-mode.md hp41-gui/src-tauri/src/tray.rs hp41-gui/src-tauri/src/tray_helpers.rs`
Expected: all three exist.

- [ ] **Step 4: Commit**

Use `/git-workflow:commit --with-skills` with message:

```
📝 docs(gui): record macOS menu-bar mode (ADR-v4.1-001)

Add ADR-v4.1-001 documenting the menu-bar decision (Accessory policy,
runtime-gated, autostart, dialog-suppress flag, HP41_SHOW_ON_START) and a
matching bullet under GUI specifics in CLAUDE.md.
```

---

## Final verification (whole feature)

- [ ] **Run the full GUI gate locally:**

```bash
cd hp41-gui && npm run build && npm test            # frontend: tsc + Vitest
cd hp41-gui/src-tauri && cargo test --all-features  # backend + tray_helpers
cd hp41-gui/src-tauri && cargo clippy --all-targets # no unwrap_used, no warnings
```
Expected: all green. Clippy must be clean (the crate sets `#![deny(clippy::unwrap_used)]`; the new code uses `unwrap_or`, `try_state`, `if let`, and `.lock().ok()` — no bare `unwrap`).

- [ ] **SC-4 invariant still holds:**

```bash
grep -rn "fn op_\(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum\)" hp41-gui/src-tauri/src/
```
Expected: empty (no core ops duplicated in the GUI).

- [ ] **Confirm the unrelated `Justfile` working-tree change was never bundled** into any feature commit (`git log --stat` for the feature commits shows only files this plan touches).

---

## Self-review notes (author)

- **Spec coverage:** §2 Backend → Tasks 4 (tray, menu, Accessory, autostart) + 5 (blur/suppress). §3 Frontend scaling → Task 1. §4 Config/assets → Tasks 2 (autostart dep/perm), 4 (icon), 6 (visible:false). §5 Tests/risks → Tasks 3 (unit tests), 7 (E2E escape hatch). §"Open Questions" → flicker-guard (Task 3 `should_show_after_hide` + Task 4 wiring), dialog/modal suppression (Task 5 + the in-webview-modal note), About = system dialog (Task 4 `dialog().message()`), `HP41_SHOW_ON_START` (Tasks 4 + 7). ADR/CLAUDE.md (Task 8) per project convention.
- **Type consistency:** `PopoverState { last_hidden: Mutex<Option<Instant>>, suppress_hide: AtomicBool }` defined in `tray.rs`, used identically in `lib.rs` (blur handler) and `commands.rs` (guard). Helpers `compute_popover_position`/`should_show_after_hide`/`fit_inner_height` defined in `tray_helpers.rs` (Task 3) and called in `tray.rs` (Task 4) with matching signatures. `computeScale`/`DESIGN_WIDTH`/`DESIGN_HEIGHT` defined in `scale.ts` (Task 1) and used in `main.tsx`.
- **No placeholders:** every code step shows complete code; commands have expected output. The two genuine unknowns (exact tauri 2.11 signatures; exact `wdio.conf.cjs` launcher shape) are flagged with explicit "verify and adapt" instructions plus fallbacks, not silent TODOs.
- **Corrections folded in from codebase verification:** `main.tsx` keeps React 19 `createRoot` and the `index.css`+`themes.css` import order (D-48.11); E2E files are `wdio.conf.cjs` / `e2e/smoke.spec.js`; CI E2E is Ubuntu-only so the macOS path doesn't affect it.
