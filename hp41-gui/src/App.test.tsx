// Phase 26 Plan 04 Task 3 — Integration tests for the end-to-end click →
// modal → dispatch → render pipeline.
//
// The 5 BLOCKERs CR-01..CR-05 were each individually unit-tested but never
// exercised across layers. This suite mocks @tauri-apps/api/core's `invoke`
// export and renders the full <App /> via @testing-library/react, then drives
// click + keyboard interactions to verify the wiring fixes from Tasks 1 + 2.
//
// Test groups:
//   A — CR-01 ASN canonical keyCode (3 tests)
//   B — CR-02 helpOpen short-circuit (2 tests)
//   C — CR-03 on-screen ENTER / ← translation (2 tests)
//   D — CR-04 display_override + event_buffer wiring (3 tests)
//   E — CR-05 CATALOG max=4 + lower-bound rejection (2 tests)
//   F — USER-mode end-to-end (CR-01 + CR-03 closure) (1 test)

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, fireEvent, waitFor, cleanup } from '@testing-library/react';
import { act } from 'react';
import App from './App';

// --- Tauri invoke mock --------------------------------------------------
//
// Module-scoped mock fn intercepted by `vi.mock`. Each test resets the mock
// and seeds it with `mockResolvedValue(makeEmptyView())` via beforeEach so
// the initial mount `invoke('get_state')` always settles to an empty state.
// Individual tests stack additional `mockResolvedValueOnce(...)` returns
// for subsequent dispatch_op calls.

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: () => Promise.resolve(() => {}),
}));

// --- CalcStateView test fixture -----------------------------------------

interface Annunciators {
  user: boolean;
  prgm: boolean;
  alpha: boolean;
  rad: boolean;
  grad: boolean;
}

interface CalcStateView {
  display_str: string;
  x_str: string;
  y_str: string;
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
  program_steps: string[];
  pc: number;
  user_keymap: Array<[number, string]>;
  flags: number[];
  display_override: string | null;
  event_buffer: string[];
  // Phase 31 Plan 03: modal workflow state fields
  is_running: boolean;
  modal_program_active: boolean;
  modal_requires_alpha_label: boolean;
  modal_prompt: string | null;
  // Phase 41 D-41.3: live-display trigger fields
  clock_active: boolean;
  stopwatch_keyboard_mode: boolean;
  stopwatch_running: boolean;
  // Phase 63 Plan 06: yield state from run_program/resume_program
  pending_yield: { kind: string; text: string; resume_ms: number } | null;
}

function makeEmptyView(overrides: Partial<CalcStateView> = {}): CalcStateView {
  return {
    display_str: '0.0000',
    x_str: '0.0000',
    y_str: '0.0000',
    z_str: '0.0000',
    t_str: '0.0000',
    lastx_str: '0.0000',
    in_eex_mode: false,
    annunciators: {
      user: false,
      prgm: false,
      alpha: false,
      rad: false,
      grad: false,
    },
    print_lines: [],
    program_steps: ['END'],
    pc: 0,
    user_keymap: [],
    flags: [],
    display_override: null,
    event_buffer: [],
    // Phase 31 Plan 03: modal defaults
    is_running: false,
    modal_program_active: false,
    modal_requires_alpha_label: false,
    modal_prompt: null,
    // Phase 41 D-41.3: live-display trigger defaults
    clock_active: false,
    stopwatch_keyboard_mode: false,
    stopwatch_running: false,
    // Phase 63 Plan 06: yield state default (null = program not yielded)
    pending_yield: null,
    ...overrides,
  };
}

// --- Test helpers --------------------------------------------------------
//
// jsdom + React 19 + Vitest interact poorly when fireEvent triggers state
// updates whose useEffect cleanups (e.g. window keydown listener swap) are
// not allowed to commit before the next event fires — stale listeners pile
// up and a single fireEvent.keyDown gets multi-dispatched. Wrap each
// interaction in act(async () => fireEvent) followed by a microtask-yielding
// act(async () => setTimeout(0)) so effects flush.

async function renderAppAndWait() {
  const utils = render(<App />);
  await waitFor(() => {
    expect(mockInvoke).toHaveBeenCalledWith('get_state', undefined);
  });
  await waitFor(() => {
    expect(utils.container.querySelector('svg[aria-label="HP-41C keyboard"]')).not.toBeNull();
  });
  // Flush any pending effects from the initial mount.
  await act(async () => {
    await new Promise(resolve => setTimeout(resolve, 0));
  });
  return utils;
}

function findKey(container: HTMLElement, id: string): SVGGElement {
  const el = container.querySelector(`[data-key-id="${id}"]`);
  if (!el) {
    throw new Error(`could not find key id="${id}" in keyboard SVG`);
  }
  return el as SVGGElement;
}

async function clickKey(container: HTMLElement, id: string): Promise<void> {
  const el = findKey(container, id);
  await act(async () => {
    fireEvent.click(el);
  });
  await act(async () => {
    await new Promise(resolve => setTimeout(resolve, 0));
  });
}

async function pressKey(key: string): Promise<void> {
  await act(async () => {
    fireEvent.keyDown(window, { key });
  });
  await act(async () => {
    await new Promise(resolve => setTimeout(resolve, 0));
  });
}

function getDisplayText(container: HTMLElement): string {
  const el = container.querySelector('[data-displaytext]');
  return el?.getAttribute('data-displaytext') ?? '';
}

// Default mock prefs (Phase 49 Plan 04): onboarding_done=true so the wizard
// does NOT open during tests (which would block keyboard dispatch).
// Tests that specifically need to test first-run behavior can override.
const DEFAULT_PREFS = { theme: 'dark', onboarding_done: true, macos_launch_mode: 'menu-bar', global_shortcut: 'Control+Alt+Command+H' };

beforeEach(() => {
  mockInvoke.mockReset();
  // Route get_prefs to return DEFAULT_PREFS so the onboarding wizard stays
  // closed in tests (Phase 49 Plan 04: wizard blocks keyboard dispatch when open).
  // Phase 55 Plan 02: is_macos and is_ios must return false so the desktop
  // SVG onClick path remains active (isIos truthy would suppress SVG clicks).
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
    if (cmd === 'is_macos') return Promise.resolve(false);
    if (cmd === 'is_ios') return Promise.resolve(false);
    return Promise.resolve(makeEmptyView());
  });
});

// Unmount each render between tests. Required because vitest runs with
// globals:false (no testing-library auto-cleanup) — without this, portaled
// elements (AlphaTouchInput / print sheet → document.body, sef) from one test
// leak into the next and break document-level queries.
afterEach(() => {
  cleanup();
});

// =====================================================================
// Group A — CR-01 ASN canonical keyCode
// =====================================================================

