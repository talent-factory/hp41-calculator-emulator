// Phase 26 D-26.1, D-26.2, D-26.3, D-26.4, D-26.5 — frontend-owned modal state.
//
// PendingInput mirrors hp41-cli's PendingInput hybrid struct-variants (Phase 25
// Plan 02) at the user-observable layer. CLI ↔ GUI parity invariant D-25.6
// requires identical logical states, one-shot SHIFT lifetime, ALPHA-overrides-
// SHIFT precedence, and IND-toggle via shift-0 — NOT implementation identity.
// The discriminated union lives in TypeScript so the modal preview, key
// accumulation, and parameterized-id dispatch happen entirely in React state
// (no new IPC surface per D-26.1).
//
// `handleModalKey` and `renderModalLcd` are pure functions for testability —
// they don't touch React state, just compute the next PendingInput + dispatch
// id + LCD preview string from inputs alone.

// --- TypeScript mirror of hp41-core enums (Phase 21/22/24) ----------------
// These hand-typed unions mirror the Rust enum membership exactly. A future
// codegen step (quicktype or similar) could automate this — deferred to
// Phase 27 if drift becomes a problem (per CONTEXT Claude's Discretion).

/// Mirrors hp41-core's FlagTestKind (4 variants).
/// The naming uses the CLI's PromptKind discriminator alias for clarity:
/// SF and CF are not part of FlagTestKind (they are SfFlag / CfFlag ops)
/// but share the same modal accumulator path, so we merge them here.
export type FlagPromptKind =
  | 'SF' // SfFlag — sets flag, no skip
  | 'CF' // CfFlag — clears flag, no skip
  | 'FsQuery' // FS? — skip if NOT set (FlagTestKind::IsSet)
  | 'FcQuery' // FC? — skip if NOT clear (FlagTestKind::IsClear)
  | 'FsQueryClear' // FS?C — IsSetThenClear (always clear after test)
  | 'FcQueryClear'; // FC?C — IsClearThenClear (always clear after test)

/// Mirrors the CLI's RegisterOpKind (8 variants × {direct, IND}).
/// StoArith is omitted — Phase 25 routes STO-arith through a separate
/// 3-step modal flow per W3 fix.
export type RegisterOpKind =
  | 'Sto'
  | 'Rcl'
  | 'View'
  | 'Arcl'
  | 'Asto'
  | 'Isg'
  | 'Dse';

export type FmtMode = 'fix' | 'sci' | 'eng';

export type SingleDigitOp = 'Tone' | 'Catalog';

/// Discriminated union — 14 variants. Each variant carries exactly the state
/// needed to render the LCD preview and decide the final parameterized
/// dispatch id at end-of-accumulation. Mirrors hp41-cli::app::PendingInput
/// 1:1 by intent (modulo TS naming idioms) per D-25.6.
export type PendingInput =
  // ── 12 base variants from D-26.4 ─────────────────────────────────────
  | { kind: 'flag'; testKind: FlagPromptKind; ind: boolean; acc: string }
  | { kind: 'register'; op: RegisterOpKind; ind: boolean; acc: string }
  | { kind: 'clp'; acc: string }
  | { kind: 'del'; acc: string }
  | { kind: 'xeq_name'; acc: string; dispatchPrefix: 'xeq' | 'gto' | 'lbl' }
  | { kind: 'fmt'; mode: FmtMode }
  | { kind: 'assign_key' }
  | { kind: 'assign_label'; keyCode: number; acc: string }
  | { kind: 'confirm_load'; programIdx: number }
  | { kind: 'hex'; acc: string }
  | { kind: 'print' }
  // ── B2 — Tone (max=9) + Catalog (max=3) collapse into single_digit ───
  | { kind: 'single_digit'; op: SingleDigitOp; max: number }
  // ── B1 — direct-dispatch: 4 conditional-test prompts open the modal
  //         and immediately resolve to a real op id on the same tick.
  | { kind: 'direct'; dispatchId: string };

/// Return shape of `handleModalKey`. Caller (App.tsx::handleClick) translates
/// these three fields into React state updates + invokeForKey calls.
export type ModalKeyResult = {
  /// PendingInput to keep the modal open with updated state, OR null to close.
  nextPending: PendingInput | null;
  /// Parameterized op id to dispatch via invokeForKey, OR null = no dispatch.
  dispatchId: string | null;
  /// True when the keystroke consumed shiftActive (IND-toggle path).
  consumesShift: boolean;
};

