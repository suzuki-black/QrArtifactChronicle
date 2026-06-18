// swift-tools-version:5.9
import PackageDescription

// QrArtifactChronicle — macOS アプリ ＋ Rust コア(xcframework)統合。
// QracFFIBinary: Rust 静的ライブラリ(libqrac_ffi.a)を XCFramework 化したもの。
// QracFFI:       UniFFI 生成の Swift API（C モジュール qrac_ffiFFI を import）。
// QrArtifactApp: SwiftUI アプリ本体。
let package = Package(
    name: "QracKit",
    platforms: [.macOS(.v13), .iOS(.v16)],
    products: [
        .executable(name: "QrArtifactApp", targets: ["QrArtifactApp"]),
        .library(name: "QracFFI", targets: ["QracFFI"]),
    ],
    targets: [
        .binaryTarget(name: "QracFFIBinary", path: "../QracFFI.xcframework"),
        .target(
            name: "QracFFI",
            dependencies: ["QracFFIBinary"],
            // Rust 静的ライブラリが必要とするシステムライブラリ。
            linkerSettings: [
                .linkedLibrary("System"),
                .linkedFramework("CoreFoundation"),
            ]
        ),
        .executableTarget(name: "QrArtifactApp", dependencies: ["QracFFI"]),
    ]
)
