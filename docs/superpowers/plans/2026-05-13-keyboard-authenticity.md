# Keyboard Authenticity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `hp41-gui` keyboard with an authentic HP-41C 5×7-grid layout featuring a single one-shot SHIFT key, three-label keys (shift/primary/alpha), and explicit stub-error handling for v2.2 backlog functions.

**Architecture:** Frontend-only state for SHIFT (no IPC schema change). `App.tsx` owns mode resolution; `Keyboard.tsx` renders. `key_map.rs` gains resolvers for already-implemented core ops plus a stub branch for v2.2 functions. Zero changes to `hp41-core`, `hp41-cli`, persistence, or display panel.

**Tech Stack:** React 19 + TypeScript + Vite (no test runner configured for frontend — `tsc --noEmit` only), Tauri v2.11 (Rust commands with `cargo test`), inline SVG keyboard, CSS in `App.css`.

**Spec:** [docs/superpowers/specs/2026-05-13-keyboard-authenticity-design.md](../specs/2026-05-13-keyboard-authenticity-design.md)

**Phase id:** Phase 19 (under v2.1 milestone)

**Test commands cheat sheet:**
- Rust: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- TypeScript typecheck: `cd hp41-gui && npx tsc --noEmit`
- Full GUI CI: `just gui-ci`
- Interactive dev: `just gui-dev`

**Commit convention:** Use `/git-workflow:commit --with-skills` per CLAUDE.md (German Emoji Conventional Commits in English text).

---

## Task 1: Define new `KeyDef` type and replace `KEY_DEFS` with full 35-key data

**Why:** The current `KeyDef` carries only `label` + optional `fShiftLabel`. The new model needs structured shift action (id + label) and an alpha character. Doing the type + data in one task keeps the file compiling.

**Files:**
- Modify: `hp41-gui/src/Keyboard.tsx:1-72` (type + KEY_DEFS array)

- [ ] **Step 1: Replace the `KeyDef` type and `KEY_DEFS` array**

Open `hp41-gui/src/Keyboard.tsx`. Replace lines 3-72 with:

```typescript
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
```

- [ ] **Step 2: Run typecheck**

```bash
cd hp41-gui && npx tsc --noEmit
```

Expected: Errors will appear in the rest of `Keyboard.tsx` (the old layout code references `fShiftLabel`, etc.). Note them; they will be fixed in Tasks 2-5. **As long as the error count is finite and the messages all point inside `Keyboard.tsx`, proceed.** If errors leak into `App.tsx`, stop and fix.

- [ ] **Step 3: Commit**

```bash
git add hp41-gui/src/Keyboard.tsx
```

Use `/git-workflow:commit --with-skills` with subject:
`🎹 refactor(gui): introduce authentic HP-41C KeyDef model and 35-key table`

Body explains: replaces `fShiftLabel` cosmetic-only field with structured `shifted` action; adds `alphaChar` for ALPHA mode; introduces top-row variant. SVG render code intentionally left broken in this commit — Tasks 2-5 finish the rewrite.

---

## Task 2: Rewrite SVG layout math for 5-column grid + top-row band

**Why:** Old layout used 9 columns and uniform `FSHIFT_H + KEY_H + GAP` rows. New layout has a separate top-row band, then 8 main-grid rows.

**Files:**
- Modify: `hp41-gui/src/Keyboard.tsx:12-16` (constants) and `:104-150` (viewBox + body rects)

- [ ] **Step 1: Update layout constants**

Replace lines 12-16:

```typescript
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
```

- [ ] **Step 2: Update SVG viewBox and body rect**

Locate the `<svg>` element (currently `viewBox="0 0 400 230"`). Change to:

```tsx
<svg
  width="100%"
  viewBox={`0 0 ${KEYBOARD_W} ${KEYBOARD_H}`}
  xmlns="http://www.w3.org/2000/svg"
  aria-label="HP-41C keyboard"
>
```

Update the body rect (currently `<rect width="400" height="230" ...>`) to:

```tsx
<rect width={KEYBOARD_W} height={KEYBOARD_H} fill="url(#body-grad)" rx={10} />
<rect width={KEYBOARD_W} height={14} fill="url(#bevel-hi)" rx={10} opacity={0.4} />
```

- [ ] **Step 3: Update body gradient to black**

In `<defs>`, replace the `body-grad` definition (lines 113-116 originally):

```tsx
<linearGradient id="body-grad" x1="0" y1="0" x2="0" y2="1">
  <stop offset="0%"   stopColor="#1a1a1a" />
  <stop offset="100%" stopColor="#000000" />
</linearGradient>
```

- [ ] **Step 4: Implement positioning helper**

Add this helper function just below `KEY_DEFS`:

```typescript
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
```

- [ ] **Step 5: Run typecheck**

```bash
cd hp41-gui && npx tsc --noEmit
```

Expected: still has errors in the old render loop (which Task 3 replaces). Proceed.

