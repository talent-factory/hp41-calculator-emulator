import { useState, type MutableRefObject } from 'react';

const COLS = 5;
const KEY_W = 64;
const KEY_H = 44;
const GAP = 6;
const PAD = 10;
const TOP_GAP = 12;         // gap between top-row band and main grid
const TOP_CENTER_GAP = 96;  // visible gap between ON/USER and PRGM/ALPHA
const SHIFT_LABEL_H = 11;   // space reserved above each key for orange shift label
const ALPHA_LABEL_H = 11;   // space reserved below each key for blue alpha letter

const KEYBOARD_W = PAD * 2 + COLS * KEY_W + (COLS - 1) * GAP;
const TOP_ROW_H = KEY_H;
const GRID_ROW_H = SHIFT_LABEL_H + KEY_H + ALPHA_LABEL_H + GAP;
const KEYBOARD_H = PAD * 2 + TOP_ROW_H + TOP_GAP + 8 * GRID_ROW_H;

export type KeyDef = {
  id: string;                                  // primary op key id; '' = shift key (handled specially)
  label: string;                               // primary visible label
  shifted?: { id: string; label: string };    // shifted op + orange label above
  alphaChar?: string;                          // ALPHA-mode character (blue label below)
  row: number;                                 // 0 = top-row band, 1..8 = main grid rows
  col: number;                                 // 0..4 within row
  colSpan?: number;                            // default 1 (ENTER spans 2)
  variant?: 'top' | 'shift' | 'enter';        // styling hint
};

// Top row — separated from main grid by gap. ON/USER on the left, PRGM/ALPHA on the right.
// Row 0 reserved for top row. No shift/alpha labels on top-row buttons.
const TOP_ROW: KeyDef[] = [
  { id: '',             label: 'ON',    row: 0, col: 0, variant: 'top' },
  { id: 'user_mode',    label: 'USER',  row: 0, col: 1, variant: 'top' },
  { id: 'prgm_mode',    label: 'PRGM',  row: 0, col: 3, variant: 'top' },
  { id: 'alpha_toggle', label: 'ALPHA', row: 0, col: 4, variant: 'top' },
];

