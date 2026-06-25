import XCTest
@testable import HP41GUI

final class NativePlatformTests: XCTestCase {
    func testDeclaredSwiftPackageTargetUsesCompileTimePlatformFacts() {
        #if os(macOS)
        XCTAssertEqual(NativePlatform.current, .macOS)
        XCTAssertTrue(NativePlatform.isMacOS)
        XCTAssertFalse(NativePlatform.isIOS)
        #else
        XCTAssertEqual(NativePlatform.current, .iOS)
        XCTAssertFalse(NativePlatform.isMacOS)
        XCTAssertTrue(NativePlatform.isIOS)
        #endif
    }

    func testTouchTargetPolicyIsPlatformNative() {
        XCTAssertEqual(NativePlatform.minimumTouchTarget, NativePlatform.isIOS ? 44 : 0)
    }
}
