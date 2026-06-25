import XCTest
@testable import HP41GUI

final class KeyboardLayoutTests: XCTestCase {
    func testRequiredRPNKeysAreReachable() {
        let ids = Set(KeyboardLayout.rows.flatMap { $0 }.map(\.commandID))
        XCTAssertTrue(["0", "1", "2", "3", "enter", "plus", "minus", "mul", "div"].allSatisfy(ids.contains))
    }

    func testKeysHaveUniqueNonemptyIDs() {
        let ids = KeyboardLayout.rows.flatMap { $0 }.map(\.commandID).filter { !$0.isEmpty }
        XCTAssertEqual(ids.count, Set(ids).count)
    }

    func testEveryEnabledKeyIsAcceptedByNativeBridgeSurface() {
        let enabled = KeyboardLayout.modes + KeyboardLayout.rows.flatMap { $0 }
        let supported = Set([
            "user_mode", "prgm_mode", "alpha_toggle", "sigma_plus", "recip", "sqrt", "log", "ln",
            "xge_y", "rdn", "sin", "cos", "tan", "shift", "sst", "enter", "chs", "e", "clx_or_a", "minus",
            "plus", "mul", "div", "r_s", "xeq_prompt", "sto_prompt", "rcl_prompt",
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".",
        ])
        for key in enabled where !key.commandID.isEmpty {
            XCTAssertTrue(supported.contains(key.commandID), "Unsupported enabled key: \(key.commandID)")
        }
    }

    func testShiftedOperationsAreSupportedOrIntentionallyDisabled() {
        let supported = Set([
            "sigma_minus", "ypow", "sq", "tenpow", "exp", "cl_sigma_stat", "pct_change", "asin", "acos", "atan",
            "bst", "rtn", "clx", "beep", "polar_to_rect", "rect_to_polar", "pi", "lastx",
            "lbl_prompt", "gto_prompt",
            "asn",
            "catalog", "isg_prompt", "sf_prompt", "cf_prompt", "fs_prompt",
            "fix_prompt", "sci_prompt", "eng_prompt", "view",
            "x_eq_y_prompt", "x_le_y_prompt", "x_gt_y_prompt", "x_eq_0_prompt", "clx_or_a",
        ])
        for key in KeyboardLayout.rows.flatMap({ $0 }) {
            guard let shifted = key.shiftedID, !shifted.isEmpty else { continue }
            XCTAssertTrue(supported.contains(shifted), "Unsupported shifted key: \(shifted)")
        }
    }

    func testModeRowMatchesNativeTopRowContract() {
        XCTAssertEqual(KeyboardLayout.modes.map(\.label), ["ON", "USER", "PRGM", "ALPHA"])
    }

    func testAlphaAlphabetIsReachable() {
        let labels = Set(KeyboardLayout.rows.flatMap { $0 }.compactMap(\.alpha))
        for letter in "ABCDEFGHIJKLMNOPQRSTUVWXYZ" {
            XCTAssertTrue(labels.contains(String(letter)), "Missing ALPHA key \(letter)")
        }
    }

    func testEverySwiftUIIdentityIsUnique() {
        let keys = KeyboardLayout.modes + KeyboardLayout.rows.flatMap { $0 }
        XCTAssertEqual(keys.map(\.id).count, Set(keys.map(\.id)).count)
    }

    func testEveryRenderedCalculatorKeyHasAStableUniqueAccessibilityIdentifier() {
        let keys = KeyboardLayout.modes + KeyboardLayout.rows.flatMap { $0 }
        let identifiers = keys.map(\.accessibilityIdentifier)
        XCTAssertEqual(identifiers.count, Set(identifiers).count)
        XCTAssertTrue(identifiers.allSatisfy { $0.hasPrefix("key-") && $0.count > 4 })
        XCTAssertTrue(["key-on", "key-enter", "key-shift", "key-r_s"].allSatisfy(identifiers.contains))
    }

    func testProgramModeShiftedCLPOverrideMatchesTauriSkin() {
        let sqrt = KeyboardLayout.rows.flatMap { $0 }.first(where: { $0.commandID == "sqrt" })
        XCTAssertEqual(sqrt?.shiftedInProgramID, "clp_prompt")
        XCTAssertEqual(sqrt?.shiftedInProgramLabel, "CLP")
    }

