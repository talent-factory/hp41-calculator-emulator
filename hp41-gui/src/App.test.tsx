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

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/react';
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

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(makeEmptyView());
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

  it('A2: ASN + SIN + type "TEST" + ENTER dispatches asn_25_TEST (canonical, not 23)', async () => {
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
    await clickKey(container, 'enter');
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
  it('C1: assign_label modal with acc=TEST → click ENTER dispatches asn_25_TEST', async () => {
    const { container } = await renderAppAndWait();
    await clickKey(container, 'shift');
    await clickKey(container, 'xeq_prompt');
    await clickKey(container, 'sin');
    for (const ch of ['T', 'E', 'S', 'T']) {
      await pressKey(ch);
    }
    expect(getDisplayText(container)).toBe('ASN TEST_');
    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'enter');
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
    await clickKey(container, 'enter');
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
  it('G3: LBL modal — click Σ+, 1/x, √x, LOG (alphaChars A/B/C/D) → acc=ABCD; ENTER dispatches lbl_ABCD', async () => {
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
    await clickKey(container, 'enter');
    expect(mockInvoke).toHaveBeenCalledWith('dispatch_op', { keyId: 'lbl_ABCD' });
  });

  it('G4: CLP modal — same flow; ENTER dispatches clp_ABCD', async () => {
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
    await clickKey(container, 'enter');
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
    mockInvoke.mockResolvedValue(
      makeEmptyView({ modal_program_active: true, modal_prompt: 'ORDER=?' }),
    );
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
    mockInvoke.mockResolvedValue(
      makeEmptyView({ is_running: true }),
    );
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

  it('H3: R/S with neither flag calls run_stop (existing baseline)', async () => {
    // Default state: no modal, not running.
    const { container } = await renderAppAndWait();

    mockInvoke.mockResolvedValueOnce(makeEmptyView());
    await clickKey(container, 'r_s');

    expect(mockInvoke).toHaveBeenCalledWith('run_stop', undefined);
  });

  it('H4: Esc with modal_program_active calls cancel_modal', async () => {
    // Seed initial state: modal is active.
    mockInvoke.mockResolvedValue(
      makeEmptyView({ modal_program_active: true }),
    );
    const { container } = await renderAppAndWait();

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