// Main grid — 5 columns × 8 rows. ENTER spans 2 columns in row 4.
// Stub labels (ASN, CATALOG, BEEP, P→R, R→P, x=y?, x≤y?, x>y?, x=0?, π, VIEW, SF, CF, FS?)
// resolve via key_map.rs stub branch and surface a toast.
const MAIN_GRID: KeyDef[] = [
  // Row 1 — math
  { id: 'sigma_plus', label: 'Σ+',  shifted: { id: 'sigma_minus', label: 'Σ−' },   alphaChar: 'A', row: 1, col: 0 },
  { id: 'recip',      label: '1/x', shifted: { id: 'ypow',        label: 'yˣ' },   alphaChar: 'B', row: 1, col: 1 },
  { id: 'sqrt',       label: '√x',  shifted: { id: 'sq',          label: 'x²' },   alphaChar: 'C', row: 1, col: 2 },
  { id: 'log',        label: 'LOG', shifted: { id: 'tenpow',      label: '10ˣ' }, alphaChar: 'D', row: 1, col: 3 },
  { id: 'ln',         label: 'LN',  shifted: { id: 'exp',         label: 'eˣ' },   alphaChar: 'E', row: 1, col: 4 },
  // Row 2 — trig + stack
  { id: 'xge_y',      label: 'x≥y', shifted: { id: 'cl_sigma_stat', label: 'CLΣ' }, alphaChar: 'F', row: 2, col: 0 },
  { id: 'rdn',        label: 'R↓',  shifted: { id: 'pct_change',  label: '%' },    alphaChar: 'G', row: 2, col: 1 },
  { id: 'sin',        label: 'SIN', shifted: { id: 'asin',        label: 'SIN⁻¹' }, alphaChar: 'H', row: 2, col: 2 },
  { id: 'cos',        label: 'COS', shifted: { id: 'acos',        label: 'COS⁻¹' }, alphaChar: 'I', row: 2, col: 3 },
  { id: 'tan',        label: 'TAN', shifted: { id: 'atan',        label: 'TAN⁻¹' }, alphaChar: 'J', row: 2, col: 4 },
  // Row 3 — program
  { id: 'shift',      label: '',    row: 3, col: 0, variant: 'shift' },
  { id: 'xeq_prompt', label: 'XEQ', shifted: { id: 'asn',     label: 'ASN' }, alphaChar: 'K', row: 3, col: 1 },
  { id: 'sto_prompt', label: 'STO', shifted: { id: 'lbl_prompt', label: 'LBL' }, alphaChar: 'L', row: 3, col: 2 },
  { id: 'rcl_prompt', label: 'RCL', shifted: { id: 'gto_prompt', label: 'GTO' }, alphaChar: 'M', row: 3, col: 3 },
  { id: 'sst',        label: 'SST', shifted: { id: 'bst',     label: 'BST' }, row: 3, col: 4 },
  // Row 4 — entry (ENTER spans 2)
  { id: 'enter',      label: 'ENTER↑', shifted: { id: 'catalog', label: 'CATALOG' }, alphaChar: 'N', row: 4, col: 0, colSpan: 2, variant: 'enter' },
  { id: 'chs',        label: 'CHS', shifted: { id: 'isg_prompt', label: 'ISG' }, alphaChar: 'O', row: 4, col: 2 },
  { id: 'e',          label: 'EEX', shifted: { id: 'rtn',         label: 'RTN' }, alphaChar: 'P', row: 4, col: 3 },
  { id: 'clx_or_a',   label: '←',   shifted: { id: 'clx_or_a',    label: 'CL X/A' }, row: 4, col: 4 },
  // Row 5 — operator − + digits 7/8/9
  { id: 'minus',      label: '−', shifted: { id: 'x_eq_y_prompt', label: 'x=y?' }, alphaChar: 'Q', row: 5, col: 0 },
  { id: '7',          label: '7', shifted: { id: 'sf_prompt',     label: 'SF' },   alphaChar: 'R', row: 5, col: 1 },
  { id: '8',          label: '8', shifted: { id: 'cf_prompt',     label: 'CF' },   alphaChar: 'S', row: 5, col: 2 },
  { id: '9',          label: '9', shifted: { id: 'fs_prompt',     label: 'FS?' },  alphaChar: 'T', row: 5, col: 3 },
  // Row 6 — operator + + digits 4/5/6
  { id: 'plus',       label: '+', shifted: { id: 'x_le_y_prompt', label: 'x≤y?' }, alphaChar: 'U', row: 6, col: 0 },
  { id: '4',          label: '4', shifted: { id: 'beep',          label: 'BEEP' }, alphaChar: 'V', row: 6, col: 1 },
  { id: '5',          label: '5', shifted: { id: 'polar_to_rect', label: 'P→R' },  alphaChar: 'W', row: 6, col: 2 },
  { id: '6',          label: '6', shifted: { id: 'rect_to_polar', label: 'R→P' },  alphaChar: 'X', row: 6, col: 3 },
  // Row 7 — operator × + digits 1/2/3
  { id: 'mul',        label: '×', shifted: { id: 'x_gt_y_prompt', label: 'x>y?' }, alphaChar: 'Y', row: 7, col: 0 },
  { id: '1',          label: '1', shifted: { id: 'fix_prompt',    label: 'FIX' },  alphaChar: 'Z', row: 7, col: 1 },
  { id: '2',          label: '2', shifted: { id: 'sci_prompt',    label: 'SCI' },  alphaChar: '=', row: 7, col: 2 },
  { id: '3',          label: '3', shifted: { id: 'eng_prompt',    label: 'ENG' },  alphaChar: '?', row: 7, col: 3 },
  // Row 8 — operator ÷ + 0 . R/S
  { id: 'div',        label: '÷', shifted: { id: 'x_eq_0_prompt', label: 'x=0?' }, alphaChar: ':', row: 8, col: 0 },
  { id: '0',          label: '0', shifted: { id: 'pi',            label: 'π' },    alphaChar: ' ', row: 8, col: 1 },
  { id: '.',          label: '.', shifted: { id: 'lastx',         label: 'LAST X' }, alphaChar: ',', row: 8, col: 2 },
  { id: 'r_s',        label: 'R/S', shifted: { id: 'view',        label: 'VIEW' }, row: 8, col: 3 },
];

const KEY_DEFS: KeyDef[] = [...TOP_ROW, ...MAIN_GRID];

function keyPosition(key: KeyDef): { x: number; y: number; w: number; h: number } {
  const cs = key.colSpan ?? 1;
  const w = cs * KEY_W + (cs - 1) * GAP;
  const h = KEY_H;

  if (key.row === 0) {
    // Top row: ON/USER on left (cols 0-1), PRGM/ALPHA on right (cols 3-4) with center gap.
    const leftWidth = 2 * KEY_W + GAP;
    const xLeftStart = PAD;
    const xRightStart = PAD + leftWidth + TOP_CENTER_GAP;
    const xCols = [xLeftStart, xLeftStart + KEY_W + GAP, 0, xRightStart, xRightStart + KEY_W + GAP];
    return { x: xCols[key.col], y: PAD, w, h };
  }

  const gridRow = key.row - 1; // rows 1..8 → indices 0..7
  const x = PAD + key.col * (KEY_W + GAP);
  const y = PAD + TOP_ROW_H + TOP_GAP + gridRow * GRID_ROW_H + SHIFT_LABEL_H;
  return { x, y, w, h };
}

