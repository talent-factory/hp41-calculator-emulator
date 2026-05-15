import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { Keyboard, type KeyDef } from './Keyboard';
import Display14Seg from './Display14Seg';
import HelpOverlay from './HelpOverlay';
import {
  handleModalKey,
  renderModalLcd,
  makeKeyCodeMagic,
  type PendingInput,
  type ModalKeyResult,
} from './pending_input';

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
  program_steps: string[];  // Phase 18 D-01: pre-formatted step strings from Rust
  pc: number;               // Phase 18 D-01: current program counter index
  // Phase 26 D-26.11 (BLOCKER B5): TS-side mirror of the new Rust projections.
  user_keymap: Array<[number, string]>;   // mirrors Vec<(u8, String)>
  flags: number[];                         // mirrors Vec<u8> of set-flag indices
  display_override: string | null;         // mirrors Option<String>
  event_buffer: string[];                  // mirrors Vec<String> (drained per IPC)
}

// Tauri rejects with GuiError { message: string } — String(err) yields
// "[object Object]". Extract the message field so toasts are readable.
// For other object-shaped rejections (raw Tauri framework errors, third-
// party rejections without `.message`) fall back to JSON.stringify so the
// "[object Object]" failure mode this helper exists to prevent cannot leak
// through. Strings, numbers, null, undefined fall through to String().
function extractErrMessage(err: unknown): string {
  if (typeof err === 'object' && err !== null) {
    if ('message' in err) return String((err as { message: unknown }).message);
    try {
      return JSON.stringify(err);
    } catch {
      // Circular reference or non-serialisable value — fall through to String.
    }
  }
  return String(err);
}

// Route a resolved op id to the right Tauri command. SST/BST/R-S have
// dedicated commands; everything else flows through dispatch_op.
async function invokeForKey(effectiveId: string): Promise<CalcStateView> {
  if (effectiveId === 'sst') return invoke<CalcStateView>('sst_step');
  if (effectiveId === 'bst') return invoke<CalcStateView>('bst_step');
  if (effectiveId === 'r_s') return invoke<CalcStateView>('run_stop');
  return invoke<CalcStateView>('dispatch_op', { keyId: effectiveId });
}

function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
  // Phase 18 D-07: F7/F8 → SST/BST keyboard bindings
  // Use e.code (physical key) so macOS media-key remapping doesn't block these
  if (e.key === 'F7' || e.code === 'F7') return 'sst';
  if (e.key === 'F8' || e.code === 'F8') return 'bst';

  // ALPHA-mode pass-through: when the ALPHA annunciator is active, route
  // single printable A-Z / 0-9 / space keys to alpha_<X> so the backend
  // resolves them as Op::AlphaAppend.  This must come BEFORE the normal key
  // map so typing "QUAD" in ALPHA mode appends to the alpha register rather
  // than dispatching SIN/UNDEF/ASIN/SDEV.
  if (state?.annunciators?.alpha && e.key.length === 1) {
    const ch = e.key.toUpperCase();
    if (/^[A-Z0-9 ]$/.test(ch)) {
      return `alpha_${ch}`;
    }
  }

  // EEX-CHS: 'n' routes based on current in_eex_mode (D-06)
  if (e.key === 'n') return state?.in_eex_mode ? 'eex_chs' : 'chs';
  // Digit entry
  if (e.key.length === 1 && '0123456789'.includes(e.key)) return e.key;
  if (e.key === '.') return '.';
  if (e.key === 'e') return 'e';
  // Modal-trigger keys — silently ignore, no invoke (D-05).
  // 'P' was in this list pre-Phase-26 but Phase 26 D-26.10 reassigns it
  // to 'prx' (SHIFT+P prints X). 'p' (lowercase) was 'prx' pre-Phase-26
  // and is now 'prgm_mode' per D-26.10.
  if (e.key.length === 1 && 'SRfFX'.includes(e.key)) return null;
  // Named op mapping — authoritative source: hp41-cli/src/keys.rs key_to_op()
  const MAP: Record<string, string> = {
    'Enter': 'enter', 'Backspace': 'clx',
    '+': 'plus', '-': 'minus', '*': 'mul', '/': 'div',
    'r': 'rdn', 'x': 'xy_swap', 'l': 'lastx', 's': 'sqrt',
    // Phase 26 D-26.10 — physical-keyboard 'p' remap.
    // Pre-Phase-26: 'p' was 'prx'. The conflict with the cluster of
    // letter-key shortcuts (every other letter is a math/program op,
    // not a print directive) was deferred from v2.0. v2.2 resolves it:
    // lowercase 'p' now toggles PRGM mode; SHIFT+'P' prints X.
    'p': 'prgm_mode',
    'P': 'prx',
    'a': 'asin', 'c': 'acos', 'k': 'atan',
    'C': 'cos', 'T': 'tan', 'L': 'ln', 'G': 'log', 'E': 'exp',
    'H': 'tenpow', 'I': 'recip', 'W': 'sq', 'Y': 'ypow',
    '%': 'pct_change',
    'u': 'user_mode',
    'z': 'sigma_plus', 'Z': 'sigma_minus', 'm': 'mean', 'D': 'sdev',
    'y': 'yhat', 'b': 'lr', 'O': 'corr', 'V': 'cl_sigma_stat',
    'h': 'hms_to_h', 'j': 'hms_add', 'J': 'hms_sub',
    'q': 'sin',    // Phase 8 reassignment: 'q' = SIN
    'g': 'clreg',  // Phase 8 addition: 'g' = CLREG
  };
  return MAP[e.key] ?? null;
}

