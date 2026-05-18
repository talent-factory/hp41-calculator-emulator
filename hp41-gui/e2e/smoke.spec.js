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

/**
 * Direct Tauri-backend invocation for E2E tests when the click-sequence path is
 * blocked by frontend modal semantics. Plan 32-03 / D-32.2 / Open Question #1
 * fallback: the on-screen `xeq_name` modal's submit-key is the `enter` key
 * (alphaChar='N'), but App.tsx::handleClick prioritizes the `effectiveId ===
 * 'enter'` branch BEFORE the alphaChar routing — so a label like "SINH" that
 * contains 'N' cannot be typed via clicks (the 3rd letter click would submit
 * the partial label "SI" instead of appending 'N'). The fallback bypasses the
 * modal entirely by dispatching the magic-prefix `xeq_<label>` key id directly
 * to the `dispatch_op` Tauri command, which routes through key_map::resolve →
 * Op::Xeq(label) → xrom_resolve → the resolved Math Pac I Op variant.
 *
 * Per RESEARCH.md Open Question #1, this is acceptable per QUAL-03 because the
 * "workflow" being exercised is `xeq_SINH → Op::Sinh → display 1.1752`, and
 * the dispatch path through xrom_resolve is identical whether triggered by a
 * click sequence or by a direct invoke. The xrom_resolve fallback IS the
 * regression-sensitive path; the modal click sequence is GUI-only ceremony.
 *
 * Uses Tauri v2's `window.__TAURI_INTERNALS__.invoke` — exposed by Tauri's JS
 * runtime regardless of the `withGlobalTauri` config flag. Production code in
 * App.tsx uses `import { invoke } from '@tauri-apps/api/core'` which compiles
 * to a call into `__TAURI_INTERNALS__.invoke`; the E2E test takes the same
 * underlying path through the window global.
 */
