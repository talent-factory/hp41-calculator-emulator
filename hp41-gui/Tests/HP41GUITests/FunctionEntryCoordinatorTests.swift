import Foundation
import XCTest
@testable import HP41GUI

@MainActor
final class FunctionEntryCoordinatorTests: XCTestCase {
    func testGeneratedCatalogContainsOnlyRunnableCanonicalFunctions() {
        XCTAssertEqual(FunctionCatalog.all.count, 296)
        XCTAssertTrue(FunctionCatalog.all.contains(where: { $0.name == "WPRGM" }))
        XCTAssertTrue(FunctionCatalog.all.contains(where: { $0.name == "SIN" }))
        XCTAssertFalse(FunctionCatalog.all.contains(where: { $0.name == "STO" }))
        XCTAssertEqual(Set(FunctionCatalog.all.map(\.name)).count, FunctionCatalog.all.count)
    }

    func testNativeHelpCatalogCoversCanonicalReferenceAndShortcuts() throws {
        XCTAssertEqual(HelpCatalog.all.count, 363)
        XCTAssertEqual(HelpCatalog.shortcuts.count, 61)
        XCTAssertTrue(["HP-41CV Built-in", "Math 1", "Stat 1", "Time", "Adv Conv", "Adv Math", "Extended Memory"]
            .allSatisfy(HelpCatalog.modules.contains))

        let add = try XCTUnwrap(HelpCatalog.all.first { $0.opVariant == "Add" })
        XCTAssertFalse(add.runnable)
        XCTAssertNotNil(add.example)
        let abs = try XCTUnwrap(HelpCatalog.all.first { $0.name == "ABS" })
        XCTAssertTrue(abs.runnable)
        XCTAssertTrue(abs.matches("absolute value"))
    }

    func testCatalogSearchUsesNamesDescriptionsCategoriesAndAliases() {
        let coordinator = FunctionEntryCoordinator()
        coordinator.query = "stopwatch start"
        XCTAssertTrue(coordinator.filteredFunctions.contains(where: { $0.name == "RUNSW" }))
        coordinator.query = "trigonometric sine"
        XCTAssertTrue(coordinator.filteredFunctions.contains(where: { $0.name == "SIN" }))
    }

    func testExecuteDispatchesCanonicalXEQAndDismissesOnSuccess() {
        let model = CalculatorModel(stateURL: temporaryStateURL("execute"))
        model.press(id: "2")
        model.press(id: "chs")
        let coordinator = FunctionEntryCoordinator()
        coordinator.open()
        coordinator.execute("ABS", on: model)
        XCTAssertEqual(model.state.x, "2.0000")
        XCTAssertFalse(coordinator.isPresented)
        XCTAssertNil(model.state.error)
    }

    func testInvalidCustomLabelKeepsSheetOpenAndSurfacesCoreError() {
        let model = CalculatorModel(stateURL: temporaryStateURL("invalid"))
        let coordinator = FunctionEntryCoordinator()
        coordinator.open()
        coordinator.execute("NO SUCH LABEL", on: model)
        XCTAssertTrue(coordinator.isPresented)
        XCTAssertNotNil(model.state.error)
    }

    func testModalFunctionTransitionsSheetToAlphaLabelCollection() {
        let model = CalculatorModel(stateURL: temporaryStateURL("modal"))
        let coordinator = FunctionEntryCoordinator()
        coordinator.open()
        coordinator.execute("SOLVE", on: model)
        XCTAssertTrue(model.state.modalProgramActive)
        XCTAssertTrue(model.state.modalRequiresAlphaLabel)
        XCTAssertEqual(model.state.modalPrompt, "FUNCTION NAME?")
        XCTAssertTrue(coordinator.isPresented)
    }

    private func temporaryStateURL(_ name: String) -> URL {
        FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-function-\(name)-\(UUID().uuidString).json")
    }
}
