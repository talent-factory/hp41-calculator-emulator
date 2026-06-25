import Foundation

enum PhysicalKeyboardAction: Equatable {
    case dispatch(String)
    case saveState
}

/// Framework-independent mirror of the retained Tauri `resolveKeyId` map.
/// Keeping this pure makes every shortcut testable without synthesizing AppKit events.
enum PhysicalKeyboardRouter {
    static let mappedActions: Set<String> = [
        ".", "__save_state__", "acos", "alpha_backspace", "asin", "atan", "bst",
        "cl_sigma_stat", "clreg", "corr", "cos", "div", "e", "enter",
        "entry_backspace", "exp", "hms_add", "hms_sub", "hms_to_h", "lastx", "ln",
        "log", "lr", "mean", "minus", "mul", "pct_change", "plus", "prgm_mode",
        "prx", "rdn", "recip", "sdev", "sigma_minus", "sigma_plus", "sin", "sq",
        "sqrt", "sst", "tan", "tenpow", "user_mode", "xeq_RDPRGM", "xeq_RDTA",
        "xeq_WDTA", "xeq_WPRGM", "xy_swap", "yhat", "ypow",
    ]

    static func route(key rawKey: String, commandOrControl: Bool = false,
                      alphaActive: Bool = false, eexActive: Bool = false) -> PhysicalKeyboardAction? {
        let key = normalized(rawKey)

        if commandOrControl {
            switch key.lowercased() {
            case "w": return .dispatch("xeq_WPRGM")
            case "r": return .dispatch("xeq_RDPRGM")
            case "d": return .dispatch("xeq_WDTA")
            case "f": return .dispatch("xeq_RDTA")
            case "s": return .saveState
            default: return nil
            }
        }

        if key == "F5" { return .saveState }
        if key == "F7" { return .dispatch("sst") }
        if key == "F8" { return .dispatch("bst") }

        if alphaActive, key.count == 1,
           let scalar = key.uppercased().unicodeScalars.first,
           CharacterSet.alphanumerics.union(.whitespaces).contains(scalar) {
            return .dispatch("alpha_\(key.uppercased())")
        }
        if alphaActive, key == "Backspace" { return .dispatch("alpha_backspace") }

        if key == "n" { return .dispatch(eexActive ? "eex_chs" : "chs") }
        if key.count == 1, key.first?.isNumber == true { return .dispatch(key) }
        if key == "." || key == "e" { return .dispatch(key) }
        if key.count == 1, "SRfFX".contains(key) { return nil }

        return shortcutMap[key].map(PhysicalKeyboardAction.dispatch)
    }

    private static let shortcutMap: [String: String] = [
        "Enter": "enter", "Backspace": "entry_backspace",
        "+": "plus", "-": "minus", "*": "mul", "/": "div",
        "r": "rdn", "x": "xy_swap", "l": "lastx", "s": "sqrt",
        "p": "prgm_mode", "P": "prx",
        "a": "asin", "c": "acos", "k": "atan",
        "C": "cos", "T": "tan", "L": "ln", "G": "log", "E": "exp",
        "H": "tenpow", "I": "recip", "W": "sq", "Y": "ypow",
        "%": "pct_change", "u": "user_mode",
        "z": "sigma_plus", "Z": "sigma_minus", "m": "mean", "D": "sdev",
        "y": "yhat", "b": "lr", "O": "corr", "V": "cl_sigma_stat",
        "h": "hms_to_h", "j": "hms_add", "J": "hms_sub",
        "q": "sin", "g": "clreg",
    ]

    private static func normalized(_ key: String) -> String {
        switch key {
        case "\r", "\n": return "Enter"
        case "\u{7f}", "\u{8}": return "Backspace"
        case "\u{F708}": return "F5"
        case "\u{F70A}": return "F7"
        case "\u{F70B}": return "F8"
        default: return key
        }
    }
}