// --- helpers -------------------------------------------------------------

/// Map RegisterOpKind → key_map.rs prefix.
function regPrefix(op: RegisterOpKind): string {
  switch (op) {
    case 'Sto':
      return 'sto';
    case 'Rcl':
      return 'rcl';
    case 'View':
      return 'view';
    case 'Arcl':
      return 'arcl';
    case 'Asto':
      return 'asto';
    case 'Isg':
      return 'isg';
    case 'Dse':
      return 'dse';
  }
}

/// Map FlagPromptKind → key_map.rs prefix. The four FlagTestKind variants
/// (FS?, FC?, FS?C, FC?C) use distinct prefixes per key_map.rs §
/// "Phase 21 flag prefixes".
function flagPrefix(testKind: FlagPromptKind): string {
  switch (testKind) {
    case 'SF':
      return 'sf';
    case 'CF':
      return 'cf';
    case 'FsQuery':
      return 'fs';
    case 'FcQuery':
      return 'fc';
    case 'FsQueryClear':
      return 'fs_c';
    case 'FcQueryClear':
      return 'fc_c';
  }
}

/// Map FlagPromptKind → HP-41CV LCD display label.
function flagDisplayLabel(testKind: FlagPromptKind): string {
  switch (testKind) {
    case 'SF':
      return 'SF';
    case 'CF':
      return 'CF';
    case 'FsQuery':
      return 'FS?';
    case 'FcQuery':
      return 'FC?';
    case 'FsQueryClear':
      return 'FS?C';
    case 'FcQueryClear':
      return 'FC?C';
  }
}

/// Right-pad with underscore-as-cursor to 12 chars (typical HP-41 LCD width).
function padCursor(s: string, cursors = 1): string {
  // Visible cursors = trailing underscores AFTER the cursor count is consumed.
  // Caller passes the count of cursor positions still waiting for input.
  const cursorStr = '_'.repeat(cursors);
  return (s + cursorStr).slice(0, 12);
}

// --- pure-function modal engine -------------------------------------------

/**
 * Process one keystroke against an open modal. Returns the next pending
 * state, an optional dispatch id, and whether shiftActive was consumed.
 *
 * `key` is the resolved string id of the keystroke — a digit ("0".."9"),
 * a letter, or a special token ("Enter", "Escape", "Backspace").
 *
 * Phase 26 D-26.2 IND-toggle: inside a `flag` or `register` modal, pressing
 * SHIFT then "0" toggles `pending.ind` (hardware-faithful per HP-41C/CV
 * Quick Reference Guide p.14). The SHIFT bit lives on App.tsx as `shiftActive`
 * (reused, not duplicated — same precedent as v2.1 `shiftActive` per D-26.1).
 * This function does NOT mutate `shiftActive`; it signals consumption via
 * `consumesShift: true` and the caller sets `setShiftActive(false)`.
 */
