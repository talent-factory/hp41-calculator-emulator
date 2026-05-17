// Phase 26 D-26.1..D-26.5 — frontend modal state machine tests.
//
// Targets the pure functions `handleModalKey` and `renderModalLcd` extracted
// from App.tsx for testability. No React rendering, no Tauri runtime — just
// (input PendingInput, key, shiftActive) → (next PendingInput, dispatchId,
// consumesShift) transitions and LCD preview strings.

import { describe, it, expect } from 'vitest';
import {
  handleModalKey,
  renderModalLcd,
  makeKeyCodeMagic,
  type PendingInput,
} from './pending_input';
import {
  KEY_DEFS_PRIMARY_IDS,
  KEY_DEFS_SHIFTED_IDS,
} from './key_defs_ids';

describe('handleModalKey — register modal (D-26.2 / D-26.4)', () => {
  it('accumulates two digits then dispatches sto_05', () => {
    let pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: false,
      acc: '',
    };
    let r = handleModalKey('0', pending, false);
    expect(r.nextPending).toEqual({ ...pending, acc: '0' });
    expect(r.dispatchId).toBeNull();
    expect(r.consumesShift).toBe(false);
    pending = r.nextPending!;
    r = handleModalKey('5', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('sto_05');
    expect(r.consumesShift).toBe(false);
  });

  it('with ind=true dispatches sto_ind_NN', () => {
    const pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: true,
      acc: '0',
    };
    const r = handleModalKey('7', pending, false);
    expect(r.dispatchId).toBe('sto_ind_07');
  });

  it('IND-toggle via shift-0 sets ind=true AND consumesShift=true', () => {
    const pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: false,
      acc: '',
    };
    const r = handleModalKey('0', pending, true);
    expect(r.consumesShift).toBe(true);
    expect(r.nextPending).toEqual({ ...pending, ind: true });
    expect(r.dispatchId).toBeNull();
  });

  it('IND-toggle does NOT append 0 to acc (W2 verification)', () => {
    const pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: false,
      acc: '5',
    };
    const r = handleModalKey('0', pending, true);
    // acc preserved verbatim; only ind toggled.
    expect(r.nextPending).toEqual({ ...pending, ind: true });
    expect(
      (r.nextPending as { acc: string }).acc,
      'IND-toggle keystroke must NOT append "0" to the accumulator',
    ).toBe('5');
  });

  it('digit when not shifted normally appends', () => {
    const pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: false,
      acc: '',
    };
    const r = handleModalKey('0', pending, false);
    expect((r.nextPending as { acc: string }).acc).toBe('0');
  });

  it('ignores non-digit non-shift-0 keys', () => {
    const pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: false,
      acc: '',
    };
    const r = handleModalKey('a', pending, false);
    expect(r.nextPending).toEqual(pending);
    expect(r.dispatchId).toBeNull();
  });
});

describe('handleModalKey — flag modal', () => {
  it('SF prompt: digit accumulates, dispatches sf_12', () => {
    let pending: PendingInput = {
      kind: 'flag',
      testKind: 'SF',
      ind: false,
      acc: '',
    };
    pending = handleModalKey('1', pending, false).nextPending!;
    const r = handleModalKey('2', pending, false);
    expect(r.dispatchId).toBe('sf_12');
  });

  it('IND-toggle via shift-0 sets ind=true (W2)', () => {
    const pending: PendingInput = {
      kind: 'flag',
      testKind: 'SF',
      ind: false,
      acc: '',
    };
    const r = handleModalKey('0', pending, true);
    expect(r.consumesShift).toBe(true);
    expect(r.nextPending).toEqual({ ...pending, ind: true });
  });

  it('FsQueryClear dispatches fs_c_NN', () => {
    let pending: PendingInput = {
      kind: 'flag',
      testKind: 'FsQueryClear',
      ind: false,
      acc: '',
    };
    pending = handleModalKey('0', pending, false).nextPending!;
    const r = handleModalKey('5', pending, false);
    expect(r.dispatchId).toBe('fs_c_05');
  });
});

