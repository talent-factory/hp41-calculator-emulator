import { useState, type MutableRefObject } from 'react';

const COLS = 5;
const KEY_W = 68;
const KEY_H = 44;
const GAP = 8;
const PAD = 10;
const TOP_GAP = 12;         // gap between top-row band and main grid
const SHIFT_LABEL_H = 11;   // space reserved above each key for orange shift label
const ALPHA_LABEL_H = 11;   // space reserved below each key for blue alpha letter

// Rows 5-8 (operator + digits) are 4 wider keys spanning the same total
// width as 5 normal-col keys + 4 gaps in the upper rows. Width derives so
// `4*WIDE_KEY_W + 3*GAP == 5*KEY_W + 4*GAP`, i.e. four wide cols visually
// fill the right edge of the keyboard (no empty col 4 below ENTER).
const WIDE_KEY_W = (COLS * KEY_W + GAP) / 4;

const KEYBOARD_W = PAD * 2 + COLS * KEY_W + (COLS - 1) * GAP;
const TOP_ROW_H = KEY_H;
const GRID_ROW_H = SHIFT_LABEL_H + KEY_H + ALPHA_LABEL_H + GAP;
const KEYBOARD_H = PAD * 2 + TOP_ROW_H + TOP_GAP + 8 * GRID_ROW_H;

