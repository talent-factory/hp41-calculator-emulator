// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "HP41GUI",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "hp41-gui", targets: ["HP41GUI"])],
    targets: [
        .target(name: "CHP41", publicHeadersPath: "include"),
        .executableTarget(
            name: "HP41GUI",
            dependencies: ["CHP41"],
            linkerSettings: [
                .unsafeFlags(["-Lhp41-bridge/target/release", "-lhp41_bridge"]),
                .linkedFramework("AppIntents"),
                .linkedFramework("SwiftUI"),
            ]
        ),
        .testTarget(
            name: "HP41GUITests",
            dependencies: ["HP41GUI"],
            resources: [.copy("Fixtures")]
        ),
    ]
)