- [ ] **Step 6: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`🎹 refactor(gui): switch keyboard layout to 5-col grid with top-row band`

Body: black body gradient, new positioning helper, separated top-row band with center gap. Render loop still broken — Task 3 finalizes it.

---

## Task 3: Replace the render loop with three-label rendering and variant handling

**Why:** Old loop emitted one `<rect>` + optional `fShiftLabel` text. New loop must emit primary, shifted (top), alpha (bottom), and handle three variants (`top`, `shift`, `enter`, default).

**Files:**
- Modify: `hp41-gui/src/Keyboard.tsx:152-231` (the entire `KEY_DEFS.map(...)` render block)

- [ ] **Step 1: Add the new gradient definitions**

In `<defs>`, **add** these gradients (keep existing `grad-dark`, `grad-enter`, `bevel-hi`, drop `grad-row0` and `grad-cream` from later steps):

```tsx
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
```

After this step, the old `grad-row0` and `grad-cream` definitions can be deleted — no keys reference them anymore. Verify by removing lines 124-138 (the two unused gradients).

- [ ] **Step 2: Replace `isCreamKey` and `getKeyGrad`**

Replace lines 74-86 with:

```typescript
function getKeyGrad(key: KeyDef, shiftActive: boolean): string {
  if (key.variant === 'shift') {
    return shiftActive ? 'url(#grad-shift-active)' : 'url(#grad-shift-idle)';
  }
  if (key.variant === 'enter') return 'url(#grad-enter)';
  return 'url(#grad-dark)';
}
```

- [ ] **Step 3: Extend the `KeyboardProps` interface**

Replace the existing `KeyboardProps` interface with:

```typescript
export interface KeyboardProps {
  onKey: (key: KeyDef) => void;          // App.tsx decides mode + dispatches
  busyRef: MutableRefObject<boolean>;
  shiftActive: boolean;
  alphaActive: boolean;
}
```

- [ ] **Step 4: Replace the render loop**

Replace the entire `KEY_DEFS.map(...)` block (lines 152-231) with:

```tsx
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
    >
      {/* Orange shift label above (skip on top-row keys) */}
      {key.shifted && (
        <text
          x={x + w / 2}
          y={y - 2}
          textAnchor="middle"
          fill="#d68a1c"
          fontSize={9}
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

      {/* Primary label */}
      <text
        x={x + w / 2}
        y={y + h / 2 + 5}
        textAnchor="middle"
        fill={labelColor}
        fontSize={key.variant === 'enter' ? 13 : 14}
        fontWeight="bold"
      >
        {key.label}
      </text>

      {/* Blue alpha letter below (skip on top-row, shift, and ENTER) */}
      {key.alphaChar && key.variant !== 'top' && (
        <text
          x={x + w / 2}
          y={y + h + 10}
          textAnchor="middle"
          fill={alphaActive ? '#7fb9e0' : '#5b8fb9'}
          fontSize={9}
          fontWeight="bold"
        >
          {key.alphaChar === ' ' ? 'SPACE' : key.alphaChar}
        </text>
      )}
    </g>
  );
})}
```

- [ ] **Step 5: Update `handleKeyClick` signature**

Replace the existing `handleKeyClick` (around line 96):

```typescript
const handleKeyClick = (key: KeyDef) => {
  if (!key.id) return;                 // ON and other unwired keys
  if (busyRef.current) return;
  setPressedKey(key.id);
  setTimeout(() => setPressedKey(prev => (prev === key.id ? null : prev)), 150);
  onKey(key);
};
```

- [ ] **Step 6: Update component signature**

Replace the `Keyboard` function signature line (around line 93):

```typescript
export function Keyboard({ onKey, busyRef, shiftActive, alphaActive }: KeyboardProps) {
```

- [ ] **Step 7: Run typecheck**

```bash
cd hp41-gui && npx tsc --noEmit
```

Expected: One error in `App.tsx` — `<Keyboard onKey={handleClick} busyRef={busyRef} />` is missing two new required props. **App.tsx is wired up in Task 5** — temporarily fix by passing literal `false` for both new props:

In `App.tsx:173`, change to:
```tsx
<Keyboard onKey={(_def) => {}} busyRef={busyRef} shiftActive={false} alphaActive={false} />
```

Note: this leaves the keyboard click-deaf until Task 5; that's intentional. Run typecheck again — should now pass.

- [ ] **Step 8: Visual smoke test**

```bash
just gui-dev
```

In the Tauri window, verify:
- Body is black, not brown
- Top-row buttons are clearly separated with a center gap
- 5-column main grid is visible with 8 rows
- Each key shows three labels (orange top, white center, blue bottom) except SHIFT (no labels, orange) and top-row (single label, no shift/alpha)
- SHIFT key is visible in row 3 col 0, solid orange

Close the dev session. Do not try clicking yet — handler is stubbed.

- [ ] **Step 9: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`🎹 feat(gui): render three-label authentic HP-41C keys (visual only)`

