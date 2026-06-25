#if SWIFT_PACKAGE
import CHP41
#endif
import Foundation
#if os(macOS)
import AppKit
#endif

struct OutputNotice: Identifiable, Equatable {
    let id = UUID()
    let message: String
}

struct AnnunciatorState: Codable, Equatable, Sendable {
    var user = false
    var prgm = false
    var alpha = false
    var rad = false
    var grad = false
}

struct UserKeyAssignment: Codable, Equatable, Sendable {
    var keyCode: Int
    var label: String

    init(keyCode: Int, label: String) {
        self.keyCode = keyCode
        self.label = label
    }

    init(from decoder: Decoder) throws {
        var values = try decoder.unkeyedContainer()
        keyCode = try values.decode(Int.self)
        label = try values.decode(String.self)
    }

    func encode(to encoder: Encoder) throws {
        var values = encoder.unkeyedContainer()
        try values.encode(keyCode)
        try values.encode(label)
    }
}

struct YieldStateView: Codable, Equatable, Sendable {
    var kind: String
    var text: String
    var resumeMS: UInt64

    enum CodingKeys: String, CodingKey {
        case kind, text
        case resumeMS = "resume_ms"
    }
}

struct RawProgramInfo: Codable, Equatable, Identifiable, Sendable {
    var label: String
    var index: Int
    var byteLength: Int
    var stepCount: Int

    var id: Int { index }

    enum CodingKeys: String, CodingKey {
        case label, index
        case byteLength = "byte_len"
        case stepCount = "step_count"
    }
}

struct FileTransferResult: Codable, Equatable, Sendable {
    var kind: String
    var path: String?
    var programs: [RawProgramInfo]?
}

struct CalculatorState: Codable, Equatable, Sendable {
    var display = "0.0000"
    var x = "0.0000"
    var y = "0.0000"
    var z = "0.0000"
    var t = "0.0000"
    var lastX = "0.0000"
    var annunciators = AnnunciatorState()
    var inEEXMode = false
    var printLines: [String] = []
    var programSteps: [String] = []
    var pc = 0
    var userKeymap: [UserKeyAssignment] = []
    var flags: [Int] = []
    var displayOverride: String?
    var eventBuffer: [String] = []
    var isRunning = false
    var modalProgramActive = false
    var modalRequiresAlphaLabel = false
    var modalPrompt: String?
    var clockActive = false
    var stopwatchKeyboardMode = false
    var stopwatchRunning = false
    var pendingYield: YieldStateView?
    var error: String?

    enum CodingKeys: String, CodingKey {
        case display = "display_str"
        case x = "x_str"
        case y = "y_str"
        case z = "z_str"
        case t = "t_str"
        case lastX = "lastx_str"
        case annunciators, flags, error, pc
        case inEEXMode = "in_eex_mode"
        case printLines = "print_lines"
        case programSteps = "program_steps"
        case userKeymap = "user_keymap"
        case displayOverride = "display_override"
        case eventBuffer = "event_buffer"
        case isRunning = "is_running"
        case modalProgramActive = "modal_program_active"
        case modalRequiresAlphaLabel = "modal_requires_alpha_label"
        case modalPrompt = "modal_prompt"
        case clockActive = "clock_active"
        case stopwatchKeyboardMode = "stopwatch_keyboard_mode"
        case stopwatchRunning = "stopwatch_running"
        case pendingYield = "pending_yield"
        case legacyDisplay = "display"
        case legacyX = "x"
        case legacyY = "y"
        case legacyZ = "z"
        case legacyT = "t"
        case legacyLastX = "last_x"
    }