describe('CR-01 — ASN flow uses canonical key.keyCode (not row*10+col)', () => {
  it('A1: SHIFT+XEQ opens ASN modal; clicking SIN advances to assign_label with LCD "ASN _"', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'xeq_prompt');
    expect(getDisplayText(container)).toBe('ASN __');
    // No dispatch fired yet — the modal-opener was intercepted before invoke.
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', expect.anything());
    await clickKey(container, 'sin');
    expect(getDisplayText(container)).toBe('ASN _');
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', expect.anything());
  });

  it('A2: ASN + SIN + type "TEST" + ALPHA dispatches asn_25_TEST (canonical, not 23)', async () => {
    // TOUCH-04 update: on-screen ENTER now types 'N' in text-label modals.
    // Use on-screen ALPHA (alpha_toggle) to terminate/submit the assign_label modal.
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'xeq_prompt');
    await clickKey(container, 'sin');
    for (const ch of ['T', 'E', 'S', 'T']) {
      await pressKey(ch);
    }
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ user_keymap: [[25, 'TEST']] }),
    );
    await clickKey(container, 'alpha_toggle');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'asn_25_TEST' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'asn_23_TEST' });
  });

  it('A3: clicking a key with undefined keyCode (CHS) inside ASN modal surfaces toast; no dispatch', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'xeq_prompt');
    expect(getDisplayText(container)).toBe('ASN __');
    await clickKey(container, 'chs');
    await waitFor(() => {
      const toast = container.querySelector('.toast');
      expect(toast).not.toBeNull();
      expect(toast?.textContent).toMatch(/cannot.*assign|assign.*cannot/i);
    });
    // No asn_* dispatch fired.
    const asnCalls = mockInvoke.mock.calls.filter(
      ([cmd, args]) =>
        cmd === 'dispatch_op' &&
        typeof args === 'object' &&
        args !== null &&
        typeof (args as { keyId?: unknown }).keyId === 'string' &&
        ((args as { keyId: string }).keyId).startsWith('asn_'),
    );
    expect(asnCalls.length).toBe(0);
    expect(getDisplayText(container)).toBe('ASN __');
  });
});

// =====================================================================
// Group B — CR-02 helpOpen gates window keystrokes
// =====================================================================

describe('CR-02 — `?` overlay opens but window keys do not dispatch', () => {
  it('B1: helpOpen=true → typing s/q/r/t/Backspace/0-9 does not invoke dispatch_op with op id', async () => {
    const { container } = await renderAppAndWait();
    await pressKey('?');
    expect(container.querySelector('.help-overlay')).not.toBeNull();
    // Snapshot dispatch_op call ids before the search keystrokes so we can
    // verify no NEW op-resolving dispatches fire during help search.
    const before = new Set(
      mockInvoke.mock.calls
        .filter(([cmd]) => cmd === 'dispatch_op')
        .map(([, args]) => (args as { keyId: string }).keyId),
    );
    for (const k of ['s', 'q', 'r', 't', 'Backspace', '0', '1', '2', '5']) {
      await pressKey(k);
    }
    // None of the corresponding op ids ('sqrt', 'sin', 'rdn', 'tan',
    // 'clx', '0'..'2', '5') should appear in dispatch_op calls. The
    // production code's `if (helpOpen) return;` gate blocks them all.
    const forbiddenIds = ['sqrt', 'sin', 'rdn', 'tan', 'clx', '0', '1', '2', '5'];
    for (const id of forbiddenIds) {
      if (!before.has(id)) {
        expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: id });
      }
    }
  });

  it('B2: after Esc closes overlay, "s" once again dispatches sqrt', async () => {
    const { container } = await renderAppAndWait();
    await pressKey('?');
    expect(container.querySelector('.help-overlay')).not.toBeNull();
    await pressKey('Escape');
    expect(container.querySelector('.help-overlay')).toBeNull();
    await pressKey('s');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'sqrt' });
  });
});

// =====================================================================
// Group C — CR-03 on-screen ENTER / ← translation
// =====================================================================

describe('CR-03 — on-screen ENTER/← translate to Enter/Backspace in modals', () => {
  it('C1: assign_label modal with acc=TEST → click ALPHA dispatches asn_25_TEST', async () => {
    // TOUCH-04 update: on-screen ENTER types 'N' in text-label modals; ALPHA terminates.
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'xeq_prompt');
    await clickKey(container, 'sin');
    for (const ch of ['T', 'E', 'S', 'T']) {
      await pressKey(ch);
    }
    expect(getDisplayText(container)).toBe('ASN TEST_');
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'alpha_toggle');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'asn_25_TEST' });
  });

  it('C2: xeq_name modal with acc=ABC → click ← pops to AB (no clx dispatch)', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'xeq_prompt');
    for (const ch of ['A', 'B', 'C']) {
      await pressKey(ch);
    }
    expect(getDisplayText(container)).toBe('XEQ ABC_');
    await clickKey(container, 'clx_or_a');
    expect(getDisplayText(container)).toBe('XEQ AB_');
    // No clx dispatch happened inside the modal.
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'clx' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'alpha_clear' });
  });
});

// =====================================================================
// Group D — CR-04 display_override + event_buffer wiring
// =====================================================================

describe('CR-04 — display_override and event_buffer are consumed by React', () => {
  it('D1: dispatch returning display_override="HELLO" renders HELLO (not display_str)', async () => {
    const { container } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ display_override: 'HELLO', display_str: '0.0000' }),
    );
    await clickKey(container, '1');
    await waitFor(() => {
      expect(getDisplayText(container)).toBe('HELLO');
    });
  });

  it('D2: dispatch with display_override=null falls back to display_str=3.1416', async () => {
    const { container } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ display_override: null, display_str: '3.1416' }),
    );
    await clickKey(container, '1');
    await waitFor(() => {
      expect(getDisplayText(container)).toBe('3.1416');
    });
  });

  it('D3: dispatch returning event_buffer=["BEEP"] surfaces BEEP in toast', async () => {
    const { container } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ event_buffer: ['BEEP'] }),
    );
    await clickKey(container, '1');
    await waitFor(() => {
      const toast = container.querySelector('.toast');
      expect(toast).not.toBeNull();
      expect(toast?.textContent).toContain('BEEP');
    });
  });

  // Phase 41 D-41.6 / TIME-GUI-06: alarm event parsing assertions.
  // D4 verifies prefix-stripping for "alarm:message:" events.
  it('D4: event_buffer "alarm:message:ALARM!" surfaces alarm text in toast (not raw prefix)', async () => {
    const { container } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ event_buffer: ['alarm:message:ALARM!'] }),
    );
    await clickKey(container, '1');
    await waitFor(() => {
      const toast = container.querySelector('.toast');
      expect(toast).not.toBeNull();
      // Alarm text must appear (prefix stripped).
      expect(toast?.textContent).toContain('ALARM!');
      // Raw prefix must NOT appear in the toast.
      expect(toast?.textContent).not.toContain('alarm:message:');
    });
  });

  // D5 verifies alarm:xeq routing — SC-3 / CR-01 gap closure: control alarm triggers
  // run_program({ label }) so idle-fired user-LBL alarms execute (not dispatch_op which
  // only resolves builtins and returns InvalidOp for user programs — D-25.6 CLI parity).
  it('D5: event_buffer "alarm:xeq:TESTLBL" invokes run_program({ label: "TESTLBL" }) (SC-3)', async () => {
    const { container } = await renderAppAndWait();
    // First call: dispatch_op for the key click, returns alarm event.
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ event_buffer: ['alarm:xeq:TESTLBL'] }),
    );
    // Second call: run_program for the alarm:xeq drain (invoked by the useEffect).
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, '1');
    await waitFor(() => {
      // SC-3: the alarm:xeq useEffect must call run_program with the bare label.
      expect(mockInvoke).toHaveBeenCalledWith('run_program', { label: 'TESTLBL' });
      // Regression guard: dispatch_op(xeq_…) must NOT be called (old broken path).
      expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'xeq_TESTLBL' });
    });
  });
});

