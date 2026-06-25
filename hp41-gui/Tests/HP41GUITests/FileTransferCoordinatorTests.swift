import Foundation
import XCTest
@testable import HP41GUI

@MainActor
final class FileTransferCoordinatorTests: XCTestCase {
    private var directory: URL!

    override func setUpWithError() throws {
        directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-files-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let directory { try? FileManager.default.removeItem(at: directory) }
    }

    func testMultiProgramPickerImportsOnlySelectedPrograms() throws {
        let firstURL = directory.appendingPathComponent("first.raw")
        let secondURL = directory.appendingPathComponent("second.raw")
        let archiveURL = directory.appendingPathComponent("archive.raw")
        let first = model(named: "first")
        record("plus", in: first)
        XCTAssertNotNil(first.exportRaw(to: firstURL))
        let second = model(named: "second")
        record("minus", in: second)
        XCTAssertNotNil(second.exportRaw(to: secondURL))
        var archive = try Data(contentsOf: firstURL)
        archive.append(try Data(contentsOf: secondURL))
        try archive.write(to: archiveURL)

        let target = model(named: "target")
        let coordinator = FileTransferCoordinator()
        coordinator.prepareRawImport(from: archiveURL, for: target)
        XCTAssertEqual(coordinator.pendingRawImport?.programs.count, 2)
        XCTAssertEqual(coordinator.selectedProgramIndices, [0, 1])

        coordinator.selectedProgramIndices = [1]
        coordinator.confirmRawImport(for: target)
        XCTAssertNil(coordinator.pendingRawImport)
        XCTAssertTrue(target.state.programSteps.contains(where: { $0.contains("-") }))
        XCTAssertFalse(target.state.programSteps.contains(where: { $0.contains("+") }))
        XCTAssertEqual(coordinator.notice?.title, "Program Import Complete")
    }

    func testEmptyRawArchiveProducesNoticeWithoutPicker() throws {
        let url = directory.appendingPathComponent("empty.raw")
        try Data().write(to: url)
        let coordinator = FileTransferCoordinator()
        coordinator.prepareRawImport(from: url, for: model(named: "empty"))
        XCTAssertNil(coordinator.pendingRawImport)
        XCTAssertEqual(coordinator.notice?.title, "No Programs Found")
    }

    func testDataCardExportAndImportUseNativeCoordinatorBoundary() {
        let url = directory.appendingPathComponent("data.card.json")
        let coordinator = FileTransferCoordinator()
        let source = model(named: "data-source")
        source.press(id: "4")
        source.press(id: "sto_07")
        coordinator.exportData(to: url, for: source)
        XCTAssertTrue(FileManager.default.fileExists(atPath: url.path))

        let target = model(named: "data-target")
        coordinator.importData(from: url, for: target)
        target.press(id: "rcl_07")
        XCTAssertEqual(target.state.x, "4.0000")
        XCTAssertEqual(coordinator.notice?.title, "Data Card Imported")
    }

    private func model(named name: String) -> CalculatorModel {
        CalculatorModel(stateURL: directory.appendingPathComponent("\(name).json"))
    }

    private func record(_ operation: String, in model: CalculatorModel) {
        model.press(id: "prgm_mode")
        model.press(id: operation)
        model.press(id: "prgm_mode")
    }
}
