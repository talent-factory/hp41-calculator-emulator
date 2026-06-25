import XCTest
@testable import HP41GUI

@MainActor
final class AppIntentMailboxTests: XCTestCase {
    func testMailboxAtomicallyTakesRequestExactlyOnce() throws {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-intent-\(UUID().uuidString).json")
        let mailbox = AppIntentMailbox(url: url)
        let request = PendingNativeAppIntent(kind: .runProgram, value: "SOLVE")
        try mailbox.enqueue(request)
        XCTAssertEqual(try mailbox.take(), request)
        XCTAssertNil(try mailbox.take())
    }

    func testMalformedMailboxIsRemovedInsteadOfLooping() throws {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-intent-bad-\(UUID().uuidString).json")
        try Data("not json".utf8).write(to: url)
        let mailbox = AppIntentMailbox(url: url)
        XCTAssertThrowsError(try mailbox.take())
        XCTAssertFalse(FileManager.default.fileExists(atPath: url.path))
    }

    func testModelConsumesExecuteFunctionThroughTypedDispatch() async throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let model = CalculatorModel(stateURL: directory.appendingPathComponent("state.json"))
        model.press(id: "9")
        let mailbox = AppIntentMailbox(url: directory.appendingPathComponent("intent.json"))
        try mailbox.enqueue(.init(kind: .executeFunction, value: "sqrt"))
        model.consumePendingAppIntent(from: mailbox)
        while model.isExecutionBusy { try await Task.sleep(for: .milliseconds(10)) }
        XCTAssertEqual(model.state.x, "3.0000")
        XCTAssertNil(try mailbox.take())
    }

    func testMailboxRejectsEmptyAndOversizedValues() {
        let mailbox = AppIntentMailbox(url: FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-intent-invalid-\(UUID().uuidString).json"))
        XCTAssertThrowsError(try mailbox.enqueue(.init(kind: .executeFunction, value: "   ")))
        XCTAssertThrowsError(try mailbox.enqueue(.init(kind: .runProgram, value: String(repeating: "A", count: 129))))
    }
}