Body: three-label key rendering (orange shift / white primary / blue alpha), distinct visual variants for SHIFT and ENTER, click handler temporarily stubbed pending Task 5 state wiring.

---

## Task 4: Add `key_map.rs` resolvers for newly-clickable core ops (TDD)

**Why:** The new layout exposes ~14 already-implemented core ops that had no clickable surface. Before wiring App.tsx click handling, ensure the backend can resolve every new id. TDD: tests first.

**Files:**
- Modify: `hp41-gui/src-tauri/src/key_map.rs` (the `resolve` match arms and the test module)

- [ ] **Step 1: Audit which ids in `KEY_DEFS` need new resolvers**

The KEY_DEFS ids needing verification:
- Already in `resolve()`: `sigma_plus`, `sigma_minus`, `cl_sigma_stat`, `recip`, `sqrt`, `log`, `ln`, `rdn`, `pct_change`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `enter`, `chs`, `rtn`, `lastx`, `plus`, `minus`, `mul`, `div`, `user_mode`, `prgm_mode`, `alpha_toggle`
- Already in `resolve()` but no current KEY_DEFS user: `int`, `mean`, `sdev`, `lr`, `yhat`, `corr`, `hms_to_h`, `h_to_hms`, `hms_add`, `hms_sub`
- **NEW resolvers needed**: `sq`, `ypow`, `tenpow`, `exp`, `xge_y`
- **Digit/decimal/EEX** (special-cased in `dispatch_op`): `0`–`9`, `.`, `e` — no change.
- **Prompts/modal-openers** (frontend-only ids, NEVER reach `dispatch_op`): `sto_prompt`, `rcl_prompt`, `xeq_prompt`, `gto_prompt`, `lbl_prompt`, `isg_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt`. Modal handling is **out of scope** — these are stubbed in Task 5.
- **Special-routed in App.tsx** (NOT via `dispatch_op`): `sst`, `bst`, `r_s`, `clx_or_a`, `shift`. Handled in Task 6.
- **Stub-error path** (Task 5): `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`.

- [ ] **Step 2: Write failing tests for new named ops**

Open `hp41-gui/src-tauri/src/key_map.rs`. In the `tests` module, add this test at the end:

```rust
#[test]
fn test_new_named_op_resolvers() {
    // Newly exposed ops that became clickable in Phase 19.
    assert_eq!(resolve("sq").unwrap(), Op::Sq);
    assert_eq!(resolve("ypow").unwrap(), Op::YPow);
    assert_eq!(resolve("tenpow").unwrap(), Op::TenPow);
    assert_eq!(resolve("exp").unwrap(), Op::Exp);
    assert_eq!(resolve("xge_y").unwrap(), Op::Test(hp41_core::ops::TestKind::XGeY));
}
```

- [ ] **Step 3: Run tests to verify failure**

```bash
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_new_named_op_resolvers
```

Expected: FAIL with `unknown key: sq` (or similar) for at least one assertion.

- [ ] **Step 4: Add the resolvers**

In `resolve()` in `key_map.rs`, add these arms within the `// ── Unary math ─` block (after the existing `recip => Ok(Op::Recip)` line):

```rust
"sq" => Ok(Op::Sq),
"ypow" => Ok(Op::YPow),
"tenpow" => Ok(Op::TenPow),
"exp" => Ok(Op::Exp),
```

Add a new section just before `// ── Print ─` for comparison ops:

```rust
// ── Comparison (Test variants — keyboard-accessible without label arg) ───
"xge_y" => Ok(Op::Test(hp41_core::ops::TestKind::XGeY)),
```

- [ ] **Step 5: Run tests to verify pass**

```bash
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_new_named_op_resolvers
```

Expected: PASS.

Also re-run the full key_map test suite:

```bash
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --lib key_map
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`⚙️ feat(gui): map new authentic-keyboard ops (sq, ypow, tenpow, exp, xge_y)`

Body: enables click access to Sq, YPow, TenPow, Exp, and Test(XGeY) — ops already in core but previously dead behind the cosmetic `fShiftLabel` field.

---

## Task 5: Add stub-error branch in `key_map.rs` for v2.2-backlog functions (TDD)

**Why:** Spec decision D-5 — show all reference labels but return an explicit `GuiError` for not-yet-implemented functions. Honors D-07 (no silent discard).

**Files:**
- Modify: `hp41-gui/src-tauri/src/key_map.rs` (resolve match arms + tests)

- [ ] **Step 1: Write failing test for stub branch**

Add to the `tests` module in `key_map.rs`:

```rust
#[test]
fn test_stub_error_for_v22_backlog_ops() {
    // Spec D-5: these ids resolve to an explicit GuiError, not Ok(Op).
    let stub_ids = [
        "pi", "polar_to_rect", "rect_to_polar",
        "beep", "asn", "catalog", "view",
    ];
    for id in stub_ids {
        let err = resolve(id).unwrap_err();
        assert!(
            err.message.contains("planned for a future phase"),
            "id {id:?} expected stub message, got: {}",
            err.message
        );
        assert!(
            err.message.contains(id),
            "id {id:?} expected message to contain id, got: {}",
            err.message
        );
    }
}