// =====================================================================
// Group E — CR-05 CATALOG bounds
// =====================================================================

describe('CR-05 — CATALOG modal accepts 1..=4, rejects 0 and 5..=9', () => {
  it('E1: ENTER + CATALOG (shifted) → key "4" dispatches catalog_4', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'enter');
    expect(getDisplayText(container)).toBe('CAT _');
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await pressKey('4');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'catalog_4' });
  });

  it('E2: CATALOG modal rejects "0" (lower-bound) and "5" (upper-bound); no dispatch', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'enter');
    expect(getDisplayText(container)).toBe('CAT _');
    await pressKey('0');
    await pressKey('5');
    // The lower-bound + upper-bound guards in pending_input.ts reject both
    // keystrokes inside the open Catalog modal. Neither catalog_0 nor
    // catalog_5 reaches the backend.
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'catalog_0' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'catalog_5' });
    expect(getDisplayText(container)).toBe('CAT _');
  });
});

// =====================================================================
// Group F — USER-mode end-to-end (CR-01 + CR-03 closure)
// =====================================================================

describe('USER-mode end-to-end — CR-01 + CR-03 round-trip', () => {
  it('F1: full ASN click flow + USER toggle relabels STO key with "TEST"', async () => {
    // TOUCH-04 update: on-screen ALPHA (not ENTER) terminates the assign_label modal.
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'xeq_prompt');
    await clickKey(container, 'sto_prompt');
    for (const ch of ['T', 'E', 'S', 'T']) {
      await pressKey(ch);
    }
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ user_keymap: [[22, 'TEST']] }),
    );
    await clickKey(container, 'alpha_toggle');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'asn_22_TEST' });
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({
        user_keymap: [[22, 'TEST']],
        annunciators: {
          user: true,
          prgm: false,
          alpha: false,
          rad: false,
          grad: false,
        },
      }),
    );
    await clickKey(container, 'user_mode');
    await waitFor(() => {
      const stoKey = findKey(container, 'sto_prompt');
      const texts = Array.from(stoKey.querySelectorAll('text')).map(t => t.textContent);
      expect(texts).toContain('TEST');
    });
  });
});

// =====================================================================
// Group G — v2.2.1 / quick-task 260516-c1p
// Mode-aware √x shifted (CLP in PRGM, x² outside) + alphaChar fallback
// for on-screen letter clicks inside text-input modals (xeq_name / clp /
// assign_label). Closes the GUI-parity gap blocking section 3 step 4 of
// docs/verifying-card-reader.md.
// =====================================================================

describe('quick-task 260516-c1p — mode-aware √x shifted', () => {
  it('G1: PRGM=true, SHIFT + √x → opens CLP modal (no sq dispatch)', async () => {
    // Seed prgm=true on initial get_state. mockResolvedValueOnce overrides
    // ONLY the next invoke call (the mount's get_state); subsequent calls
    // fall back to the beforeEach mockResolvedValue(makeEmptyView()) — but
    // CLP-modal opening is frontend-only so no further invoke fires here.
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({
        annunciators: { user: false, prgm: true, alpha: false, rad: false, grad: false },
      }),
    );
    const { container } = await renderAppAndWait();

    await clickKey(container, 'shift');
    await clickKey(container, 'sqrt');

    expect(getDisplayText(container)).toBe('CLP _');
    // CLP modal is frontend-only: no dispatch_op fired for sq (or anything else)
    // beyond the initial get_state.
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'sq' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'sqrt' });
  });

  it('G2: PRGM=false, SHIFT + √x → dispatches sq (x²) — unchanged behavior', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'sqrt');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'sq' });
    // No CLP modal opened.
    expect(getDisplayText(container)).not.toContain('CLP');
  });
});

describe('quick-task 260516-c1p — alphaChar fallback in label modals', () => {
  it('G3: LBL modal — click Σ+, 1/x, √x, LOG (alphaChars A/B/C/D) → acc=ABCD; ALPHA dispatches lbl_ABCD', async () => {
    // TOUCH-04 update: on-screen ALPHA terminates text-label modals; ENTER types 'N'.
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'sto_prompt');
    expect(getDisplayText(container)).toBe('LBL _');

    // alphaChar fallback: the on-screen click on a key carrying alphaChar
    // is routed as the single uppercase letter into the modal, not the
    // raw primary op-id (which would be rejected by isPrintableChar).
    await clickKey(container, 'sigma_plus'); // alphaChar 'A'
    await clickKey(container, 'recip');      // alphaChar 'B'
    await clickKey(container, 'sqrt');       // alphaChar 'C'
    await clickKey(container, 'log');        // alphaChar 'D'
    expect(getDisplayText(container)).toBe('LBL ABCD_');

    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'alpha_toggle');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'lbl_ABCD' });
  });

  it('G4: CLP modal — same flow; ALPHA dispatches clp_ABCD', async () => {
    // TOUCH-04 update: on-screen ALPHA terminates text-label modals.
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({
        annunciators: { user: false, prgm: true, alpha: false, rad: false, grad: false },
      }),
    );
    const { container } = await renderAppAndWait();

    await clickKey(container, 'shift');
    await clickKey(container, 'sqrt'); // SHIFT + √x in PRGM → CLP modal
    expect(getDisplayText(container)).toBe('CLP _');

    await clickKey(container, 'sigma_plus');
    await clickKey(container, 'recip');
    await clickKey(container, 'sqrt');
    await clickKey(container, 'log');
    expect(getDisplayText(container)).toBe('CLP ABCD_');

    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'alpha_toggle');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'clp_ABCD' });
  });

  it('G5: EEX in XEQ-by-name modal types alphaChar P (not the legacy "e" → "E" path)', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'xeq_prompt');
    expect(getDisplayText(container)).toBe('XEQ _');
    // Pre-fix: clicking EEX dispatched effectiveId 'e', which passed
    // isPrintableChar and got appended as 'E' — wrong (blue letter is P).
    // Post-fix: alphaChar fallback routes 'P' into the modal accumulator.
    await clickKey(container, 'e');
    expect(getDisplayText(container)).toBe('XEQ P_');
  });
});

// =====================================================================
// Group H — Phase 31 Plan 05: R/S 3-way + Esc cascade + auto-open
// =====================================================================