    init() {}

    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        display = try values.decodeIfPresent(String.self, forKey: .display)
            ?? values.decodeIfPresent(String.self, forKey: .legacyDisplay) ?? "0.0000"
        x = try values.decodeIfPresent(String.self, forKey: .x)
            ?? values.decodeIfPresent(String.self, forKey: .legacyX) ?? "0.0000"
        y = try values.decodeIfPresent(String.self, forKey: .y)
            ?? values.decodeIfPresent(String.self, forKey: .legacyY) ?? "0.0000"
        z = try values.decodeIfPresent(String.self, forKey: .z)
            ?? values.decodeIfPresent(String.self, forKey: .legacyZ) ?? "0.0000"
        t = try values.decodeIfPresent(String.self, forKey: .t)
            ?? values.decodeIfPresent(String.self, forKey: .legacyT) ?? "0.0000"
        lastX = try values.decodeIfPresent(String.self, forKey: .lastX)
            ?? values.decodeIfPresent(String.self, forKey: .legacyLastX) ?? "0.0000"
        annunciators = try values.decodeIfPresent(AnnunciatorState.self, forKey: .annunciators) ?? .init()
        inEEXMode = try values.decodeIfPresent(Bool.self, forKey: .inEEXMode) ?? false
        printLines = try values.decodeIfPresent([String].self, forKey: .printLines) ?? []
        programSteps = try values.decodeIfPresent([String].self, forKey: .programSteps) ?? []
        pc = try values.decodeIfPresent(Int.self, forKey: .pc) ?? 0
        userKeymap = try values.decodeIfPresent([UserKeyAssignment].self, forKey: .userKeymap) ?? []
        flags = try values.decodeIfPresent([Int].self, forKey: .flags) ?? []
        displayOverride = try values.decodeIfPresent(String.self, forKey: .displayOverride)
        eventBuffer = try values.decodeIfPresent([String].self, forKey: .eventBuffer) ?? []
        isRunning = try values.decodeIfPresent(Bool.self, forKey: .isRunning) ?? false
        modalProgramActive = try values.decodeIfPresent(Bool.self, forKey: .modalProgramActive) ?? false
        modalRequiresAlphaLabel = try values.decodeIfPresent(Bool.self, forKey: .modalRequiresAlphaLabel) ?? false
        modalPrompt = try values.decodeIfPresent(String.self, forKey: .modalPrompt)
        clockActive = try values.decodeIfPresent(Bool.self, forKey: .clockActive) ?? false
        stopwatchKeyboardMode = try values.decodeIfPresent(Bool.self, forKey: .stopwatchKeyboardMode) ?? false
        stopwatchRunning = try values.decodeIfPresent(Bool.self, forKey: .stopwatchRunning) ?? false
        pendingYield = try values.decodeIfPresent(YieldStateView.self, forKey: .pendingYield)
        error = try values.decodeIfPresent(String.self, forKey: .error)
    }

    func encode(to encoder: Encoder) throws {
        var values = encoder.container(keyedBy: CodingKeys.self)
        try values.encode(display, forKey: .display)
        try values.encode(x, forKey: .x)
        try values.encode(y, forKey: .y)
        try values.encode(z, forKey: .z)
        try values.encode(t, forKey: .t)
        try values.encode(lastX, forKey: .lastX)
        try values.encode(annunciators, forKey: .annunciators)
        try values.encode(inEEXMode, forKey: .inEEXMode)
        try values.encode(printLines, forKey: .printLines)
        try values.encode(programSteps, forKey: .programSteps)
        try values.encode(pc, forKey: .pc)
        try values.encode(userKeymap, forKey: .userKeymap)
        try values.encode(flags, forKey: .flags)
        try values.encodeIfPresent(displayOverride, forKey: .displayOverride)
        try values.encode(eventBuffer, forKey: .eventBuffer)
        try values.encode(isRunning, forKey: .isRunning)
        try values.encode(modalProgramActive, forKey: .modalProgramActive)
        try values.encode(modalRequiresAlphaLabel, forKey: .modalRequiresAlphaLabel)
        try values.encodeIfPresent(modalPrompt, forKey: .modalPrompt)
        try values.encode(clockActive, forKey: .clockActive)
        try values.encode(stopwatchKeyboardMode, forKey: .stopwatchKeyboardMode)
        try values.encode(stopwatchRunning, forKey: .stopwatchRunning)
        try values.encodeIfPresent(pendingYield, forKey: .pendingYield)
        try values.encodeIfPresent(error, forKey: .error)
    }
}

@MainActor
final class CalculatorModel: ObservableObject {
    @Published private(set) var state = CalculatorState()
    @Published var shiftActive = false
    @Published private(set) var printLog: [String] = []
    @Published var isPrinterPresented = false
    @Published private(set) var outputNotice: OutputNotice?
    @Published private(set) var isExecutionBusy = false
    @Published private var resumedDisplay: String?

    private let service: CalculatorService?
    private var scheduledYield: YieldStateView?
    private var resumeTask: Task<Void, Never>?
    private var liveTickTask: Task<Void, Never>?
    private var sceneIsActive = true
    private var executionTask: Task<Void, Never>?

    init(stateURL: URL? = nil) {
        let url = stateURL ?? Self.defaultStateURL()
        service = CalculatorService(stateURL: url)
        refresh()
    }

    func press(_ key: CalculatorKey) {
        if key.commandID == "shift" {
            shiftActive.toggle()
            return
        }
        if state.annunciators.alpha, let alpha = key.alpha {
            shiftActive = false
            press(id: "alpha:\(alpha == "SPACE" ? " " : alpha)")
            return
        }
        let id = shiftActive ? (key.shiftedID ?? key.commandID) : key.commandID
        shiftActive = false
        guard !id.isEmpty else { return }
        press(id: id)
    }

