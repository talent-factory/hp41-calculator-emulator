import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { Keyboard, type KeyDef } from './Keyboard';

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
}

// Tauri rejects with GuiError { message: string } — String(err) yields
// "[object Object]". Extract the message field so toasts are readable.
function extractErrMessage(err: unknown): string {
  return typeof err === 'object' && err && 'message' in err
    ? String((err as { message: unknown }).message)
    : String(err);
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
  // EEX-CHS: 'n' routes based on current in_eex_mode (D-06)
  if (e.key === 'n') return state?.in_eex_mode ? 'eex_chs' : 'chs';
  // Digit entry
  if (e.key.length === 1 && '0123456789'.includes(e.key)) return e.key;
  if (e.key === '.') return '.';
  if (e.key === 'e') return 'e';
  // Modal-trigger keys — silently ignore, no invoke (D-05)
  if (e.key.length === 1 && 'SRfFPX'.includes(e.key)) return null;
  // Named op mapping — authoritative source: hp41-cli/src/keys.rs key_to_op()
  const MAP: Record<string, string> = {
    'Enter': 'enter', 'Backspace': 'clx',
    '+': 'plus', '-': 'minus', '*': 'mul', '/': 'div',
    'r': 'rdn', 'x': 'xy_swap', 'l': 'lastx', 's': 'sqrt', 'p': 'prx',
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
  // Phase 19 D-4: frontend-owned SHIFT one-shot prefix (no IPC round-trip).
  const [shiftActive, setShiftActive] = useState(false);
  // Toast overlay for GuiError responses (single-toast policy, 2s auto-dismiss).
  const [toastMsg, setToastMsg] = useState<string | null>(null);

  // Auto-dismiss toast after 2 seconds.
  useEffect(() => {
    if (!toastMsg) return;
    const t = setTimeout(() => setToastMsg(null), 2000);
    return () => clearTimeout(t);
  }, [toastMsg]);

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
      .catch(err => setToastMsg(extractErrMessage(err)))
      .finally(() => { busyRef.current = false; });
  }, []);

  // On-screen keyboard click router — implements spec D-4 priority:
  //   1. SHIFT key toggles local shiftActive, no dispatch.
  //   2. ALPHA mode → alpha_<char> (shift ignored in ALPHA mode).
  //   3. shiftActive && key.shifted → shifted.id, consumes shift one-shot.
  //   4. otherwise → primary id.
  // Special routes: sst/bst/r_s go to dedicated commands; clx_or_a branches on
  // the live ALPHA annunciator into clx | alpha_clear.
  const handleClick = useCallback(async (key: KeyDef) => {
    if (busyRef.current) return;

    // D-4 priority 1: SHIFT key itself toggles state, no dispatch.
    if (key.id === 'shift') {
      setShiftActive(prev => !prev);
      return;
    }

    // Resolve the effective id.
    const alphaOn = calcState?.annunciators.alpha ?? false;
    let effectiveId: string;
    let consumesShift = false;

    if (alphaOn && key.alphaChar) {
      // D-4 priority 2: ALPHA mode — character append. Shift ignored.
      effectiveId = `alpha_${key.alphaChar}`;
    } else if (shiftActive && key.shifted) {
      // D-4 priority 3: shifted action, consumes shift one-shot.
      effectiveId = key.shifted.id;
      consumesShift = true;
    } else {
      // D-4 priority 4: primary.
      effectiveId = key.id;
    }

    if (!effectiveId) return;

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
      setToastMsg(extractErrMessage(err));
    } finally {
      if (consumesShift) setShiftActive(false);
      busyRef.current = false;
    }
  }, [calcState, shiftActive]);

  // Physical-keyboard handler — useCallback with calcState dep so 'n' reads latest in_eex_mode.
  // Phase 19: Tab toggles SHIFT, Esc cancels SHIFT (no IPC).
  const handleKey = useCallback((e: KeyboardEvent) => {
    if (e.repeat) return;        // SC-4 fix: ignore OS key-repeat events — each IPC round-trip
                                 // completes before the next repeat fires, defeating busyRef alone
    if (e.key === 'Escape') {
      setShiftActive(false);
      return;
    }
    if (e.key === 'Tab') {
      e.preventDefault();
      setShiftActive(prev => !prev);
      return;
    }
    if (busyRef.current) return; // debounce: ignore while invoke pending
    const keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;  // unmapped or modal-trigger key — silent ignore
    e.preventDefault();
    dispatchKeyId(keyId);
  }, [calcState, dispatchKeyId]);

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
      <div className="display">{calcState.display_str}</div>
      {toastMsg && (
        <div className="toast" role="status">{toastMsg}</div>
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
    </div>
  );
}

export default App;