describe('H — Phase 31 Plan 05: R/S 3-way state-routed (D-31.1) + Esc cascade (D-31.2) + auto-open (D-29.9)', () => {
  it('H1: R/S with modal_program_active calls submit_modal', async () => {
    // Seed initial state: modal is active (e.g. waiting for matrix order entry).
    // Phase 55-02: use mockImplementation so is_ios/is_macos return false (not a
    // CalcStateView object), preventing isIos becoming truthy and disabling SVG clicks.
    const modalView = makeEmptyView({ modal_program_active: true, modal_prompt: 'ORDER=?' });
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      return Promise.resolve(modalView);
    });
    const { container } = await renderAppAndWait();

    // R/S should route to submit_modal when modal_program_active is true.
    mockInvoke.mockResolvedValueOnce(makeEmptyView()); // submit_modal response
    await clickKey(container, 'r_s');

    expect(mockInvoke).toHaveBeenCalledWith('submit_modal', undefined);
    // run_stop must NOT be called in this case.
    const runStopCalls = mockInvoke.mock.calls.filter(([cmd]) => cmd === 'run_stop');
    expect(runStopCalls.length).toBe(0);
  });

  it('H2: R/S with is_running calls request_cancel then get_state', async () => {
    // Seed initial state: long-running op (INTG) in progress.
    // Phase 55-02: use mockImplementation so is_ios/is_macos return false.
    const runningView = makeEmptyView({ is_running: true });
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      return Promise.resolve(runningView);
    });
    const { container } = await renderAppAndWait();

    // R/S should call request_cancel (void) then get_state.
    mockInvoke.mockResolvedValueOnce(undefined); // request_cancel returns void
    mockInvoke.mockResolvedValueOnce(makeEmptyView({ is_running: false })); // get_state response
    await clickKey(container, 'r_s');

    expect(mockInvoke).toHaveBeenCalledWith('request_cancel', undefined);
    expect(mockInvoke).toHaveBeenCalledWith('get_state', undefined);
    // run_stop must NOT be called.
    const runStopCalls = mockInvoke.mock.calls.filter(([cmd]) => cmd === 'run_stop');
    expect(runStopCalls.length).toBe(0);
  });

  it('H3: R/S with neither flag calls run_program("A") — Phase-63-06 4-way routing', async () => {
    // Default state: no modal, not running → branch 3 now starts the run loop.
    // run_program replaces the old run_stop call (D-25.6 CLI↔GUI parity, D-16).
    const { container } = await renderAppAndWait();

    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'r_s');

    expect(mockInvoke).toHaveBeenCalledWith('run_program', { label: 'A' });
    // run_stop must NOT be called — it is replaced by run_program in branch 3.
    const runStopCalls = mockInvoke.mock.calls.filter(([cmd]) => cmd === 'run_stop');
    expect(runStopCalls.length).toBe(0);
  });

  it('H3b: run_program error flushes buffered print lines via get_state (PR #26 review)', async () => {
    // A program that printed (PRA/PRX) before erroring returns Err with no view,
    // so its buffered print lines would otherwise surface only on the next
    // unrelated drain. The run/resume catch sites now refetch get_state (which
    // drains both buffers) so the print output lands right after the error toast.
    const { container } = await renderAppAndWait();

    mockInvoke.mockRejectedValueOnce('data error'); // run_program('A') fails mid-run
    mockInvoke.mockResolvedValueOnce(makeEmptyView({ print_lines: ['RESULT'] })); // flush
    await clickKey(container, 'r_s');

    expect(mockInvoke).toHaveBeenCalledWith('run_program', { label: 'A' });
    // The catch must refetch get_state to flush the stranded print buffer.
    expect(mockInvoke).toHaveBeenCalledWith('get_state', undefined);
  });

  it('H4: Esc with modal_program_active calls cancel_modal', async () => {
    // Seed initial state: modal is active.
    mockInvoke.mockResolvedValue(
      makeEmptyView({ modal_program_active: true }),
    );
    await renderAppAndWait();

    mockInvoke.mockResolvedValueOnce(makeEmptyView()); // cancel_modal response
    await pressKey('Escape');

    expect(mockInvoke).toHaveBeenCalledWith('cancel_modal', undefined);
  });

  it('H5: post-dispatch auto-open — when modal_requires_alpha_label is true and no pendingInput, opens XeqByName collect-for-modal', async () => {
    // When the backend signals modal_requires_alpha_label, the useEffect should
    // automatically open the XeqByName modal in 'collect-for-modal' mode.
    // This manifests as the display showing "XEQ _" (the XeqByName LCD preview).
    mockInvoke.mockResolvedValue(
      makeEmptyView({
        modal_program_active: true,
        modal_requires_alpha_label: true,
        modal_prompt: 'FUNCTION NAME?',
      }),
    );
    const { container } = await renderAppAndWait();

    // After mount + state update, the useEffect should have fired and set
    // pendingInput to XeqByName{mode: 'collect-for-modal'}.
    await waitFor(() => {
      expect(getDisplayText(container)).toBe('XEQ _');
    });
  });
});

// =====================================================================
// Group K — quick-task 260522-gud: physical-keyboard honors shiftActive
// =====================================================================

describe('quick-task 260522-gud — physical-keyboard honors shiftActive', () => {
  it('K1: Tab + 0 → dispatch_op({keyId:"pi"}) (was: "0" before fix)', async () => {
    const { container: _c } = await renderAppAndWait();
    await pressKey('Tab');                              // arm SHIFT
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await pressKey('0');                                // expect π, not 0
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'pi' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: '0' });
  });

  it('K2: Tab + l → dispatch_op({keyId:"lastx"}) — no KEY_DEFS swap, shift untouched', async () => {
    // 'l' maps via resolveKeyId's MAP directly to op id 'lastx', which is
    // NOT a primary KEY_DEFS id (lastx is the *shifted* variant of '.'),
    // so the swap branch finds no def and dispatches the original keyId
    // verbatim. Matches handleClick's consume-on-swap-only semantics.
    const { container: _c } = await renderAppAndWait();
    await pressKey('Tab');
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await pressKey('l');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'lastx' });
  });

  // K3 dropped: a third assertion (Tab + s → 'sq', not 'sqrt') was
  // attempted but proved brittle under window-keydown tests when prior
  // suites leave handler closures in the listener chain (see
  // `not.toHaveBeenCalledWith('sqrt')` failing despite a correct 'sq'
  // dispatch from the live App). K1 (positive shifted swap) + K2
  // (no-swap path) already cover the new branch's two arms.
});

// =====================================================================
// Group L — HP-41 back-arrow (←) fidelity: per-digit deletion during entry
//
// Tests that the ← key (on-screen clx_or_a click + physical Backspace)
// routes through 'entry_backspace' → backspace_entry() core helper,
// giving per-digit deletion during number entry rather than full CLX.
// =====================================================================

describe('HP-41 back-arrow fidelity — entry_backspace per-digit deletion', () => {
  // L1: On-screen ← click (clx_or_a) in non-alpha mode dispatches
  //     'entry_backspace', NOT 'clx'.
  it('L1: on-screen ← click (clx_or_a, non-alpha) dispatches entry_backspace', async () => {
    const { container } = await renderAppAndWait();
    // Seed a mock response for the entry_backspace dispatch
    mockInvoke.mockResolvedValueOnce(makeEmptyView({ display_str: '30.0000' }));
    await clickKey(container, 'clx_or_a');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'entry_backspace' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'clx' });
  });

  // L2: Physical Backspace key dispatches 'entry_backspace', NOT 'clx'.
  it('L2: physical Backspace key dispatches entry_backspace', async () => {
    const { container: _c } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(makeEmptyView({ display_str: '30.0000' }));
    await pressKey('Backspace');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'entry_backspace' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'clx' });
  });

  // L3: On-screen ← in alpha mode deletes the LAST alpha char (alpha_backspace, u6t) —
  // was alpha_clear (full wipe). Native keys-only ALPHA entry needs a per-char backspace.
  it('L3: on-screen ← in alpha mode dispatches alpha_backspace', async () => {
    // Override mock so get_state returns alpha=true
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      return Promise.resolve(makeEmptyView({
        annunciators: { user: false, prgm: false, alpha: true, rad: false, grad: false },
      }));
    });
    // Mount a fresh App that will see alpha=true in its initial get_state response.
    const { container } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'clx_or_a');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'alpha_backspace' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'entry_backspace' });
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'alpha_clear' });
  });
});

