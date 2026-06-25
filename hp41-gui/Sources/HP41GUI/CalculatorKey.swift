import Foundation

struct CalculatorKey: Identifiable, Hashable {
    let id: UUID
    let commandID: String
    let label: String
    let shiftedID: String?
    let shiftedLabel: String?
    let shiftedInProgramID: String?
    let shiftedInProgramLabel: String?
    let alpha: String?
    let keyCode: Int?
    let columnSpan: Int
    let kind: Kind

    enum Kind: Hashable { case normal, enter, shift, mode }

    var accessibilityIdentifier: String {
        if label == "ON" { return "key-on" }
        return "key-\(commandID.isEmpty ? label.lowercased() : commandID)"
    }

    init(_ id: String, _ label: String, shifted: (String, String)? = nil,
         shiftedInProgram: (String, String)? = nil, alpha: String? = nil,
         keyCode: Int? = nil, columnSpan: Int = 1, kind: Kind = .normal) {
        self.id = UUID()
        commandID = id
        self.label = label
        shiftedID = shifted?.0
        shiftedLabel = shifted?.1
        shiftedInProgramID = shiftedInProgram?.0
        shiftedInProgramLabel = shiftedInProgram?.1
        self.alpha = alpha
        self.keyCode = keyCode
        self.columnSpan = columnSpan
        self.kind = kind
    }
}

enum KeyboardLayout {
    static let modes = [
        CalculatorKey("", "ON", kind: .mode),
        CalculatorKey("user_mode", "USER", kind: .mode),
        CalculatorKey("prgm_mode", "PRGM", kind: .mode),
        CalculatorKey("alpha_toggle", "ALPHA", kind: .mode),
    ]

    static let rows: [[CalculatorKey]] = [
        [
            .init("sigma_plus", "Σ+", shifted: ("sigma_minus", "Σ−"), alpha: "A", keyCode: 11),
            .init("recip", "1/x", shifted: ("ypow", "yˣ"), alpha: "B", keyCode: 12),
            .init("sqrt", "√x", shifted: ("sq", "x²"), shiftedInProgram: ("clp_prompt", "CLP"), alpha: "C", keyCode: 13),
            .init("log", "LOG", shifted: ("tenpow", "10ˣ"), alpha: "D", keyCode: 14),
            .init("ln", "LN", shifted: ("exp", "eˣ"), alpha: "E", keyCode: 15),
        ],
        [
            .init("xge_y", "x≥y", shifted: ("cl_sigma_stat", "CLΣ"), alpha: "F"),
            .init("rdn", "R↓", shifted: ("pct_change", "%"), alpha: "G", keyCode: 24),
            .init("sin", "SIN", shifted: ("asin", "SIN⁻¹"), alpha: "H", keyCode: 25),
            .init("cos", "COS", shifted: ("acos", "COS⁻¹"), alpha: "I", keyCode: 34),
            .init("tan", "TAN", shifted: ("atan", "TAN⁻¹"), alpha: "J", keyCode: 35),
        ],
        [
            .init("shift", "", kind: .shift),
            .init("xeq_prompt", "XEQ", shifted: ("asn", "ASN"), alpha: "K", keyCode: 21),
            .init("sto_prompt", "STO", shifted: ("lbl_prompt", "LBL"), alpha: "L", keyCode: 22),
            .init("rcl_prompt", "RCL", shifted: ("gto_prompt", "GTO"), alpha: "M", keyCode: 23),
            .init("sst", "SST", shifted: ("bst", "BST"), keyCode: 32),
        ],
        [
            .init("enter", "ENTER↑", shifted: ("catalog", "CATALOG"), alpha: "N", keyCode: 84, columnSpan: 2, kind: .enter),
            .init("chs", "CHS", shifted: ("isg_prompt", "ISG"), alpha: "O"),
            .init("e", "EEX", shifted: ("rtn", "RTN"), alpha: "P", keyCode: 83),
            .init("clx_or_a", "←", shifted: ("clx_or_a", "CL X/A")),
        ],
        [
            .init("minus", "−", shifted: ("x_eq_y_prompt", "x=y?"), alpha: "Q", keyCode: 64), .init("7", "7", shifted: ("sf_prompt", "SF"), alpha: "R", keyCode: 51),
            .init("8", "8", shifted: ("cf_prompt", "CF"), alpha: "S", keyCode: 52), .init("9", "9", shifted: ("fs_prompt", "FS?"), alpha: "T", keyCode: 53),
        ],
        [
            .init("plus", "+", shifted: ("x_le_y_prompt", "x≤y?"), alpha: "U", keyCode: 74), .init("4", "4", shifted: ("beep", "BEEP"), alpha: "V", keyCode: 61),
            .init("5", "5", shifted: ("polar_to_rect", "P→R"), alpha: "W", keyCode: 62),
            .init("6", "6", shifted: ("rect_to_polar", "R→P"), alpha: "X", keyCode: 63),
        ],
        [
            .init("mul", "×", shifted: ("x_gt_y_prompt", "x>y?"), alpha: "Y", keyCode: 54), .init("1", "1", shifted: ("fix_prompt", "FIX"), alpha: "Z", keyCode: 71),
            .init("2", "2", shifted: ("sci_prompt", "SCI"), alpha: "=", keyCode: 72), .init("3", "3", shifted: ("eng_prompt", "ENG"), alpha: "?", keyCode: 73),
        ],
        [
            .init("div", "÷", shifted: ("x_eq_0_prompt", "x=0?"), alpha: ":", keyCode: 45), .init("0", "0", shifted: ("pi", "π"), alpha: "SPACE", keyCode: 81),
            .init(".", ".", shifted: ("lastx", "LAST X"), alpha: ",", keyCode: 82), .init("r_s", "R/S", shifted: ("view", "VIEW"), keyCode: 31),
        ],
    ]

    static let canonicalCommandIDs: Set<String> = {
        let keys = modes + rows.flatMap { $0 }
        return Set(keys.flatMap { key in
            [key.commandID, key.shiftedID, key.shiftedInProgramID].compactMap { id in
                guard let id, !id.isEmpty else { return nil }
                return id
            }
        })
    }()

    static func primaryLabel(for key: CalculatorKey, userActive: Bool,
                             assignments: [UserKeyAssignment]) -> String {
        guard userActive, let keyCode = key.keyCode,
              let assignment = assignments.first(where: { $0.keyCode == keyCode })
        else { return key.label }
        return String(assignment.label.prefix(7))
    }
}