// Phase 26 D-26.5 — modal-opener factory table. Mapping from clickable
// modal-opener id → factory function returning the initial PendingInput.
// The 13 *_prompt ids + asn/view/catalog/xeq_prompt/gto_prompt/lbl_prompt are
// intercepted in `handleClick` BEFORE they reach `dispatch_op` — the backend
// stub-error arm in key_map.rs stays as defense-in-depth (D-07 invariant).
//
// The 4 conditional-test prompts (x_eq_y_prompt, etc.) route through the
// `direct` variant (B1) — they dispatch immediately on the next handleModalKey
// call; no accumulator, no IND-toggle.
const MODAL_OPENERS: Record<string, () => PendingInput> = {
  // Register-modal openers (accumulator + IND-toggle).
  sto_prompt: () => ({ kind: 'register', op: 'Sto', ind: false, acc: '' }),
  rcl_prompt: () => ({ kind: 'register', op: 'Rcl', ind: false, acc: '' }),
  isg_prompt: () => ({ kind: 'register', op: 'Isg', ind: false, acc: '' }),
  // Flag-modal openers (accumulator + IND-toggle).
  sf_prompt: () => ({ kind: 'flag', testKind: 'SF', ind: false, acc: '' }),
  cf_prompt: () => ({ kind: 'flag', testKind: 'CF', ind: false, acc: '' }),
  fs_prompt: () => ({ kind: 'flag', testKind: 'FsQuery', ind: false, acc: '' }),
  // Fmt-modal openers (single-digit FIX/SCI/ENG N).
  fix_prompt: () => ({ kind: 'fmt', mode: 'fix' }),
  sci_prompt: () => ({ kind: 'fmt', mode: 'sci' }),
  eng_prompt: () => ({ kind: 'fmt', mode: 'eng' }),
  // BLOCKER B1: conditional-test prompts dispatch IMMEDIATELY via `direct`.
  x_eq_y_prompt: () => ({ kind: 'direct', dispatchId: 'x_eq_y' }),
  x_le_y_prompt: () => ({ kind: 'direct', dispatchId: 'x_le_y' }),
  x_gt_y_prompt: () => ({ kind: 'direct', dispatchId: 'x_gt_y' }),
  x_eq_0_prompt: () => ({ kind: 'direct', dispatchId: 'x_eq_0' }),
  // Label-bearing modals (text input + Enter) — xeq_name shape reused.
  xeq_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'xeq' }),
  gto_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'gto' }),
  lbl_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'lbl' }),
  // BLOCKER B2: catalog + tone share single_digit with op + max discriminator.
  // Phase 26 Plan 04 CR-05 — Catalog max raised from 3 to 4 so XFNS (CAT 4) is
  // reachable from the GUI; matches hp41-core op_catalog (accepts n in 1..=4).
  catalog: () => ({ kind: 'single_digit', op: 'Catalog', max: 4 }),
  tone: () => ({ kind: 'single_digit', op: 'Tone', max: 9 }),
  // ASN flow: AssignKey → (next key click via __keycode__NN) → AssignLabel.
  asn: () => ({ kind: 'assign_key' }),
  // VIEW — takes a register, same shape as register-modal but op='View'.
  view: () => ({ kind: 'register', op: 'View', ind: false, acc: '' }),
};