export type KeyDef = {
  // Primary op key id resolved via key_map::resolve(), OR the empty string
  // for unwired top-row buttons (currently only ON). The SHIFT key is
  // identified by variant: 'shift' and uses id: 'shift' as a sentinel that
  // App.tsx short-circuits before any dispatch.
  id: string;
  label: string;                               // primary visible label
  shifted?: { id: string; label: string };    // shifted op + orange label above
  alphaChar?: string;                          // ALPHA-mode character (blue label below)
  row: number;                                 // 0 = top-row band, 1..8 = main grid rows
  col: number;                                 // 0..4 within row
  colSpan?: number;                            // default 1 (ENTER spans 2)
  variant?: 'top' | 'shift' | 'enter';        // styling hint
  // Phase 26 D-26.9 / W9 — HP-41 hardware key code (row×10 + col, 1-indexed)
  // per hp41-cli/src/keys.rs::keycode_to_hp41_code. Used by USER-mode
  // per-key relabel: when annunciators.user is true and an ASN entry in
  // CalcStateView.user_keymap has this keyCode, the keycap renders the
  // ASN'd label INSTEAD of the primary label.
  //
  // W9 RESOLUTION: these are HARDCODED LITERALS from the CLI canonical
  // mapping, NOT computed `row * 10 + col`. The GUI 5×8 grid uses 0-indexed
  // cols and a row numbering that does NOT match HP-41 hardware (e.g. SIN
  // at GUI (row 2, col 2) is HP-41 code 25 from CLI, but `2*10+2=22` is
  // STO, not SIN). Any drift from the CLI mapping fails the W9 sentinel
  // parity test in Keyboard.test.tsx before the USER overlay can mis-label.
  //
  // Variants 'top' and 'shift' and empty-id entries leave this `undefined`
  // (USER overlay does not relabel them). Some keys (CHS, CL X/A, etc.)
  // have no unambiguous CLI-canonical mapping and are also left undefined.
  keyCode?: number;
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
//
// W9 keyCode literals: HARDCODED from hp41-cli/src/keys.rs canonical mapping.
// See KeyDef.keyCode doc above. Keys without a CLI mapping (chs, clx_or_a,
// xge_y) leave keyCode undefined — USER-mode relabel will skip them.
const MAIN_GRID: KeyDef[] = [
  // Row 1 — math. HP-41 hardware row 1: Σ+(11), 1/x(12), √x(13), LOG(14), LN(15).
  { id: 'sigma_plus', label: 'Σ+',  shifted: { id: 'sigma_minus', label: 'Σ−' },   alphaChar: 'A', row: 1, col: 0, keyCode: 11 },
  { id: 'recip',      label: '1/x', shifted: { id: 'ypow',        label: 'yˣ' },   alphaChar: 'B', row: 1, col: 1, keyCode: 12 },
  { id: 'sqrt',       label: '√x',  shifted: { id: 'sq',          label: 'x²' },   alphaChar: 'C', row: 1, col: 2, keyCode: 13 },
  { id: 'log',        label: 'LOG', shifted: { id: 'tenpow',      label: '10ˣ' }, alphaChar: 'D', row: 1, col: 3, keyCode: 14 },
  { id: 'ln',         label: 'LN',  shifted: { id: 'exp',         label: 'eˣ' },   alphaChar: 'E', row: 1, col: 4, keyCode: 15 },
  // Row 2 — trig + stack. HP-41 hardware row 2: XEQ(21), STO(22), RCL(23), R↓(24), SIN(25).
  // The GUI puts XEQ on row 3 (program row), so row 2 col 0 (xge_y) has no
  // direct CLI mapping — leave keyCode undefined.
  { id: 'xge_y',      label: 'x≥y', shifted: { id: 'cl_sigma_stat', label: 'CLΣ' }, alphaChar: 'F', row: 2, col: 0 },
  { id: 'rdn',        label: 'R↓',  shifted: { id: 'pct_change',  label: '%' },    alphaChar: 'G', row: 2, col: 1, keyCode: 24 },
  { id: 'sin',        label: 'SIN', shifted: { id: 'asin',        label: 'SIN⁻¹' }, alphaChar: 'H', row: 2, col: 2, keyCode: 25 },
  { id: 'cos',        label: 'COS', shifted: { id: 'acos',        label: 'COS⁻¹' }, alphaChar: 'I', row: 2, col: 3, keyCode: 34 },
  { id: 'tan',        label: 'TAN', shifted: { id: 'atan',        label: 'TAN⁻¹' }, alphaChar: 'J', row: 2, col: 4, keyCode: 35 },
  // Row 3 — program. HP-41 hardware row 3: R/S(31), SST(32), GTO(33), COS(34), TAN(35).
  // GUI moves COS/TAN to row 2, SST to row 3 col 4; XEQ/STO/RCL/RCL use row-2 codes.
  { id: 'shift',      label: '',    row: 3, col: 0, variant: 'shift' },
  { id: 'xeq_prompt', label: 'XEQ', shifted: { id: 'asn',     label: 'ASN' }, alphaChar: 'K', row: 3, col: 1, keyCode: 21 },
  { id: 'sto_prompt', label: 'STO', shifted: { id: 'lbl_prompt', label: 'LBL' }, alphaChar: 'L', row: 3, col: 2, keyCode: 22 },
  { id: 'rcl_prompt', label: 'RCL', shifted: { id: 'gto_prompt', label: 'GTO' }, alphaChar: 'M', row: 3, col: 3, keyCode: 23 },
  { id: 'sst',        label: 'SST', shifted: { id: 'bst',     label: 'BST' }, row: 3, col: 4, keyCode: 32 },
  // Row 4 — entry (ENTER spans 2). HP-41 hardware row 4: USER(41), f(42), g(43), ENTER(44/84), ÷(45).
  // CLI keys.rs encodes ENTER as 84 (row 8 col 4 in some HP-41C variants).
  // CHS has no unambiguous CLI mapping (hardware row 4 col 5 = 45 conflicts
  // with div's mapping); leave keyCode undefined. clx_or_a likewise.
  { id: 'enter',      label: 'ENTER↑', shifted: { id: 'catalog', label: 'CATALOG' }, alphaChar: 'N', row: 4, col: 0, colSpan: 2, variant: 'enter', keyCode: 84 },
  { id: 'chs',        label: 'CHS', shifted: { id: 'isg_prompt', label: 'ISG' }, alphaChar: 'O', row: 4, col: 2 },
  { id: 'e',          label: 'EEX', shifted: { id: 'rtn',         label: 'RTN' }, alphaChar: 'P', row: 4, col: 3, keyCode: 83 },
  { id: 'clx_or_a',   label: '←',   shifted: { id: 'clx_or_a',    label: 'CL X/A' }, row: 4, col: 4 },
  // Row 5 — operator − + digits 7/8/9. HP-41 hardware row 5: 7(51), 8(52), 9(53), ×(54).
  // CLI keys.rs encodes − as 64 (HP-41 row 6 col 4) — the wide-row operator
  // column 0 in the GUI doesn't align to HP-41's column 5; we use the CLI's
  // canonical key→code mapping (where '-' → 64), preserving the GETKEY contract.
  { id: 'minus',      label: '−', shifted: { id: 'x_eq_y_prompt', label: 'x=y?' }, alphaChar: 'Q', row: 5, col: 0, keyCode: 64 },
  { id: '7',          label: '7', shifted: { id: 'sf_prompt',     label: 'SF' },   alphaChar: 'R', row: 5, col: 1, keyCode: 51 },
  { id: '8',          label: '8', shifted: { id: 'cf_prompt',     label: 'CF' },   alphaChar: 'S', row: 5, col: 2, keyCode: 52 },
  { id: '9',          label: '9', shifted: { id: 'fs_prompt',     label: 'FS?' },  alphaChar: 'T', row: 5, col: 3, keyCode: 53 },
  // Row 6 — operator + + digits 4/5/6. HP-41 hardware row 6: 4(61), 5(62), 6(63), −(64).
  // CLI maps '+' → 74 (hardware row 7 col 4).
  { id: 'plus',       label: '+', shifted: { id: 'x_le_y_prompt', label: 'x≤y?' }, alphaChar: 'U', row: 6, col: 0, keyCode: 74 },
  { id: '4',          label: '4', shifted: { id: 'beep',          label: 'BEEP' }, alphaChar: 'V', row: 6, col: 1, keyCode: 61 },
  { id: '5',          label: '5', shifted: { id: 'polar_to_rect', label: 'P→R' },  alphaChar: 'W', row: 6, col: 2, keyCode: 62 },
  { id: '6',          label: '6', shifted: { id: 'rect_to_polar', label: 'R→P' },  alphaChar: 'X', row: 6, col: 3, keyCode: 63 },
  // Row 7 — operator × + digits 1/2/3. HP-41 hardware row 7: 1(71), 2(72), 3(73), +(74).
  // CLI maps '*' → 54 (hardware row 5 col 4).
  { id: 'mul',        label: '×', shifted: { id: 'x_gt_y_prompt', label: 'x>y?' }, alphaChar: 'Y', row: 7, col: 0, keyCode: 54 },
  { id: '1',          label: '1', shifted: { id: 'fix_prompt',    label: 'FIX' },  alphaChar: 'Z', row: 7, col: 1, keyCode: 71 },
  { id: '2',          label: '2', shifted: { id: 'sci_prompt',    label: 'SCI' },  alphaChar: '=', row: 7, col: 2, keyCode: 72 },
  { id: '3',          label: '3', shifted: { id: 'eng_prompt',    label: 'ENG' },  alphaChar: '?', row: 7, col: 3, keyCode: 73 },
  // Row 8 — operator ÷ + 0 . R/S. HP-41 hardware row 8: 0(81), .(82), EEX(83), ENTER(84/85).
  // CLI maps '/' → 45 (hardware row 4 col 5). R/S → 31 per CLI Phase 19 binding.
  { id: 'div',        label: '÷', shifted: { id: 'x_eq_0_prompt', label: 'x=0?' }, alphaChar: ':', row: 8, col: 0, keyCode: 45 },
  { id: '0',          label: '0', shifted: { id: 'pi',            label: 'π' },    alphaChar: ' ', row: 8, col: 1, keyCode: 81 },
  { id: '.',          label: '.', shifted: { id: 'lastx',         label: 'LAST X' }, alphaChar: ',', row: 8, col: 2, keyCode: 82 },
  { id: 'r_s',        label: 'R/S', shifted: { id: 'view',        label: 'VIEW' }, row: 8, col: 3, keyCode: 31 },
];

// Exported so Vitest can assert KEY_DEFS keyCode parity with the hp41-cli
// canonical mapping (W9 drift-catch). Treat as readonly outside this file.
export const KEY_DEFS: readonly KeyDef[] = [...TOP_ROW, ...MAIN_GRID];

function keyPosition(key: KeyDef): { x: number; y: number; w: number; h: number } {
  const cs = key.colSpan ?? 1;
  const h = KEY_H;

  if (key.row === 0) {
    // Top row keys at cols 0, 1, 3, 4 align with main-grid cols 0, 1, 3, 4
    // using the same x formula — col 2 is intentionally skipped (no key)
    // and the visual centre gap is exactly one missing key + two gaps.
    const w = cs * KEY_W + (cs - 1) * GAP;
    const x = PAD + key.col * (KEY_W + GAP);
    return { x, y: PAD, w, h };
  }

  const gridRow = key.row - 1; // rows 1..8 → indices 0..7
  const y = PAD + TOP_ROW_H + TOP_GAP + gridRow * GRID_ROW_H + SHIFT_LABEL_H;

  if (key.row >= 5) {
    // Wide rows (5-8): 4 cols of WIDE_KEY_W. col 0 is the operator
    // (−/+/×/÷), cols 1-3 are the digits. No col 4 — the rightmost key
    // ends at the same x as col 4 of the upper rows.
    const w = cs * WIDE_KEY_W + (cs - 1) * GAP;
    const x = PAD + key.col * (WIDE_KEY_W + GAP);
    return { x, y, w, h };
  }

  const w = cs * KEY_W + (cs - 1) * GAP;
  const x = PAD + key.col * (KEY_W + GAP);
  return { x, y, w, h };
}

function getKeyGrad(key: KeyDef, shiftActive: boolean): string {
  if (key.variant === 'shift') {
    return shiftActive ? 'url(#grad-shift-active)' : 'url(#grad-shift-idle)';
  }
  if (key.variant === 'enter') return 'url(#grad-enter)';
  return 'url(#grad-dark)';
}

export interface KeyboardProps {
  onKey: (key: KeyDef) => void;          // App.tsx decides mode + dispatches
  busyRef: MutableRefObject<boolean>;
  shiftActive: boolean;
  alphaActive: boolean;
  // Phase 26 D-26.9 — USER-mode per-key relabel. When `userActive=true` and
  // a KEY_DEFS entry has `keyCode` matching an ASN entry in `userKeymap`,
  // the keycap renders the ASN'd label INSTEAD of the primary label.
  // Wired by Task 3; the props are declared in Task 2 so App.tsx can
  // pass them now without a TypeScript error.
  userActive?: boolean;
  userKeymap?: ReadonlyArray<[number, string]>;
}

export function Keyboard({
  onKey,
  busyRef,
  shiftActive,
  alphaActive,
  userActive = false,
  userKeymap = [],
}: KeyboardProps) {
  const [pressedKey, setPressedKey] = useState<string | null>(null);

  // Phase 26 D-26.9 — USER-mode relabel resolver. Returns the ASN'd label
  // for a given keyCode, or null if no ASN entry exists for this key.
  // Defense-in-depth length cap of 7 chars: HP-41 ASN labels are up to 6
  // chars per the ALPHA pack convention; the slice caps the visual blast
  // radius of a malicious or accidental long label. React's default text-
  // node rendering escapes content automatically — `<script>` becomes
  // literal text in the SVG <text> element, NOT an injected script tag
  // (T-26-03-04 mitigation; covered by Keyboard.test.tsx XSS test).
  function resolveUserLabel(keyCode: number | undefined): string | null {
    if (!userActive || keyCode === undefined) return null;
    const entry = userKeymap.find(([code]) => code === keyCode);
    if (!entry) return null;
    return entry[1].slice(0, 7);
  }

  const handleKeyClick = (key: KeyDef) => {
    if (!key.id) return;                 // ON and other unwired keys
    if (busyRef.current) return;
    setPressedKey(key.id);
    setTimeout(() => setPressedKey(prev => (prev === key.id ? null : prev)), 150);
    onKey(key);
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
        <linearGradient id="grad-enter" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#346034" />
          <stop offset="60%"  stopColor="#1a3a1a" />
          <stop offset="100%" stopColor="#0a180a" />
        </linearGradient>
        <linearGradient id="grad-shift-idle" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#d68a1c" />
          <stop offset="60%"  stopColor="#b06811" />
          <stop offset="100%" stopColor="#7a4708" />
        </linearGradient>
        <linearGradient id="grad-shift-active" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#ffb742" />
          <stop offset="60%"  stopColor="#f5a423" />
          <stop offset="100%" stopColor="#c97d10" />
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
        const { x, y, w, h } = keyPosition(key);
        const isPressed = pressedKey === key.id && Boolean(key.id);
        const isShiftKey = key.variant === 'shift';
        const labelColor = '#e8e8e8';
        const labelKey = `${key.row}-${key.col}`;

        // The SHIFT key has no labels — solid orange cap only.
        if (isShiftKey) {
          return (
            <g
              key={labelKey}
              onClick={() => handleKeyClick(key)}
              className={
                shiftActive ? 'key key-shift-active' : isPressed ? 'key key-pressed' : 'key'
              }
              // Phase 26 Plan 04 Task 3 — test-only locator for integration
              // tests in App.test.tsx (`container.querySelector(
              // '[data-key-id="shift"]')`). React passes data-* attributes
              // through to the DOM without listener attachment, so no
              // production effect.
              data-key-id={key.id || undefined}
            >
              <rect x={x + 1} y={y + 2} width={w} height={h} rx={5} ry={5} fill="#000" opacity={0.45} />
              <rect x={x} y={y} width={w} height={h} rx={5} ry={5}
                    fill={getKeyGrad(key, shiftActive)}
                    stroke="#3a2208" strokeWidth={0.8} />
              <rect x={x + 1} y={y + 1} width={w - 2} height={h / 2} rx={4} ry={4}
                    fill="url(#bevel-hi)" />
            </g>
          );
        }

        return (
          <g
            key={labelKey}
            onClick={() => handleKeyClick(key)}
            className={isPressed ? 'key key-pressed' : 'key'}
            // Phase 26 Plan 04 Task 3 — test-only locator (see SHIFT branch above).
            data-key-id={key.id || undefined}
          >
            {/* Orange shift label above (skip on top-row keys) */}
            {key.shifted && (
              <text
                x={x + w / 2}
                y={y - 2}
                textAnchor="middle"
                fill="#d68a1c"
                fontSize={11}
                fontWeight="bold"
              >
                {key.shifted.label}
              </text>
            )}

            {/* Shadow under cap */}
            <rect x={x + 1} y={y + 2} width={w} height={h} rx={5} ry={5} fill="#000" opacity={0.45} />

            {/* Cap */}
            <rect x={x} y={y} width={w} height={h} rx={5} ry={5}
                  fill={getKeyGrad(key, shiftActive)}
                  stroke="#0a0a0a" strokeWidth={0.8} />

            {/* Bevel highlight */}
            <rect x={x + 1} y={y + 1} width={w - 2} height={h / 2} rx={4} ry={4}
                  fill="url(#bevel-hi)" className="key-bevel" />

            {/* Primary label — D-26.9: replace with USER-mode ASN'd label
                when annunciators.user is active AND this key has a keyCode
                that matches an entry in userKeymap. Text-node rendering
                escapes content (T-26-03-04 XSS mitigation). */}
            <text
              x={x + w / 2}
              y={y + h / 2 + 5}
              textAnchor="middle"
              fill={labelColor}
              fontSize={key.variant === 'enter' ? 13 : 14}
              fontWeight="bold"
            >
              {resolveUserLabel(key.keyCode) ?? key.label}
            </text>

            {/* Blue alpha letter below (skip on top-row, shift, and ENTER) */}
            {key.alphaChar && key.variant !== 'top' && (
              <text
                x={x + w / 2}
                y={y + h + 11}
                textAnchor="middle"
                fill={alphaActive ? '#7fb9e0' : '#5b8fb9'}
                fontSize={11}
                fontWeight="bold"
              >
                {key.alphaChar === ' ' ? 'SPACE' : key.alphaChar}
              </text>
            )}
          </g>
        );
      })}
    </svg>
  );
}