export function handleModalKey(
  key: string,
  pending: PendingInput,
  shiftActive: boolean,
): ModalKeyResult {
  // Esc cancels any open modal, no dispatch.
  if (key === 'Escape') {
    return { nextPending: null, dispatchId: null, consumesShift: false };
  }

  switch (pending.kind) {
    case 'direct': {
      // B1: opens and resolves on the same tick — dispatchId is the real id.
      return {
        nextPending: null,
        dispatchId: pending.dispatchId,
        consumesShift: false,
      };
    }

    case 'flag': {
      // IND-toggle path (D-26.2 / Pitfall 10): SHIFT armed + "0" toggles ind.
      if (shiftActive && key === '0') {
        return {
          nextPending: { ...pending, ind: !pending.ind },
          dispatchId: null,
          consumesShift: true,
        };
      }
      if (isDigit(key)) {
        const acc = pending.acc + key;
        if (acc.length === 2) {
          const prefix = flagPrefix(pending.testKind);
          const dispatchId = pending.ind
            ? `${prefix}_ind_${acc}`
            : `${prefix}_${acc}`;
          return { nextPending: null, dispatchId, consumesShift: false };
        }
        return {
          nextPending: { ...pending, acc },
          dispatchId: null,
          consumesShift: false,
        };
      }
      // Ignore unmapped keys (preserve the modal).
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'register': {
      // IND-toggle path (D-26.2 / Pitfall 10).
      if (shiftActive && key === '0') {
        return {
          nextPending: { ...pending, ind: !pending.ind },
          dispatchId: null,
          consumesShift: true,
        };
      }
      if (isDigit(key)) {
        const acc = pending.acc + key;
        if (acc.length === 2) {
          const prefix = regPrefix(pending.op);
          const dispatchId = pending.ind
            ? `${prefix}_ind_${acc}`
            : `${prefix}_${acc}`;
          return { nextPending: null, dispatchId, consumesShift: false };
        }
        return {
          nextPending: { ...pending, acc },
          dispatchId: null,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'clp': {
      // Text-input modal — Enter dispatches; Backspace pops a char; any other
      // single printable char appends. Empty labels reject (no-op).
      if (key === 'Enter') {
        if (pending.acc.length === 0) {
          return { nextPending: pending, dispatchId: null, consumesShift: false };
        }
        return {
          nextPending: null,
          dispatchId: `clp_${pending.acc}`,
          consumesShift: false,
        };
      }
      if (key === 'Backspace') {
        return {
          nextPending: { ...pending, acc: pending.acc.slice(0, -1) },
          dispatchId: null,
          consumesShift: false,
        };
      }
      if (isPrintableChar(key) && pending.acc.length < 7) {
        return {
          nextPending: { ...pending, acc: pending.acc + key.toUpperCase() },
          dispatchId: null,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'del': {
      // 3-digit numeric — auto-dispatch on the 3rd digit.
      // BLOCKER B3: backend clamps at 255; frontend preview shows "DEL ERR"
      // for accumulators that would parse to >255. Dispatch still fires;
      // key_map.rs returns the GuiError and the toast surfaces.
      if (isDigit(key)) {
        const acc = pending.acc + key;
        if (acc.length === 3) {
          return {
            nextPending: null,
            dispatchId: `del_${acc}`,
            consumesShift: false,
          };
        }
        return {
          nextPending: { ...pending, acc },
          dispatchId: null,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'single_digit': {
      // B2: single 0..=max keystroke. Reject digits > max silently.
      if (isDigit(key)) {
        const digit = Number(key);
        if (digit > pending.max) {
          return { nextPending: pending, dispatchId: null, consumesShift: false };
        }
        const idPrefix = pending.op === 'Tone' ? 'tone' : 'catalog';
        return {
          nextPending: null,
          dispatchId: `${idPrefix}_${digit}`,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'fmt': {
      // Single 0..=9 digit dispatches fix_N / sci_N / eng_N.
      if (isDigit(key)) {
        return {
          nextPending: null,
          dispatchId: `${pending.mode}_${key}`,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'xeq_name': {
      // Text input + Enter. Empty label dispatches the bare prompt id
      // (consumed by the resolver as `xeq_` → label "" → no-op label match).
      if (key === 'Enter') {
        if (pending.acc.length === 0) {
          return { nextPending: pending, dispatchId: null, consumesShift: false };
        }
        return {
          nextPending: null,
          dispatchId: `${pending.dispatchPrefix}_${pending.acc}`,
          consumesShift: false,
        };
      }
      if (key === 'Backspace') {
        return {
          nextPending: { ...pending, acc: pending.acc.slice(0, -1) },
          dispatchId: null,
          consumesShift: false,
        };
      }
      if (isPrintableChar(key) && pending.acc.length < 24) {
        return {
          nextPending: { ...pending, acc: pending.acc + key.toUpperCase() },
          dispatchId: null,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'assign_key': {
      // The keystroke here MUST be resolved to a keyCode by the caller
      // (handleClick has the KeyDef in scope; physical-keyboard rejects
      // assign_key for now). The caller passes the keyCode via a magic
      // prefix `__keycode__<NN>`. If `key` doesn't match that pattern,
      // ignore.
      const kc = parseKeyCodeMagic(key);
      if (kc === null) {
        return { nextPending: pending, dispatchId: null, consumesShift: false };
      }
      return {
        nextPending: { kind: 'assign_label', keyCode: kc, acc: '' },
        dispatchId: null,
        consumesShift: false,
      };
    }

    case 'assign_label': {
      // Text input + Enter dispatches asn_NN_NAME (key_map.rs::resolve_asn).
      if (key === 'Enter') {
        return {
          nextPending: null,
          dispatchId: `asn_${pending.keyCode}_${pending.acc}`,
          consumesShift: false,
        };
      }
      if (key === 'Backspace') {
        return {
          nextPending: { ...pending, acc: pending.acc.slice(0, -1) },
          dispatchId: null,
          consumesShift: false,
        };
      }
      if (isPrintableChar(key) && pending.acc.length < 7) {
        return {
          nextPending: { ...pending, acc: pending.acc + key.toUpperCase() },
          dispatchId: null,
          consumesShift: false,
        };
      }
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }

    case 'confirm_load':
    case 'hex':
    case 'print': {
      // Phase 26 scope: scaffolds only — v1.x CLI semantics ported in a
      // later plan if/when these modals are needed in the GUI. Esc cancels.
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }
  }
}

/**
 * Emit the LCD preview string for an active modal per D-26.3. The string is
 * 12 chars or fewer (HP-41 display width); the caller (App.tsx) passes it to
 * <Display14Seg /> in Plan 26-02 or to the existing .display div until then.
 *
 * Cursor convention: trailing underscore = "waiting for input" digit position;
 * non-cursor chars are the already-accumulated state. Examples:
 *   register Sto, ind=false, acc=""     → "STO __"
 *   register Sto, ind=true, acc="5"     → "STO IND _5"
 *   flag SF, acc="12"                   → "SF 12"
 *   clp acc="MYPRG"                     → "CLP MYPRG_"
 *   single_digit Tone                   → "TONE _"
 *   del acc="256"                       → "DEL ERR"   (BLOCKER B3)
 */
export function renderModalLcd(pending: PendingInput): string {
  switch (pending.kind) {
    case 'direct': {
      // Never visible (modal closes on the same tick it opens).
      return '';
    }
    case 'flag': {
      // Spelt-out modal labels — match HP-41CV display conventions.
      const display = flagDisplayLabel(pending.testKind);
      const indPart = pending.ind ? ' IND' : '';
      const accPart = pending.acc.padEnd(2, '_');
      return `${display}${indPart} ${accPart}`;
    }
    case 'register': {
      const display = pending.op.toUpperCase();
      const indPart = pending.ind ? ' IND' : '';
      const accPart = pending.acc.padEnd(2, '_');
      return `${display}${indPart} ${accPart}`;
    }
    case 'clp': {
      return padCursor(`CLP ${pending.acc}`, 1);
    }
    case 'del': {
      // BLOCKER B3 — when accumulator would resolve to >255, render "DEL ERR"
      // instead of the numeric preview.
      if (pending.acc.length === 3) {
        const n = Number(pending.acc);
        if (!Number.isNaN(n) && n > 255) {
          return 'DEL ERR';
        }
      }
      return `DEL ${pending.acc.padEnd(3, '_')}`;
    }
    case 'single_digit': {
      const display = pending.op === 'Tone' ? 'TONE' : 'CAT';
      return `${display} _`;
    }
    case 'fmt': {
      const display = pending.mode.toUpperCase();
      return `${display} _`;
    }
    case 'xeq_name': {
      const display = pending.dispatchPrefix.toUpperCase();
      return padCursor(`${display} ${pending.acc}`, 1);
    }
    case 'assign_key': {
      return 'ASN __';
    }
    case 'assign_label': {
      return padCursor(`ASN ${pending.acc}`, 1);
    }
    case 'confirm_load': {
      return `LOAD Y/N?`;
    }
    case 'hex': {
      return `HEX ${pending.acc.padEnd(2, '_')}`;
    }
    case 'print': {
      return 'PRINT _';
    }
  }
}

// --- predicates ----------------------------------------------------------

function isDigit(key: string): boolean {
  return key.length === 1 && key >= '0' && key <= '9';
}

function isPrintableChar(key: string): boolean {
  // Letters, digits, common punctuation usable in HP-41 labels.
  return key.length === 1 && /^[A-Za-z0-9 +\-*/]$/.test(key);
}

/// Recognise the magic prefix `__keycode__<NN>` used by the caller to pass a
/// resolved keyCode into the assign_key modal.
export function makeKeyCodeMagic(keyCode: number): string {
  return `__keycode__${keyCode}`;
}

function parseKeyCodeMagic(key: string): number | null {
  const match = key.match(/^__keycode__(\d+)$/);
  if (!match) return null;
  const n = Number(match[1]);
  return Number.isFinite(n) ? n : null;
}
