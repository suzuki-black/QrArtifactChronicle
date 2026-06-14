import SwiftUI

@main
struct QrArtifactApp: App {
    var body: some Scene {
        WindowGroup("QR考古学") {
            ContentView()
        }
        .defaultSize(width: 420, height: 720)
        .windowResizability(.contentMinSize)
    }
}
