// Phase 26 W3 — exhaustive list of all KEY_DEFS primary + shifted ids.
//
// MUST match the union of primary+shifted ids in hp41-gui/src/Keyboard.tsx
// KEY_DEFS. When adding a key to KEY_DEFS, update both:
//   1. This file (the TS source of truth for the W3 audit)
//   2. The hand-mirrored array in
//      hp41-gui/src-tauri/src/key_map.rs::tests::test_keyboard_skin_ids_resolve_or_are_modal_openers
//
// The Rust-side test asserts every id either resolves via key_map::resolve,
// hits the modal-opener stub arm, or is in HANDLED_OUTSIDE_RESOLVE. Drift in
// either direction surfaces at `just gui-check` time.

export const KEY_DEFS_PRIMARY_IDS = [
  // TOP_ROW (4 entries — "" for ON filtered separately).
  'user_mode',
  'prgm_mode',
  'alpha_toggle',
  // MAIN_GRID row 1 (math).
  'sigma_plus',
  'recip',
  'sqrt',
  'log',
  'ln',
  // MAIN_GRID row 2 (trig + stack).
  'xge_y',
  'rdn',
  'sin',
  'cos',
  'tan',
  // MAIN_GRID row 3 (program — SHIFT is at col 0, no primary id).
  'xeq_prompt',
  'sto_prompt',
  'rcl_prompt',
  'sst',
  // MAIN_GRID row 4 (entry).
  'enter',
  'chs',
  // 'e' is a digit-input id handled outside resolve.
  'clx_or_a',
  // MAIN_GRID rows 5-8 (digits + operators). The numeric ids 0-9 and '.'
  // are handled outside resolve (commands.rs digit branch).
  'minus',
  'plus',
  'mul',
  'div',
  'r_s',
] as const;

export const KEY_DEFS_SHIFTED_IDS = [
  // MAIN_GRID row 1
  'sigma_minus',
  'ypow',
  'sq',
  'tenpow',
  'exp',
  // MAIN_GRID row 2
  'cl_sigma_stat',
  'pct_change',
  'asin',
  'acos',
  'atan',
  // MAIN_GRID row 3
  'asn',
  'lbl_prompt',
  'gto_prompt',
  'bst',
  // MAIN_GRID row 4
  'catalog',
  'isg_prompt',
  'rtn',
  'clx_or_a',
  // MAIN_GRID row 5
  'x_eq_y_prompt',
  'sf_prompt',
  'cf_prompt',
  'fs_prompt',
  // MAIN_GRID row 6
  'x_le_y_prompt',
  'beep',
  'polar_to_rect',
  'rect_to_polar',
  // MAIN_GRID row 7
  'x_gt_y_prompt',
  'fix_prompt',
  'sci_prompt',
  'eng_prompt',
  // MAIN_GRID row 8
  'x_eq_0_prompt',
  'pi',
  'lastx',
  'view',
] as const;

/// Ids handled by App.tsx OUTSIDE key_map::resolve — special routes through
/// dedicated Tauri commands or alpha-aware branches.
export const KEY_DEFS_HANDLED_OUTSIDE_RESOLVE = [
  'sst', // → sst_step Tauri command
  'bst', // → bst_step Tauri command
  'r_s', // → run_stop Tauri command
  'shift', // → frontend-only shiftActive toggle
  '', // → ON (unwired)
  'clx_or_a', // → branches on annunciators.alpha into clx | alpha_clear
  'e', // → digit-entry path (commands.rs digit branch)
] as const;

export type KeyDefId =
  | (typeof KEY_DEFS_PRIMARY_IDS)[number]
  | (typeof KEY_DEFS_SHIFTED_IDS)[number]
  | (typeof KEY_DEFS_HANDLED_OUTSIDE_RESOLVE)[number];
