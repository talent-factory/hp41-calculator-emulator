//! Static help data for the HP-41 function reference overlay (D-18).
//!
//! Format: (key_binding, hp41_op_name, description)
//! Category headers: key="" op="" desc="=== Category ===" for visual separation in Table.
//!
//! This is the SINGLE SOURCE OF TRUTH for key-to-operation descriptions.
//! Keys derived from KEY_REF_TABLE in keys.rs; descriptions added here.
//! ≥83 entries covering all keyboard-accessible HP-41 operations.

/// (key_binding, hp41_op_name, description) — ≥83 entries, 13 categories.
pub const HELP_DATA: &[(&str, &str, &str)] = &[
    // ── Stack ─────────────────────────────────────────────────────────────────
    ("", "", "=== Stack ==="),
    ("Enter", "ENTER", "Lift stack and duplicate X into Y"),
    ("Bksp", "CLX", "Clear X register (entry cancel)"),
    ("n", "CHS", "Change sign of X (negate)"),
    ("r", "R↓", "Roll stack down: X←Y, Y←Z, Z←T, T←X"),
    ("x", "X⟷Y", "Swap X and Y registers"),
    ("l", "LASTX", "Recall last X (value before last operation)"),
    // ── Arithmetic ────────────────────────────────────────────────────────────
    ("", "", "=== Arithmetic ==="),
    ("+", "+", "Add: X ← Y + X, drop stack"),
    ("-", "-", "Subtract: X ← Y − X, drop stack"),
    ("*", "×", "Multiply: X ← Y × X, drop stack"),
    ("/", "÷", "Divide: X ← Y ÷ X, drop stack"),
    ("I", "1/x", "Reciprocal of X"),
    ("s", "√x", "Square root of X"),
    ("W", "x²", "Square of X"),
    ("Y", "Yˣ", "Y raised to power X"),
    // ── Trig ──────────────────────────────────────────────────────────────────
    ("", "", "=== Trig ==="),
    ("q", "SIN", "Sine of X (in current angle mode)"),
    ("C", "COS", "Cosine of X (in current angle mode)"),
    ("T", "TAN", "Tangent of X (in current angle mode)"),
    ("a", "ASIN", "Arc sine of X → result in current angle mode"),
    (
        "c",
        "ACOS",
        "Arc cosine of X → result in current angle mode",
    ),
    (
        "k",
        "ATAN",
        "Arc tangent of X → result in current angle mode",
    ),
    (
        "d",
        "DEG/RAD/GRAD",
        "Cycle angle mode: DEG → RAD → GRAD → DEG",
    ),
    // ── Math ──────────────────────────────────────────────────────────────────
    ("", "", "=== Math ==="),
    ("L", "LN", "Natural logarithm of X"),
    ("G", "LOG", "Base-10 logarithm of X"),
    ("E", "eˣ", "Natural exponential of X"),
    ("H", "10ˣ", "Base-10 exponential of X"),
    // ── Registers ─────────────────────────────────────────────────────────────
    ("", "", "=== Registers ==="),
    (
        "R",
        "RCL [nn]",
        "Recall register nn (00–99) to X — press R then 2 digits",
    ),
    (
        "S",
        "STO [nn]",
        "Store X to register nn (00–99) — press S then 2 digits",
    ),
    (
        "S +",
        "STO+ [nn]",
        "Add X to register nn or stack Y/Z/T/L — press S then +, then nn or Y/Z/T/L",
    ),
    (
        "S -",
        "STO- [nn]",
        "Subtract X from register nn or stack Y/Z/T/L — press S then -, then nn or Y/Z/T/L",
    ),
    (
        "S *",
        "STO× [nn]",
        "Multiply register nn or stack Y/Z/T/L by X — press S then *, then nn or Y/Z/T/L",
    ),
    (
        "S /",
        "STO÷ [nn]",
        "Divide register nn or stack Y/Z/T/L by X — press S then /, then nn or Y/Z/T/L",
    ),
    ("g", "CLREG", "Clear all storage registers R00-R99 to zero"),
    // ── ALPHA ─────────────────────────────────────────────────────────────────
    ("", "", "=== ALPHA Mode ==="),
    (
        "a",
        "ALPHA",
        "Toggle ALPHA mode (while not in ALPHA: enters mode)",
    ),
    (
        "a",
        "ALPHA exit",
        "While in ALPHA mode: press a or Enter to exit",
    ),
    (
        "Bksp",
        "ALPHA ←",
        "While in ALPHA mode: delete last character",
    ),
    (
        "Del",
        "CLRALPHA",
        "While in ALPHA mode: clear entire ALPHA register",
    ),
    (
        "Esc",
        "ALPHA Esc",
        "While in ALPHA mode: exit without clearing register",
    ),
    (
        "any",
        "ALPHA char",
        "While in ALPHA mode: append character to ALPHA register",
    ),
    // ── Programming ───────────────────────────────────────────────────────────
    ("", "", "=== Programming ==="),
    (
        "p",
        "PRGM",
        "Toggle PRGM recording mode (record keystrokes as program)",
    ),
    ("F5", "R/S", "Run program from label A (Run/Stop)"),
    ("F7", "SST", "Single-step forward in program (Step)"),
    ("F8", "BST", "Back-step in program"),
    // ── Display ───────────────────────────────────────────────────────────────
    ("", "", "=== Display ==="),
    (
        "f",
        "FIX/SCI/ENG",
        "Cycle display format (FIX/SCI/ENG), keeps current digit count",
    ),
    (
        "F",
        "FIX/SCI/ENG n",
        "Open digit-count modal: enter 0\u{2013}9 to set digits; f cycles format",
    ),
    ("0-9", "digit", "Digit entry (append to X entry buffer)"),
    (".", "decimal", "Decimal point in entry buffer"),
    ("e", "EEX", "Scientific notation exponent entry"),
    // ── Persistence ───────────────────────────────────────────────────────────
    ("", "", "=== Persistence ==="),
    (
        "Ctrl+S",
        "SAVE",
        "Save full state to active file immediately",
    ),
    (
        "Ctrl+P",
        "PROGRAMS",
        "Open program library (load bundled sample programs)",
    ),
    // ── USER Mode ─────────────────────────────────────────────────────────────
    ("", "", "=== USER Mode ==="),
    (
        "u",
        "USER",
        "Toggle USER mode (key assignments active when lit)",
    ),
    (
        "Ctrl+A",
        "ASSIGN",
        "Assign a key to a program label (two-step: key then label)",
    ),
    (
        "F1",
        "USER key a",
        "Run program assigned to key a (USER mode only)",
    ),
    (
        "F2",
        "USER key b",
        "Run program assigned to key b (USER mode only)",
    ),
    (
        "F3",
        "USER key c",
        "Run program assigned to key c (USER mode only)",
    ),
    (
        "F4",
        "USER key d",
        "Run program assigned to key d (USER mode only)",
    ),
    // ── Science & Engineering ─────────────────────────────────────────────────
    ("", "", "=== Science & Engineering ==="),
    (
        "z",
        "\u{03A3}+",
        "Accumulate X,Y into \u{03A3} registers R01\u{2013}R06; push count n to X",
    ),
    (
        "Z",
        "\u{03A3}\u{2212}",
        "Remove X,Y from \u{03A3} registers R01\u{2013}R06; push count n to X",
    ),
    (
        "m",
        "MEAN",
        "Mean: X \u{2190} x\u{0305}, Y \u{2190} y\u{0305} from \u{03A3} registers",
    ),
    (
        "D",
        "SDEV",
        "Sample std dev (n\u{2212}1): X \u{2190} \u{03C3}x, Y \u{2190} \u{03C3}y",
    ),
    (
        "y",
        "YHAT",
        "\u{0177} prediction: read x from X, compute \u{0177} via L.R.",
    ),
    (
        "b",
        "L.R.",
        "Linear regression: slope m to Y, intercept b to X",
    ),
    ("O", "CORR", "Correlation coefficient r in X"),
    (
        "V",
        "CL\u{03A3}",
        "Clear \u{03A3} statistics registers R01\u{2013}R06 to zero",
    ),
    (
        "h",
        "HMS\u{2192}",
        "Convert H.MMSS (hours.minutesseconds) to decimal hours",
    ),
    (
        "(none)",
        "\u{2192}HMS",
        "Convert decimal hours to H.MMSS format (key unbound; use dispatch)",
    ),
    (
        "j",
        "HMS+",
        "Add two H.MMSS values: Y + X \u{2192} X (base-60 carry)",
    ),
    (
        "J",
        "HMS\u{2212}",
        "Subtract H.MMSS values: Y \u{2212} X \u{2192} X (base-60 borrow)",
    ),
    // ── Print ─────────────────────────────────────────────────────────────────
    ("", "", "=== Print ==="),
    (
        "P X",
        "PRX",
        "Print X register to console (right-aligned, 24 chars)",
    ),
    (
        "P A",
        "PRA",
        "Print ALPHA register to console (left-aligned, 24 chars)",
    ),
    (
        "P S",
        "PRSTK",
        "Print full stack T/Z/Y/X/LASTX/ALPHA (6 lines) to console",
    ),
    // ── Synthetic Programming ─────────────────────────────────────────────────
    ("", "", "=== Synthetic Programming ==="),
    (
        "X nn",
        "HEX",
        "Insert synthetic hex byte at current PC (PRGM mode) — press X then 2 hex digits",
    ),
    (
        "S M",
        "STO M",
        "Store X to hidden register M — press S then M",
    ),
    (
        "S N",
        "STO N",
        "Store X to hidden register N — press S then N",
    ),
    (
        "S O",
        "STO O",
        "Store X to hidden register O — press S then O",
    ),
    (
        "R M",
        "RCL M",
        "Recall hidden register M into X — press R then M",
    ),
    (
        "R N",
        "RCL N",
        "Recall hidden register N into X — press R then N",
    ),
    (
        "R O",
        "RCL O",
        "Recall hidden register O into X — press R then O",
    ),
    // ── Card Reader ───────────────────────────────────────────────────────────
    ("", "", "=== Card Reader ==="),
    (
        "Ctrl+W",
        "WPRGM",
        "Write current program to card (uses ALPHA register as filename)",
    ),
    (
        "Ctrl+R",
        "RDPRGM",
        "Read program from card (uses ALPHA register as filename)",
    ),
    (
        "Ctrl+D",
        "WDTA",
        "Write data registers to card (uses ALPHA register as filename)",
    ),
    (
        "Ctrl+F",
        "RDTA",
        "Read data registers from card (uses ALPHA register as filename)",
    ),
    // ── Help ──────────────────────────────────────────────────────────────────
    ("", "", "=== Help ==="),
    ("?", "HELP", "Toggle this function reference overlay"),
    (
        "Esc/q/?",
        "HELP close",
        "Close this overlay (Esc, q, or ? again)",
    ),
    ("↑/↓/j/k", "scroll", "Scroll this table up/down"),
    // ── Quit ──────────────────────────────────────────────────────────────────
    ("", "", "=== Quit ==="),
    ("Ctrl+C", "QUIT", "Quit (saves state first)"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_data_has_minimum_entries() {
        // At least 80 entries (all keyboard-accessible ops including category headers)
        assert!(
            HELP_DATA.len() >= 80,
            "HELP_DATA must have at least 80 entries, got {}",
            HELP_DATA.len()
        );
    }

    #[test]
    fn test_all_sixteen_categories_present() {
        // Phase 19 added "=== Card Reader ===" — now 16 categories total.
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
            "=== Science & Engineering ===",
            "=== Print ===",
            "=== Synthetic Programming ===",
            "=== Card Reader ===",
            "=== Help ===",
            "=== Quit ===",
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
            assert!(!op.is_empty(), "Row with key='{key}' has empty op field");
        }
    }
}
