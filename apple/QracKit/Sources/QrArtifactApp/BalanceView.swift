import SwiftUI
import QracFFI

/// ゲームバランス確認シート（デバッグ）。固定シードで大量サンプリングし分布を可視化。
struct BalanceView: View {
    @ObservedObject var model: GameModel
    @ObservedObject var settings: Settings
    @Environment(\.dismiss) private var dismiss

    @State private var n = 5000
    @State private var withOldYear = false
    @State private var result: GameModel.Balance?
    @State private var running = false

    private let target: [Double] = [40.49, 25, 15, 10, 6, 2, 1, 0.4, 0.1, 0.01]

    var body: some View {
        VStack(spacing: 12) {
            HStack {
                Text(settings.t("📊 バランス確認", "📊 Balance")).font(.title3.bold())
                Spacer()
                Button(settings.t("閉じる", "Close")) { dismiss() }
            }

            Picker("", selection: $n) {
                Text("1,000").tag(1000); Text("5,000").tag(5000); Text("20,000").tag(20000)
            }.pickerStyle(.segmented).labelsHidden()

            Toggle(settings.t("1985年の補正ありでサンプリング（時代補正の効きを見る）",
                              "Sample with the 1985 era bonus (see the era-bonus effect)"),
                   isOn: $withOldYear).font(.caption)

            Button { run() } label: {
                Label(running ? settings.t("計算中…", "Computing…") : settings.t("サンプリング実行", "Run sampling"),
                      systemImage: "play.fill").frame(maxWidth: .infinity)
            }.buttonStyle(.borderedProminent).disabled(running)

            if let r = result {
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        raritySection(r)
                        axisSection(settings.t("文明", "Civilization"), r.civ, r.n, settings.civName)
                        axisSection(settings.t("時代", "Era"), r.era, r.n, settings.eraName)
                        axisSection(settings.t("カテゴリ", "Category"), r.category, r.n, settings.categoryName)
                    }
                }
            } else {
                Spacer()
                Text(settings.t("「サンプリング実行」で分布を計算します", "Tap Run sampling to compute the distribution"))
                    .foregroundStyle(.secondary)
                Spacer()
            }
        }
        .padding(18)
        .frame(width: 460, height: 720)
        .preferredColorScheme(.light)
        .onAppear { if result == nil { run() } }
    }

    private func run() {
        running = true
        let count = n
        let year: Int32? = withOldYear ? Int32(1985) : nil
        DispatchQueue.main.async {
            result = model.sample(count, year: year)
            running = false
        }
    }

    private func raritySection(_ r: GameModel.Balance) -> some View {
        let stars = withOldYear ? r.finalStars : r.baseStars
        let maxStar = withOldYear ? 13 : 10
        return VStack(alignment: .leading, spacing: 6) {
            Text(withOldYear ? settings.t("最終レア度分布（補正あり）", "Final rarity (with era bonus)")
                             : settings.t("基本レア度分布（補正なし）", "Base rarity (no bonus)"))
                .font(.subheadline.bold())
            ForEach(1...maxStar, id: \.self) { s in
                let pct = Double(stars[s]) / Double(r.n) * 100
                HStack(spacing: 6) {
                    Text("★\(s)").font(.caption.monospaced()).frame(width: 36, alignment: .leading)
                    GeometryReader { geo in
                        ZStack(alignment: .leading) {
                            RoundedRectangle(cornerRadius: 3).fill(Color(white: 0.92))
                            RoundedRectangle(cornerRadius: 3).fill(rarityColor(s))
                                .frame(width: max(2, geo.size.width * barScale(pct)))
                        }
                    }.frame(height: 14)
                    Text(String(format: "%.2f%%", pct)).font(.caption2.monospaced())
                        .frame(width: 56, alignment: .trailing)
                    if !withOldYear, s <= 10 {
                        Text(settings.t(String(format: "(目標%.2f)", target[s - 1]),
                                        String(format: "(target %.2f)", target[s - 1])))
                            .font(.caption2).foregroundStyle(.secondary)
                            .frame(width: 86, alignment: .trailing)
                    }
                }
            }
        }
    }

    private func barScale(_ pct: Double) -> Double { min(1, (pct / 50).squareRoot()) }

    private func axisSection(_ title: String, _ data: [(String, Int)], _ n: Int,
                             _ nameFn: @escaping (String) -> String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.subheadline.bold())
            ForEach(data, id: \.0) { name, c in
                let pct = Double(c) / Double(n) * 100
                HStack(spacing: 6) {
                    Text(nameFn(name)).font(.caption).frame(width: 110, alignment: .leading).lineLimit(1)
                    GeometryReader { geo in
                        ZStack(alignment: .leading) {
                            RoundedRectangle(cornerRadius: 3).fill(Color(white: 0.92))
                            RoundedRectangle(cornerRadius: 3).fill(Color.accentColor.opacity(0.7))
                                .frame(width: max(2, geo.size.width * (pct / 100)))
                        }
                    }.frame(height: 12)
                    Text(String(format: "%.1f%%", pct)).font(.caption2.monospaced())
                        .frame(width: 46, alignment: .trailing)
                }
            }
        }
    }
}
