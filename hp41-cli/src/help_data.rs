//! Static help data for the HP-41 function reference overlay (D-18).
//!
//! Format: (key_binding, hp41_op_name, description)
//! Category headers: key="" op="" desc="=== Category ===" for visual separation in Table.
//!
//! This is the SINGLE SOURCE OF TRUTH for key-to-operation descriptions.
//! Keys derived from KEY_REF_TABLE in keys.rs; descriptions added here.
//! ≥50 entries covering all keyboard-accessible HP-41 operations.

/// (key_binding, hp41_op_name, description) — ≥50 entries, 10 categories.
pub const HELP_DATA: &[(&str, &str, &str)] = &[
    // ── Stack ─────────────────────────────────────────────────────────────────
    ("",        "",          "=== Stack ==="),
    ("Enter",   "ENTER",     "Lift stack and duplicate X into Y"),
    ("Bksp",    "CLX",       "Clear X register (entry cancel)"),
    ("n",       "CHS",       "Change sign of X (negate)"),
    ("r",       "R↓",        "Roll stack down: X←Y, Y←Z, Z←T, T←X"),
    ("x",       "X⟷Y",      "Swap X and Y registers"),
    ("l",       "LASTX",     "Recall last X (value before last operation)"),
    // ── Arithmetic ────────────────────────────────────────────────────────────
    ("",        "",          "=== Arithmetic ==="),
    ("+",       "+",         "Add: X ← Y + X, drop stack"),
    ("-",       "-",         "Subtract: X ← Y − X, drop stack"),
    ("*",       "×",         "Multiply: X ← Y × X, drop stack"),
    ("/",       "÷",         "Divide: X ← Y ÷ X, drop stack"),
    ("I",       "1/x",       "Reciprocal of X"),
    ("s",       "√x",        "Square root of X"),
    ("W",       "x²",        "Square of X"),
    ("Y",       "Yˣ",        "Y raised to power X"),
    // ── Trig ──────────────────────────────────────────────────────────────────
    ("",        "",          "=== Trig ==="),
    ("S",       "SIN",       "Sine of X (in current angle mode)"),
    ("C",       "COS",       "Cosine of X (in current angle mode)"),
    ("T",       "TAN",       "Tangent of X (in current angle mode)"),
    ("a",       "ASIN",      "Arc sine of X → result in current angle mode"),
    ("c",       "ACOS",      "Arc cosine of X → result in current angle mode"),
    ("k",       "ATAN",      "Arc tangent of X → result in current angle mode"),
    ("d",       "DEG/RAD/GRAD", "Cycle angle mode: DEG → RAD → GRAD → DEG"),
    // ── Math ──────────────────────────────────────────────────────────────────
    ("",        "",          "=== Math ==="),
    ("L",       "LN",        "Natural logarithm of X"),
    ("G",       "LOG",       "Base-10 logarithm of X"),
    ("E",       "eˣ",        "Natural exponential of X"),
    ("H",       "10ˣ",       "Base-10 exponential of X"),
    // ── Registers ─────────────────────────────────────────────────────────────
    ("",        "",          "=== Registers ==="),
    ("R",       "RCL [nn]",  "Recall register nn (00–99) to X — press R then 2 digits"),
    ("Shift+R", "STO [nn]",  "Store X to register nn (00–99) — press S then 2 digits"),
    ("Shift+R+", "STO+ [nn]","Add X to register nn — press Shift+R+, then 2 digits"),
    ("Shift+R-", "STO- [nn]","Subtract X from register nn"),
    ("Shift+R*", "STO× [nn]","Multiply register nn by X"),
    ("Shift+R/", "STO÷ [nn]","Divide register nn by X"),
    // ── ALPHA ─────────────────────────────────────────────────────────────────
    ("",        "",          "=== ALPHA Mode ==="),
    ("a",       "ALPHA",     "Toggle ALPHA mode (while not in ALPHA: enters mode)"),
    ("a",       "ALPHA exit","While in ALPHA mode: press a or Enter to exit"),
    ("Bksp",    "ALPHA ←",   "While in ALPHA mode: delete last character"),
    ("Esc",     "ALPHA Esc", "While in ALPHA mode: exit without clearing register"),
    ("any",     "ALPHA char","While in ALPHA mode: append character to ALPHA register"),
    // ── Programming ───────────────────────────────────────────────────────────
    ("",        "",          "=== Programming ==="),
    ("p",       "PRGM",      "Toggle PRGM recording mode (record keystrokes as program)"),
    ("F5",      "R/S",       "Run program from label A (Run/Stop)"),
    ("F7",      "SST",       "Single-step forward in program (Step)"),
    ("F8",      "BST",       "Back-step in program"),
    // ── Display ───────────────────────────────────────────────────────────────
    ("",        "",          "=== Display ==="),
    ("f",       "FIX/SCI/ENG","Cycle display format: FIX 4 → SCI 4 → ENG 4 → FIX 4"),
    ("0-9",     "digit",     "Digit entry (append to X entry buffer)"),
    (".",       "decimal",   "Decimal point in entry buffer"),
    ("e",       "EEX",       "Scientific notation exponent entry"),
    // ── Persistence ───────────────────────────────────────────────────────────
    ("",        "",          "=== Persistence ==="),
    ("Ctrl+S",  "SAVE",      "Save full state to active file immediately"),
    ("Ctrl+P",  "PROGRAMS",  "Open program library (load bundled sample programs)"),
    // ── USER Mode ─────────────────────────────────────────────────────────────
    ("",        "",          "=== USER Mode ==="),
    ("u",       "USER",      "Toggle USER mode (key assignments active when lit)"),
    ("Ctrl+A",  "ASSIGN",    "Assign a key to a program label (two-step: key then label)"),
    ("F1",      "USER key a","Run program assigned to key a (USER mode only)"),
    ("F2",      "USER key b","Run program assigned to key b (USER mode only)"),
    ("F3",      "USER key c","Run program assigned to key c (USER mode only)"),
    ("F4",      "USER key d","Run program assigned to key d (USER mode only)"),
    // ── Help ──────────────────────────────────────────────────────────────────
    ("",        "",          "=== Help ==="),
    ("?",       "HELP",      "Toggle this function reference overlay"),
    ("Esc/q/?", "HELP close","Close this overlay (Esc, q, or ? again)"),
    ("↑/↓/j/k", "scroll",   "Scroll this table up/down"),
    // ── Quit ──────────────────────────────────────────────────────────────────
    ("",        "",          "=== Quit ==="),
    ("q",       "QUIT",      "Quit (saves state first)"),
    ("Ctrl+C",  "QUIT",      "Quit (saves state first)"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_data_has_minimum_entries() {
        // At least 50 entries (all keyboard-accessible ops including category headers)
        assert!(
            HELP_DATA.len() >= 50,
            "HELP_DATA must have at least 50 entries, got {}",
            HELP_DATA.len()
        );
    }

    #[test]
    fn test_all_ten_categories_present() {
        let categories = [
            "=== Stack ===",
            "=== Arithmetic ===",
            "=== Trig ===",
            "=== Math ===",
            "=== Registers ===",
            "=== ALPHA Mode ===",
            "=== Programming ===",
            "=== Display ===",
            "=== Persistence ===",
            "=== USER Mode ===",
        ];
        for cat in &categories {
            assert!(
                HELP_DATA.iter().any(|(_, _, desc)| desc == cat),
                "Missing category header: {cat}"
            );
        }
    }

    #[test]
    fn test_no_empty_key_or_op_in_data_rows() {
        // Non-header rows must have non-empty key and op fields
        for &(key, op, desc) in HELP_DATA {
            if desc.starts_with("===") {
                // Category header — key and op are empty by design
                continue;
            }
            assert!(!key.is_empty(), "Row with op='{op}' has empty key field");
            assert!(!op.is_empty(),  "Row with key='{key}' has empty op field");
        }
    }
}
