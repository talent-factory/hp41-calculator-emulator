#if canImport(AppIntents)
import AppIntents

@available(macOS 13.0, *)
enum NativeHP41Function: String, AppEnum {
    case squareRoot = "sqrt"
    case reciprocal = "recip"
    case square = "sq"
    case sine = "sin"
    case cosine = "cos"
    case tangent = "tan"
    case naturalLogarithm = "ln"
    case commonLogarithm = "log"
    case exponential = "exp"

    static let typeDisplayRepresentation = TypeDisplayRepresentation(name: "HP-41 Function")
    static let caseDisplayRepresentations: [NativeHP41Function: DisplayRepresentation] = [
        .squareRoot: "Square Root", .reciprocal: "Reciprocal", .square: "Square",
        .sine: "Sine", .cosine: "Cosine", .tangent: "Tangent",
        .naturalLogarithm: "Natural Logarithm", .commonLogarithm: "Common Logarithm",
        .exponential: "Exponential",
    ]
}

@available(macOS 13.0, *)
struct OpenNativeHP41Intent: AppIntent {
    static let title: LocalizedStringResource = "Open HP-41"
    static let openAppWhenRun = true
    func perform() async throws -> some IntentResult { .result() }
}

@available(macOS 13.0, *)
struct ExecuteNativeHP41FunctionIntent: AppIntent {
    static let title: LocalizedStringResource = "Execute HP-41 Function"
    static let openAppWhenRun = true
    @Parameter(title: "Function") var function: NativeHP41Function
    static var parameterSummary: some ParameterSummary { Summary("Execute \(\.$function)") }

    func perform() async throws -> some IntentResult & ProvidesDialog {
        try AppIntentMailbox().enqueue(.init(kind: .executeFunction, value: function.rawValue))
        return .result(dialog: "Executing the selected function.")
    }
}

@available(macOS 13.0, *)
struct RunNativeHP41ProgramIntent: AppIntent {
    static let title: LocalizedStringResource = "Run HP-41 Program"
    static let openAppWhenRun = true
    @Parameter(title: "Program Label", requestValueDialog: "What program label should I run?")
    var label: String
    static var parameterSummary: some ParameterSummary { Summary("Run program \(\.$label)") }

    func perform() async throws -> some IntentResult & ProvidesDialog {
        let normalized = label.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard !normalized.isEmpty else { throw $label.needsValueError("Enter a program label.") }
        try AppIntentMailbox().enqueue(.init(kind: .runProgram, value: normalized))
        return .result(dialog: "Running program \(normalized).")
    }
}

@available(macOS 13.0, *)
struct XEQNativeHP41Intent: AppIntent {
    static let title: LocalizedStringResource = "XEQ"
    static let description = IntentDescription("Executes an HP-41 function or program by name.")
    static let openAppWhenRun = true
    @Parameter(title: "Name", requestValueDialog: "What function or program should I execute?")
    var name: String
    @Dependency private var calculator: CalculatorModel
    static var parameterSummary: some ParameterSummary { Summary("XEQ \(\.$name)") }

    func perform() async throws -> some IntentResult & ProvidesDialog {
        let normalized = name.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard !normalized.isEmpty else { throw $name.needsValueError("Enter a function or program name.") }
        await calculator.runProgramInBackground(label: normalized)
        return .result(dialog: "Executing \(normalized).")
    }
}

@available(macOS 13.0, *)
struct NativeHP41AppShortcuts: AppShortcutsProvider {
    static var appShortcuts: [AppShortcut] {
        AppShortcut(intent: OpenNativeHP41Intent(), phrases: ["Open \(.applicationName)"],
                    shortTitle: "Open HP-41", systemImageName: "function")
        AppShortcut(intent: ExecuteNativeHP41FunctionIntent(),
                    phrases: ["Execute a function in \(.applicationName)"],
                    shortTitle: "Execute Function", systemImageName: "function")
        AppShortcut(intent: RunNativeHP41ProgramIntent(),
                    phrases: ["Run a program in \(.applicationName)"],
                    shortTitle: "Run Program", systemImageName: "play.fill")
        AppShortcut(intent: XEQNativeHP41Intent(),
                    phrases: ["XEQ in \(.applicationName)"],
                    shortTitle: "XEQ", systemImageName: "function")
    }
}
#endif
