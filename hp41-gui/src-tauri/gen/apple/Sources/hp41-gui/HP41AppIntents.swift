import AppIntents
import Foundation

@_silgen_name("hp41_app_intent_enqueued")
private func hp41AppIntentEnqueued()

@available(iOS 16.0, macOS 13.0, *)
private struct PendingHP41Intent: Codable {
    let kind: String
    let value: String
}

@available(iOS 16.0, macOS 13.0, *)
private enum HP41IntentMailbox {
    static func enqueue(kind: String, value: String) throws {
        guard let applicationSupport = FileManager.default.urls(
            for: .applicationSupportDirectory,
            in: .userDomainMask
        ).first else {
            throw CocoaError(.fileNoSuchFile)
        }

        let bundleID = Bundle.main.bundleIdentifier ?? "ch.talent-factory.hp41"
        let directory = applicationSupport.appendingPathComponent(bundleID, isDirectory: true)
        try FileManager.default.createDirectory(
            at: directory,
            withIntermediateDirectories: true
        )

        let request = PendingHP41Intent(kind: kind, value: value)
        let data = try JSONEncoder().encode(request)
        try data.write(
            to: directory.appendingPathComponent("pending-app-intent.json"),
            options: .atomic
        )
        hp41AppIntentEnqueued()
    }
}

@available(iOS 16.0, macOS 13.0, *)
enum HP41Function: String, AppEnum {
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

    static let caseDisplayRepresentations: [HP41Function: DisplayRepresentation] = [
        .squareRoot: "Square Root",
        .reciprocal: "Reciprocal",
        .square: "Square",
        .sine: "Sine",
        .cosine: "Cosine",
        .tangent: "Tangent",
        .naturalLogarithm: "Natural Logarithm",
        .commonLogarithm: "Common Logarithm",
        .exponential: "Exponential",
    ]
}

@available(iOS 16.0, macOS 13.0, *)
struct OpenHP41Intent: AppIntent {
    static let title: LocalizedStringResource = "Open HP-41"
    static let description = IntentDescription("Opens the HP-41 calculator.")
    static let openAppWhenRun = true

    func perform() async throws -> some IntentResult & ProvidesDialog {
        .result(dialog: "Opening HP-41.")
    }
}

@available(iOS 16.0, macOS 13.0, *)
struct ExecuteHP41FunctionIntent: AppIntent {
    static let title: LocalizedStringResource = "Execute HP-41 Function"
    static let description = IntentDescription("Executes a function using the current HP-41 stack.")
    static let openAppWhenRun = true

    @Parameter(title: "Function")
    var function: HP41Function

    static var parameterSummary: some ParameterSummary {
        Summary("Execute \(\.$function)")
    }

    func perform() async throws -> some IntentResult & ProvidesDialog {
        try HP41IntentMailbox.enqueue(kind: "execute_function", value: function.rawValue)
        return .result(dialog: "Executing the selected function.")
    }
}

@available(iOS 16.0, macOS 13.0, *)
struct RunHP41ProgramIntent: AppIntent {
    static let title: LocalizedStringResource = "Run HP-41 Program"
    static let description = IntentDescription("Runs a stored HP-41 program by label.")
    static let openAppWhenRun = true

    @Parameter(title: "Program Label", requestValueDialog: "What program label should I run?")
    var label: String

    static var parameterSummary: some ParameterSummary {
        Summary("Run program \(\.$label)")
    }

    func perform() async throws -> some IntentResult & ProvidesDialog {
        let normalizedLabel = label.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard !normalizedLabel.isEmpty else {
            throw $label.needsValueError("Enter a program label.")
        }
        try HP41IntentMailbox.enqueue(kind: "run_program", value: normalizedLabel)
        return .result(dialog: "Running program \(normalizedLabel).")
    }
}

@available(iOS 16.0, macOS 13.0, *)
struct HP41AppShortcuts: AppShortcutsProvider {
    static var appShortcuts: [AppShortcut] {
        AppShortcut(
            intent: OpenHP41Intent(),
            phrases: [
                "Open \(.applicationName)",
                "Show my \(.applicationName)",
            ],
            shortTitle: "Open HP-41",
            systemImageName: "function"
        )
        AppShortcut(
            intent: ExecuteHP41FunctionIntent(),
            phrases: [
                "Calculate with \(.applicationName)",
                "Execute a function in \(.applicationName)",
            ],
            shortTitle: "Execute Function",
            systemImageName: "function"
        )
        AppShortcut(
            intent: RunHP41ProgramIntent(),
            phrases: [
                "Run a program in \(.applicationName)",
                "Start an HP-41 program with \(.applicationName)",
            ],
            shortTitle: "Run Program",
            systemImageName: "play.fill"
        )
    }
}