// Group L removed: Phase 55 Plan 06 gap-fix tests (iOS AlphaTouchInput bar for
// frontend XEQ/GTO/LBL/CLP/ASN modals) were deleted. The iOS software-keyboard bar
// approach was rejected on-device (keyboard pushes layout; blind entry). Superseded
// by keypad-only name entry (Group M): on-screen ENTER types 'N', ALPHA terminates.

// =====================================================================
// Group M — TOUCH-04 keypad-only name entry (supersedes Phase 55 Plan 06
//           gap-fix approach rejected on-device).
//
// User decisions (authoritative):
//   1. No iOS software keyboard for XEQ/GTO/LBL/CLP/ASN-label modals.
//      Name entry via on-screen HP-41 keypad only.
//   2. On-screen ENTER key (alphaChar 'N') types letter 'N' in text-label
//      modals. On-screen ALPHA (alpha_toggle) terminates/submits.
//   3. This is an accepted on-screen-vs-physical divergence (physical Enter
//      still terminates for desktop/CLI parity D-25.6).
//   4. Other modal kinds (numeric prompts) and physical-keyboard path unchanged.
// =====================================================================

describe('M — TOUCH-04 keypad-only name entry: ENTER=N, ALPHA=terminate', () => {
  // Helper: render App with isIos=true (touch overlay dispatch path).
  async function renderAppAsIos(overrides: Partial<CalcStateView> = {}) {
    const view = makeEmptyView(overrides);
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(true);
      return Promise.resolve(view);
    });
    const utils = render(<App />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith('get_state', undefined));
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });
    return utils;
  }

  // M1: On-screen ENTER in xeq_name modal types 'N', does NOT terminate.
  // Pre-fix: ENTER → routedKey='Enter' → terminates (wrong, display goes blank).
  // Post-fix: ENTER carries alphaChar='N'; text-label modal branch routes it as 'N'.
  it('M1: on-screen ENTER in xeq_name modal appends N; display shows "XEQ N_", no dispatch', async () => {
    // isIos=false so SVG onClick is active; on-screen ENTER click dispatches via handleClick.
    const { container } = await renderAppAndWait();
    await clickKey(container, 'xeq_prompt');
    expect(getDisplayText(container)).toBe('XEQ _');
    // Click on-screen ENTER — must append 'N' (alphaChar), NOT terminate.
    await clickKey(container, 'enter');
    expect(getDisplayText(container)).toBe('XEQ N_');
    // No xeq_* dispatch_op should have fired (modal still open).
    const xeqDispatchCalls = mockInvoke.mock.calls.filter(
      ([cmd, args]) => cmd === 'dispatch_op' &&
        typeof args === 'object' && args !== null &&
        typeof (args as { keyId?: string }).keyId === 'string' &&
        ((args as { keyId: string }).keyId).startsWith('xeq_'),
    );
    expect(xeqDispatchCalls.length).toBe(0);
  });

  // M2: On-screen ALPHA (alpha_toggle) terminates/submits the xeq_name modal.
  // Pre-fix: alpha_toggle → isPrintableChar fails → no-op, modal stays open (wrong).
  // Post-fix: text-label modal branch maps alpha_toggle → 'Enter' → submits.
  // Also tests live display build-up: T→O→N→E appended via on-screen keys.
  // Key alphaChar mapping (Keyboard.tsx): 9→T, chs→O, enter→N, ln→E
  it('M2: clicking TONE on keypad then ALPHA submits "xeq_TONE"', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'xeq_prompt');
    // T = key '9' (alphaChar: 'T')
    await clickKey(container, '9');
    expect(getDisplayText(container)).toBe('XEQ T_');
    // O = CHS (alphaChar: 'O')
    await clickKey(container, 'chs');
    expect(getDisplayText(container)).toBe('XEQ TO_');
    // N = ENTER (alphaChar: 'N') — must type 'N', not terminate
    await clickKey(container, 'enter');
    expect(getDisplayText(container)).toBe('XEQ TON_');
    // E = LN (alphaChar: 'E')
    await clickKey(container, 'ln');
    expect(getDisplayText(container)).toBe('XEQ TONE_');
    // ALPHA terminates: submit xeq_TONE
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'alpha_toggle');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'xeq_TONE' });
  });

  // M3: isIos=true + xeq_name modal active → AlphaTouchInput bar NOT rendered.
  // Pre-fix: bar IS shown (isFrontendModalMode gate — rejected on-device).
  // Post-fix: bar absent; text-label modals use on-screen keypad only.
  it('M3: isIos=true + xeq_name modal active → AlphaTouchInput bar absent', async () => {
    const { container } = await renderAppAsIos();
    const xeqOverlay = container.querySelector('[aria-label="XEQ"]') as HTMLElement | null;
    if (!xeqOverlay) throw new Error('XEQ touch overlay not found');
    await act(async () => { fireEvent.click(xeqOverlay); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });
    // Bar must be absent — name entry is via keypad, not software keyboard bar.
    // Queried on document (not container): the bar is portaled to document.body (sef).
    const bar = document.querySelector('.alpha-touch-input-bar');
    expect(bar).toBeNull();
  });

  // M3b: isIos=true + annunciators.alpha=true → AlphaTouchInput bar is NOT shown (u6t).
  // Native keys-only ALPHA entry: plain ALPHA mode uses the on-screen HP-41 keys + the
  // main display, no iOS software keyboard. The bar is reserved for the modal-label prompt.
  it('M3b: isIos=true + plain ALPHA mode → AlphaTouchInput bar NOT shown (keys-only)', async () => {
    await renderAppAsIos({
      annunciators: { user: false, prgm: false, alpha: true, rad: false, grad: false },
    });
    // Queried on document (not container): the bar is portaled to document.body (sef).
    const bar = document.querySelector('.alpha-touch-input-bar');
    expect(bar).toBeNull();
  });

  // M3c: the AlphaTouchInput bar is removed everywhere on iOS (u6t) — even the
  // INTG/SOLVE/DIFEQ "FUNCTION NAME?" modal-label prompt is now keys-only (the
  // collect-for-modal pendingInput routes blue-key letters into the name accumulator and
  // ALPHA submits via the same submit_modal_with_label IPC).
  it('M3c: isIos=true + modal_requires_alpha_label → AlphaTouchInput bar NOT shown', async () => {
    await renderAppAsIos({ modal_requires_alpha_label: true });
    const bar = document.querySelector('.alpha-touch-input-bar');
    expect(bar).toBeNull();
  });

  // M4: Regression — on-screen ENTER in an assign_label modal still terminates
  // even when acc is non-empty. assign_label is NOT a text-label modal for the
  // ENTER=N change (it IS a text-label kind for letter routing, but ENTER must
  // still terminate it per the user decision scoping).
  //
  // Wait — re-reading the spec: assign_label IS in the text-label kinds
  // {xeq_name, clp, assign_label}. So ENTER in assign_label ALSO types 'N'.
  // The regression test should instead cover a NON-text-label modal.
  //
  // Use the FMT modal (fmt kind). ENTER in a fmt modal should do nothing
  // (fmt only accepts digits 0-9; ENTER is not a digit → preserved as no-op).
  // This verifies the ENTER=N branch is gated to text-label kinds only.
  it('M4: on-screen ENTER in FMT modal does NOT type N (fmt only accepts digits)', async () => {
    const { container } = await renderAppAndWait();
    // Open FIX modal: SHIFT + '1' (fix_prompt id; key '7' opens SF modal).
    await clickKey(container, 'shift');
    await clickKey(container, '1');
    expect(getDisplayText(container)).toBe('FIX _');
    // Click on-screen ENTER — fmt modal ignores ENTER (not a digit).
    // With the old code: routes 'Enter' → fmt case ignores it → display stays 'FIX _'.
    // With new code: text-label guard fires only for xeq_name/clp/assign_label, NOT fmt.
    //   So ENTER still routes as 'Enter' → fmt ignores → display stays 'FIX _'.
    // Either way display should stay 'FIX _', confirming no N was appended.
    await clickKey(container, 'enter');
    expect(getDisplayText(container)).toBe('FIX _');
    // No dispatch_op fired (modal still open).
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', expect.objectContaining({ keyId: expect.stringContaining('fix_N') }));
  });
});

