import SwiftUI

@main
struct QrArtifactApp: App {
    var body: some Scene {
        WindowGroup("QrArtifactChronicle") {
            ContentView()
                // UIは明色テーマ前提。端末がダークモードでも文字が白で溶けないよう固定。
                .preferredColorScheme(.light)
        }
        // ウィンドウサイズ系は macOS の Scene 修飾子（iOS では全画面なので不要）。
        #if os(macOS)
        .defaultSize(width: 420, height: 720)
        .windowResizability(.contentMinSize)
        #endif
    }
}
