import Foundation
import XCTest
@testable import HP41GUI

@MainActor
final class PerformanceSoakTests: XCTestCase {
    private var directory: URL!

    override func setUpWithError() throws {
        directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-soak-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let directory { try? FileManager.default.removeItem(at: directory) }
    }

    func testRapidKeyInputMaintainsDeterministicStackState() {
        let model = makeModel("rapid-input")
        for _ in 0..<100 {
            ["1", "enter", "1", "plus", "clx"].forEach(model.press(id:))
        }
        XCTAssertEqual(model.state.x, "0.0000")
        XCTAssertNil(model.state.error)
    }

    func testLongImportedProgramRunsToCompletion() throws {
        let model = makeModel("long-program")
        let raw = directory.appendingPathComponent("long.raw")
        try rawProgram(nullCount: 2_500, loops: false).write(to: raw)
        XCTAssertNotNil(model.importRaw(at: raw, indices: [0]))

        model.runProgram(label: "A")
        XCTAssertFalse(model.state.isRunning)
        XCTAssertNil(model.state.pendingYield)
        XCTAssertNil(model.state.error)
    }

    func testRepeatedTickCyclesRemainStable() {
        let model = makeModel("ticks")
        model.press(id: "xeq_CLOCK")
        for _ in 0..<500 { model.tickTime() }
        XCTAssertTrue(model.state.clockActive)
        XCTAssertNil(model.state.error)
    }

    func testCancellationStopsLoopingBackgroundProgram() async throws {
        let model = makeModel("cancellation")
        let raw = directory.appendingPathComponent("loop.raw")
        try rawProgram(nullCount: 1_000, loops: true).write(to: raw)
        XCTAssertNotNil(model.importRaw(at: raw, indices: [0]))

        model.runProgramInBackground(label: "A")
        while !model.isExecutionBusy { try await Task.sleep(for: .milliseconds(1)) }
        model.requestCancel()
        while model.isExecutionBusy { try await Task.sleep(for: .milliseconds(2)) }

        XCTAssertFalse(model.state.isRunning)
        XCTAssertNil(model.state.pendingYield)
    }

    func testPrinterBurstAccumulatesAndClearsEveryLine() {
        let model = makeModel("printer")
        model.press(id: "sf_55")
        for value in 0..<50 {
            model.press(id: "clx")
            for digit in String(value) { model.press(id: String(digit)) }
            model.press(id: "prx")
        }
        XCTAssertEqual(model.printLog.count, 50)
        XCTAssertTrue(model.printLog.last?.contains("49") == true)
        model.clearPrintLog()
        XCTAssertTrue(model.printLog.isEmpty)
    }

    func testEventBurstDrainsInOrderWithoutTransportArtifacts() {
        let model = makeModel("events")
        let events = (0..<100).map { "EVENT \($0)" }
        model.consumeTransientOutput(printLines: [], events: events)
        XCTAssertEqual(model.outputNotice?.message, "EVENT 99")
        XCTAssertFalse(model.outputNotice?.message.contains("alarm:") == true)
        model.dismissOutputNotice()
        XCTAssertNil(model.outputNotice)
    }

    func testRepeatedRawImportsPreserveProgramListingIntegrity() throws {
        let model = makeModel("imports")
        let raw = directory.appendingPathComponent("sqrt.raw")
        try Data([0x52, 0xC0, 0x00, 0x0D]).write(to: raw)
        for _ in 0..<25 {
            XCTAssertNotNil(model.importRaw(at: raw, indices: [0]))
        }
        XCTAssertEqual(model.state.programSteps.filter { $0.contains("√x") }.count, 25)
        XCTAssertEqual(model.state.programSteps.last, String(format: "%03d END", 25))
        XCTAssertNil(model.state.error)
    }

    private func makeModel(_ name: String) -> CalculatorModel {
        CalculatorModel(stateURL: directory.appendingPathComponent("\(name).json"))
    }

    private func rawProgram(nullCount: Int, loops: Bool) -> Data {
        var bytes = Data([0xCF, 0xF1, 0x41]) // LBL "A"
        bytes.append(Data(repeating: 0xCD, count: nullCount))
        if loops { bytes.append(contentsOf: [0x1D, 0xF1, 0x41]) } // GTO "A"
        bytes.append(contentsOf: [0xC0, 0x00, 0x0D])
        return bytes
    }
}