function isCreamKey(key: KeyDef): boolean {
  return (
    ['user_mode', 'prgm_mode', 'alpha_toggle'].includes(key.id) ||
    (key.id === '' && (key.label === 'f' || key.label === 'g'))
  );
}

function getKeyGrad(key: KeyDef): string {
  if (key.row === 0) return 'url(#grad-row0)';
  if (key.id === 'enter') return 'url(#grad-enter)';
  if (isCreamKey(key)) return 'url(#grad-cream)';
  return 'url(#grad-dark)';
}

interface KeyboardProps {
  onKey: (keyId: string) => void;
  busyRef: MutableRefObject<boolean>;
}

export function Keyboard({ onKey, busyRef }: KeyboardProps) {
  const [pressedKey, setPressedKey] = useState<string | null>(null);

  const handleKeyClick = (keyId: string) => {
    if (!keyId) return;
    if (busyRef.current) return;
    setPressedKey(keyId);
    setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150);
    onKey(keyId);
  };

  return (
    <svg
      width="100%"
      viewBox={`0 0 ${KEYBOARD_W} ${KEYBOARD_H}`}
      xmlns="http://www.w3.org/2000/svg"
      aria-label="HP-41C keyboard"
    >
      <defs>
        <linearGradient id="body-grad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#1a1a1a" />
          <stop offset="100%" stopColor="#000000" />
        </linearGradient>

        {/* Key cap gradients — lighter at top, darker at bottom (convex 3D look) */}
        <linearGradient id="grad-dark" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#303030" />
          <stop offset="60%"  stopColor="#181818" />
          <stop offset="100%" stopColor="#080808" />
        </linearGradient>
        <linearGradient id="grad-row0" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#6a4830" />
          <stop offset="60%"  stopColor="#3a2418" />
          <stop offset="100%" stopColor="#1e1008" />
        </linearGradient>
        <linearGradient id="grad-enter" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#346034" />
          <stop offset="60%"  stopColor="#1a3a1a" />
          <stop offset="100%" stopColor="#0a180a" />
        </linearGradient>
        <linearGradient id="grad-cream" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#ede8d4" />
          <stop offset="60%"  stopColor="#c8bd98" />
          <stop offset="100%" stopColor="#a89870" />
        </linearGradient>

        {/* Inner bevel highlight — white gradient fading out (top of key only) */}
        <linearGradient id="bevel-hi" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#ffffff" stopOpacity="0.22" />
          <stop offset="100%" stopColor="#ffffff" stopOpacity="0" />
        </linearGradient>
      </defs>

      {/* Calculator body */}
      <rect width={KEYBOARD_W} height={KEYBOARD_H} fill="url(#body-grad)" rx={10} />
      <rect width={KEYBOARD_W} height={14} fill="url(#bevel-hi)" rx={10} opacity={0.4} />

      {KEY_DEFS.map(key => {
        const cs = key.colSpan ?? 1;
        const x = PAD + key.col * (KEY_W + GAP);
        const rectY = PAD + key.row * (FSHIFT_H + KEY_H + GAP) + FSHIFT_H;
        const w = cs * KEY_W + (cs - 1) * GAP;
        const isPressed = pressedKey === key.id && Boolean(key.id);
        const cream = isCreamKey(key);
        const labelColor = cream ? '#1e1008' : '#ffffff';

        return (
          <g
            key={`${key.row}-${key.col}`}
            onClick={() => handleKeyClick(key.id)}
            className={isPressed ? 'key key-pressed' : 'key'}
          >
            {/* f-shift label above key */}
            {key.fShiftLabel && (
              <text
                x={x + w / 2}
                y={rectY - 2}
                textAnchor="middle"
                fill="#d4a800"
                fontSize={8}
                fontWeight="bold"
              >
                {key.fShiftLabel}
              </text>
            )}

            {/* Key shadow (separate rect under the cap for depth) */}
            <rect
              x={x + 1}
              y={rectY + 2}
              width={w}
              height={KEY_H}
              rx={4}
              ry={4}
              fill="#000000"
              opacity={0.45}
            />

            {/* Key cap with gradient fill */}
            <rect
              x={x}
              y={rectY}
              width={w}
              height={KEY_H}
              rx={4}
              ry={4}
              fill={getKeyGrad(key)}
              stroke={cream ? '#806848' : '#0a0a0a'}
              strokeWidth={0.8}
            />

            {/* Bevel highlight — top portion of key cap */}
            <rect
              x={x + 1}
              y={rectY + 1}
              width={w - 2}
              height={KEY_H / 2}
              rx={3}
              ry={3}
              fill="url(#bevel-hi)"
              className="key-bevel"
            />

            {/* Primary label */}
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