describe('handleModalKey — clp modal (label text input)', () => {
  it('accumulates label and dispatches on Enter', () => {
    let pending: PendingInput = { kind: 'clp', acc: '' };
    pending = handleModalKey('M', pending, false).nextPending!;
    pending = handleModalKey('Y', pending, false).nextPending!;
    pending = handleModalKey('P', pending, false).nextPending!;
    pending = handleModalKey('R', pending, false).nextPending!;
    pending = handleModalKey('G', pending, false).nextPending!;
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('clp_MYPRG');
  });

  it('Backspace pops a character', () => {
    const pending: PendingInput = { kind: 'clp', acc: 'TEST' };
    const r = handleModalKey('Backspace', pending, false);
    expect((r.nextPending as { acc: string }).acc).toBe('TES');
  });
});

describe('handleModalKey — del modal (BLOCKER B3)', () => {
  it('accumulates 3 digits then dispatches del_010', () => {
    let pending: PendingInput = { kind: 'del', acc: '' };
    pending = handleModalKey('0', pending, false).nextPending!;
    pending = handleModalKey('1', pending, false).nextPending!;
    const r = handleModalKey('0', pending, false);
    expect(r.dispatchId).toBe('del_010');
  });

  it('dispatches del_256 — backend produces GuiError (B3 backend side)', () => {
    let pending: PendingInput = { kind: 'del', acc: '' };
    pending = handleModalKey('2', pending, false).nextPending!;
    pending = handleModalKey('5', pending, false).nextPending!;
    const r = handleModalKey('6', pending, false);
    // Frontend dispatches; key_map.rs::resolve produces the GuiError.
    expect(r.dispatchId).toBe('del_256');
  });

  it('renderModalLcd emits "DEL ERR" for acc="256" (B3 frontend preview)', () => {
    const pending: PendingInput = { kind: 'del', acc: '256' };
    expect(renderModalLcd(pending)).toBe('DEL ERR');
  });

  it('renderModalLcd emits "DEL ERR" for acc="999"', () => {
    const pending: PendingInput = { kind: 'del', acc: '999' };
    expect(renderModalLcd(pending)).toBe('DEL ERR');
  });

  it('renderModalLcd emits "DEL 255" for acc="255" (boundary)', () => {
    const pending: PendingInput = { kind: 'del', acc: '255' };
    expect(renderModalLcd(pending)).toBe('DEL 255');
  });
});

describe('handleModalKey — single_digit modal (BLOCKER B2)', () => {
  it('Tone dispatches on first 0-9 keystroke', () => {
    const pending: PendingInput = { kind: 'single_digit', op: 'Tone', max: 9 };
    const r = handleModalKey('5', pending, false);
    expect(r.dispatchId).toBe('tone_5');
  });

  it('Catalog accepts 1..=3', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 3,
    };
    const r = handleModalKey('2', pending, false);
    expect(r.dispatchId).toBe('catalog_2');
  });

  it('Catalog rejects digits > 3 (B2 max enforcement)', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 3,
    };
    const r = handleModalKey('5', pending, false);
    expect(r.nextPending).toEqual(pending);
    expect(r.dispatchId).toBeNull();
  });
});