    func testCanonicalKeyIdentitySetMatchesRetainedTauriKeyboard() {
        let expected = Set([
            ".", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "acos",
            "alpha_toggle", "asin", "asn", "atan", "beep", "bst", "catalog", "cf_prompt",
            "chs", "cl_sigma_stat", "clp_prompt", "clx_or_a", "cos", "div", "e", "eng_prompt",
            "enter", "exp", "fix_prompt", "fs_prompt", "gto_prompt", "isg_prompt", "lastx",
            "lbl_prompt", "ln", "log", "minus", "mul", "pct_change", "pi", "plus",
            "polar_to_rect", "prgm_mode", "r_s", "rcl_prompt", "rdn", "recip",
            "rect_to_polar", "rtn", "sci_prompt", "sf_prompt", "shift", "sigma_minus",
            "sigma_plus", "sin", "sq", "sqrt", "sst", "sto_prompt", "tan", "tenpow",
            "user_mode", "view", "x_eq_0_prompt", "x_eq_y_prompt", "x_gt_y_prompt",
            "x_le_y_prompt", "xeq_prompt", "xge_y", "ypow",
        ])
        XCTAssertEqual(KeyboardLayout.canonicalCommandIDs, expected)
        XCTAssertEqual(expected.count, 72)
    }

    func testCanonicalLayoutPreservesEnterSpanAndContextualClearIdentity() {
        let enter = KeyboardLayout.rows.flatMap { $0 }.first { $0.commandID == "enter" }
        let clear = KeyboardLayout.rows.flatMap { $0 }.first { $0.commandID == "clx_or_a" }
        XCTAssertEqual(enter?.columnSpan, 2)
        XCTAssertEqual(clear?.shiftedID, "clx_or_a")
        XCTAssertTrue(KeyboardLayout.canonicalCommandIDs.contains("cl_sigma_stat"))
    }

    func testUserAssignmentsRelabelOnlyCanonicalHardwareKeysAndCapLength() throws {
        let sigma = try XCTUnwrap(KeyboardLayout.rows.flatMap { $0 }.first { $0.commandID == "sigma_plus" })
        let chs = try XCTUnwrap(KeyboardLayout.rows.flatMap { $0 }.first { $0.commandID == "chs" })
        let assignments = [UserKeyAssignment(keyCode: 11, label: "LONGNAME")]

        XCTAssertEqual(KeyboardLayout.primaryLabel(for: sigma, userActive: false, assignments: assignments), "Σ+")
        XCTAssertEqual(KeyboardLayout.primaryLabel(for: sigma, userActive: true, assignments: assignments), "LONGNAM")
        XCTAssertEqual(KeyboardLayout.primaryLabel(for: chs, userActive: true, assignments: assignments), "CHS")
    }

    func testPhysicalShortcutInventoryMatchesTauriLedgerExactly() {
        let expected = Set([
            ".", "__save_state__", "acos", "alpha_backspace", "asin", "atan", "bst",
            "cl_sigma_stat", "clreg", "corr", "cos", "div", "e", "enter",
            "entry_backspace", "exp", "hms_add", "hms_sub", "hms_to_h", "lastx", "ln",
            "log", "lr", "mean", "minus", "mul", "pct_change", "plus", "prgm_mode",
            "prx", "rdn", "recip", "sdev", "sigma_minus", "sigma_plus", "sin", "sq",
            "sqrt", "sst", "tan", "tenpow", "user_mode", "xeq_RDPRGM", "xeq_RDTA",
            "xeq_WDTA", "xeq_WPRGM", "xy_swap", "yhat", "ypow",
        ])
        XCTAssertEqual(PhysicalKeyboardRouter.mappedActions, expected)
    }

    func testPhysicalShortcutMapMirrorsCaseSensitiveTauriBindings() {
        let cases: [(String, String)] = [
            ("r", "rdn"), ("x", "xy_swap"), ("p", "prgm_mode"), ("P", "prx"),
            ("q", "sin"), ("C", "cos"), ("E", "exp"), ("Z", "sigma_minus"),
            ("j", "hms_add"), ("J", "hms_sub"), ("%", "pct_change"),
            ("\r", "enter"), ("\u{7f}", "entry_backspace"), ("\u{F70A}", "sst"),
            ("\u{F70B}", "bst"),
        ]
        for (key, expected) in cases {
            XCTAssertEqual(PhysicalKeyboardRouter.route(key: key), .dispatch(expected), key)
        }
    }

    func testPhysicalShortcutContextRoutesModifiersAlphaAndExponentSign() {
        XCTAssertEqual(PhysicalKeyboardRouter.route(key: "s", commandOrControl: true), .saveState)
        XCTAssertEqual(PhysicalKeyboardRouter.route(key: "w", commandOrControl: true), .dispatch("xeq_WPRGM"))
        XCTAssertNil(PhysicalKeyboardRouter.route(key: "v", commandOrControl: true))
        XCTAssertEqual(PhysicalKeyboardRouter.route(key: "q", alphaActive: true), .dispatch("alpha_Q"))
        XCTAssertEqual(PhysicalKeyboardRouter.route(key: "\u{7f}", alphaActive: true), .dispatch("alpha_backspace"))
        XCTAssertEqual(PhysicalKeyboardRouter.route(key: "n", eexActive: false), .dispatch("chs"))
        XCTAssertEqual(PhysicalKeyboardRouter.route(key: "n", eexActive: true), .dispatch("eex_chs"))
    }
}
