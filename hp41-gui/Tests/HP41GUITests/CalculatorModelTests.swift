import Foundation
import XCTest
@testable import HP41GUI

@MainActor
final class CalculatorModelTests: XCTestCase {
    private var temporaryDirectory: URL!
    private var stateURL: URL { temporaryDirectory.appendingPathComponent("autosave.json") }

    override func setUpWithError() throws {
        temporaryDirectory = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-swift-tests-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: temporaryDirectory, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let temporaryDirectory {
            try? FileManager.default.removeItem(at: temporaryDirectory)
        }
    }

    func testFreshCalculatorProjectsCompleteStateView() {
        let model = CalculatorModel(stateURL: stateURL)
        XCTAssertEqual(model.state.display, "0.0000")
        XCTAssertEqual(model.state.x, "0.0000")
        XCTAssertEqual(model.state.y, "0.0000")
        XCTAssertEqual(model.state.z, "0.0000")
        XCTAssertEqual(model.state.t, "0.0000")
        XCTAssertEqual(model.state.lastX, "0.0000")
        XCTAssertEqual(model.state.annunciators, AnnunciatorState())
        XCTAssertNil(model.state.error)
    }

    func testRPNDispatchSmoke() {
        let model = CalculatorModel(stateURL: stateURL)
        ["2", "enter", "3", "plus"].forEach(model.press(id:))
        XCTAssertEqual(model.state.x, "5.0000")
    }

    func testAllArithmeticDispatches() {
        let cases: [(String, String)] = [
            ("plus", "5.0000"), ("minus", "-1.0000"),
            ("mul", "6.0000"), ("div", "0.6667"),
        ]
        for (operation, expected) in cases {
            let url = temporaryDirectory.appendingPathComponent("\(operation).json")
            let model = CalculatorModel(stateURL: url)
            ["2", "enter", "3", operation].forEach(model.press(id:))
            XCTAssertEqual(model.state.x, expected, operation)
        }
    }