describe('handleModalKey — single_digit Catalog bounds (CR-05)', () => {
  // Phase 26 Plan 04 CR-05 — frontend Catalog modal must mirror
  // hp41-core's op_catalog acceptance (n in 1..=4). Without the
  // op-specific lower-bound guard the modal accepted '0' (which the
  // backend rejects with InvalidOp) and the App.tsx MODAL_OPENERS.catalog
  // cap of max:3 prevented '4' (XFNS catalog) from ever being reachable.

  it('Catalog accepts digit 1 → dispatches catalog_1', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 4,
    };
    const r = handleModalKey('1', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('catalog_1');
    expect(r.consumesShift).toBe(false);
  });

  it('Catalog accepts digit 4 → dispatches catalog_4', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 4,
    };
    const r = handleModalKey('4', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('catalog_4');
    expect(r.consumesShift).toBe(false);
  });

  it('Catalog REJECTS digit 0 (lower-bound guard, CR-05)', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 4,
    };
    const r = handleModalKey('0', pending, false);
    expect(r.nextPending).toEqual(pending);
    expect(r.dispatchId).toBeNull();
    expect(r.consumesShift).toBe(false);
  });

  it('Catalog REJECTS digit 5 (upper-bound guard)', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 4,
    };
    const r = handleModalKey('5', pending, false);
    expect(r.nextPending).toEqual(pending);
    expect(r.dispatchId).toBeNull();
    expect(r.consumesShift).toBe(false);
  });

  it('Tone accepts digit 0 → dispatches tone_0 (asymmetric lower bound)', () => {
    const pending: PendingInput = { kind: 'single_digit', op: 'Tone', max: 9 };
    const r = handleModalKey('0', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('tone_0');
  });

  it('Tone accepts digit 9 → dispatches tone_9', () => {
    const pending: PendingInput = { kind: 'single_digit', op: 'Tone', max: 9 };
    const r = handleModalKey('9', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('tone_9');
  });

  it('Catalog non-digit keys leave modal unchanged', () => {
    const pending: PendingInput = {
      kind: 'single_digit',
      op: 'Catalog',
      max: 4,
    };
    for (const key of ['Enter', 'a', 'Backspace']) {
      const r = handleModalKey(key, pending, false);
      expect(r.nextPending).toEqual(pending);
      expect(r.dispatchId).toBeNull();
    }
  });

  it('Tone non-digit keys leave modal unchanged', () => {
    const pending: PendingInput = { kind: 'single_digit', op: 'Tone', max: 9 };
    for (const key of ['Enter', 'a', 'Backspace']) {
      const r = handleModalKey(key, pending, false);
      expect(r.nextPending).toEqual(pending);
      expect(r.dispatchId).toBeNull();
    }
  });
});

describe('handleModalKey — fmt modal', () => {
  it('dispatches fix_3 / sci_5 / eng_2', () => {
    expect(handleModalKey('3', { kind: 'fmt', mode: 'fix' }, false).dispatchId).toBe(
      'fix_3',
    );
    expect(handleModalKey('5', { kind: 'fmt', mode: 'sci' }, false).dispatchId).toBe(
      'sci_5',
    );
    expect(handleModalKey('2', { kind: 'fmt', mode: 'eng' }, false).dispatchId).toBe(
      'eng_2',
    );
  });
});

describe('handleModalKey — assign_key / assign_label (ASN flow)', () => {
  it('assign_key advances to assign_label on a keycode-magic input', () => {
    const pending: PendingInput = { kind: 'assign_key' };
    const r = handleModalKey(makeKeyCodeMagic(22), pending, false);
    expect(r.nextPending).toEqual({
      kind: 'assign_label',
      keyCode: 22,
      acc: '',
    });
  });

  it('assign_label Enter dispatches asn_NN_NAME', () => {
    const pending: PendingInput = {
      kind: 'assign_label',
      keyCode: 22,
      acc: 'TEST',
    };
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('asn_22_TEST');
  });
});

describe('handleModalKey — xeq_name (label-bearing prompts)', () => {
  it('xeq prefix dispatches xeq_LABEL', () => {
    let pending: PendingInput = {
      kind: 'xeq_name',
      acc: '',
      dispatchPrefix: 'xeq',
    };
    pending = handleModalKey('S', pending, false).nextPending!;
    pending = handleModalKey('U', pending, false).nextPending!;
    pending = handleModalKey('B', pending, false).nextPending!;
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('xeq_SUB');
  });

  it('gto prefix dispatches gto_LABEL', () => {
    let pending: PendingInput = {
      kind: 'xeq_name',
      acc: '',
      dispatchPrefix: 'gto',
    };
    pending = handleModalKey('A', pending, false).nextPending!;
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('gto_A');
  });

  it('lbl prefix dispatches lbl_LABEL', () => {
    let pending: PendingInput = {
      kind: 'xeq_name',
      acc: '',
      dispatchPrefix: 'lbl',
    };
    pending = handleModalKey('X', pending, false).nextPending!;
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('lbl_X');
  });
});

