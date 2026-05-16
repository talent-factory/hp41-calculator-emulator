// FN-QUAL-05 smoke — literal ROADMAP success criterion (Phase 27, Plan 27-04).
//
// Boots the production Tauri release binary via tauri-driver (WebDriver classic
// on 127.0.0.1:4444), clicks `2 ENTER 3 +` on the SVG keyboard, asserts the LCD
// reads `5.0000`. Per D-27.13 this is the SOLE E2E spec — broader flows (modal
// interactions, autosave persistence roundtrip) are deferred to v2.3+.
//
// Selectors:
//   data-key-id    Keyboard.tsx — set on each SVG <g class="key"> wrapper
//   data-testid    Display14Seg.tsx — set on the LCD outer <svg>
//   data-text      Display14Seg.tsx — mirrors the displayed string for tests
//                  (the 14-segment LCD renders SVG <path> only, so .getText()
//                  returns empty; data-text is the contract)
//
// Per D-27.15 AMENDED (2026-05-15) this is WebdriverIO + Mocha, NOT Playwright
// — `tauri-driver` 2.0.6 speaks WebDriver classic which Playwright does NOT.
//
// Plain `.js` (not TS) — Mocha + WDIO globals (`describe`, `it`, `$`, `browser`)
// resolve at runtime; the spec has no real type dependence. Sticking to `.js`
// removes the WDIO 9 → tsx auto-detection footgun (no tsx devDep required).

/**
 * Dispatch a synthetic `click` MouseEvent directly on the SVG element matched
 * by `[data-key-id="${keyId}"]`. WebKitGTK's WebDriver implementation does not
 * consider SVG `<g>` elements "interactable" via the standard element-click
 * flow even when CSS `pointer-events: all` is set — the interactability check
 * uses element bounding-box heuristics that don't account for SVG group
 * containers. The canonical workaround is to bypass the WebDriver
 * interactability gate by dispatching the click event through the DOM API
 * directly. React's onClick handler attached at the `<g>` level still fires
 * (it listens for `click` events bubbling through the SVG namespace), so the
 * dispatch path through `App.tsx::handleClick → invokeForKey → dispatch_op`
 * is exercised end-to-end exactly as it would be from a real user click.
 */
async function clickKey(keyId) {
    const dispatched = await browser.execute((sel) => {
        const el = document.querySelector(sel);
        if (!el) return false;
        el.dispatchEvent(
            new MouseEvent('click', { bubbles: true, cancelable: true }),
        );
        return true;
    }, `[data-key-id="${keyId}"]`);
    if (!dispatched) {
        throw new Error(`element [data-key-id="${keyId}"] not found in DOM`);
    }
}

describe('HP-41 GUI smoke (FN-QUAL-05, D-27.13 literal ROADMAP scope)', () => {
    it('2 ENTER 3 + displays 5.0000', async () => {
        // Wait for the React tree to mount and SVG keyboard to render before
        // dispatching clicks. The LCD lives at the top of the tree and is
        // always present once the app initializes.
        const display = await $('[data-testid="lcd-display"]');
        await display.waitForExist({ timeout: 10000 });

        // Click sequence: 2 ENTER 3 + → X = 5.
        await clickKey('2');
        await clickKey('enter');
        await clickKey('3');
        await clickKey('plus');

        // Give React a moment to round-trip through the Tauri IPC and re-render.
        await browser.pause(250);

        // Display14Seg.tsx unconditionally sets `data-text={text}` on the
        // outer <svg>, and the 14-segment LCD renders SVG <path> only (no
        // <text> nodes), so `data-text` is the ONLY meaningful assertion
        // surface. If it's absent the contract is broken — fail loudly
        // rather than fall through to `.getText()` (which would return ""
        // and either mask a regression OR silently pass against the
        // accidental empty string).
        const dataText = await display.getAttribute('data-text');
        if (dataText === null) {
            throw new Error(
                "[data-testid='lcd-display'] is missing data-text — Display14Seg contract broken (see hp41-gui/src/Display14Seg.tsx)",
            );
        }
        if (dataText !== '5.0000') {
            throw new Error(
                `expected [data-testid="lcd-display"] data-text='5.0000', got '${dataText}'`,
            );
        }
    });
});
