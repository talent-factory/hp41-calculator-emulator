import Foundation

enum NativePlatform: Equatable {
    case macOS
    case iOS

    static var current: NativePlatform {
        #if os(iOS)
        .iOS
        #else
        .macOS
        #endif
    }

    static var isMacOS: Bool { current == .macOS }
    static var isIOS: Bool { current == .iOS }
    static var minimumTouchTarget: Double { isIOS ? 44 : 0 }
}
