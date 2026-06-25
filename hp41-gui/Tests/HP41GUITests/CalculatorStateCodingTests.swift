import Foundation
import XCTest
@testable import HP41GUI

final class CalculatorStateCodingTests: XCTestCase {
    func testDecodesBridgeSnakeCaseLastX() throws {
        let json = #"{"display":"1.0000","x":"1.0000","y":"2.0000","z":"3.0000","t":"4.0000","last_x":"5.0000","annunciators":{"user":true,"prgm":false,"alpha":false,"rad":true,"grad":false},"error":null}"#
        let state = try JSONDecoder().decode(CalculatorState.self, from: Data(json.utf8))
        XCTAssertEqual(state.lastX, "5.0000")
        XCTAssertTrue(state.annunciators.user)
        XCTAssertTrue(state.annunciators.rad)
    }

    func testErrorPayloadDecodes() throws {
        let json = #"{"display":"0.0000","x":"0.0000","y":"0.0000","z":"0.0000","t":"0.0000","last_x":"0.0000","annunciators":{"user":false,"prgm":false,"alpha":false,"rad":false,"grad":false},"error":"DATA ERROR"}"#
        let state = try JSONDecoder().decode(CalculatorState.self, from: Data(json.utf8))
        XCTAssertEqual(state.error, "DATA ERROR")
    }

    func testDecodesCompleteSharedStateView() throws {
        let json = #"{"display_str":"RUNNING","x_str":"1.00","y_str":"2.00","z_str":"3.00","t_str":"4.00","lastx_str":"5.00","in_eex_mode":true,"annunciators":{"user":true,"prgm":false,"alpha":false,"rad":true,"grad":false},"print_lines":["PRINT"],"program_steps":["000 END"],"pc":0,"user_keymap":[[11,"SIN"]],"flags":[5,48],"display_override":"VIEW","event_buffer":["BEEP"],"is_running":true,"modal_program_active":true,"modal_requires_alpha_label":true,"modal_prompt":"FUNCTION?","clock_active":true,"stopwatch_keyboard_mode":true,"stopwatch_running":true,"pending_yield":{"kind":"pse","text":"WAIT","resume_ms":1000},"error":null}"#

        let state = try JSONDecoder().decode(CalculatorState.self, from: Data(json.utf8))
        XCTAssertEqual(state.display, "RUNNING")
        XCTAssertTrue(state.inEEXMode)
        XCTAssertEqual(state.printLines, ["PRINT"])
        XCTAssertEqual(state.userKeymap, [UserKeyAssignment(keyCode: 11, label: "SIN")])
        XCTAssertEqual(state.flags, [5, 48])
        XCTAssertEqual(state.eventBuffer, ["BEEP"])
        XCTAssertTrue(state.modalRequiresAlphaLabel)
        XCTAssertEqual(state.pendingYield, YieldStateView(kind: "pse", text: "WAIT", resumeMS: 1000))
    }

    func testRoundTripPreservesViewContract() throws {
        var source = CalculatorState()
        source.display = "HELLO"
        source.lastX = "9.0000"
        source.annunciators.alpha = true
        XCTAssertEqual(try JSONDecoder().decode(CalculatorState.self, from: JSONEncoder().encode(source)), source)
    }

    func testDecodesRawArchivePickerMetadata() throws {
        let json = #"{"kind":"raw_programs","programs":[{"label":"LBL A (8 bytes)","index":0,"byte_len":8,"step_count":2}]}"#
        let result = try JSONDecoder().decode(FileTransferResult.self, from: Data(json.utf8))
        XCTAssertEqual(
            result.programs,
            [RawProgramInfo(label: "LBL A (8 bytes)", index: 0, byteLength: 8, stepCount: 2)]
        )
    }
}