    func testDecimalOnEmptySeedsLeadingZero() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: ".")
        XCTAssertEqual(model.state.display, "0.")
        model.press(id: "1")
        XCTAssertEqual(model.state.display, "0.1")
    }

    func testSecondDecimalIsIgnored() {
        let model = CalculatorModel(stateURL: stateURL)
        ["0", ".", "1", "."].forEach(model.press(id:))
        XCTAssertEqual(model.state.display, "0.1")
    }

    func testEEXOnEmptyUsesImplicitMantissa() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "e")
        XCTAssertEqual(model.state.display, "1e")
    }

    func testExponentEntryCapsAtTwoDigitsAndSupportsSignToggle() {
        let model = CalculatorModel(stateURL: stateURL)
        ["1", "e", "2", "3", "4"].forEach(model.press(id:))
        XCTAssertEqual(model.state.display, "1e23")

        model.press(id: "eex_chs")
        XCTAssertEqual(model.state.display, "1e-23")
        model.press(id: "eex_chs")
        XCTAssertEqual(model.state.display, "1e23")
    }

    func testBackspaceEditsEntryBeforeClearingX() {
        let model = CalculatorModel(stateURL: stateURL)
        ["1", "2", "backspace"].forEach(model.press(id:))
        XCTAssertEqual(model.state.display, "1")
        model.press(id: "enter")
        model.press(id: "backspace")
        XCTAssertEqual(model.state.x, "0.0000")
    }

    func testUnknownKeySurfacesErrorAndDoesNotCrash() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "not_a_real_key")
        XCTAssertEqual(model.state.error, "unknown key: not_a_real_key")
    }

    func testSuccessfulKeyClearsPreviousError() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "unknown")
        XCTAssertNotNil(model.state.error)
        model.press(id: "1")
        XCTAssertNil(model.state.error)
    }

    func testShiftIsOneShotAndDispatchesShiftedOperation() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "4")
        model.press(KeyboardLayout.rows[2][0])
        XCTAssertTrue(model.shiftActive)
        model.press(KeyboardLayout.rows[7][1]) // shifted 0 = PI
        XCTAssertFalse(model.shiftActive)
        XCTAssertEqual(model.state.x, "3.1416")
    }

    func testUserModeAnnunciatorToggles() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "user_mode")
        XCTAssertTrue(model.state.annunciators.user)
        model.press(id: "user_mode")
        XCTAssertFalse(model.state.annunciators.user)
    }

    func testProgramModeAnnunciatorToggles() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        XCTAssertTrue(model.state.annunciators.prgm)
        model.press(id: "prgm_mode")
        XCTAssertFalse(model.state.annunciators.prgm)
    }

    func testProgramModeShowsEndInsteadOfXRegister() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        XCTAssertEqual(model.state.display, "000 END")
        XCTAssertNotEqual(model.state.display, model.state.x)
        XCTAssertEqual(model.state.programSteps, ["000 END"])
        XCTAssertEqual(model.state.pc, 0)
    }

    func testProgramEntryBufferOverridesStepDisplay() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        model.press(id: "4")
        XCTAssertEqual(model.state.display, "4")
    }

    func testProgramRecordingProducesAuthenticListingWithEnd() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["2", "enter", "3", "plus"].forEach(model.press(id:))
        XCTAssertEqual(model.state.programSteps, ["000 2.000000000", "001 ENTER", "002 3.000000000", "003 + ", "004 END"])
        XCTAssertEqual(model.state.display, "000 2.000000000")
    }

    func testSSTAndBSTNavigateAndClampProgramCounter() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["2", "enter"].forEach(model.press(id:))
        model.press(id: "sst")
        XCTAssertEqual(model.state.pc, 1)
        XCTAssertEqual(model.state.display, "001 ENTER")
        model.press(id: "sst")
        model.press(id: "sst")
        XCTAssertEqual(model.state.pc, 2)
        XCTAssertEqual(model.state.display, "002 END")
        model.press(id: "bst")
        model.press(id: "bst")
        model.press(id: "bst")
        XCTAssertEqual(model.state.pc, 0)
        XCTAssertEqual(model.state.display, "000 2.000000000")
    }

    func testRecordedProgramRunsFromKeyboard() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["2", "enter", "3", "plus"].forEach(model.press(id:))
        model.press(id: "prgm_mode")
        model.press(id: "r_s")
        XCTAssertEqual(model.state.x, "5.0000")
        XCTAssertEqual(model.state.display, "5.0000")
        XCTAssertFalse(model.state.isRunning)
    }

    func testLabeledProgramRunsThroughSharedRequestBoundary() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["lbl_A", "2", "enter", "3", "plus"].forEach(model.press(id:))
        model.press(id: "prgm_mode")

        model.runProgram(label: "A")

        XCTAssertEqual(model.state.x, "5.0000")
        XCTAssertNil(model.state.pendingYield)
        XCTAssertNil(model.state.error)
    }

    func testStateRoutedRunStopStartsLabelAProgram() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["lbl_A", "4"].forEach(model.press(id:))
        model.press(id: "prgm_mode")

        model.startOrStopProgram()

        XCTAssertEqual(model.state.x, "4.0000")
        XCTAssertFalse(model.state.isRunning)
        XCTAssertNil(model.state.error)
    }

    func testBackgroundProgramExecutionReturnsThroughServiceBoundary() async throws {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["lbl_A", "4"].forEach(model.press(id:))
        model.press(id: "prgm_mode")

        model.runProgramInBackground(label: "A")
        while model.isExecutionBusy { try await Task.sleep(for: .milliseconds(10)) }

        XCTAssertEqual(model.state.x, "4.0000")
        XCTAssertFalse(model.state.isRunning)
        XCTAssertNil(model.state.error)
    }

    func testBackgroundRunStopExecutesCurrentUnlabeledProgram() async throws {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["2", "enter", "3", "plus"].forEach(model.press(id:))
        model.press(id: "prgm_mode")

        model.startOrStopProgramInBackground()
        while model.isExecutionBusy { try await Task.sleep(for: .milliseconds(10)) }

        XCTAssertEqual(model.state.x, "5.0000")
        XCTAssertFalse(model.state.isRunning)
        XCTAssertNil(model.state.error)
    }

    func testLiveTimeDriverTracksCalculatorModeAndSceneActivity() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "xeq_CLOCK")
        XCTAssertTrue(model.state.clockActive)
        XCTAssertTrue(model.isLiveTickScheduled)

        model.setSceneActive(false)
        XCTAssertFalse(model.isLiveTickScheduled)

        model.setSceneActive(true)
        XCTAssertTrue(model.isLiveTickScheduled)
    }

    func testGETKEYYieldResumesThroughSharedRequestBoundary() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["lbl_A", "getkey", "1", "plus"].forEach(model.press(id:))
        model.press(id: "prgm_mode")

        model.runProgram(label: "A")
        XCTAssertEqual(model.state.pendingYield?.kind, "wait_for_key")

        XCTAssertTrue(model.capturePendingKey(keyCode: 11))
        XCTAssertEqual(model.state.x, "12.0000")
        XCTAssertNil(model.state.pendingYield)
        XCTAssertNil(model.state.error)
    }

    func testGETKEYSwallowsKeysWithoutHardwareCodesAndSupportsCancelSentinel() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["lbl_A", "getkey", "1", "plus"].forEach(model.press(id:))
        model.press(id: "prgm_mode")
        model.runProgram(label: "A")

        XCTAssertTrue(model.capturePendingKey(keyCode: nil))
        XCTAssertEqual(model.state.pendingYield?.kind, "wait_for_key")
        XCTAssertTrue(model.capturePendingKey(keyCode: 0))
        XCTAssertNil(model.state.pendingYield)
    }

    func testTimedProgramYieldDisplaysAndAutomaticallyResumes() async throws {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["lbl_A", "1", "pse", "2", "plus"].forEach(model.press(id:))
        model.press(id: "prgm_mode")

        model.runProgram(label: "A")
        let yield = try XCTUnwrap(model.state.pendingYield)
        XCTAssertEqual(yield.kind, "pse")
        XCTAssertEqual(model.displayText, yield.text)

        try await Task.sleep(for: .milliseconds(Int(yield.resumeMS + 150)))
        XCTAssertNil(model.state.pendingYield)
        XCTAssertEqual(model.state.x, "3.0000")
        XCTAssertEqual(model.displayText, "3.0000")
        XCTAssertNil(model.state.error)
    }

    func testAlphaModeUsesKeyAlphaLabels() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "alpha_toggle")
        XCTAssertTrue(model.state.annunciators.alpha)
        model.press(KeyboardLayout.rows[0][0])
        model.press(KeyboardLayout.rows[0][1])
        XCTAssertEqual(model.state.display.trimmingCharacters(in: .whitespaces), "AB")
    }

    func testAlphaBackspaceRemovesCharacter() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "alpha_toggle")
        model.press(id: "alpha:A")
        model.press(id: "backspace")
        XCTAssertEqual(model.state.display.trimmingCharacters(in: .whitespaces), "")
    }

    func testStatePersistsAndReloads() {
        var model: CalculatorModel? = CalculatorModel(stateURL: stateURL)
        ["2", "enter", "3", "mul"].forEach { model?.press(id: $0) }
        XCTAssertTrue(FileManager.default.fileExists(atPath: stateURL.path))
        model = nil
        let reloaded = CalculatorModel(stateURL: stateURL)
        XCTAssertEqual(reloaded.state.x, "6.0000")
    }

    func testExplicitSaveRecreatesAutosave() throws {
        let model = CalculatorModel(stateURL: stateURL)
        ["4", "enter"].forEach(model.press(id:))
        try FileManager.default.removeItem(at: stateURL)

        model.saveState()

        XCTAssertTrue(FileManager.default.fileExists(atPath: stateURL.path))
        XCTAssertEqual(CalculatorModel(stateURL: stateURL).state.x, "4.0000")
    }

    func testSoftAndFullResetPersistTheirSemantics() {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "prgm_mode")
        ["9", "enter"].forEach(model.press(id:))
        model.press(id: "prgm_mode")
        model.press(id: "user_mode")

        model.resetSoft()
        XCTAssertEqual(model.state.x, "0.0000")
        XCTAssertFalse(model.state.annunciators.user)
        XCTAssertNotEqual(model.state.programSteps, ["000 END"])

        model.resetFull()
        XCTAssertEqual(model.state.x, "0.0000")
        XCTAssertEqual(model.state.programSteps, ["000 END"])
        XCTAssertEqual(CalculatorModel(stateURL: stateURL).state.x, "0.0000")
    }

    func testResetTiersClearOneShotShiftState() {
        let model = CalculatorModel(stateURL: stateURL)
        model.shiftActive = true
        model.resetSoft()
        XCTAssertFalse(model.shiftActive)

        model.shiftActive = true
        model.resetFull()
        XCTAssertFalse(model.shiftActive)
    }

    func testStateFileContainsVersionWrapper() throws {
        let model = CalculatorModel(stateURL: stateURL)
        model.press(id: "1")
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: stateURL)) as? [String: Any])
        XCTAssertEqual(object["version"] as? Int, 1)
        XCTAssertNotNil(object["state"] as? [String: Any])
    }

    func testRawFileRequestsRoundTripThroughTypedBridge() throws {
        let rawURL = temporaryDirectory.appendingPathComponent("roundtrip.raw")
        let source = CalculatorModel(stateURL: stateURL)
        source.press(id: "prgm_mode")
        source.press(id: "plus")
        source.press(id: "prgm_mode")

        XCTAssertNotNil(source.exportRaw(to: rawURL))
        XCTAssertTrue(FileManager.default.fileExists(atPath: rawURL.path))
        let programs = source.inspectRaw(at: rawURL)
        XCTAssertEqual(programs?.count, 1)
        XCTAssertEqual(programs?.first?.stepCount, 1)

        let target = CalculatorModel(
            stateURL: temporaryDirectory.appendingPathComponent("imported-state.json")
        )
        XCTAssertNotNil(target.importRaw(at: rawURL, indices: [0]))
        XCTAssertTrue(target.state.programSteps.contains(where: { $0.contains("+") }))
        XCTAssertNil(target.state.error)
    }

    func testCalculatorCardCommandsUseConfiguredNativeCardsDirectory() {
        let cardsURL = temporaryDirectory.appendingPathComponent("cards", isDirectory: true)
        setenv("HP41_CARDS_PATH", cardsURL.path, 1)
        defer { unsetenv("HP41_CARDS_PATH") }

        let writer = CalculatorModel(stateURL: stateURL)
        writer.press(id: "prgm_mode")
        writer.press(id: "plus")
        writer.press(id: "prgm_mode")
        writer.press(id: "alpha:C")
        writer.press(id: "xeq_WPRGM")
        XCTAssertTrue(FileManager.default.fileExists(atPath: cardsURL.appendingPathComponent("C.raw").path))
        XCTAssertNil(writer.state.error)

        let reader = CalculatorModel(
            stateURL: temporaryDirectory.appendingPathComponent("card-reader.json")
        )
        reader.press(id: "alpha:C")
        reader.press(id: "xeq_RDPRGM")
        XCTAssertTrue(reader.state.programSteps.contains(where: { $0.contains("+") }))
        XCTAssertNil(reader.state.error)
    }

    func testCorruptStateFallsBackToFreshCalculator() throws {
        try Data("not json".utf8).write(to: stateURL)
        let model = CalculatorModel(stateURL: stateURL)
        XCTAssertEqual(model.state.x, "0.0000")
        XCTAssertNil(model.state.error)
    }

    func testMissingStateFileIsNormalFirstRun() {
        let model = CalculatorModel(stateURL: stateURL)
        XCTAssertEqual(model.state.display, "0.0000")
    }

    func testLoadedRunningStateIsForcedToStopped() throws {
        let writer = CalculatorModel(stateURL: stateURL)
        writer.press(id: "1")
        var text = try String(contentsOf: stateURL, encoding: .utf8)
        text = text.replacingOccurrences(of: "\"is_running\": false", with: "\"is_running\": true")
        try text.write(to: stateURL, atomically: true, encoding: .utf8)
        let reloaded = CalculatorModel(stateURL: stateURL)
        reloaded.press(id: "2")
        XCTAssertEqual(reloaded.state.display, "12")
    }

    func testDefaultStatePathPreservesCLIInteropContract() {
        XCTAssertTrue(CalculatorModel.defaultStateURL().path.hasSuffix("/.hp41/autosave.json"))
    }

    func testPrinterOutputAccumulatesRepeatedIdenticalDrainedLines() {
        let model = CalculatorModel(stateURL: stateURL)

        model.press(id: "sf_55")
        model.press(id: "7")
        model.press(id: "prx")
        let firstLine = model.printLog
        model.press(id: "prx")

        XCTAssertEqual(firstLine.count, 1)
        XCTAssertEqual(model.printLog, firstLine + firstLine)
        XCTAssertTrue(model.isPrinterPresented)

        model.clearPrintLog()
        XCTAssertTrue(model.printLog.isEmpty)
    }

    func testAlarmEventsArePresentedWithoutTransportPrefixes() {
        let model = CalculatorModel(stateURL: stateURL)

        model.consumeTransientOutput(printLines: [], events: ["alarm:message:WAKE UP"])
        XCTAssertEqual(model.outputNotice?.message, "WAKE UP")

        model.consumeTransientOutput(printLines: [], events: ["alarm:missing:MORNING"])
        XCTAssertEqual(model.outputNotice?.message, "Alarm XEQ MORNING: label not found")

        model.dismissOutputNotice()
        XCTAssertNil(model.outputNotice)
    }
}