    func press(id: String) {
        request(.dispatch(keyID: id))
        if id == "r_s" { publishResumedDisplay() }
    }

    func singleStep() { request(.sstStep) }
    func backStep() { request(.bstStep) }
    func runStop() { request(.runStop) }
    func runProgram(label: String) { request(.runProgram(label: label)) }
    func resumeProgram() {
        request(.resumeProgram)
        publishResumedDisplay()
    }
    func resumeProgram(keycode: UInt8) {
        request(.resumeProgramWithKey(keycode: keycode))
        publishResumedDisplay()
    }
    func requestCancel() { request(.requestCancel) }
    func submitModal() { request(.submitModal) }
    func cancelModal() { request(.cancelModal) }
    func submitModal(label: String) {
        request(.submitModalWithLabel(label: label))
    }
    func tickTime() { request(.tickTime) }
    func saveState() { request(.saveState) }
    func resetSoft() {
        shiftActive = false
        request(.resetSoft)
    }
    func resetFull() {
        shiftActive = false
        request(.resetFull)
    }

    var displayText: String {
        state.pendingYield?.text ?? resumedDisplay ?? state.displayOverride ?? state.display
    }

    var isLiveTickScheduled: Bool { liveTickTask != nil }

    /// Returns true whenever GETKEY owns the keyboard. Keys without an HP-41
    /// hardware code are intentionally swallowed while the program remains suspended.
    @discardableResult
    func capturePendingKey(keyCode: Int?) -> Bool {
        guard state.pendingYield?.kind == "wait_for_key" else { return false }
        if let keyCode, let code = UInt8(exactly: keyCode) { resumeProgram(keycode: code) }
        return true
    }

    func startOrStopProgram() {
        if state.modalProgramActive {
            submitModal()
        } else if state.isRunning {
            requestCancel()
        } else {
            runProgram(label: "A")
        }
    }

    func startOrStopProgramInBackground() {
        if state.modalProgramActive {
            submitModal()
        } else if isExecutionBusy {
            requestCancel()
        } else {
            runStopInBackground()
        }
    }

    private func runStopInBackground() {
        guard !isExecutionBusy, let service else { return }
        isExecutionBusy = true
        state.isRunning = true
        executionTask = Task { @MainActor [weak self] in
            let response = await service.requestAsync(.dispatch(keyID: "r_s"))
            guard let self else { return }
            _ = self.applyBoundaryResult(response)
            self.isExecutionBusy = false
            self.executionTask = nil
        }
    }

    func runProgramInBackground(label: String) {
        guard !isExecutionBusy, let service else { return }
        isExecutionBusy = true
        state.isRunning = true
        executionTask = Task { @MainActor [weak self] in
            let response = await service.requestAsync(.runProgram(label: label))
            guard let self else { return }
            _ = self.applyBoundaryResult(response)
            self.isExecutionBusy = false
            self.executionTask = nil
        }
    }

    func setSceneActive(_ active: Bool) {
        sceneIsActive = active
        reconcileLiveTicking()
        if active, state.clockActive || state.stopwatchKeyboardMode { tickTime() }
    }

    func clearPrintLog() {
        printLog.removeAll()
    }

    func dismissOutputNotice() {
        outputNotice = nil
    }

    func consumePendingAppIntent(from mailbox: AppIntentMailbox = AppIntentMailbox()) {
        do {
            guard let intent = try mailbox.take() else { return }
            switch intent.kind {
            case .executeFunction:
                press(id: intent.value)
            case .runProgram:
                runProgramInBackground(label: intent.value)
            }
        } catch {
            showOutputNotice("Invalid App Intent request")
        }
    }

    func inspectRaw(at url: URL) -> [RawProgramInfo]? {
        request(.inspectRaw(path: url.path))?.programs
    }

    @discardableResult
    func importRaw(at url: URL, indices: [Int]) -> FileTransferResult? {
        request(.importRaw(path: url.path, indices: indices))
    }

    @discardableResult
    func exportRaw(to url: URL) -> FileTransferResult? {
        request(.exportRaw(path: url.path))
    }

    @discardableResult
    func importData(at url: URL) -> FileTransferResult? {
        request(.importData(path: url.path))
    }

    @discardableResult
    func exportData(to url: URL) -> FileTransferResult? {
        request(.exportData(path: url.path))
    }