describe('handleModalKey — direct variant (BLOCKER B1)', () => {
  it('dispatches dispatchId immediately on first call', () => {
    const pending: PendingInput = {
      kind: 'direct',
      dispatchId: 'x_eq_y',
    };
    const r = handleModalKey('', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('x_eq_y');
    expect(r.consumesShift).toBe(false);
  });
});

describe('handleModalKey — Esc cancels (D-26.4)', () => {
  it('Esc returns nextPending=null, dispatchId=null', () => {
    const pending: PendingInput = {
      kind: 'register',
      op: 'Sto',
      ind: false,
      acc: '5',
    };
    const r = handleModalKey('Escape', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBeNull();
    expect(r.consumesShift).toBe(false);
  });
});

describe('renderModalLcd — preview strings (D-26.3)', () => {
  it('STO __ for register Sto direct, empty acc', () => {
    expect(
      renderModalLcd({ kind: 'register', op: 'Sto', ind: false, acc: '' }),
    ).toBe('STO __');
  });

  it('STO IND _5 for register Sto IND, acc="5"', () => {
    expect(
      renderModalLcd({ kind: 'register', op: 'Sto', ind: true, acc: '5' }),
    ).toBe('STO IND 5_');
  });

  it('SF 12 for flag SF direct, acc="12"', () => {
    expect(
      renderModalLcd({
        kind: 'flag',
        testKind: 'SF',
        ind: false,
        acc: '12',
      }),
    ).toBe('SF 12');
  });

  it('FS? __ for flag FsQuery direct, empty acc', () => {
    expect(
      renderModalLcd({
        kind: 'flag',
        testKind: 'FsQuery',
        ind: false,
        acc: '',
      }),
    ).toBe('FS? __');
  });

  it('FS?C IND __ for flag FsQueryClear, ind=true', () => {
    expect(
      renderModalLcd({
        kind: 'flag',
        testKind: 'FsQueryClear',
        ind: true,
        acc: '',
      }),
    ).toBe('FS?C IND __');
  });

  it('CLP MYPRG_ for clp acc="MYPRG"', () => {
    expect(renderModalLcd({ kind: 'clp', acc: 'MYPRG' })).toBe('CLP MYPRG_');
  });

  it('TONE _ for single_digit Tone (B2)', () => {
    expect(renderModalLcd({ kind: 'single_digit', op: 'Tone', max: 9 })).toBe(
      'TONE _',
    );
  });

  it('CAT _ for single_digit Catalog (B2)', () => {
    expect(
      renderModalLcd({ kind: 'single_digit', op: 'Catalog', max: 3 }),
    ).toBe('CAT _');
  });

  it('FIX _ for fmt fix', () => {
    expect(renderModalLcd({ kind: 'fmt', mode: 'fix' })).toBe('FIX _');
  });

  it('ASN __ for assign_key', () => {
    expect(renderModalLcd({ kind: 'assign_key' })).toBe('ASN __');
  });

  it('ASN TEST_ for assign_label acc="TEST"', () => {
    expect(
      renderModalLcd({ kind: 'assign_label', keyCode: 22, acc: 'TEST' }),
    ).toBe('ASN TEST_');
  });

  it('empty for direct variant (modal closes on the same tick)', () => {
    expect(
      renderModalLcd({ kind: 'direct', dispatchId: 'x_eq_y' }),
    ).toBe('');
  });
});

// ── Phase 31 Plan 05: XeqByName mode discriminator (D-29.9 GUI mirror) ──────
//
// Tests added per Task 2 requirements:
// (a) xeq_name normal mode Enter → existing xeq_<acc> behavior (backward-compat)
// (b) xeq_name collect-for-modal Enter → __submit_modal_with_label__<acc> magic-prefix
// (c) xeq_name collect-for-modal Backspace → pops last char, mode unchanged

describe('handleModalKey — xeq_name with mode discriminator (Phase 31 Plan 05)', () => {
  it('(a) normal mode Enter returns xeq_<acc> dispatchId (backward-compat)', () => {
    // Mode 'normal' (explicit) — existing behavior must be preserved.
    const pending: PendingInput = {
      kind: 'xeq_name',
      acc: 'SINH',
      dispatchPrefix: 'xeq',
      mode: 'normal',
    };
    const r = handleModalKey('Enter', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('xeq_SINH');
    expect(r.consumesShift).toBe(false);
  });

  it('(a-default) mode omitted (undefined) Enter also returns xeq_<acc> (default = normal)', () => {
    // mode is optional — omitting it must default to 'normal' behavior.
    const pending: PendingInput = {
      kind: 'xeq_name',
      acc: 'TANH',
      dispatchPrefix: 'xeq',
      // mode intentionally omitted
    };
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('xeq_TANH');
  });

  it('(b) collect-for-modal Enter returns __submit_modal_with_label__<acc> magic-prefix', () => {
    // Phase 31 Plan 05 / Pitfall 15: CollectForModal Enter must produce the
    // magic-prefix dispatch id so App.tsx::invokeForKey routes to submit_modal_with_label.
    const pending: PendingInput = {
      kind: 'xeq_name',
      acc: 'F',
      dispatchPrefix: 'xeq',
      mode: 'collect-for-modal',
    };
    const r = handleModalKey('Enter', pending, false);
    expect(r.nextPending).toBeNull();
    expect(r.dispatchId).toBe('__submit_modal_with_label__F');
    expect(r.consumesShift).toBe(false);
  });

  it('(b-full-label) collect-for-modal Enter with multi-char acc', () => {
    // Full function name label (e.g. "INTEG" for INTG).
    const pending: PendingInput = {
      kind: 'xeq_name',
      acc: 'INTEG',
      dispatchPrefix: 'xeq',
      mode: 'collect-for-modal',
    };
    const r = handleModalKey('Enter', pending, false);
    expect(r.dispatchId).toBe('__submit_modal_with_label__INTEG');
  });

  it('(c) collect-for-modal Backspace removes last char without changing mode', () => {
    // Backspace during CollectForModal must preserve the mode field.
    const pending: PendingInput = {
      kind: 'xeq_name',
      acc: 'AB',
      dispatchPrefix: 'xeq',
      mode: 'collect-for-modal',
    };
    const r = handleModalKey('Backspace', pending, false);
    expect(r.nextPending).toEqual({
      kind: 'xeq_name',
      acc: 'A',
      dispatchPrefix: 'xeq',
      mode: 'collect-for-modal',
    });
    expect(r.dispatchId).toBeNull();
  });

  it('collect-for-modal Enter with empty acc is a no-op (modal stays open)', () => {
    // Empty label must not dispatch — matches existing 'normal' behavior.
    const pending: PendingInput = {
      kind: 'xeq_name',
      acc: '',
      dispatchPrefix: 'xeq',
      mode: 'collect-for-modal',
    };
    const r = handleModalKey('Enter', pending, false);
    expect(r.nextPending).toEqual(pending);
    expect(r.dispatchId).toBeNull();
  });
});

describe('Phase 26 W3 — KEY_DEFS audit (TypeScript side)', () => {
  it('every KEY_DEFS primary id is non-empty', () => {
    for (const id of KEY_DEFS_PRIMARY_IDS) {
      expect(typeof id).toBe('string');
    }
  });

  it('every KEY_DEFS shifted id is non-empty', () => {
    for (const id of KEY_DEFS_SHIFTED_IDS) {
      expect(typeof id).toBe('string');
    }
  });

  it('no duplicate ids within primary list', () => {
    const set = new Set(KEY_DEFS_PRIMARY_IDS);
    expect(set.size).toBe(KEY_DEFS_PRIMARY_IDS.length);
  });
});