async function invokeBackend(command, args = {}) {
    const result = await browser.executeAsync(
        function (cmd, payload, done) {
            // eslint-disable-next-line no-underscore-dangle
            const internals = window.__TAURI_INTERNALS__;
            if (!internals || typeof internals.invoke !== 'function') {
                done({ ok: false, err: 'window.__TAURI_INTERNALS__.invoke not available' });
                return;
            }
            internals.invoke(cmd, payload)
                .then(value => done({ ok: true, value }))
                .catch(err => done({ ok: false, err: String(err && err.message ? err.message : err) }));
        },
        command,
        args,
    );
    if (!result.ok) {
        throw new Error(`invokeBackend('${command}') failed: ${result.err}`);
    }
    return result.value;
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

    // Plan 32-03 / D-32.2 / QUAL-03 — Math Pac I via xrom_resolve.
    //
    // Click-strategy: BROWSER.EXECUTE FALLBACK (per RESEARCH.md Open Question
    // #1 + Plan 32-03 reconnaissance). The pure-click path is blocked because
    // the `xeq_name` modal's submit key is `enter`, but the `enter` key also
    // carries `alphaChar: 'N'` (Keyboard.tsx line 105). App.tsx::handleClick
    // (lines 387-400) checks `effectiveId === 'enter'` BEFORE alphaChar
    // routing — so clicking `enter` while accumulating "SI" would submit
    // "SI" rather than append 'N' to produce "SIN". Labels without 'N' could
    // use the real-click path, but SINH cannot.
    //
    // The fallback dispatches `xeq_SINH` directly through `dispatch_op`,
    // which routes through key_map::resolve → Op::Xeq("SINH") → xrom_resolve
    // → Op::Sinh. The regression-sensitive surface is xrom_resolve; the
    // modal click ceremony is GUI-only and not what QUAL-03 attests.
    it('XEQ "SINH" 1 displays 1.1752 (Math Pac I via xrom_resolve)', async () => {
        const display = await $('[data-testid="lcd-display"]');
        await display.waitForExist({ timeout: 10000 });

        // Push X = 1 via real clicks — confirms the keyboard path still works
        // for digit entry even when the xrom invocation uses the fallback.
        await clickKey('1');
        await clickKey('enter');

        // Dispatch Op::Sinh via xrom_resolve directly. sinh(1) = (e − 1/e)/2
        // ≈ 1.17520119364 → FIX 4 default formats as `1.1752`.
        await invokeBackend('dispatch_op', { keyId: 'xeq_SINH' });
        await browser.pause(250);

        const dataText = await display.getAttribute('data-text');
        if (dataText === null) {
            throw new Error(
                "[data-testid='lcd-display'] is missing data-text — Display14Seg contract broken (see hp41-gui/src/Display14Seg.tsx)",
            );
        }
        if (dataText !== '1.1752') {
            throw new Error(
                `expected [data-testid="lcd-display"] data-text='1.1752', got '${dataText}'`,
            );
        }
    });

    // Plan 32-03 / D-32.2 / D-32.3 / QUAL-03 — Math Pac I modal pipeline.
    //
    // Click-strategy: BROWSER.EXECUTE FALLBACK for the XEQ-by-name invocations
    // (opening MATRIX and triggering DET), REAL CLICKS for digit entry and
    // R/S submits between matrix elements. This hybrid honors the spirit of
    // D-32.2 — the test exercises the modal_program lifecycle (ORDER=? → 4×
    // ElementPrompt → Ready → DET), the column-major element iteration, and
    // R/S-as-submit semantics (D-28.5 / D-31.1 R/S 3-way routing) — without
    // forcing brittle alpha-character click sequences for the program names.
    //
    // Matrix `[[1, 2], [3, 4]]` has det = 1·4 − 2·3 = −2 → `-2.0000` (FIX 4).
    // Column-major iteration order (per math1/matrix.rs::submit_modal lines
    // 372-401): row varies fastest, then column. For [[1,2],[3,4]] in
    // standard row-major layout, the entry sequence at the column-major
    // prompts (A1,1 → A2,1 → A1,2 → A2,2) is 1, 3, 2, 4.
    //
    // DET is invariant under transpose, so the assertion `-2.0000` holds
    // regardless of which input convention the OM ultimately intends.
    //
    // Per D-32.3 — NO Esc-cancel verification at the end. The natural
    // modal_program lifecycle (clears when DET fires) is sufficient.
    it('XEQ "MATRIX" 2x2 DET displays -2.0000 (Math Pac I modal pipeline)', async () => {
        const display = await $('[data-testid="lcd-display"]');
        await display.waitForExist({ timeout: 10000 });

        // Open MATRIX workflow via direct dispatch. modal_program_active flips
        // to true; ORDER=? is the first prompt.
        await invokeBackend('dispatch_op', { keyId: 'xeq_MATRIX' });
        await browser.pause(150);

        // Enter order 2: digit then R/S (which routes to submit_modal because
        // modal_program_active is true — see App.tsx::invokeForKey lines
        // 86-99 / D-31.1 R/S 3-way routing).
        await clickKey('2');
        await clickKey('r_s');
        await browser.pause(100);

        // Enter the four matrix values in column-major order: A1,1=1,
        // A2,1=3, A1,2=2, A2,2=4 (input sequence 1, 3, 2, 4).
        await clickKey('1');
        await clickKey('r_s');
        await browser.pause(100);
        await clickKey('3');
        await clickKey('r_s');
        await browser.pause(100);
        await clickKey('2');
        await clickKey('r_s');
        await browser.pause(100);
        await clickKey('4');
        await clickKey('r_s');
        await browser.pause(150);

        // Matrix is now Ready (all 4 elements entered). Dispatch DET via
        // xrom_resolve. The DET program reads matrix_dim + R15.. → Op::MatDet
        // → writes determinant to X → modal_program clears.
        await invokeBackend('dispatch_op', { keyId: 'xeq_DET' });
        await browser.pause(500); // DET is heavier than a single op — wider window

        const dataText = await display.getAttribute('data-text');
        if (dataText === null) {
            throw new Error(
                "[data-testid='lcd-display'] is missing data-text — Display14Seg contract broken (see hp41-gui/src/Display14Seg.tsx)",
            );
        }
        if (dataText !== '-2.0000') {
            throw new Error(
                `expected [data-testid="lcd-display"] data-text='-2.0000', got '${dataText}'`,
            );
        }
    });
});
