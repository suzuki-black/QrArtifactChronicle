import Foundation
import SwiftUI
import QracFFI

/// ゲーム画面の状態。すべて Rust コア(deriveQr/renderQr…)を呼ぶだけ。
@MainActor
final class GameModel: ObservableObject {
    // デバッグ手入力
    @Published var input: String = ""
    @Published var useYear: Bool = false
    @Published var year: Int32 = 1985

    // 表示中の遺物
    @Published var artifact: Artifact?
    @Published var image: NSImage?
    @Published var baseName: String = ""
    @Published var stage: UInt8 = 0
    @Published var descriptionText: String = ""

    // 発掘演出
    enum DigPhase: Equatable { case idle, digging, scolding }
    @Published var phase: DigPhase = .idle
    private var discovered = Set<String>()        // 発掘済み artifactHash
    private var pendingArt: Artifact?
    private var pendingRender: RenderedImage?
    private var isNewDiscovery = false

    // 展示室（発掘済み一覧）
    struct Collected: Identifiable, Equatable, Codable {
        let id: String          // artifactHash
        let name: String
        let civ: String
        let era: String
        let category: String
        let baseRarity: Int
        let eraBonus: Int
        let finalRarity: Int
        let isMythic: Bool
        let preservation: Double
        let dirt: String
        let chip: Bool
        let crack: Bool
        let wear: Bool
        let stage: Int
        let png: Data
        let description: String
        var haystack: String { "\(name) \(civ) \(era) \(category) \(description)" }
    }
    @Published var collected: [Collected] = []

    private var assetsConfigured = false
    private let saveURL: URL

    init() {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let dir = base.appendingPathComponent("QrArtifactChronicle", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        saveURL = dir.appendingPathComponent("collection.json")
        load()
    }

    private func load() {
        guard let data = try? Data(contentsOf: saveURL),
              let items = try? JSONDecoder().decode([Collected].self, from: data) else { return }
        collected = items
        discovered = Set(items.map(\.id))
    }

    private func save() {
        if let data = try? JSONEncoder().encode(collected) { try? data.write(to: saveURL) }
    }

    /// 新規なら展示室に追加して保存（自動保存）。既出なら何もしない。
    private func registerIfNew(_ a: Artifact, _ r: RenderedImage) {
        guard !discovered.contains(a.artifactHash) else { return }
        discovered.insert(a.artifactHash)
        collected.append(Collected(
            id: a.artifactHash, name: r.baseName,
            civ: a.civ, era: a.era, category: a.category,
            baseRarity: Int(a.baseRarity), eraBonus: Int(a.eraBonus),
            finalRarity: Int(a.finalRarity), isMythic: a.isMythic,
            preservation: a.preservationScore, dirt: a.dirtLayerId,
            chip: a.damage.chip, crack: a.damage.crack, wear: a.damage.wear,
            stage: Int(r.matchedStage),
            png: Data(r.png), description: r.description))
        save()
    }

    /// 展示室をリセット（デバッグ専用）。
    func resetCollection() {
        collected = []
        discovered = []
        try? FileManager.default.removeItem(at: saveURL)
    }

    func configureAssetsIfBundled() {
        guard !assetsConfigured else { return }
        assetsConfigured = true
        if let res = Bundle.main.resourceURL?.appendingPathComponent("assets"),
           FileManager.default.fileExists(atPath: res.path) {
            configureAssets(dir: res.path)
        }
    }

    private func derive() -> Artifact {
        useYear ? deriveQrWithYear(text: input, year: year) : deriveQr(text: input)
    }
    private func render() -> RenderedImage {
        useYear ? renderQrWithYear(text: input, year: year) : renderQr(text: input)
    }

    /// 入力が空（空白のみ含む）か。空文字のQRは現実に存在しないため発掘不可。
    var canExcavate: Bool {
        !input.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    /// 無演出で即表示（起動・プリセット・年変更用）。発掘済み登録はしない。
    func dig() {
        guard canExcavate else { return }
        commit(derive(), render())
    }

    private func commit(_ a: Artifact, _ r: RenderedImage) {
        artifact = a
        image = NSImage(data: Data(r.png))
        baseName = r.baseName
        stage = r.matchedStage
        descriptionText = r.description
    }

    /// 「発掘」ボタン: 新規なら採掘アニメ→新発見、既出なら師匠が叱る→持ち出し。いずれも最後に遺物表示。
    func excavate() {
        guard canExcavate else { return }
        let a = derive()
        let r = render()
        pendingArt = a
        pendingRender = r
        isNewDiscovery = !discovered.contains(a.artifactHash)
        phase = isNewDiscovery ? .digging : .scolding
    }

    /// ランダム（デバッグ）: アニメ無しで即・次々に表示。新規は自動保存。
    func randomExcavate() {
        input = "QR-\(Int.random(in: 0..<10_000_000))"
        let a = derive()
        let r = render()
        commit(a, r)
        registerIfNew(a, r)
    }

    /// モーダル（アニメ→バナー、約3秒）終了時に呼ばれる。新規・既出いずれも遺物を表示し、新規は自動保存。
    func finishDigAnimation() {
        if let a = pendingArt, let r = pendingRender {
            commit(a, r)
            registerIfNew(a, r)
        }
        pendingArt = nil
        pendingRender = nil
        phase = .idle
    }

    // MARK: - バランス確認（大量サンプリング）

    struct Balance {
        var n: Int
        var finalStars: [Int]              // index 1..13
        var baseStars: [Int]               // index 1..10（年補正なし＝基本分布）
        var civ: [(String, Int)]
        var era: [(String, Int)]
        var category: [(String, Int)]
    }

    /// 固定シード列 "balance#i" で n 件サンプリング（再現可能）。
    /// year=nil なら年補正なし（＝基本レア度分布の確認）。year 指定で時代補正の効きを確認。
    func sample(_ n: Int, year: Int32?) -> Balance {
        var fin = [Int](repeating: 0, count: 14)
        var bas = [Int](repeating: 0, count: 11)
        var civ: [String: Int] = [:]
        var era: [String: Int] = [:]
        var cat: [String: Int] = [:]
        for i in 0..<n {
            let key = "balance#\(i)"
            let a = year.map { deriveQrWithYear(text: key, year: $0) } ?? deriveQr(text: key)
            fin[Int(a.finalRarity)] += 1
            bas[Int(a.baseRarity)] += 1
            civ[a.civ, default: 0] += 1
            era[a.era, default: 0] += 1
            cat[a.category, default: 0] += 1
        }
        let sortDesc: ([String: Int]) -> [(String, Int)] = { d in
            d.sorted { $0.value > $1.value }.map { ($0.key, $0.value) }
        }
        return Balance(n: n, finalStars: fin, baseStars: bas,
                       civ: sortDesc(civ), era: sortDesc(era), category: sortDesc(cat))
    }
}

/// レア度ごとの色（演出用）。
func rarityColor(_ r: Int) -> Color {
    switch r {
    case ...2: return .gray
    case 3...4: return .green
    case 5...6: return .blue
    case 7...8: return .purple
    case 9...10: return .orange
    default: return .pink   // ★11..13 神話級
    }
}
