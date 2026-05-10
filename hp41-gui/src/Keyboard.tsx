import { useState, type MutableRefObject } from 'react';

type KeyDef = {
  id: string;
  label: string;
  fShiftLabel?: string;
  row: number;
  col: number;
  colSpan?: number;
};

const KEY_W = 39;
const KEY_H = 26;
const GAP = 4;
const PAD = 8;
const FSHIFT_H = 12;

const KEY_DEFS: KeyDef[] = [
  // Row 0 — top math/function row (9 keys)
  { id: 'sigma_plus', label: 'Σ+',   fShiftLabel: 'x²',   row: 0, col: 0 },
  { id: 'recip',      label: '1/x',  fShiftLabel: 'yˣ',   row: 0, col: 1 },
  { id: 'sqrt',       label: '√x',                         row: 0, col: 2 },
  { id: 'log',        label: 'LOG',  fShiftLabel: '10ˣ',  row: 0, col: 3 },
  { id: 'ln',         label: 'LN',   fShiftLabel: 'eˣ',   row: 0, col: 4 },
  { id: '',           label: 'XEQ',                        row: 0, col: 5 },
  { id: '',           label: 'STO',                        row: 0, col: 6 },
  { id: '',           label: 'RCL',                        row: 0, col: 7 },
  { id: 'clx',        label: '←',                          row: 0, col: 8 },

  // Row 1 — trig row with double-wide ENTER (8 physical keys, 9 column-slots)
  { id: 'sin',     label: 'SIN',   fShiftLabel: 'ASIN', row: 1, col: 0 },
  { id: 'cos',     label: 'COS',   fShiftLabel: 'ACOS', row: 1, col: 1 },
  { id: 'tan',     label: 'TAN',   fShiftLabel: 'ATAN', row: 1, col: 2 },
  { id: 'rdn',     label: 'R↓',                         row: 1, col: 3 },
  { id: 'xy_swap', label: 'x↔y',                        row: 1, col: 4 },
  { id: 'enter',   label: 'ENTER',                       row: 1, col: 5, colSpan: 2 },
  { id: 'div',     label: '÷',                           row: 1, col: 7 },
  { id: 'mul',     label: '×',                           row: 1, col: 8 },

  // Row 2 — mode keys + digits 7/8/9 (9 keys)
  { id: 'user_mode',    label: 'USER',  row: 2, col: 0 },
  { id: '',             label: 'f',     row: 2, col: 1 },
  { id: '',             label: 'g',     row: 2, col: 2 },
  { id: '7',            label: '7',     row: 2, col: 3 },
  { id: '8',            label: '8',     row: 2, col: 4 },
  { id: '9',            label: '9',     row: 2, col: 5 },
  { id: 'minus',        label: '−',     row: 2, col: 6 },
  { id: 'prgm_mode',    label: 'PRGM',  row: 2, col: 7 },
  { id: 'alpha_toggle', label: 'ALPHA', row: 2, col: 8 },

  // Row 3 — entry keys + digits 4/5/6 (9 keys)
  { id: 'chs',  label: 'CHS',  row: 3, col: 0 },
  { id: 'e',    label: 'EEX',  row: 3, col: 1 },
  { id: '',     label: 'SST',  row: 3, col: 2 },
  { id: '4',    label: '4',    row: 3, col: 3 },
  { id: '5',    label: '5',    row: 3, col: 4 },
  { id: '6',    label: '6',    row: 3, col: 5 },
  { id: 'plus', label: '+',    row: 3, col: 6 },
  { id: '',     label: 'GTO',  row: 3, col: 7 },
  { id: '',     label: 'R/S',  row: 3, col: 8 },

  // Row 4 — digits 0/1/2/3 + special (9 keys)
  { id: '0',     label: '0',    row: 4, col: 0 },
  { id: '.',     label: '.',    row: 4, col: 1 },
  { id: '',      label: 'ON',   row: 4, col: 2 },
  { id: '1',     label: '1',    row: 4, col: 3 },
  { id: '2',     label: '2',    row: 4, col: 4 },
  { id: '3',     label: '3',    row: 4, col: 5 },
  { id: 'lastx', label: 'LSTx', row: 4, col: 6 },
  { id: 'clreg', label: 'CLRG', row: 4, col: 7 },
  { id: '',      label: 'BST',  row: 4, col: 8 },
];

function getKeyColor(key: KeyDef): string {
  if (key.row === 0) return '#3a2a1e';
  if (key.id === 'enter') return '#1a3a1a';
  if (
    ['user_mode', 'prgm_mode', 'alpha_toggle'].includes(key.id) ||
    (key.id === '' && (key.label === 'f' || key.label === 'g'))
  ) return '#d4c9b0';
  return '#1a1a1a';
}

interface KeyboardProps {
  onKey: (keyId: string) => void;
  busyRef: MutableRefObject<boolean>;
}

export function Keyboard({ onKey, busyRef }: KeyboardProps) {
  const [pressedKey, setPressedKey] = useState<string | null>(null);

  const handleKeyClick = (keyId: string) => {
    if (!keyId) return;                   // visual-only key guard (D-07, Pitfall 2)
    if (busyRef.current) return;         // debounce guard (D-13)
    setPressedKey(keyId);
    setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150);
    onKey(keyId);
  };

  return (
    <svg
      width="100%"
      viewBox="0 0 400 230"
      xmlns="http://www.w3.org/2000/svg"
      aria-label="HP-41C keyboard"
    >
      <rect width="400" height="230" fill="#3d2b1f" rx={6} />
      {KEY_DEFS.map(key => {
        const cs = key.colSpan ?? 1;
        const x = PAD + key.col * (KEY_W + GAP);
        const rectY = PAD + key.row * (FSHIFT_H + KEY_H + GAP) + FSHIFT_H;
        const w = cs * KEY_W + (cs - 1) * GAP;
        const isPressed = pressedKey === key.id && Boolean(key.id);
        const labelColor =
          ['user_mode', 'prgm_mode', 'alpha_toggle'].includes(key.id) ||
          (key.id === '' && (key.label === 'f' || key.label === 'g'))
            ? '#2a1a0a'
            : '#ffffff';
        return (
          <g
            key={`${key.row}-${key.col}`}
            onClick={() => handleKeyClick(key.id)}
            className={isPressed ? 'key key-pressed' : 'key'}
            style={{ pointerEvents: 'all' }}
          >
            {key.fShiftLabel && (
              <text
                x={x + w / 2}
                y={rectY - 2}
                textAnchor="middle"
                fill="#c8a400"
                fontSize={8}
              >
                {key.fShiftLabel}
              </text>
            )}
            <rect
              x={x}
              y={rectY}
              width={w}
              height={KEY_H}
              rx={3}
              ry={3}
              fill={getKeyColor(key)}
              stroke="#111"
              strokeWidth={0.5}
            />
            <text
              x={x + w / 2}
              y={rectY + KEY_H / 2 + 4}
              textAnchor="middle"
              fill={labelColor}
              fontSize={10}
              fontWeight="bold"
            >
              {key.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
}
