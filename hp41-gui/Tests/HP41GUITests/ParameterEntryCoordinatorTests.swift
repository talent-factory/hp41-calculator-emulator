import Foundation
import XCTest
@testable import HP41GUI

@MainActor
final class ParameterEntryCoordinatorTests: XCTestCase {
    func testStoreAndRecallUseCanonicalTwoDigitRegisterIDs() {
        let model = CalculatorModel(stateURL: stateURL("register"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "7")
        XCTAssertTrue(coordinator.open(for: "sto_prompt"))
        coordinator.input = "5"
        coordinator.submit(on: model)
        XCTAssertFalse(coordinator.isPresented)

        model.press(id: "clx")
        XCTAssertTrue(coordinator.open(for: "rcl_prompt"))
        coordinator.input = "05"
        coordinator.submit(on: model)
        XCTAssertEqual(model.state.x, "7.0000")
        XCTAssertNil(model.state.error)
    }

    func testIndirectRegisterToggleDispatchesIndirectVariant() {
        let model = CalculatorModel(stateURL: stateURL("indirect"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "5")
        model.press(id: "sto_01")
        model.press(id: "9")
        XCTAssertTrue(coordinator.open(for: "sto_prompt"))
        coordinator.input = "1"
        coordinator.indirect = true
        coordinator.submit(on: model)
        model.press(id: "clx")
        model.press(id: "rcl_05")
        XCTAssertEqual(model.state.x, "9.0000")
    }

    func testLabelEntryUppercasesAndRecordsInProgramMode() {
        let model = CalculatorModel(stateURL: stateURL("label"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "prgm_mode")
        XCTAssertTrue(coordinator.open(for: "lbl_prompt"))
        coordinator.input = "work"
        coordinator.submit(on: model)
        XCTAssertTrue(model.state.programSteps.contains(where: { $0.contains("LBL WORK") }))
        XCTAssertFalse(coordinator.isPresented)
    }

    func testInvalidRegisterKeepsEntryOpenWithoutDispatch() {
        let model = CalculatorModel(stateURL: stateURL("invalid-register"))
        let coordinator = ParameterEntryCoordinator()
        XCTAssertTrue(coordinator.open(for: "rcl_prompt"))
        coordinator.input = "100"
        coordinator.submit(on: model)
        XCTAssertTrue(coordinator.isPresented)
        XCTAssertNil(model.state.error)
    }

    func testASNUsesCanonicalHardwareKeyCodeAndUppercaseLabel() {
        let model = CalculatorModel(stateURL: stateURL("asn"))
        let coordinator = ParameterEntryCoordinator()
        XCTAssertTrue(coordinator.open(for: "asn"))
        coordinator.assignmentKeyCode = 25
        coordinator.input = "work"
        coordinator.submit(on: model)
        XCTAssertEqual(
            model.state.userKeymap,
            [UserKeyAssignment(keyCode: 25, label: "WORK")]
        )
        XCTAssertFalse(coordinator.isPresented)
    }

    func testFormatAndFlagParametersDispatchCanonicalIDs() {
        let model = CalculatorModel(stateURL: stateURL("format-flag"))
        let coordinator = ParameterEntryCoordinator()
        ["1", "enter", "3", "div"].forEach(model.press(id:))
        XCTAssertTrue(coordinator.open(for: "fix_prompt"))
        coordinator.input = "2"
        coordinator.submit(on: model)
        XCTAssertEqual(model.state.display, "0.33")

        XCTAssertTrue(coordinator.open(for: "sf_prompt"))
        coordinator.input = "5"
        coordinator.submit(on: model)
        XCTAssertTrue(model.state.flags.contains(5))
    }

    func testViewToneAndCatalogSingleParameterFlows() {
        let model = CalculatorModel(stateURL: stateURL("single-parameter"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "8")
        model.press(id: "sto_05")
        XCTAssertTrue(coordinator.open(for: "view"))
        coordinator.input = "5"
        coordinator.submit(on: model)
        XCTAssertNil(model.state.error)

        XCTAssertTrue(coordinator.open(for: "tone"))
        coordinator.input = "3"
        coordinator.submit(on: model)
        XCTAssertTrue(model.state.eventBuffer.contains(where: { $0.contains("TONE") }))

        XCTAssertTrue(coordinator.open(for: "catalog"))
        coordinator.input = "1"
        coordinator.submit(on: model)
        XCTAssertNil(model.state.error)
    }

    func testConditionalPromptDispatchesDirectOperationInProgramMode() {
        let model = CalculatorModel(stateURL: stateURL("conditional"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "prgm_mode")
        XCTAssertTrue(coordinator.dispatchDirect("x_eq_y_prompt", on: model))
        XCTAssertTrue(model.state.programSteps.contains(where: { $0.contains("TEST") }))
    }

    func testExtendedRegisterAndFlagVariantsResolve() {
        let model = CalculatorModel(stateURL: stateURL("extended-registers"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "prgm_mode")
        for opener in ["arcl_prompt", "asto_prompt", "dse_prompt", "fc_prompt", "fs_c_prompt", "fc_c_prompt"] {
            XCTAssertTrue(coordinator.open(for: opener), opener)
            coordinator.input = "5"
            coordinator.submit(on: model)
            XCTAssertNil(model.state.error, opener)
        }
        XCTAssertTrue(model.state.programSteps.contains(where: { $0.contains("ARCL 05") }))
        XCTAssertTrue(model.state.programSteps.contains(where: { $0.contains("DSE 05") }))
    }

    func testSTOArithmeticSupportsRegisterStackAndIndirectTargets() {
        let model = CalculatorModel(stateURL: stateURL("sto-arithmetic"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "1"); model.press(id: "0"); model.press(id: "sto_05")
        model.press(id: "2")
        XCTAssertTrue(coordinator.open(for: "sto_arith_prompt"))
        coordinator.arithmeticOperation = "plus"
        coordinator.input = "5"
        coordinator.submit(on: model)
        model.press(id: "rcl_05")
        XCTAssertEqual(model.state.x, "12.0000")

        model.press(id: "prgm_mode")
        XCTAssertTrue(coordinator.open(for: "sto_arith_prompt"))
        coordinator.arithmeticOperation = "mul"
        coordinator.input = "y"
        coordinator.submit(on: model)
        XCTAssertTrue(model.state.programSteps.contains(where: { $0.contains("STO× Y") }))
    }

    func testDeleteSizeAndIndirectBranchParametersResolve() {
        let model = CalculatorModel(stateURL: stateURL("remaining-parameters"))
        let coordinator = ParameterEntryCoordinator()
        XCTAssertTrue(coordinator.open(for: "size_prompt"))
        coordinator.input = "120"
        coordinator.submit(on: model)
        XCTAssertNil(model.state.error)

        model.press(id: "prgm_mode")
        XCTAssertTrue(coordinator.open(for: "del_prompt"))
        coordinator.input = "0"
        coordinator.submit(on: model)
        for opener in ["gto_ind_prompt", "xeq_ind_prompt"] {
            XCTAssertTrue(coordinator.open(for: opener))
            coordinator.input = "5"
            coordinator.submit(on: model)
            XCTAssertNil(model.state.error, opener)
        }
    }

    func testCLPClearsNamedProgramThroughProgramModeOverrideWorkflow() {
        let model = CalculatorModel(stateURL: stateURL("clp"))
        let coordinator = ParameterEntryCoordinator()
        model.press(id: "prgm_mode")
        XCTAssertTrue(coordinator.open(for: "lbl_prompt"))
        coordinator.input = "work"
        coordinator.submit(on: model)
        model.press(id: "plus")
        XCTAssertTrue(model.state.programSteps.contains(where: { $0.contains("LBL WORK") }))

        XCTAssertTrue(coordinator.open(for: "clp_prompt"))
        coordinator.input = "work"
        coordinator.submit(on: model)
        XCTAssertFalse(model.state.programSteps.contains(where: { $0.contains("LBL WORK") }))
        XCTAssertNil(model.state.error)
    }

    private func stateURL(_ name: String) -> URL {
        FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-parameter-\(name)-\(UUID().uuidString).json")
    }
}