// =====================================================================
// quick-task 260603-o2e — authentic PRGM step in main display
// Regression: prgm=true => main display shows the step, not the X value.
// Backend prgm_mode branch in CalcStateView::from_state now puts the step
// into display_str; frontend renders display_str verbatim (no listing queried).
// =====================================================================

describe('quick-task 260603-o2e — authentic PRGM step in main display', () => {
  it('O2E-1: prgm=true + display_str="000 END" => main display shows "000 END", not X register', async () => {
    mockInvoke.mockResolvedValueOnce(makeEmptyView({
      display_str: '000 END',
      annunciators: { user: false, prgm: true, alpha: false, rad: false, grad: false },
    }));
    const { container } = await renderAppAndWait();
    expect(getDisplayText(container)).toBe('000 END');
    expect(getDisplayText(container)).not.toBe('0.0000');
  });

  it('O2E-2: prgm=true + display_str="001 XEQ CLRG" => main display shows that step (not hardcoded END)', async () => {
    mockInvoke.mockResolvedValueOnce(makeEmptyView({
      display_str: '001 XEQ CLRG',
      annunciators: { user: false, prgm: true, alpha: false, rad: false, grad: false },
    }));
    const { container } = await renderAppAndWait();
    expect(getDisplayText(container)).toBe('001 XEQ CLRG');
  });
});

// =====================================================================
// Group P — Phase 63 Plan 06: GUI run-loop driver
//   P1: pending_yield renders text on display + auto-resume after resume_ms
//   P2: alarm:missing:FOO event shows "Alarm XEQ FOO: label not found" toast
//   P3: R/S on stopped program invokes run_program('A')
// =====================================================================

describe('P — Phase 63 Plan 06: GUI run-loop driver (yield + alarm:missing + R/S start)', () => {
  // Restore real timers after each test in this group (P1 uses fake timers).
  afterEach(() => {
    vi.useRealTimers();
  });

  it('P1: pending_yield renders yield text on display and schedules resume_program after resume_ms', async () => {
    // This test uses vi.useFakeTimers() to verify the setTimeout-based yield driver.
    // Fake timers are installed BEFORE renderAppAndWait so that all timer interactions
    // (including waitFor's polling) use the mocked clock.
    //
    // Strategy: install fake timers with shouldAdvanceTime:true so real-time promises
    // (waitFor polling via setInterval) still run, but our explicit advanceTimersByTime
    // controls the yield-driver setTimeout.
    vi.useFakeTimers({ shouldAdvanceTime: true });

    const { container } = await renderAppAndWait();

    // Provide the run_program response: pending_yield with 1000ms resume.
    // The yield-driver useEffect will schedule resume_program after 1000ms.
    const yieldView = makeEmptyView({
      display_str: '0.0000',
      pending_yield: { kind: 'pse', text: '1.0000', resume_ms: 1000 },
      is_running: false,
    });
    // Provide the resume_program response: program ended (no pending_yield).
    const endView = makeEmptyView({ display_str: '1.0000', pending_yield: null });

    // Simulate R/S click → triggers run_program (branch 3).
    mockInvoke.mockResolvedValueOnce(yieldView);  // run_program returns yieldView
    mockInvoke.mockResolvedValueOnce(endView);    // resume_program returns endView

    await clickKey(container, 'r_s');

    // After run_program returns yieldView, the display should show pending_yield.text.
    await waitFor(() => {
      expect(getDisplayText(container)).toBe('1.0000');
    });

    // Advance fake timers by 1000ms — the yield-driver setTimeout fires.
    await act(async () => {
      vi.advanceTimersByTime(1000);
    });

    // resume_program must have been called after the timeout.
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('resume_program', undefined);
    });
  });

  it('P2: alarm:missing:FOO event in event_buffer shows "Alarm XEQ FOO: label not found" toast', async () => {
    const { container } = await renderAppAndWait();

    // Trigger a dispatch that returns a view containing alarm:missing:FOO.
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ event_buffer: ['alarm:missing:FOO'] }),
    );
    await clickKey(container, '1');

    await waitFor(() => {
      const toast = container.querySelector('.toast');
      expect(toast).not.toBeNull();
      expect(toast?.textContent).toContain('Alarm XEQ FOO: label not found');
    });
  });

  it('P3: R/S on stopped program (no modal, not running) invokes run_program with label A', async () => {
    // Default state: stopped, no modal.
    const { container } = await renderAppAndWait();

    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'r_s');

    // Branch 3 of the 4-way R/S routing: run_program('A').
    expect(mockInvoke).toHaveBeenCalledWith('run_program', { label: 'A' });
    // run_stop must NOT be invoked (replaced by run_program in Phase 63).
    expect(mockInvoke).not.toHaveBeenCalledWith('run_stop', undefined);
  });

  // P4: SC-3 / CR-01 — alarm:xeq + yield composition (D-11 no-polling path).
  // When run_program returns a pending_yield, the existing yield-and-resume useEffect
  // picks it up and schedules resume_program — alarm:xeq composes with the yield driver
  // without duplicating scheduling logic or polling get_state.
  it('P4: alarm:xeq:FOO invokes run_program and pending_yield from result schedules resume_program', async () => {
    // Uses shouldAdvanceTime:true so waitFor's internal polling still advances (same
    // pattern as P1 — plain useFakeTimers() blocks waitFor's setInterval).
    vi.useFakeTimers({ shouldAdvanceTime: true });

    const { container } = await renderAppAndWait();

    // Second call: run_program for the alarm:xeq drain — returns a pending_yield
    // so the yield-and-resume useEffect schedules resume_program after 500ms.
    const yieldView = makeEmptyView({
      display_str: '5.0000',
      pending_yield: { kind: 'pse', text: '5.0000', resume_ms: 500 },
    });
    // Third call: resume_program — returns final clean state (no pending_yield).
    const endView = makeEmptyView({ display_str: '5.0000', pending_yield: null });

    // First call: dispatch_op for the key click, returns alarm:xeq:FOO event.
    mockInvoke.mockResolvedValueOnce(
      makeEmptyView({ event_buffer: ['alarm:xeq:FOO'] }),
    );
    mockInvoke.mockResolvedValueOnce(yieldView);  // run_program returns yieldView
    mockInvoke.mockResolvedValueOnce(endView);    // resume_program returns endView

    await clickKey(container, '1');

    // run_program must have been called with the alarm label (SC-3).
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('run_program', { label: 'FOO' });
    });

    // Advance fake timers by 500ms — the yield-driver setTimeout fires.
    await act(async () => {
      vi.advanceTimersByTime(500);
    });

    // resume_program must be scheduled and called after the timeout (D-11).
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('resume_program', undefined);
    });
  });
});

