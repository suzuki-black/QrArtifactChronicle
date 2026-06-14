import SwiftUI
import QracFFI

/// ゲームバランス確認シート。固定シードで大量サンプリングし、分布を可視化。
struct BalanceView: View {
    @ObservedObject var model: GameModel
    @Environment(\.dismiss) private var dismiss

    @State private var n = 5000
    @State private var withOldYear = false      // true: 1985年補正ありでサンプリング
    @State private var result: GameModel.Balance?
    @State private var running = false

    // 基本レア度の目標分布(%)（docs/02 2.1, ★1は余り込み40.49%）
    private let target: [Double] = [40.49, 25, 15, 10, 6, 2, 1, 0.4, 0.1, 0.01]

    var body: some View {
        VStack(spacing: 12) {
            HStack {
                Text("📊 バランス確認").font(.title3.bold())
                Spacer()
                Button("閉じる") { dismiss() }
            }

            HStack {
                Picker("件数", selection: $n) {
                    Text("1,000").tag(1000); Text("5,000").tag(5000); Text("20,000").tag(20000)
                }.pickerStyle(.segmented)
            }
            Toggle("1985年の補正ありでサンプリング（時代補正の効きを見る）", isOn: $withOldYear)
                .font(.caption)

            Button {
                run()
            } label: {
                Label(running ? "計算中…" : "サンプリング実行", systemImage: "play.fill")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent).disabled(running)

            if let r = result {
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        raritySection(r)
                        axisSection("文明", r.civ, r.n)
                        axisSection("時代", r.era, r.n)
                        axisSection("カテゴリ", r.category, r.n)
                    }
                }
            } else {
                Spacer()
                Text("「サンプリング実行」で分布を計算します").foregroundStyle(.secondary)
                Spacer()
            }
        }
        .padding(18)
        .frame(width: 460, height: 720)
        .onAppear { if result == nil { run() } }
    }

    private func run() {
        running = true
        let count = n
        let year: Int32? = withOldYear ? Int32(1985) : nil
        // FFI 呼び出しは高速。"計算中" を一瞬見せてから main で実行（@MainActor の model を直接呼ぶ）。
        DispatchQueue.main.async {
            result = model.sample(count, year: year)
            running = false
        }
    }

    // レア度分布（観測% と 目標%）
    private func raritySection(_ r: GameModel.Balance) -> some View {
        let stars = withOldYear ? r.finalStars : r.baseStars
        let maxStar = withOldYear ? 13 : 10
        return VStack(alignment: .leading, spacing: 6) {
            Text(withOldYear ? "最終レア度分布（補正あり）" : "基本レア度分布（補正なし）")
                .font(.subheadline.bold())
            ForEach(1...maxStar, id: \.self) { s in
                let pct = Double(stars[s]) / Double(r.n) * 100
                HStack(spacing: 6) {
                    Text("★\(s)").font(.caption.monospaced()).frame(width: 36, alignment: .leading)
                    GeometryReader { geo in
                        ZStack(alignment: .leading) {
                            RoundedRectangle(cornerRadius: 3).fill(Color(white: 0.92))
                            RoundedRectangle(cornerRadius: 3)
                                .fill(rarityColor(s))
                                .frame(width: max(2, geo.size.width * barScale(pct)))
                        }
                    }.frame(height: 14)
                    Text(String(format: "%.2f%%", pct)).font(.caption2.monospaced())
                        .frame(width: 56, alignment: .trailing)
                    if !withOldYear, s <= 10 {
                        Text(String(format: "(目標%.2f)", target[s - 1]))
                            .font(.caption2).foregroundStyle(.secondary)
                            .frame(width: 78, alignment: .trailing)
                    }
                }
            }
        }
    }

    // sqrt スケール（レア層も見えるように）
    private func barScale(_ pct: Double) -> Double { min(1, (pct / 50).squareRoot()) }

    private func axisSection(_ title: String, _ data: [(String, Int)], _ n: Int) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.subheadline.bold())
            ForEach(data, id: \.0) { name, c in
                let pct = Double(c) / Double(n) * 100
                HStack(spacing: 6) {
                    Text(name).font(.caption).frame(width: 96, alignment: .leading).lineLimit(1)
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