    @discardableResult
    private func request(_ request: CalculatorRequest) -> FileTransferResult? {
        switch request {
        case .resumeProgram, .resumeProgramWithKey:
            break
        default:
            resumedDisplay = nil
        }
        if request.isCancellation, isExecutionBusy {
            service?.cancel()
            return nil
        }
        guard !isExecutionBusy, let service else { return nil }
        return applyBoundaryResult(service.request(request))
    }

    private func refresh() {
        guard let service else { return }
        _ = applyBoundaryResult(service.state())
    }

    private func publishResumedDisplay() {
        guard state.pendingYield == nil else { return }
        resumedDisplay = state.x
        var resumedState = state
        resumedState.displayOverride = nil
        resumedState.display = resumedState.x
        state = resumedState
    }

    @discardableResult
    func applyBoundaryResult(
        _ result: Result<CalculatorResponse, CalculatorServiceError>
    ) -> FileTransferResult? {
        switch result {
        case let .success(response):
            let wasShowingYield = state.pendingYield != nil
            var nextState = response.state
            if wasShowingYield && nextState.pendingYield == nil {
                nextState.displayOverride = nil
                nextState.display = nextState.x
            }
            state = nextState
            consumeTransientOutput(
                printLines: response.state.printLines,
                events: response.state.eventBuffer
            )
            reconcileExecutionDrivers()
            return response.result
        case let .failure(error):
            // Keep every last-known-good calculator field and surface only the
            // boundary failure. A malformed response must never zero the stack.
            state.error = error.message
            return nil
        }
    }

    /// Consumes the bridge's drain-on-read output exactly once for each decoded response.
    /// Kept internal so the prefix routing contract can be regression tested without
    /// constructing alarm records in the calculator core.
    func consumeTransientOutput(printLines: [String], events: [String]) {
        if !printLines.isEmpty {
            printLog.append(contentsOf: printLines)
            isPrinterPresented = true
        }

        for event in events {
            if event.hasPrefix("alarm:message:") {
                showOutputNotice(String(event.dropFirst("alarm:message:".count)))
            } else if event.hasPrefix("alarm:xeq:") {
                let label = String(event.dropFirst("alarm:xeq:".count))
                Task { @MainActor [weak self] in self?.runProgram(label: label) }
            } else if event.hasPrefix("alarm:missing:") {
                let label = String(event.dropFirst("alarm:missing:".count))
                showOutputNotice("Alarm XEQ \(label): label not found")
            } else {
                #if os(macOS)
                if event == "BEEP" || event.hasPrefix("TONE ") { NSSound.beep() }
                #endif
                showOutputNotice(event)
            }
        }
    }

    private func showOutputNotice(_ message: String) {
        let notice = OutputNotice(message: message)
        outputNotice = notice
        Task { @MainActor [weak self] in
            try? await Task.sleep(for: .seconds(2))
            if self?.outputNotice?.id == notice.id { self?.outputNotice = nil }
        }
    }

    private func reconcileExecutionDrivers() {
        reconcileYieldResume()
        reconcileLiveTicking()
    }

    private func reconcileYieldResume() {
        guard let pending = state.pendingYield, pending.kind != "wait_for_key" else {
            resumeTask?.cancel()
            resumeTask = nil
            scheduledYield = nil
            return
        }
        guard scheduledYield != pending else { return }
        resumeTask?.cancel()
        scheduledYield = pending
        resumeTask = Task { @MainActor [weak self] in
            do {
                try await Task.sleep(for: .milliseconds(Int(pending.resumeMS)))
            } catch {
                return
            }
            guard let self, self.scheduledYield == pending else { return }
            self.scheduledYield = nil
            self.resumeTask = nil
            self.resumeProgram()
        }
    }

    private func reconcileLiveTicking() {
        let needsTick = sceneIsActive && (state.clockActive || state.stopwatchKeyboardMode)
        if !needsTick {
            liveTickTask?.cancel()
            liveTickTask = nil
        } else if liveTickTask == nil {
            liveTickTask = Task { @MainActor [weak self] in
                while !Task.isCancelled {
                    do { try await Task.sleep(for: .milliseconds(100)) }
                    catch { return }
                    guard let self else { return }
                    self.tickTime()
                }
            }
        }
    }

    nonisolated static func defaultStateURL() -> URL {
        if let override = ProcessInfo.processInfo.environment["HP41_STATE_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        let manager = FileManager.default
        #if os(macOS)
        return manager.homeDirectoryForCurrentUser
            .appendingPathComponent(".hp41", isDirectory: true)
            .appendingPathComponent("autosave.json")
        #else
        let support = manager.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        return support.appendingPathComponent("autosave.json")
        #endif
    }
}