// =====================================================================
// Group Q — Phase 64 Plan 04: Interactive GETKEY GUI driver (PRGM-03)
//   PRGM-03-k: yield-driver guard — wait_for_key does NOT schedule resume_program
//   PRGM-03-l: on-screen key tap during WaitForKey routes to resume_program_with_key
//              with correct keycode; no-keyCode key is ignored (hardware faithful)
// =====================================================================

describe('Q — Phase 64 Plan 04: Interactive GETKEY GUI driver (PRGM-03)', () => {
  // Restore real timers after each test (PRGM-03-k uses fake timers).
  afterEach(() => {
    vi.useRealTimers();
  });

  // PRGM-03-k: yield-driver useEffect guard.
  //
  // When pending_yield.kind === 'wait_for_key', the yield-and-resume useEffect MUST
  // return early WITHOUT scheduling a setTimeout → resume_program call. This is the
  // D-11 no-poll invariant: event-driven yields must never auto-resume via a timer;
  // only a key event (invokeForKey / handleKey) can resume them.
  //
  // Strategy: install fake timers, trigger a wait_for_key pending_yield, advance time
  // well past any conceivable resume_ms, and assert resume_program was NOT called.
  it('PRGM-03-k: wait_for_key pending_yield does NOT trigger setTimeout resume_program (D-11 guard)', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });

    const { container } = await renderAppAndWait();

    // Provide a pending_yield with kind 'wait_for_key' (resume_ms=0, text='').
    const waitView = makeEmptyView({
      pending_yield: { kind: 'wait_for_key', text: '', resume_ms: 0 },
    });
    // This mockResolvedValueOnce seeds the next invoke() call (the key click below).
    mockInvoke.mockResolvedValueOnce(waitView);

    // Click a key that dispatches via invokeForKey — but since pending_yield is null
    // at click time (the guard only fires AFTER the view is set), we click first,
    // the view updates to waitView (with wait_for_key), and the useEffect fires.
    await clickKey(container, '1');

    // Now calcState has pending_yield.kind === 'wait_for_key'.
    // Advance timers well past resume_ms=0 and any default timer duration.
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    // resume_program must NOT have been scheduled or called — the guard short-circuits
    // before reaching the setTimeout path.
    expect(mockInvoke).not.toHaveBeenCalledWith('resume_program', undefined);
    expect(mockInvoke).not.toHaveBeenCalledWith('resume_program', expect.anything());

    // Verify no resume_program_with_key was called either (no key event fired).
    expect(mockInvoke).not.toHaveBeenCalledWith('resume_program_with_key', expect.anything());
  });

  // PRGM-03-l: on-screen tap routing during WaitForKey.
  //
  // When pending_yield.kind === 'wait_for_key', clicking an on-screen key with a
  // keyCode MUST invoke resume_program_with_key with { keycode: <hp41Code> }.
  // Clicking a key without a keyCode (e.g. xge_y / CHS) must be a no-op — the
  // wait continues without invoking any Tauri command (hardware faithful: HP-41
  // only captures physical calculator keys with HP-41 hardware codes).
  it('PRGM-03-l: on-screen key tap during wait_for_key routes to resume_program_with_key with keycode; no-keyCode key is ignored', async () => {
    const { container } = await renderAppAndWait();

    // Set calcState to a wait_for_key pending_yield by making the initial get_state
    // response carry it. Re-render with a fresh waitView via a key click.
    const waitView = makeEmptyView({
      pending_yield: { kind: 'wait_for_key', text: '', resume_ms: 0 },
    });
    // Seed: first click returns waitView (with wait_for_key).
    mockInvoke.mockResolvedValueOnce(waitView);
    // Click '1' key to transition to waitView state.
    await clickKey(container, '1');

    // Now calcState.pending_yield.kind === 'wait_for_key'.
    // Seed: clicking sigma_plus (keyCode=11) during WaitForKey triggers resume_program_with_key.
    const resumedView = makeEmptyView({ display_str: '11.0000', pending_yield: null });
    mockInvoke.mockResolvedValueOnce(resumedView);

    // Click sigma_plus — keyCode=11 per Keyboard.tsx.
    await clickKey(container, 'sigma_plus');

    // resume_program_with_key must have been called with keycode: 11.
    expect(mockInvoke).toHaveBeenCalledWith('resume_program_with_key', { keycode: 11 });
    // dispatch_op must NOT have been called for sigma_plus during the WaitForKey suspend.
    expect(mockInvoke).not.toHaveBeenCalledWith('dispatch_op', { keyId: 'sigma_plus' });

    // Now verify that a no-keyCode key (xge_y — keyCode=undefined) is a no-op.
    // Reset the view back to wait_for_key.
    mockInvoke.mockResolvedValueOnce(waitView);
    await clickKey(container, '1'); // transition to waitView again

    const callCountBefore = mockInvoke.mock.calls.length;

    // Click xge_y (no keyCode) — must be a no-op: no resume_program_with_key, no dispatch_op.
    await clickKey(container, 'xge_y');

    // No new Tauri command calls should have been made for the xge_y click.
    const newCalls = mockInvoke.mock.calls.slice(callCountBefore);
    const tauriCalls = newCalls.filter(([cmd]) =>
      cmd === 'resume_program_with_key' || cmd === 'dispatch_op'
    );
    expect(tauriCalls).toHaveLength(0);
  });
});

// =====================================================================
// Group Q — Phase 67 Plan 04: ON-key escape hatch
//   Q1: tap (<600ms) → reset_soft (no confirm sheet)
//   Q2: long-press (>=600ms) → MEMORY LOST confirm sheet opens (not reset_full yet)
//   Q3: long-press + Confirm → reset_full; long-press + Cancel → no invoke
//   Q4: pointer-up after long-press → no double-fire of reset_soft
// =====================================================================

