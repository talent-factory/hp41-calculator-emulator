import Foundation
import SwiftUI

struct ThemeColor: Equatable {
    let red: Double
    let green: Double
    let blue: Double

    var color: Color { Color(red: red, green: green, blue: blue) }

    func contrastRatio(with other: ThemeColor) -> Double {
        func luminance(_ color: ThemeColor) -> Double {
            func channel(_ value: Double) -> Double {
                value <= 0.03928 ? value / 12.92 : pow((value + 0.055) / 1.055, 2.4)
            }
            return 0.2126 * channel(color.red) + 0.7152 * channel(color.green) + 0.0722 * channel(color.blue)
        }
        let values = [luminance(self), luminance(other)].sorted(by: >)
        return (values[0] + 0.05) / (values[1] + 0.05)
    }
}

struct AppThemePalette {
    let bodyTop: ThemeColor
    let bodyBottom: ThemeColor
    let displayBackground: ThemeColor
    let displayText: ThemeColor
    let key: ThemeColor
    let enter: ThemeColor
    let shift: ThemeColor
    let shiftActive: ThemeColor
    let primaryText: ThemeColor
    let accent: ThemeColor
    let trim: ThemeColor
}

enum AppTheme: String, CaseIterable, Codable, Identifiable {
    case dark
    case light
    case classicBeige = "classic-beige"
    case highContrast = "high-contrast"

    var id: String { rawValue }
    var title: String {
        switch self {
        case .dark: "Dark"
        case .light: "Light"
        case .classicBeige: "Classic Beige"
        case .highContrast: "High Contrast"
        }
    }

    var palette: AppThemePalette {
        func rgb(_ value: UInt32) -> ThemeColor {
            ThemeColor(
                red: Double((value >> 16) & 0xff) / 255,
                green: Double((value >> 8) & 0xff) / 255,
                blue: Double(value & 0xff) / 255
            )
        }
        switch self {
        case .dark:
            return AppThemePalette(bodyTop: rgb(0x1a1a1a), bodyBottom: rgb(0x000000),
                            displayBackground: rgb(0x8c9c7d), displayText: rgb(0x14281a),
                            key: rgb(0x181818), enter: rgb(0x1a3a1a), shift: rgb(0xb06811),
                            shiftActive: rgb(0xf5a423), primaryText: rgb(0xe8e8e8),
                            accent: rgb(0xf5a423), trim: rgb(0xc8b878))
        case .light:
            return AppThemePalette(bodyTop: rgb(0xd4d4d4), bodyBottom: rgb(0xc4c4c4),
                            displayBackground: rgb(0xc8d8c8), displayText: rgb(0x1a3a1a),
                            key: rgb(0xa8a8a8), enter: rgb(0x2e6a2e), shift: rgb(0x9a5808),
                            shiftActive: rgb(0xe09010), primaryText: rgb(0x111111),
                            accent: rgb(0xc07010), trim: rgb(0xb8a060))
        case .classicBeige:
            return AppThemePalette(bodyTop: rgb(0xb8a880), bodyBottom: rgb(0xa89870),
                            displayBackground: rgb(0x4a5030), displayText: rgb(0xd8d890),
                            key: rgb(0x282010), enter: rgb(0x1a3818), shift: rgb(0xa85e08),
                            shiftActive: rgb(0xe08818), primaryText: rgb(0xf0e8d0),
                            accent: rgb(0xc8780a), trim: rgb(0xc8a848))
        case .highContrast:
            return AppThemePalette(bodyTop: rgb(0x111111), bodyBottom: rgb(0x000000),
                            displayBackground: rgb(0x000000), displayText: rgb(0xffffff),
                            key: rgb(0x000000), enter: rgb(0x002200), shift: rgb(0x884400),
                            shiftActive: rgb(0xffaa00), primaryText: rgb(0xffffff),
                            accent: rgb(0xffaa00), trim: rgb(0xffffff))
        }
    }
}

private struct StoredPreferences: Codable {
    var theme = AppTheme.dark.rawValue
    var onboardingDone = false

    enum CodingKeys: String, CodingKey {
        case theme
        case onboardingDone = "onboarding_done"
    }

    init() {}

    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        theme = try values.decodeIfPresent(String.self, forKey: .theme) ?? AppTheme.dark.rawValue
        onboardingDone = try values.decodeIfPresent(Bool.self, forKey: .onboardingDone) ?? false
    }
}

@MainActor
final class AppPreferences: ObservableObject {
    @Published private(set) var theme: AppTheme
    @Published private(set) var onboardingDone: Bool
    private var stored: StoredPreferences
    private let url: URL

    init(url: URL? = nil) {
        self.url = url ?? Self.defaultURL()
        let decoded = (try? Data(contentsOf: self.url)).flatMap { try? JSONDecoder().decode(StoredPreferences.self, from: $0) }
            ?? StoredPreferences()
        stored = decoded
        theme = AppTheme(rawValue: decoded.theme) ?? .dark
        onboardingDone = decoded.onboardingDone
    }

    func setTheme(_ theme: AppTheme) {
        self.theme = theme
        stored.theme = theme.rawValue
        save()
    }

    func completeOnboarding() {
        onboardingDone = true
        stored.onboardingDone = true
        save()
    }

    private func save() {
        guard let data = try? JSONEncoder.pretty.encode(stored) else { return }
        try? FileManager.default.createDirectory(at: url.deletingLastPathComponent(),
                                                 withIntermediateDirectories: true)
        try? data.write(to: url, options: .atomic)
    }

    nonisolated static func defaultURL() -> URL {
        if let override = ProcessInfo.processInfo.environment["HP41_PREFS_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        return FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".hp41", isDirectory: true)
            .appendingPathComponent("prefs.json")
    }
}

private extension JSONEncoder {
    static var pretty: JSONEncoder {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return encoder
    }
}