#[test]
fn test_modal_prompt_ids_are_stubs_for_now() {
    // Out-of-scope for Phase 19: modal-opener prompts also stub until v2.2 modal infra lands.
    // Frontend MUST NOT send these to dispatch_op (App.tsx routes them to in-progress modals
    // or shows a not-yet-implemented toast). The backend stub is defense-in-depth.
    let prompt_ids = [
        "sto_prompt", "rcl_prompt", "xeq_prompt", "gto_prompt", "lbl_prompt",
        "isg_prompt", "fix_prompt", "sci_prompt", "eng_prompt",
        "sf_prompt", "cf_prompt", "fs_prompt",
        "x_eq_y_prompt", "x_le_y_prompt", "x_gt_y_prompt", "x_eq_0_prompt",
    ];
    for id in prompt_ids {
        assert!(
            resolve(id).is_err(),
            "prompt id {id:?} must not resolve to an Op without its modal"
        );
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

```bash
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_stub_error_for_v22_backlog_ops test_modal_prompt_ids_are_stubs_for_now
```

Expected: both tests FAIL — the existing fallback returns `unknown key: pi`, which contains the id but not "planned for a future phase".

- [ ] **Step 3: Add the stub branch**

In `key_map.rs`, find the `match key_id {` block in `resolve()`. **Before** the final `_ => resolve_parameterized(key_id),` line, add:

```rust
// ── v2.2 backlog (intentionally not yet implemented — D-5) ───────────
"pi" | "polar_to_rect" | "rect_to_polar"
| "beep" | "asn" | "catalog" | "view" => Err(GuiError {
    message: format!("'{key_id}' is planned for a future phase"),
}),
```

The prompt ids (`sto_prompt`, `rcl_prompt`, etc.) intentionally **fall through** to the existing `_ => resolve_parameterized` arm, where they will fail to parse as compound keys and return the existing `unknown key: <id>` error. That satisfies `test_modal_prompt_ids_are_stubs_for_now` without needing a separate match arm.

- [ ] **Step 4: Run tests to verify pass**

```bash
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
```

Expected: all tests PASS.

- [ ] **Step 5: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`⚙️ feat(gui): stub-error branch for v2.2-backlog keyboard ops`

Body: explicit GuiError with "planned for a future phase" message for π, P→R, R→P, BEEP, ASN, CATALOG, VIEW. Modal-prompt ids fall through to existing unknown-key error pending v2.2 modal infrastructure. Honors D-07 (never silent discard).

---

## Task 6: Add `run_stop` Tauri command for R/S key

**Why:** Spec Edge Case 3 — R/S becomes a primary key for the first time in GUI. It is not a single `Op` variant; it toggles `CalcState.running` to start/halt program execution. We add a dedicated Tauri command symmetric with `sst_step`/`bst_step` (zero changes to `hp41-core`).

**Files:**
- Modify: `hp41-gui/src-tauri/src/commands.rs`
- Modify: `hp41-gui/src-tauri/src/lib.rs` (`generate_handler!` registration)

- [ ] **Step 1: Inspect existing `sst_step` to mirror its pattern**

```bash
grep -n "sst_step\|bst_step\|tauri::command" hp41-gui/src-tauri/src/commands.rs | head -20
```

Read enough surrounding context to understand: AppState locking pattern, `CalcStateView` construction, `print_buffer` drain. (Do this manually — do not invent.)

- [ ] **Step 2: Add the `run_stop` command**

In `hp41-gui/src-tauri/src/commands.rs`, just after the `bst_step` command definition, add:

```rust
/// Toggle program run/stop state. If the program is running, halt it.
/// If halted (or never started), start running from the current PC.
///
/// Mirrors `sst_step`/`bst_step` shape — never goes through `dispatch_op`
/// because R/S is not a single Op variant (it toggles `CalcState.running`).
#[tauri::command]
pub fn run_stop(state: tauri::State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    if calc.running {
        calc.running = false;
    } else {
        // Resume execution from current pc. The actual stepping loop already
        // exists in hp41-core::ops::program::run_program; spawning it here
        // would block the IPC thread. v2.1 scope: just toggle the flag.
        // Programs in GUI advance via SST until v2.2 introduces a tick thread.
        calc.running = true;
    }
    Ok(CalcStateView::from(&*calc))
}
```

**Important:** verify the `CalcState` field is actually named `running`. If the actual field name differs (e.g., `is_running`), adjust both occurrences. Run:

```bash
grep -n "running\|is_running" hp41-core/src/state.rs | head -10
```

Use whichever name appears in `CalcState`. **If neither exists, stop and ask** — the spec assumed `running`; the plan must match reality.

- [ ] **Step 3: Register the command in `lib.rs`**

In `hp41-gui/src-tauri/src/lib.rs`, find the `tauri::generate_handler!` invocation. Add `commands::run_stop` to the list. Example (your existing list may differ):

```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    commands::sst_step,
    commands::bst_step,
    commands::run_stop,   // ← new
])
```

- [ ] **Step 4: Create the Tauri permission file**

Per CLAUDE.md: "Tauri v2.11 does NOT auto-generate `allow-<cmd>` permissions for inline commands."

Create `hp41-gui/src-tauri/permissions/run-stop.toml`:

```toml
[[permission]]
identifier = "allow-run-stop"
description = "Allow toggling program run/stop state."
commands.allow = ["run_stop"]
```

- [ ] **Step 5: Add the permission to `capabilities/default.json`**

Read `hp41-gui/src-tauri/capabilities/default.json`. In the `permissions` array, add the string `"allow-run-stop"` next to the other `allow-*` entries (the array order is conventional but not significant).

- [ ] **Step 6: Run `cargo check` to regenerate the permission registry**

```bash
just gui-check
```

Expected: builds clean. If the permission file is malformed Tauri will reject it with a clear error pointing to the toml file.

- [ ] **Step 7: Run the full backend test suite**

```bash
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
```

Expected: all green.

- [ ] **Step 8: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`⚙️ feat(gui): add run_stop Tauri command for R/S key`

Body: symmetric with sst_step/bst_step; toggles `CalcState.running`; permission registered in `permissions/run-stop.toml` and `capabilities/default.json`. Resolves the one open question flagged in the spec (Edge Case 3).

---

## Task 7: Wire `shiftActive` state, click routing, and special-key handling in `App.tsx`

**Why:** This is where the spec's D-4 priority logic lives. Everything before this task is plumbing; this task makes the keyboard actually work in shift / alpha / normal modes.

**Files:**
- Modify: `hp41-gui/src/App.tsx` (state, handler, Keyboard prop wiring)

- [ ] **Step 1: Import `KeyDef` from Keyboard**

At the top of `App.tsx`, ensure:

```typescript
import { Keyboard, type KeyDef } from './Keyboard';
import { invoke } from '@tauri-apps/api/core';  // already imported, verify
```

- [ ] **Step 2: Add `shiftActive` and `toastMsg` state**

Inside the `App` component function, near the existing `useState` calls, add:

```typescript
const [shiftActive, setShiftActive] = useState(false);
const [toastMsg, setToastMsg] = useState<string | null>(null);
```

- [ ] **Step 3: Add a toast auto-dismiss effect**

```typescript
useEffect(() => {
  if (!toastMsg) return;
  const t = setTimeout(() => setToastMsg(null), 2000);
  return () => clearTimeout(t);
}, [toastMsg]);
```

- [ ] **Step 4: Replace the existing `handleClick` with the new mode-aware router**

Locate the current `handleClick` (single-arg `(keyId: string) => ...`). Replace it with:

```typescript
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
    // Special routes: SST/BST/R/S go to dedicated commands.
    if (effectiveId === 'sst') {
      const view = await invoke<CalcStateView>('sst_step');
      setCalcState(view);
    } else if (effectiveId === 'bst') {
      const view = await invoke<CalcStateView>('bst_step');
      setCalcState(view);
    } else if (effectiveId === 'r_s') {
      const view = await invoke<CalcStateView>('run_stop');
      setCalcState(view);
    } else if (effectiveId === 'clx_or_a') {
      // CL X/A — branch on alpha mode at click time.
      const targetId = alphaOn ? 'alpha_clear' : 'clx';
      const view = await invoke<CalcStateView>('dispatch_op', { keyId: targetId });
      setCalcState(view);
    } else {
      const view = await invoke<CalcStateView>('dispatch_op', { keyId: effectiveId });
      setCalcState(view);
    }
  } catch (err) {
    const msg = typeof err === 'object' && err && 'message' in err
      ? String((err as { message: unknown }).message)
      : String(err);
    setToastMsg(msg);
  } finally {
    if (consumesShift) setShiftActive(false);
    busyRef.current = false;
  }
}, [calcState, shiftActive]);
```

**Note:** `CalcStateView` is the existing TypeScript type. If your file doesn't yet import it, locate the existing usage (probably already imported from `./types` or similar) and reuse. If the current `App.tsx` types `CalcStateView` inline, leave that alone.

- [ ] **Step 5: Update the physical-keyboard handler (`resolveKeyId`) to support SHIFT and ESC**

Locate the existing `useEffect` that attaches `keydown` listeners (around line 100-110 in current `App.tsx`). Inside the keydown handler, **add before** the existing `resolveKeyId` call:

```typescript
if (e.key === 'Escape') {
  setShiftActive(false);
  return;
}
if (e.key === 'Tab') {
  e.preventDefault();
  setShiftActive(prev => !prev);
  return;
}
```

- [ ] **Step 6: Update the `<Keyboard>` JSX to pass the new props**

Find the `<Keyboard ... />` element (currently `<Keyboard onKey={handleClick} busyRef={busyRef} shiftActive={false} alphaActive={false} />` from Task 3 stub). Replace with:

```tsx
<Keyboard
  onKey={handleClick}
  busyRef={busyRef}
  shiftActive={shiftActive}
  alphaActive={calcState?.annunciators.alpha ?? false}
/>
```

- [ ] **Step 7: Extend the annunciator list to include SHIFT**

Find the line (originally `App.tsx:140`):
```typescript
const annunciatorNames = ['user', 'prgm', 'alpha', 'rad', 'grad'] as const;
```

Replace with:
```typescript
const annunciatorNames = ['user', 'shift', 'prgm', 'alpha', 'rad', 'grad'] as const;
```

Locate the rendering of annunciators. The existing code reads from `calcState?.annunciators[name]`. For `'shift'`, that won't exist — add a small derivation just before the map:

```tsx
const annunciators: Record<typeof annunciatorNames[number], boolean> = {
  user:  calcState?.annunciators.user  ?? false,
  shift: shiftActive,
  prgm:  calcState?.annunciators.prgm  ?? false,
  alpha: calcState?.annunciators.alpha ?? false,
  rad:   calcState?.annunciators.rad   ?? false,
  grad:  calcState?.annunciators.grad  ?? false,
};
```

Then update the annunciator map to read from `annunciators[name]` instead of `calcState?.annunciators[name]`.

- [ ] **Step 8: Render the toast overlay**

In the JSX, just inside the top-level wrapper (so it overlays the display), add:

```tsx
{toastMsg && (
  <div className="toast" role="status">{toastMsg}</div>
)}
```

- [ ] **Step 9: Run typecheck**

```bash
cd hp41-gui && npx tsc --noEmit
```

Expected: PASS. If `useCallback` or `useEffect` aren't imported at the top, add them to the existing `import { useState, ... } from 'react'` line.

- [ ] **Step 10: Manual smoke test**

```bash
just gui-dev
```

Verify in order:
1. Click `7` → display shows `7`. (primary)
2. Click `SHIFT` → SHIFT key glows orange and SHIFT annunciator appears at top.
3. Click `7` → `SF` shift function fires → toast appears with `'sf_prompt' …` (since sf_prompt isn't yet a real op). SHIFT annunciator turns off. ✓ one-shot.
4. Press `Tab` → SHIFT toggles on. Press `Esc` → SHIFT turns off. ✓
5. Click `ALPHA` → ALPHA annunciator on. Click `√x` → display alpha buffer shows `C` (alpha mode override). ✓
6. Click `√x` again with ALPHA off → display computes √x of current X. ✓
7. Click `R/S` → no crash; running flag toggles (visible in stack panel if implemented, otherwise no-op). ✓
8. Click an orange-only stub like `π` (SHIFT then `0`) → toast appears with `'pi' is planned for a future phase`. ✓
9. Spam-click SHIFT-π five times → only one toast visible at a time (newest replaces older). ✓

If any step fails, stop and debug before committing.

- [ ] **Step 11: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`🎹 feat(gui): wire one-shot SHIFT prefix, ALPHA routing, and toast surface`

Body: implements spec D-4 click resolution priority (SHIFT → ALPHA → shifted → primary); adds Tab/Esc keyboard support for SHIFT; routes SST/BST/R-S to dedicated Tauri commands; surfaces GuiError as 2s toast; CL X/A picks clx vs alpha_clear at click time.

---

## Task 8: Add CSS for SHIFT active state, toast overlay, and annunciator extension

**Why:** Functional state exists after Task 7 — this task makes the visual feedback look right.

**Files:**
- Modify: `hp41-gui/src/App.css`

- [ ] **Step 1: Inspect current App.css structure**

```bash
grep -n "^\.\|^\#" hp41-gui/src/App.css | head -40
```

Note the existing class names and conventions. Add new rules in the same style.

- [ ] **Step 2: Add SHIFT-active styling**

Append to `App.css`:

```css
/* SHIFT key visual feedback when shift is armed (one-shot prefix). */
.key.key-shift-active rect {
  filter: brightness(1.18) drop-shadow(0 0 4px #f5a423);
}

/* Pressed-key animation already exists; keep behavior for shift key too,
   but suppress when shift-active class is already applied. */
.key.key-shift-active.key-pressed rect {
  transform: scale(0.97);
}
```

- [ ] **Step 3: Add toast overlay styling**

Append:

```css
.toast {
  position: absolute;
  top: 60px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 14px;
  border-radius: 4px;
  background: rgba(20, 20, 20, 0.92);
  color: #f5a423;
  font: 11px/1.4 monospace;
  letter-spacing: 0.02em;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.5);
  pointer-events: none;
  z-index: 50;
  animation: toast-fade 2s ease-out forwards;
}

@keyframes toast-fade {
  0%   { opacity: 0; transform: translate(-50%, -4px); }
  10%  { opacity: 1; transform: translate(-50%, 0); }
  85%  { opacity: 1; }
  100% { opacity: 0; transform: translate(-50%, -2px); }
}
```

- [ ] **Step 4: Add SHIFT annunciator color**

If the existing annunciator CSS uses one color for all, add a specific orange tone for SHIFT:

```css
.annunciator-shift {
  color: #d68a1c;
}
```

And confirm the annunciator render code applies a class like `annunciator-${name}` to each item. If it doesn't, add one in `App.tsx` next to where annunciators are rendered (it's a one-line change).

- [ ] **Step 5: Visual verification**

```bash
just gui-dev
```

Re-check the Task 7 smoke-test steps. SHIFT key should now visibly glow when armed; toast should fade in/out cleanly.

- [ ] **Step 6: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`💄 style(gui): SHIFT armed glow, toast overlay, annunciator color`

---

## Task 9: Update CLAUDE.md with v2.1 keyboard invariants

**Why:** CLAUDE.md is the project's authoritative architecture log. Future Claude sessions need to know about: the one-shot SHIFT mechanic, the three-label KeyDef structure, the stub-error pattern, and the `run_stop` command. Per project convention, settled architecture decisions go in the "Settled Architecture Decisions" section.

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Read the current v2.0 additions section**

```bash
grep -n "v2.0 additions" CLAUDE.md
```

Locate the "v2.0 additions (Tauri GUI)" block.

- [ ] **Step 2: Add a new "v2.1 additions" section immediately after the v2.0 block**

Insert (preserving Markdown style of the file):

```markdown
### v2.1 additions (Keyboard authenticity, Phase 19)

- **Authentic 5×7 layout**: `hp41-gui/src/Keyboard.tsx` renders 4 top-row mode buttons + a 5×8 main grid (ENTER 2-wide) with one orange SHIFT key — replaces the prior 8-col landscape layout with cosmetic `f`/`g` keys.
- **Three-label `KeyDef`**: each key carries `id`/`label` (primary), optional `shifted: { id, label }` (orange, above), and `alphaChar` (blue, below). Old `fShiftLabel` field is gone.
- **One-shot SHIFT is frontend-only**: `shiftActive: boolean` lives in `App.tsx`; never appears in `CalcState`, `CalcStateView`, or IPC. After a shifted op fires, `setShiftActive(false)` resets. `Tab` and clicking SHIFT toggle; `Esc` cancels. SHIFT joins the annunciator list as a frontend-derived value.
- **Click resolution priority** (D-4): SHIFT key → ALPHA + alphaChar → shiftActive + shifted.id → primary.id. ALPHA mode overrides SHIFT (known divergence from real HP-41 — v2.2 deferred).
- **Stub-error pattern** (D-5): `key_map::resolve` returns `Err(GuiError { message: "'<id>' is planned for a future phase" })` for `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`. Frontend surfaces a 2s toast overlay. NEVER silently discard — D-07 holds.
- **Modal-prompt ids** (`sto_prompt`, `fix_prompt`, `x_eq_y_prompt`, etc.) are KEY_DEFS-only frontend ids; they intentionally fail `resolve()` so a mis-routed click triggers an explicit unknown-key error. v2.2 will route these to actual modals.
- **`run_stop` Tauri command**: new dedicated command symmetric with `sst_step`/`bst_step`, toggles `CalcState.running`. R/S key is now click-reachable for the first time (was `id: ''` in v2.0). Permission file: `permissions/run-stop.toml`. Frontend special-routes id `r_s` to this command, NOT `dispatch_op`.
- **`SST`/`BST`/`CL X/A` special routes**: `App.tsx`'s click handler must route these ids to dedicated paths — `sst`/`bst` → `sst_step`/`bst_step`, `clx_or_a` → `clx` or `alpha_clear` depending on `annunciators.alpha`. Adding a new such key in the future requires updating the special-route block in the handler.
- **No core/CLI changes**: Phase 19 is hp41-gui only. SC-4 invariant preserved. Save-file backward compat unchanged.
```

- [ ] **Step 3: Update the "Status" section at top of CLAUDE.md**

Find the existing:
```markdown
**Status:**
- v1.0 CLI shipped 2026-05-08 — ...
- v1.1 CLI Feature Completeness shipped 2026-05-09 — ...
- v2.0 Tauri GUI shipped 2026-05-10 — ...
```

Add a v2.1 line (replace `<date>` with the actual ship date when shipping):
```markdown
- v2.1 Keyboard Authenticity shipped <date> — Phase 19, 9 tasks (5-col layout, one-shot SHIFT, three-label keys, run_stop command, stub-error pattern)
```

If shipping isn't done yet (typical when Task 9 runs before final verification), insert:
```markdown
- v2.1 Keyboard Authenticity (in progress) — Phase 19
```

- [ ] **Step 4: Update Phase history**

Find the "Phase history" subsection. Add:
```markdown
- v2.1 (19): Keyboard Authenticity
```

- [ ] **Step 5: Update "Key Files" — GUI table**

In the existing "GUI (`hp41-gui`):" table, update the `hp41-gui/src/Keyboard.tsx` row to:

```markdown
| `hp41-gui/src/Keyboard.tsx` | Authentic 5×8 grid + top-row band; `KEY_DEFS` with three-label model (primary + shifted + alphaChar); SHIFT key variant. |
```

Add a new row for `run_stop`:
```markdown
| `hp41-gui/src-tauri/src/commands.rs` (`run_stop`) | Toggles `CalcState.running`; symmetric with sst_step/bst_step; reaches R/S key. |
```

- [ ] **Step 6: Commit**

Use `/git-workflow:commit --with-skills` with subject:
`📚 docs: record v2.1 keyboard-authenticity architecture decisions`

Body: documents one-shot SHIFT frontend-only state, three-label KeyDef, stub-error pattern, run_stop command, and special-route block — so future sessions have the invariants in `CLAUDE.md`.

---

## Task 10: Final verification — full CI + manual click-through

**Why:** Confirms the whole phase ships green and matches the spec's success criteria.

**Files:** none (verification only)

- [ ] **Step 1: Full GUI CI**

```bash
just gui-ci
```

Expected: PASS. This runs:
- `npm install`
- `npx tsc --noEmit` (frontend typecheck)
- `cargo test` (Rust unit tests including new key_map tests)
- `cargo build --release` (production build)

If any step fails, fix and re-run.

- [ ] **Step 2: Confirm zero changes to hp41-core and hp41-cli**

```bash
git diff --stat develop..HEAD -- hp41-core hp41-cli
```

Expected: **empty output**. If anything appears, stop — Phase 19 must not touch core/CLI.

- [ ] **Step 3: Confirm SC-4 invariant**

```bash
grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/
```

Expected: **empty output** (the strict pattern from CLAUDE.md SC-4 invariant note). The lenient pattern still matches `op_display_name`, which is allowed (display formatter, not calculator logic).

- [ ] **Step 4: Manual E2E click-through checklist**

`just gui-dev`, then walk the keyboard. For each row, click every key in three modes:

| Mode | Activation | Expected behavior |
|------|-----------|-------------------|
| Default | (initial) | Primary op fires |
| SHIFT | Click SHIFT first | Shifted op fires, SHIFT auto-disarms |
| ALPHA | Click ALPHA toggle | Alpha char appends; SHIFT click is ignored |

Spec table (in `2026-05-13-keyboard-authenticity-design.md`, Layout Specification section) lists every per-key expectation. Walk it row by row. A `⚠️ stub` shifted op should produce a toast, not a state change.

- [ ] **Step 5: Visual diff against reference image**

Open the user-provided reference image side-by-side with the running GUI. Check:
- 5 columns in the math/trig/program rows ✓
- 4 columns in digit/operator rows (right column empty) ✓
- ENTER spans 2 columns ✓
- Solid orange SHIFT key in row 3 col 0 ✓
- Top-row gap between USER and PRGM ✓
- Orange labels above, white labels in center, blue letters below ✓
- Black body, not brown ✓

- [ ] **Step 6: Capture screenshot for project history (optional)**

If the project tracks UI screenshots:
```bash
mkdir -p .planning/milestones/v2.1/screenshots
# Take a screenshot of the running GUI in default/shift/alpha modes.
```

- [ ] **Step 7: Final commit (only if step 6 created files)**

Use `/git-workflow:commit --with-skills` with subject:
`📸 docs: snapshot v2.1 keyboard in default/shift/alpha modes`

If step 6 was skipped, no commit needed for Task 10.

---

## Out of scope — v2.2 backlog (reference only, NOT to be implemented in Phase 19)

These are explicitly deferred. They will become their own phases.

- `Op::Pi`, `Op::PolarToRect`, `Op::RectToPolar` (core math additions)
- `Op::Beep` (sound output — needs platform abstraction)
- `Op::Asn`, `Op::Catalog`, `Op::View` (user-key assignment and catalog menus)
- Flag system: `flags: u64` in `CalcState`, `Op::Sf(u8)`, `Op::Cf(u8)`, `Op::Fs(u8)` with a 2-digit modal
- Compare-with-label modals for `x=y?` / `x≤y?` / `x>y?` / `x=0?`
- 1-digit modal for FIX/SCI/ENG shift access
- Modal infrastructure for `STO`/`RCL`/`XEQ`/`GTO`/`LBL`/`ISG` from the GUI (currently bound to CLI flow only)
- SHIFT + ALPHA combined-mode handling (real HP-41 quirk)
- True async run loop for R/S (currently just toggles the flag — actual stepping deferred to a tick thread)
