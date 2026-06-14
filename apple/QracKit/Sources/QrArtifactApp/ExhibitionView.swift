import SwiftUI

/// 展示室ページ: 発掘済みアーティファクトの一覧。カテゴリ別フィルタ＋文字検索。
/// （ダイアログではなくページ遷移。スマホUIに合わせる）
struct ExhibitionView: View {
    @ObservedObject var model: GameModel
    let onBack: () -> Void
    let onSelect: (GameModel.Collected) -> Void
    // 検索条件は親(ContentView)が保持。詳細遷移→戻りで初期化されないように。
    @Binding var search: String
    @Binding var category: String?
    @Binding var rarity: Int?
    @State private var confirmReset = false

    private let cats = ["weapon", "ritual", "daily", "architecture", "inscription", "machine_part"]
    private let cols = [GridItem(.adaptive(minimum: 150), spacing: 10)]

    // コレクションに実在するレア度のみ（空チップを並べない）
    private var presentRarities: [Int] {
        Array(Set(model.collected.map(\.finalRarity))).sorted(by: >)
    }

    private var filtered: [GameModel.Collected] {
        model.collected.filter { c in
            (category == nil || c.category == category)
                && (rarity == nil || c.finalRarity == rarity)
                && (search.isEmpty || c.haystack.localizedCaseInsensitiveContains(search))
        }
        .sorted { $0.finalRarity > $1.finalRarity }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            VStack(spacing: 10) {
                TextField("名称・文明・解説などで検索", text: $search)
                    .textFieldStyle(.roundedBorder)

                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        catChip("すべて", key: nil)
                        ForEach(cats, id: \.self) { catChip(categoryJP($0), key: $0) }
                    }
                }

                // レア度フィルタ（存在するレア度のみ）
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        rarityChip("★すべて", value: nil)
                        ForEach(presentRarities, id: \.self) { rarityChip("★\($0)", value: $0) }
                    }
                }

                if filtered.isEmpty {
                    Spacer()
                    VStack(spacing: 6) {
                        Text(model.collected.isEmpty ? "まだ何も発掘していません" : "該当する遺物がありません")
                            .foregroundStyle(.secondary)
                        if model.collected.isEmpty {
                            Text("「発掘」で遺物を見つけると ここに並びます")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                    }
                    Spacer()
                } else {
                    ScrollView {
                        LazyVGrid(columns: cols, spacing: 10) {
                            ForEach(filtered) { card($0) }
                        }
                        .padding(.vertical, 4)
                    }
                }
            }
            .padding(14)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(red: 0.96, green: 0.94, blue: 0.89))
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button { onBack() } label: {
                Label("もどる", systemImage: "chevron.left").font(.callout.bold())
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.small)
            Text("🏛 展示室").font(.title3.bold())
            Text("\(model.collected.count)点").font(.caption).foregroundStyle(.secondary)
            Spacer()
            #if DEBUG
            Button(role: .destructive) { confirmReset = true } label: {
                Label("リセット", systemImage: "trash").font(.caption)
            }.buttonStyle(.bordered).tint(.red)
            .confirmationDialog("展示室をすべてリセットしますか？", isPresented: $confirmReset, titleVisibility: .visible) {
                Button("リセットする", role: .destructive) { model.resetCollection() }
                Button("やめる", role: .cancel) {}
            }
            #endif
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    private func catChip(_ label: String, key: String?) -> some View {
        let on = category == key
        return Button(label) { category = key }
            .font(.caption)
            .buttonStyle(.bordered)
            .tint(on ? .accentColor : .gray)
    }

    private func rarityChip(_ label: String, value: Int?) -> some View {
        let on = rarity == value
        return Button(label) { rarity = value }
            .font(.caption)
            .buttonStyle(.bordered)
            .tint(on ? (value.map { rarityColor($0) } ?? .accentColor) : .gray)
    }

    private func card(_ c: GameModel.Collected) -> some View {
        Button { onSelect(c) } label: {
            VStack(spacing: 5) {
                ZStack {
                    RoundedRectangle(cornerRadius: 8).fill(Color(white: 0.95))
                    if let img = NSImage(data: c.png) {
                        Image(nsImage: img).resizable().scaledToFit().padding(4)
                    }
                }
                .frame(height: 110)
                .overlay(RoundedRectangle(cornerRadius: 8)
                    .strokeBorder(rarityColor(c.finalRarity), lineWidth: 2))

                Text(String(repeating: "★", count: c.finalRarity))
                    .font(.caption2).foregroundStyle(rarityColor(c.finalRarity))
                    .lineLimit(1).minimumScaleFactor(0.5)
                // 名称（しっかり表示）
                Text(c.name).font(.caption.bold()).foregroundStyle(.black)
                    .lineLimit(2).multilineTextAlignment(.center)
                    .frame(maxWidth: .infinity, minHeight: 30, alignment: .top)
                Text(categoryJP(c.category)).font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
            .padding(8)
            .background(Color.white)
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(c.isMythic ? RoundedRectangle(cornerRadius: 10).strokeBorder(.pink, lineWidth: 2) : nil)
        }
        .buttonStyle(.plain)
    }
}

func categoryJP(_ key: String) -> String {
    switch key {
    case "weapon": return "武器"
    case "ritual": return "祭具"
    case "daily": return "生活用品"
    case "architecture": return "建築断片"
    case "inscription": return "碑文"
    case "machine_part": return "機械部品"
    default: return key
    }
}
