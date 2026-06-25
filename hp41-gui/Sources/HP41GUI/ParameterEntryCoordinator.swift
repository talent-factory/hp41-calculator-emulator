import Foundation

enum ParameterEntryKind: Equatable {
    case register(prefix: String, title: String)
    case label(prefix: String, title: String)
    case assignment
    case flag(prefix: String, title: String)
    case format(prefix: String, title: String)
    case singleDigit(prefix: String, title: String, range: ClosedRange<Int>)
    case delete
    case size
    case stoArithmetic

    var title: String {
        switch self {
        case let .register(_, title), let .label(_, title): title
        case .assignment: "ASN Key Assignment"
        case let .flag(_, title), let .format(_, title), let .singleDigit(_, title, _): title
        case .delete: "Delete Program Steps"
        case .size: "Register Size"
        case .stoArithmetic: "STO Arithmetic"
        }
    }
}

@MainActor
final class ParameterEntryCoordinator: ObservableObject {
    @Published var kind: ParameterEntryKind?
    @Published var input = ""
    @Published var indirect = false
    @Published var assignmentKeyCode = 11
    @Published var arithmeticOperation = "plus"

    var isPresented: Bool {
        get { kind != nil }
        set { if !newValue { cancel() } }
    }

    func open(for commandID: String) -> Bool {
        let next: ParameterEntryKind?
        switch commandID {
        case "sto_prompt": next = .register(prefix: "sto", title: "STO Register")
        case "rcl_prompt": next = .register(prefix: "rcl", title: "RCL Register")
        case "gto_prompt": next = .label(prefix: "gto", title: "GTO Label")
        case "lbl_prompt": next = .label(prefix: "lbl", title: "LBL Name")
        case "asn": next = .assignment
        case "view": next = .register(prefix: "view", title: "VIEW Register")
        case "isg_prompt": next = .register(prefix: "isg", title: "ISG Register")
        case "sf_prompt": next = .flag(prefix: "sf", title: "Set Flag")
        case "cf_prompt": next = .flag(prefix: "cf", title: "Clear Flag")
        case "fs_prompt": next = .flag(prefix: "fs", title: "Test Flag Set")
        case "fix_prompt": next = .format(prefix: "fix", title: "FIX Digits")
        case "sci_prompt": next = .format(prefix: "sci", title: "SCI Digits")
        case "eng_prompt": next = .format(prefix: "eng", title: "ENG Digits")
        case "catalog": next = .singleDigit(prefix: "catalog", title: "Catalog", range: 1...4)
        case "tone": next = .singleDigit(prefix: "tone", title: "Tone", range: 0...9)
        case "clp_prompt": next = .label(prefix: "clp", title: "Clear Program")
        case "dse_prompt": next = .register(prefix: "dse", title: "DSE Register")
        case "arcl_prompt": next = .register(prefix: "arcl", title: "ARCL Register")
        case "asto_prompt": next = .register(prefix: "asto", title: "ASTO Register")
        case "fc_prompt": next = .flag(prefix: "fc", title: "Test Flag Clear")
        case "fs_c_prompt": next = .flag(prefix: "fs_c", title: "Test Set and Clear")
        case "fc_c_prompt": next = .flag(prefix: "fc_c", title: "Test Clear and Clear")
        case "gto_ind_prompt": next = .register(prefix: "gto", title: "GTO Indirect Register")
        case "xeq_ind_prompt": next = .register(prefix: "xeq", title: "XEQ Indirect Register")
        case "del_prompt": next = .delete
        case "size_prompt": next = .size
        case "sto_arith_prompt": next = .stoArithmetic
        default: next = nil
        }
        guard let next else { return false }
        kind = next
        input = ""
        indirect = false
        if commandID == "gto_ind_prompt" || commandID == "xeq_ind_prompt" {
            indirect = true
        }
        return true
    }

    func submit(on model: CalculatorModel) {
        guard let kind else { return }
        let dispatchID: String?
        switch kind {
        case let .register(prefix, _):
            guard let register = Int(input), (0...99).contains(register) else { return }
            let formatted = String(format: "%02d", register)
            dispatchID = indirect ? "\(prefix)_ind_\(formatted)" : "\(prefix)_\(formatted)"
        case let .label(prefix, _):
            let label = input.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
            let maximum = prefix == "clp" ? 7 : 24
            guard !label.isEmpty, label.count <= maximum else { return }
            dispatchID = "\(prefix)_\(label)"
        case .assignment:
            let label = input.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
            guard !label.isEmpty, label.count <= 7 else { return }
            dispatchID = "asn_\(assignmentKeyCode)_\(label)"
        case let .flag(prefix, _):
            guard let flag = Int(input), (0...55).contains(flag) else { return }
            let formatted = String(format: "%02d", flag)
            dispatchID = indirect ? "\(prefix)_ind_\(formatted)" : "\(prefix)_\(formatted)"
        case let .format(prefix, _):
            guard let digits = Int(input), (0...9).contains(digits) else { return }
            dispatchID = "\(prefix)_\(digits)"
        case let .singleDigit(prefix, _, range):
            guard let digit = Int(input), range.contains(digit) else { return }
            dispatchID = "\(prefix)_\(digit)"
        case .delete:
            guard let count = Int(input), (0...255).contains(count) else { return }
            dispatchID = "del_\(String(format: "%03d", count))"
        case .size:
            guard let count = Int(input), (0...319).contains(count) else { return }
            dispatchID = "size_\(count)"
        case .stoArithmetic:
            let target = input.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            let stackTargets = ["y", "z", "t", "lastx"]
            if let register = Int(target), (0...99).contains(register) {
                let formatted = String(format: "%02d", register)
                dispatchID = indirect
                    ? "sto_arith_\(arithmeticOperation)_ind_\(formatted)"
                    : "sto_arith_\(arithmeticOperation)_\(formatted)"
            } else if stackTargets.contains(target), !indirect {
                dispatchID = "sto_arith_\(arithmeticOperation)_\(target)"
            } else {
                return
            }
        }
        guard let dispatchID else { return }
        model.press(id: dispatchID)
        if model.state.error == nil { cancel() }
    }

    func dispatchDirect(_ commandID: String, on model: CalculatorModel) -> Bool {
        let dispatchID: String?
        switch commandID {
        case "x_eq_y_prompt": dispatchID = "x_eq_y"
        case "x_le_y_prompt": dispatchID = "x_le_y"
        case "x_gt_y_prompt": dispatchID = "x_gt_y"
        case "x_eq_0_prompt": dispatchID = "x_eq_0"
        default: dispatchID = nil
        }
        guard let dispatchID else { return false }
        model.press(id: dispatchID)
        return true
    }

    func cancel() {
        kind = nil
        input = ""
        indirect = false
        assignmentKeyCode = 11
        arithmeticOperation = "plus"
    }
}
