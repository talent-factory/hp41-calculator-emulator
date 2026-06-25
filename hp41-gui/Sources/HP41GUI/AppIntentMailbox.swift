import Foundation

struct PendingNativeAppIntent: Codable, Equatable {
    enum Kind: String, Codable {
        case executeFunction = "execute_function"
        case runProgram = "run_program"
    }

    let kind: Kind
    let value: String
}

extension Notification.Name {
    static let hp41AppIntentEnqueued = Notification.Name("hp41.app-intent-enqueued")
}

struct AppIntentMailbox {
    let url: URL

    init(url: URL? = nil) {
        self.url = url ?? Self.defaultURL()
    }

    func enqueue(_ request: PendingNativeAppIntent) throws {
        let value = request.value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !value.isEmpty, value.count <= 128 else { throw MailboxError.invalidValue }
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(),
                                                withIntermediateDirectories: true)
        try JSONEncoder().encode(request).write(to: url, options: .atomic)
        NotificationCenter.default.post(name: .hp41AppIntentEnqueued, object: nil)
    }

    func take() throws -> PendingNativeAppIntent? {
        guard FileManager.default.fileExists(atPath: url.path) else { return nil }
        let data = try Data(contentsOf: url)
        try FileManager.default.removeItem(at: url)
        let request = try JSONDecoder().decode(PendingNativeAppIntent.self, from: data)
        let value = request.value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !value.isEmpty, value.count <= 128 else { throw MailboxError.invalidValue }
        return request
    }

    static func defaultURL() -> URL {
        if let override = ProcessInfo.processInfo.environment["HP41_APP_INTENT_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let bundleID = Bundle.main.bundleIdentifier ?? "ch.talent-factory.hp41"
        return support.appendingPathComponent(bundleID, isDirectory: true)
            .appendingPathComponent("pending-app-intent.json")
    }

    enum MailboxError: Error { case invalidValue }
}
