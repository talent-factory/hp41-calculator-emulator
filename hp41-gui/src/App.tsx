import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { Keyboard } from './Keyboard';

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
}

function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
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
  const busyRef = useRef(false);
  const [printLog, setPrintLog] = useState<string[]>([]);
  const [printPanelOpen, setPrintPanelOpen] = useState(false);
  const printEndRef = useRef<HTMLDivElement>(null);

  // Mount: load initial state via get_state (D-11 — no polling)
  useEffect(() => {
    invoke<CalcStateView>('get_state')
      .then(view => setCalcState(view))
      .catch(err => console.error('get_state error:', err));
  }, []);

  // Keyboard handler — useCallback with calcState dep so 'n' reads latest in_eex_mode
  const handleKey = useCallback((e: KeyboardEvent) => {
    if (e.repeat) return;        // SC-4 fix: ignore OS key-repeat events — each IPC round-trip
                                 // completes before the next repeat fires, defeating busyRef alone
    if (busyRef.current) return; // debounce: ignore while invoke pending
    const keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;  // unmapped or modal-trigger key — silent ignore
    e.preventDefault();
    busyRef.current = true;
    invoke<CalcStateView>('dispatch_op', { keyId })
      .then(view => setCalcState(view))
      .catch(err => console.error('dispatch_op error:', err))
      .finally(() => { busyRef.current = false; });
  }, [calcState]);

  const handleClick = useCallback((keyId: string) => {
    if (busyRef.current) return;
    busyRef.current = true;
    invoke<CalcStateView>('dispatch_op', { keyId })
      .then(view => setCalcState(view))
      .catch(err => console.error('dispatch_op error:', err))
      .finally(() => { busyRef.current = false; });
  }, []);

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

  if (!calcState) {
    return <div className="calculator"><div className="display">Loading...</div></div>;
  }

  const annunciatorNames = ['user', 'prgm', 'alpha', 'rad', 'grad'] as const;
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
            className={`annunciator${calcState.annunciators[name] ? ' active' : ''}`}
          >
            {name.toUpperCase()}
          </span>
        ))}
      </div>
      <div className="display">{calcState.display_str}</div>
      <div className="stack-panel">
        {stackRows.map(([label, value]) => (
          <div key={label} className="stack-row">
            <span className="stack-label">{label}:</span>
            <span>{value}</span>
          </div>
        ))}
      </div>
      <Keyboard onKey={handleClick} busyRef={busyRef} />
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