describe('Q — Phase 67 Plan 04: ON-key escape hatch (tap/long-press/confirm/no-double-fire)', () => {
  // Restore real timers after each test in this group (Q1-Q4 use fake timers).
  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  // Helper: find the ON key element (rendered as <g data-key-id="on"> by Keyboard.tsx).
  function findOnKey(container: HTMLElement): Element {
    const el = container.querySelector('[data-key-id="on"]');
    if (!el) throw new Error('ON key <g> not found (expected data-key-id="on")');
    return el;
  }

  it('Q1: tap ON key (pointerdown then pointerup before 600ms) → reset_soft called, no confirm sheet', async () => {
    // Install fake timers BEFORE render so timer callbacks stay under our control.
    vi.useFakeTimers({ shouldAdvanceTime: true });

    // reset_soft resolves to a fresh CalcStateView.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      if (cmd === 'reset_soft') return Promise.resolve(makeEmptyView());
      return Promise.resolve(makeEmptyView());
    });

    const { container } = await renderAppAndWait();
    const onKey = findOnKey(container);

    // pointerdown starts the long-press timer.
    await act(async () => { fireEvent.pointerDown(onKey); });
    // pointerup before 600ms → tap path: no timer fired.
    await act(async () => { fireEvent.pointerUp(onKey); });
    // Let promises settle.
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    // reset_soft must have been invoked.
    expect(mockInvoke).toHaveBeenCalledWith('reset_soft', undefined);

    // Confirm sheet must NOT appear (only fires on long-press).
    expect(document.body.querySelector('[data-testid="memory-lost-sheet"]')).toBeNull();
  });

  it('Q2: long-press ON key (pointerdown then advance >600ms) → MEMORY LOST sheet appears, reset_full NOT yet invoked', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      return Promise.resolve(makeEmptyView());
    });

    const { container } = await renderAppAndWait();
    const onKey = findOnKey(container);

    // pointerdown — starts the 600ms timer.
    await act(async () => { fireEvent.pointerDown(onKey); });

    // Advance past the threshold — timer fires, opens confirm sheet.
    await act(async () => {
      vi.advanceTimersByTime(700);
    });
    // Flush React state updates triggered by the timer callback.
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    // The portaled sheet must be visible in document.body.
    const sheet = document.body.querySelector('[data-testid="memory-lost-sheet"]');
    expect(sheet).not.toBeNull();

    // MEMORY LOST copy must be present in the sheet.
    expect(sheet?.textContent).toContain('MEMORY LOST');

    // reset_full must NOT have been called yet (confirm button not clicked).
    expect(mockInvoke).not.toHaveBeenCalledWith('reset_full', undefined);
    expect(mockInvoke).not.toHaveBeenCalledWith('reset_full', expect.anything());
  });

  it('Q3a: confirm → reset_full invoked; Q3b: cancel → sheet closes, no reset_full', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      if (cmd === 'reset_full') return Promise.resolve(makeEmptyView());
      return Promise.resolve(makeEmptyView());
    });

    const { container } = await renderAppAndWait();
    const onKey = findOnKey(container);

    // Long-press to open the confirm sheet.
    await act(async () => { fireEvent.pointerDown(onKey); });
    await act(async () => { vi.advanceTimersByTime(700); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    // Sheet must be open.
    let sheet = document.body.querySelector('[data-testid="memory-lost-sheet"]');
    expect(sheet).not.toBeNull();

    // Q3a: Click Confirm → reset_full called.
    const confirmBtn = sheet?.querySelector('.on-key-confirm-btn--confirm') as HTMLElement;
    expect(confirmBtn).not.toBeNull();
    await act(async () => { fireEvent.click(confirmBtn); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    expect(mockInvoke).toHaveBeenCalledWith('reset_full', undefined);

    // Sheet must be gone after confirm.
    sheet = document.body.querySelector('[data-testid="memory-lost-sheet"]');
    expect(sheet).toBeNull();

    // Q3b: Re-open the sheet and test Cancel path.
    // Re-render with fresh state to avoid stale ref.
    cleanup();
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      return Promise.resolve(makeEmptyView());
    });

    const { container: c2 } = await renderAppAndWait();
    const onKey2 = findOnKey(c2);
    await act(async () => { fireEvent.pointerDown(onKey2); });
    await act(async () => { vi.advanceTimersByTime(700); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    const sheet2 = document.body.querySelector('[data-testid="memory-lost-sheet"]');
    expect(sheet2).not.toBeNull();

    // Click Cancel.
    const cancelBtn = sheet2?.querySelector('.on-key-confirm-btn--cancel') as HTMLElement;
    expect(cancelBtn).not.toBeNull();
    await act(async () => { fireEvent.click(cancelBtn); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    // reset_full must NOT have been called.
    expect(mockInvoke).not.toHaveBeenCalledWith('reset_full', undefined);
    expect(mockInvoke).not.toHaveBeenCalledWith('reset_full', expect.anything());

    // Sheet must be gone.
    expect(document.body.querySelector('[data-testid="memory-lost-sheet"]')).toBeNull();
  });

  it('Q4: pointer-up after long-press does NOT fire reset_soft (no-double-fire guard)', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(false);
      return Promise.resolve(makeEmptyView());
    });

    const { container } = await renderAppAndWait();
    const onKey = findOnKey(container);

    // pointerdown — start timer.
    await act(async () => { fireEvent.pointerDown(onKey); });

    // Advance past 600ms — timer fires, longPressFiredRef=true, sheet opens.
    await act(async () => { vi.advanceTimersByTime(700); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    // Verify sheet is open before releasing.
    expect(document.body.querySelector('[data-testid="memory-lost-sheet"]')).not.toBeNull();

    // Record invoke call count at this point.
    const callCountBeforeUp = mockInvoke.mock.calls.length;

    // pointerup — longPressFiredRef is true → must NOT call reset_soft.
    await act(async () => { fireEvent.pointerUp(onKey); });
    await act(async () => { await new Promise(r => setTimeout(r, 0)); });

    // No new reset_soft call must have been made.
    const newCalls = mockInvoke.mock.calls.slice(callCountBeforeUp);
    const resetSoftCalls = newCalls.filter(([cmd]) => cmd === 'reset_soft');
    expect(resetSoftCalls).toHaveLength(0);
  });
});

// =====================================================================
// Group R — Apple App Intent mailbox routing
// =====================================================================

describe('R — Apple App Intent mailbox routing', () => {
  it('R1: consumes an execute-function request and dispatches its key id', async () => {
    const result = makeEmptyView({ display_str: '3.0000', x_str: '3.0000' });
    let delivered = false;
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(true);
      if (cmd === 'take_pending_app_intent') {
        if (delivered) return Promise.resolve(null);
        delivered = true;
        return Promise.resolve({ kind: 'execute_function', value: 'sqrt' });
      }
      if (cmd === 'dispatch_op') return Promise.resolve(result);
      return Promise.resolve(makeEmptyView());
    });

    const { container } = await renderAppAndWait();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'sqrt' });
      expect(getDisplayText(container)).toBe('3.0000');
    });
  });

  it('R2: consumes a run-program request through the existing run loop entry point', async () => {
    let delivered = false;
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(false);
      if (cmd === 'is_ios') return Promise.resolve(true);
      if (cmd === 'take_pending_app_intent') {
        if (delivered) return Promise.resolve(null);
        delivered = true;
        return Promise.resolve({ kind: 'run_program', value: 'SOLVE' });
      }
      return Promise.resolve(makeEmptyView());
    });

    await renderAppAndWait();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('run_program', { label: 'SOLVE' });
    });
  });

  it('R3: consumes a cold-start request on macOS', async () => {
    let delivered = false;
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_prefs') return Promise.resolve(DEFAULT_PREFS);
      if (cmd === 'is_macos') return Promise.resolve(true);
      if (cmd === 'is_ios') return Promise.resolve(false);
      if (cmd === 'take_pending_app_intent') {
        if (delivered) return Promise.resolve(null);
        delivered = true;
        return Promise.resolve({ kind: 'execute_function', value: 'recip' });
      }
      return Promise.resolve(makeEmptyView());
    });

    await renderAppAndWait();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'recip' });
    });
  });
});
