import Foundation
import XCTest
@testable import HP41GUI

final class CalculatorContractTests: XCTestCase {
    func testEveryFacadeRequestHasAStableTypedRoundTrip() throws {
        let requests: [CalculatorRequest] = [
            .getState,
            .dispatch(keyID: "plus"),
            .sstStep,
            .bstStep,
            .runStop,
            .runProgram(label: "A"),
            .resumeProgram,
            .resumeProgramWithKey(keycode: 11),
            .requestCancel,
            .submitModal,
            .cancelModal,
            .submitModalWithLabel(label: "F"),
            .tickTime,
            .saveState,
            .resetSoft,
            .resetFull,
            .inspectRaw(path: "/tmp/a.raw"),
            .importRaw(path: "/tmp/a.raw", indices: [0, 2]),
            .exportRaw(path: "/tmp/b.raw"),
            .importData(path: "/tmp/a.card.json"),
            .exportData(path: "/tmp/b.card.json"),
        ]

        let decoder = JSONDecoder()
        let encoder = JSONEncoder()
        for request in requests {
            XCTAssertEqual(try decoder.decode(CalculatorRequest.self, from: encoder.encode(request)), request)
        }
        XCTAssertEqual(requests.count, 21)
    }

    func testResponseStatusAndErrorMustAgree() throws {
        var state = CalculatorState()
        state.error = "DATA ERROR"
        let valid = try responseData(state: state, status: "error", error: "DATA ERROR")
        let response = try JSONDecoder().decode(CalculatorResponse.self, from: valid)
        XCTAssertEqual(response.status, .error)
        XCTAssertEqual(response.error, "DATA ERROR")

        let invalid = try responseData(state: state, status: "ok", error: "DATA ERROR")
        XCTAssertThrowsError(try JSONDecoder().decode(CalculatorResponse.self, from: invalid))
    }

    func testLegacyResponseWithoutStatusInfersItFromError() throws {
        let state = CalculatorState()
        let data = try responseData(state: state, status: nil, error: nil)
        XCTAssertEqual(try JSONDecoder().decode(CalculatorResponse.self, from: data).status, .ok)
    }

    @MainActor
    func testBoundaryFailurePreservesLastValidCalculatorState() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-contract-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }

        let model = CalculatorModel(stateURL: directory.appendingPathComponent("state.json"))
        model.press(id: "7")
        let lastDisplay = model.state.display
        let lastX = model.state.x

        model.applyBoundaryResult(.failure(.decoding("truncated JSON")))

        XCTAssertEqual(model.state.display, lastDisplay)
        XCTAssertEqual(model.state.x, lastX)
        XCTAssertEqual(
            model.state.error,
            "Calculator response decoding failed: truncated JSON"
        )
    }

    @MainActor
    func testSwiftDisplayUsesYieldThenOverrideThenSharedStatePrecedence() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-display-contract-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let model = CalculatorModel(stateURL: directory.appendingPathComponent("state.json"))

        var state = CalculatorState()
        state.display = "SHARED"
        state.displayOverride = "OVERRIDE"
        state.pendingYield = YieldStateView(kind: "pse", text: "YIELD", resumeMS: 60_000)
        model.applyBoundaryResult(.success(CalculatorResponse(status: .ok, state: state)))
        XCTAssertEqual(model.displayText, "YIELD")

        state.pendingYield = nil
        model.applyBoundaryResult(.success(CalculatorResponse(status: .ok, state: state)))
        XCTAssertEqual(model.displayText, "OVERRIDE")

        state.displayOverride = nil
        model.applyBoundaryResult(.success(CalculatorResponse(status: .ok, state: state)))
        XCTAssertEqual(model.displayText, "SHARED")
    }

    private func responseData(
        state: CalculatorState,
        status: String?,
        error: String?
    ) throws -> Data {
        var object = try XCTUnwrap(
            JSONSerialization.jsonObject(with: JSONEncoder().encode(state)) as? [String: Any]
        )
        if let status { object["status"] = status }
        object["error"] = error ?? NSNull()
        return try JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
    }
}