function App() {
  const [calcState, setCalcState] = useState<CalcStateView | null>(null);
  // Surfaces GuiError messages from the backend (DivByZero, "unknown key", load
  // failure, lock poison, …). Without this row every HpError translated via
  // From<HpError> ends up at console.error and the user sees stale state.
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const busyRef = useRef(false);
  const [printLog, setPrintLog] = useState<string[]>([]);
  const [printPanelOpen, setPrintPanelOpen] = useState(false);
  const printEndRef = useRef<HTMLDivElement>(null);
  const activeStepRef = useRef<HTMLDivElement>(null);
  // Frontend-owned SHIFT one-shot prefix (no IPC round-trip).
  const [shiftActive, setShiftActive] = useState(false);
  // Phase 26 D-26.1 — frontend-owned modal state (no IPC round-trip).
  // PendingInput | null carries the 14 logical states from CLI Phase 25
  // (parity invariant D-25.6). Accumulator keystrokes route through
  // `handleModalKey`; dispatch happens at end-of-accumulation via
  // `invokeForKey(parameterizedId)`. Esc clears both shiftActive AND
  // pendingInput.
  const [pendingInput, setPendingInput] = useState<PendingInput | null>(null);
  // Phase 26 D-26.8 — `?` help overlay open/close. Frontend-only state
  // (no IPC round-trip — the help data is bundled at build time via
  // help_data.ts's vite JSON import).
  const [helpOpen, setHelpOpen] = useState(false);
  // Toast overlay for GuiError responses (single-toast policy, 2s auto-dismiss).
  // The monotonic `seq` is required because two clicks on the same stubbed
  // key produce identical message strings — setting state to the same value
  // does not re-run the auto-dismiss effect, so the second toast would be
  // dismissed by the first click's still-running timer. `seq` makes the
  // state value distinct on each call.
  const [toast, setToast] = useState<{ msg: string; seq: number } | null>(null);
  const toastSeqRef = useRef(0);
  const showToast = useCallback((msg: string) => {
    toastSeqRef.current += 1;
    setToast({ msg, seq: toastSeqRef.current });
  }, []);

  // Auto-dismiss toast after 2 seconds. Re-runs on every showToast() call
  // because the `seq` field changes even when `msg` is the same.
  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 2000);
    return () => clearTimeout(t);
  }, [toast]);

  // Mount: load initial state via get_state (D-11 — no polling)
  useEffect(() => {
    invoke<CalcStateView>('get_state')
      .then(view => { setCalcState(view); setErrorMessage(null); })
      .catch(err => setErrorMessage(`Load failed: ${err}`));
  }, []);

  // Physical-keyboard dispatch (option B): string-id path, no SHIFT/ALPHA frontend
  // mediation. resolveKeyId already maps physical keys to op ids directly. SST/BST
  // route to their dedicated Tauri commands; everything else goes through dispatch_op.
  // Errors surface as a toast (consistent with on-screen keyboard).
  const dispatchKeyId = useCallback((keyId: string) => {
    if (busyRef.current) return;
    busyRef.current = true;
    invokeForKey(keyId)
      .then(view => { setCalcState(view); setErrorMessage(null); })
      .catch(err => showToast(extractErrMessage(err)))
      .finally(() => { busyRef.current = false; });
  }, []);

  // Apply a ModalKeyResult — updates state and optionally dispatches.
  // Returns true if a dispatch was issued (caller can short-circuit).
  const applyModalResult = useCallback(
    async (result: ModalKeyResult): Promise<boolean> => {
      setPendingInput(result.nextPending);
      if (result.consumesShift) setShiftActive(false);
      if (result.dispatchId === null) return false;
      busyRef.current = true;
      try {
        const view = await invokeForKey(result.dispatchId);
        setCalcState(view);
        setErrorMessage(null);
      } catch (err) {
        showToast(extractErrMessage(err));
      } finally {
        busyRef.current = false;
      }
      return true;
    },
    [showToast],
  );

  // On-screen keyboard click router. Resolution order:
  //   1. SHIFT key toggles local shiftActive, no dispatch.
  //   2. ALPHA mode + alphaChar → alpha_<char> (SHIFT ignored in ALPHA mode).
  //   3. shiftActive + key.shifted → shifted.id, consumes the one-shot.
  //   4. otherwise → primary id.
  //   5. Phase 26 D-26.5: if effectiveId in MODAL_OPENERS, intercept BEFORE
  //      invokeForKey and open the React modal. The `direct` variant resolves
  //      on the same tick (B1).
  //   6. If a modal is already open (pendingInput !== null), route the click
  //      through handleModalKey (D-26.2).
  // Special routes: sst/bst/r_s go to dedicated commands; clx_or_a branches on
  // the live ALPHA annunciator into clx | alpha_clear.
  const handleClick = useCallback(async (key: KeyDef) => {
    if (busyRef.current) return;

    // SHIFT key itself toggles state, no dispatch (rule 1 above).
    // CRITICAL (W2): SHIFT toggling inside an open modal goes HERE, NOT
    // through handleModalKey — the modal's IND-toggle path requires
    // shiftActive to be set BEFORE the "0" keystroke arrives.
    if (key.id === 'shift') {
      setShiftActive(prev => !prev);
      return;
    }

    // Resolve the effective id per rules 2-4.
    const alphaOn = calcState?.annunciators.alpha ?? false;
    let effectiveId: string;
    let consumesShift = false;

    if (alphaOn && key.alphaChar) {
      effectiveId = `alpha_${key.alphaChar}`;
    } else if (shiftActive && key.shifted) {
      effectiveId = key.shifted.id;
      consumesShift = true;
    } else {
      effectiveId = key.id;
    }

    if (!effectiveId) return;

    // Rule 6: if a modal is open, route through handleModalKey.
    if (pendingInput !== null) {
      // Special case: assign_key modal expects a keycode via magic prefix.
      // Compute it from the clicked key's row/col (HP-41 row×10+col).
      const routedKey =
        pendingInput.kind === 'assign_key'
          ? makeKeyCodeMagic(key.row * 10 + (key.col + 1))
          : effectiveId;
      const result = handleModalKey(routedKey, pendingInput, shiftActive);
      // Consume the click shift on a modal-key transition too.
      if (consumesShift && !result.consumesShift) setShiftActive(false);
      await applyModalResult(result);
      return;
    }

    // Rule 5: modal-opener intercept (D-26.5).
    if (MODAL_OPENERS[effectiveId]) {
      const initial = MODAL_OPENERS[effectiveId]();
      // B1 fast path: `direct` variant — open and resolve on the same tick.
      if (initial.kind === 'direct') {
        const result = handleModalKey('', initial, false);
        if (consumesShift) setShiftActive(false);
        await applyModalResult(result);
        return;
      }
      setPendingInput(initial);
      if (consumesShift) setShiftActive(false);
      return;
    }

    busyRef.current = true;
    try {
      let view: CalcStateView;
      if (effectiveId === 'clx_or_a') {
        // CL X/A — branch on alpha mode at click time. (On-screen-specific:
        // physical-keyboard has no equivalent path, so this stays out of
        // invokeForKey and lives here in handleClick.)
        const targetId = alphaOn ? 'alpha_clear' : 'clx';
        view = await invoke<CalcStateView>('dispatch_op', { keyId: targetId });
      } else {
        view = await invokeForKey(effectiveId);
      }
      setCalcState(view);
      setErrorMessage(null);
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      if (consumesShift) setShiftActive(false);
      busyRef.current = false;
    }
  }, [calcState, shiftActive, pendingInput, applyModalResult, showToast]);

  // Physical-keyboard handler — useCallback with calcState dep so 'n' reads latest in_eex_mode.
  // Tab toggles SHIFT, Esc cancels in precedence order: help → modal → shift
  // (Phase 26 D-26.8 + D-26.4).
  const handleKey = useCallback((e: KeyboardEvent) => {
    if (e.repeat) return;        // SC-4 fix: ignore OS key-repeat events — each IPC round-trip
                                 // completes before the next repeat fires, defeating busyRef alone

    // Phase 26 D-26.8 — '?' opens the help overlay. Guard against ALPHA mode
    // (where '?' is a valid ALPHA register input — same convention as the
    // CLI Phase 25 `?` overlay). Skip if the overlay is already open so
    // typing '?' in the search input doesn't re-fire the toggle.
    const alphaOn = calcState?.annunciators.alpha ?? false;
    if (e.key === '?' && !alphaOn && !helpOpen) {
      e.preventDefault();
      setHelpOpen(true);
      return;
    }

    // Esc precedence (D-26.8 + D-26.4):
    //   1. Help overlay first (closes on Esc; doesn't clear modal/shift).
    //   2. pendingInput second (closes the modal, clears shiftActive).
    //   3. shiftActive last (clears the one-shot SHIFT prefix).
    // This precedence keeps each layer independently dismissable: opening
    // help doesn't lose an in-progress modal; canceling help leaves the
    // modal intact.
    if (e.key === 'Escape') {
      if (helpOpen) {
        setHelpOpen(false);
        return;
      }
      if (pendingInput !== null) {
        setPendingInput(null);
        setShiftActive(false);
        return;
      }
      setShiftActive(false);
      return;
    }
    if (e.key === 'Tab') {
      e.preventDefault();
      setShiftActive(prev => !prev);
      return;
    }
    if (busyRef.current) return; // debounce: ignore while invoke pending

    // Phase 26 D-26.4: if a modal is open, route the key through handleModalKey
    // BEFORE the normal resolveKeyId path. Esc is already handled above.
    if (pendingInput !== null) {
      // Translate physical keys to the modal's input alphabet.
      let modalKey: string | null = null;
      if (e.key === 'Enter') modalKey = 'Enter';
      else if (e.key === 'Backspace') modalKey = 'Backspace';
      else if (e.key.length === 1) {
        // Single printable character — digit, letter, or punctuation.
        modalKey = e.key;
      }
      if (modalKey === null) return;
      e.preventDefault();
      const result = handleModalKey(modalKey, pendingInput, shiftActive);
      void applyModalResult(result);
      return;
    }

    const keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;  // unmapped or modal-trigger key — silent ignore
    e.preventDefault();
    dispatchKeyId(keyId);
  }, [calcState, dispatchKeyId, pendingInput, shiftActive, applyModalResult, helpOpen]);

  // Register keyboard listener — cleanup required for React StrictMode (D-12)
  useEffect(() => {
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [handleKey]);

  // Accumulate print_lines from each IPC response into local React state.
  // D-09: print_buffer is drained per IPC call; React retains full history.
  // D-07: setPrintPanelOpen(true) auto-shows panel on first print output.
  useEffect(() => {
    if (calcState && calcState.print_lines.length > 0) {
      setPrintLog(prev => [...prev, ...calcState.print_lines]);
      setPrintPanelOpen(true);
    }
  }, [calcState]);

  // Auto-scroll to bottom whenever the print log grows.
  useEffect(() => {
    printEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [printLog]);

  // Auto-scroll active program step into view when pc changes (D-09)
  useEffect(() => {
    activeStepRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }, [calcState?.pc]);

  if (!calcState) {
    return <div className="calculator"><div className="display">Loading...</div></div>;
  }

  const annunciatorNames = ['user', 'shift', 'prgm', 'alpha', 'rad', 'grad'] as const;
  // SHIFT is a frontend-derived annunciator; the rest come from the backend CalcStateView.
  const annunciators: Record<typeof annunciatorNames[number], boolean> = {
    user:  calcState.annunciators.user,
    shift: shiftActive,
    prgm:  calcState.annunciators.prgm,
    alpha: calcState.annunciators.alpha,
    rad:   calcState.annunciators.rad,
    grad:  calcState.annunciators.grad,
  };
  const stackRows: [string, string][] = [
    ['X', calcState.x_str],
    ['Y', calcState.y_str],
    ['Z', calcState.z_str],
    ['T', calcState.t_str],
    ['L', calcState.lastx_str],
  ];

  // Phase 26 D-26.3 — modal preview replaces LCD content during accumulation.
  // Plan 26-02 will swap the inner content to <Display14Seg text={displayText} />;
  // the derivation here is forward-compatible.
  const displayText: string = pendingInput
    ? renderModalLcd(pendingInput)
    : calcState.display_str;

  return (
    <div className="calculator">
      <div className="annunciators">
        {annunciatorNames.map(name => (
          <span
            key={name}
            className={`annunciator annunciator-${name}${annunciators[name] ? ' active' : ''}`}
          >
            {name.toUpperCase()}
          </span>
        ))}
      </div>
      <div className="display"><Display14Seg text={displayText} /></div>
      {toast && (
        <div key={toast.seq} className="toast" role="status">{toast.msg}</div>
      )}
      {errorMessage && (
        <div className="error-row" role="alert">{errorMessage}</div>
      )}
      <div className="stack-panel">
        {stackRows.map(([label, value]) => (
          <div key={label} className="stack-row">
            <span className="stack-label">{label}:</span>
            <span>{value}</span>
          </div>
        ))}
      </div>
      <Keyboard
        onKey={handleClick}
        busyRef={busyRef}
        shiftActive={shiftActive}
        alphaActive={calcState.annunciators.alpha}
        userActive={calcState.annunciators.user}
        userKeymap={calcState.user_keymap}
      />
      {calcState.annunciators.prgm && (
        <div className="prgm-panel">
          <div className="prgm-panel-header">
            PRGM &#8212; {calcState.program_steps.length - 1}{' '}
            {calcState.program_steps.length - 1 === 1 ? 'step' : 'steps'}
          </div>
          <div className="prgm-panel-content">
            {calcState.program_steps.map((step, i) => (
              <div
                key={i}
                ref={calcState.pc === i ? activeStepRef : null}
                className={`step-row${calcState.pc === i ? ' step-active' : ''}`}
              >
                {step}
              </div>
            ))}
          </div>
        </div>
      )}
      {printPanelOpen && (
        <div className="print-panel">
          <div className="print-panel-header">
            <span>PRINT</span>
            <button className="print-panel-close" onClick={() => setPrintPanelOpen(false)}>×</button>
          </div>
          <div className="print-panel-content">
            {printLog.map((line, i) => (
              <div key={i} className="print-line">{line}</div>
            ))}
            <div ref={printEndRef} />
          </div>
        </div>
      )}
      {/* Phase 26 D-26.8 — `?` help overlay. The component returns null when
          open=false, so unconditional placement in the tree is safe. Anchored
          inside `.calculator` (position: relative) so the overlay's `position:
          absolute` covers the calculator footprint only, not the page. */}
      <HelpOverlay open={helpOpen} onClose={() => setHelpOpen(false)} />
    </div>
  );
}

export default App;
