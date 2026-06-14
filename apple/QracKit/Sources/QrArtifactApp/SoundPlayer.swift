import AVFoundation
import Foundation

/// 効果音プレーヤ。同梱 Resources/sounds/<name>.wav を再生する。
final class SoundPlayer {
    static let shared = SoundPlayer()
    private var players: [String: AVAudioPlayer] = [:]

    func play(_ name: String) {
        guard let p = player(name) else { return }
        p.currentTime = 0
        p.play()
    }

    func stop(_ name: String) { players[name]?.stop() }

    private func player(_ name: String) -> AVAudioPlayer? {
        if let p = players[name] { return p }
        guard let url = Bundle.main.resourceURL?
            .appendingPathComponent("sounds/\(name).wav"),
              let p = try? AVAudioPlayer(contentsOf: url) else { return nil }
        p.prepareToPlay()
        players[name] = p
        return p
    }
}
