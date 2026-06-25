import Foundation

enum CalculatorRequest: Codable, Equatable, Sendable {
    case getState
    case dispatch(keyID: String)
    case sstStep
    case bstStep
    case runStop
    case runProgram(label: String)
    case resumeProgram
    case resumeProgramWithKey(keycode: UInt8)
    case requestCancel
    case submitModal
    case cancelModal
    case submitModalWithLabel(label: String)
    case tickTime
    case saveState
    case resetSoft
    case resetFull
    case inspectRaw(path: String)
    case importRaw(path: String, indices: [Int])
    case exportRaw(path: String)
    case importData(path: String)
    case exportData(path: String)

    private enum CodingKeys: String, CodingKey {
        case command
        case keyID = "key_id"
        case label, keycode, path, indices
    }

    private enum Command: String, Codable {
        case getState = "get_state"
        case dispatch
        case sstStep = "sst_step"
        case bstStep = "bst_step"
        case runStop = "run_stop"
        case runProgram = "run_program"
        case resumeProgram = "resume_program"
        case resumeProgramWithKey = "resume_program_with_key"
        case requestCancel = "request_cancel"
        case submitModal = "submit_modal"
        case cancelModal = "cancel_modal"
        case submitModalWithLabel = "submit_modal_with_label"
        case tickTime = "tick_time"
        case saveState = "save_state"
        case resetSoft = "reset_soft"
        case resetFull = "reset_full"
        case inspectRaw = "inspect_raw"
        case importRaw = "import_raw"
        case exportRaw = "export_raw"
        case importData = "import_data"
        case exportData = "export_data"
    }

    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        switch try values.decode(Command.self, forKey: .command) {
        case .getState: self = .getState
        case .dispatch: self = .dispatch(keyID: try values.decode(String.self, forKey: .keyID))
        case .sstStep: self = .sstStep
        case .bstStep: self = .bstStep
        case .runStop: self = .runStop
        case .runProgram: self = .runProgram(label: try values.decode(String.self, forKey: .label))
        case .resumeProgram: self = .resumeProgram
        case .resumeProgramWithKey:
            self = .resumeProgramWithKey(keycode: try values.decode(UInt8.self, forKey: .keycode))
        case .requestCancel: self = .requestCancel
        case .submitModal: self = .submitModal
        case .cancelModal: self = .cancelModal
        case .submitModalWithLabel:
            self = .submitModalWithLabel(label: try values.decode(String.self, forKey: .label))
        case .tickTime: self = .tickTime
        case .saveState: self = .saveState
        case .resetSoft: self = .resetSoft
        case .resetFull: self = .resetFull
        case .inspectRaw: self = .inspectRaw(path: try values.decode(String.self, forKey: .path))
        case .importRaw:
            self = .importRaw(
                path: try values.decode(String.self, forKey: .path),
                indices: try values.decode([Int].self, forKey: .indices)
            )
        case .exportRaw: self = .exportRaw(path: try values.decode(String.self, forKey: .path))
        case .importData: self = .importData(path: try values.decode(String.self, forKey: .path))
        case .exportData: self = .exportData(path: try values.decode(String.self, forKey: .path))
        }
    }

    func encode(to encoder: Encoder) throws {
        var values = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .getState: try values.encode(Command.getState, forKey: .command)
        case let .dispatch(keyID):
            try values.encode(Command.dispatch, forKey: .command)
            try values.encode(keyID, forKey: .keyID)
        case .sstStep: try values.encode(Command.sstStep, forKey: .command)
        case .bstStep: try values.encode(Command.bstStep, forKey: .command)
        case .runStop: try values.encode(Command.runStop, forKey: .command)
        case let .runProgram(label):
            try values.encode(Command.runProgram, forKey: .command)
            try values.encode(label, forKey: .label)
        case .resumeProgram: try values.encode(Command.resumeProgram, forKey: .command)
        case let .resumeProgramWithKey(keycode):
            try values.encode(Command.resumeProgramWithKey, forKey: .command)
            try values.encode(keycode, forKey: .keycode)
        case .requestCancel: try values.encode(Command.requestCancel, forKey: .command)
        case .submitModal: try values.encode(Command.submitModal, forKey: .command)
        case .cancelModal: try values.encode(Command.cancelModal, forKey: .command)
        case let .submitModalWithLabel(label):
            try values.encode(Command.submitModalWithLabel, forKey: .command)
            try values.encode(label, forKey: .label)
        case .tickTime: try values.encode(Command.tickTime, forKey: .command)
        case .saveState: try values.encode(Command.saveState, forKey: .command)
        case .resetSoft: try values.encode(Command.resetSoft, forKey: .command)
        case .resetFull: try values.encode(Command.resetFull, forKey: .command)
        case let .inspectRaw(path):
            try values.encode(Command.inspectRaw, forKey: .command)
            try values.encode(path, forKey: .path)
        case let .importRaw(path, indices):
            try values.encode(Command.importRaw, forKey: .command)
            try values.encode(path, forKey: .path)
            try values.encode(indices, forKey: .indices)
        case let .exportRaw(path):
            try values.encode(Command.exportRaw, forKey: .command)
            try values.encode(path, forKey: .path)
        case let .importData(path):
            try values.encode(Command.importData, forKey: .command)
            try values.encode(path, forKey: .path)
        case let .exportData(path):
            try values.encode(Command.exportData, forKey: .command)
            try values.encode(path, forKey: .path)
        }
    }

    var isCancellation: Bool {
        if case .requestCancel = self { return true }
        return false
    }
}

enum CalculatorResponseStatus: String, Codable, Sendable {
    case ok
    case error
}

struct CalculatorResponse: Decodable, Sendable {
    let status: CalculatorResponseStatus
    let state: CalculatorState
    let error: String?
    let result: FileTransferResult?

    private enum CodingKeys: String, CodingKey {
        case status, error, result
    }

    init(
        status: CalculatorResponseStatus,
        state: CalculatorState,
        error: String? = nil,
        result: FileTransferResult? = nil
    ) {
        self.status = status
        self.state = state
        self.error = error
        self.result = result
    }

    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        state = try CalculatorState(from: decoder)
        error = try values.decodeIfPresent(String.self, forKey: .error)
        result = try values.decodeIfPresent(FileTransferResult.self, forKey: .result)
        status = try values.decodeIfPresent(CalculatorResponseStatus.self, forKey: .status)
            ?? (error == nil ? .ok : .error)
        guard (status == .error) == (error != nil), state.error == error else {
            throw DecodingError.dataCorruptedError(
                forKey: .status,
                in: values,
                debugDescription: "Response status, error, and state error disagree"
            )
        }
    }
}

enum CalculatorServiceError: Error, Equatable, Sendable {
    case encoding(String)
    case transport
    case decoding(String)

    var message: String {
        switch self {
        case let .encoding(message): "Calculator request encoding failed: \(message)"
        case .transport: "Calculator bridge did not return a response"
        case let .decoding(message): "Calculator response decoding failed: \(message)"
        }
    }
}
