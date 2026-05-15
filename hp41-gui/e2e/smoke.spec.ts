// FN-QUAL-05 smoke — literal ROADMAP success criterion (Phase 27, Plan 27-04).
//
// Boots the production Tauri release binary via tauri-driver (WebDriver classic
// on 127.0.0.1:4444), clicks `2 ENTER 3 +` on the SVG keyboard, asserts the LCD
// reads `5.0000`. Per D-27.13 this is the SOLE E2E spec — broader flows (modal
// interactions, autosave persistence roundtrip) are deferred to v2.3+.
//
// Selectors (verified during Plan 27-04 read_first):
//   data-key-id    Keyboard.tsx lines 285, 303 (Phase 26 Plan 04 Task 3)
//   data-testid    Display14Seg.tsx (added by Plan 27-04 Task 1)
//
// Per D-27.15 AMENDED (2026-05-15) this is WebdriverIO + Mocha, NOT Playwright
// — `tauri-driver` 2.0.6 speaks WebDriver classic which Playwright does NOT.
//
// The tsconfig.json `include: ["src"]` means this file is NOT type-checked by
// `tsc --noEmit` in gui-ci; WDIO's built-in spec loader handles transpilation.
// Globals like `describe`, `it`, and `$` come from Mocha + WebdriverIO at
// runtime.

declare const describe: (name: string, fn: () => void) => void;
declare const it: (name: string, fn: () => Promise<void> | void) => void;
declare const $: (selector: string) => Promise<{
    click: () => Promise<void>;
    getText: () => Promise<string>;
    getAttribute: (name: string) => Promise<string | null>;
}>;

describe('HP-41 GUI smoke (FN-QUAL-05, D-27.13 literal ROADMAP scope)', () => {
    it('2 ENTER 3 + displays 5.0000', async () => {
        // Click sequence: 2 ENTER 3 + → X = 5.
        await (await $('[data-key-id="2"]')).click();
        await (await $('[data-key-id="enter"]')).click();
        await (await $('[data-key-id="3"]')).click();
        await (await $('[data-key-id="plus"]')).click();

        const display = await $('[data-testid="lcd-display"]');

        // Display14Seg renders SVG segment paths, not plain text. The
        // `data-text` attribute (added alongside `data-testid` in Plan 27-04
        // Task 1) is the primary assertion path. If `data-text` is absent
        // for any reason (older build), fall back to plain text content —
        // the assertion still fires on the same DOM node.
        const dataText = await display.getAttribute('data-text');
        if (dataText !== null) {
            if (dataText !== '5.0000') {
                throw new Error(
                    `expected [data-testid="lcd-display"] data-text='5.0000', got '${dataText}'`,
                );
            }
        } else {
            const plain = await display.getText();
            if (plain !== '5.0000') {
                throw new Error(
                    `expected [data-testid="lcd-display"] text='5.0000', got '${plain}'`,
                );
            }
        }
    });
});
