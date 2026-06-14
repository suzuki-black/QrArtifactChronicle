import SwiftUI

/// 展示室ページ: 発掘済みアーティファクトの一覧。カテゴリ別フィルタ＋文字検索。
struct ExhibitionView: View {
    @ObservedObject var model: GameModel
    @EnvironmentObject var settings: Settings
    let onBack: () -> Void
    let onSelect: (GameModel.Collected) -> Void
    @Binding var search: String
    @Binding var category: String?
    @Binding var rarity: Int?
    @State private var confirmReset = false

    private let cats = ["weapon", "ritual", "daily", "architecture", "inscription", "machine_part"]
    private let cols = [GridItem(.adaptive(minimum: 150), spacing: 10)]

    private var presentRarities: [Int] {
        Array(Set(model.collected.map(\.finalRarity))).sorted(by: >)
    }

    /// 表示言語に応じた検索対象（名称・キー・両表記）。
    private func matches(_ c: GameModel.Collected) -> Bool {
        if search.isEmpty { return true }
        let name = settings.artifactName(civ: c.civ, era: c.era, category: c.category, rarity: c.finalRarity)
        let hay = [name, c.civ, c.era, c.category,
                   settings.civName(c.civ), settings.eraName(c.era), settings.categoryName(c.category)]
            .joined(separator: " ")
        return hay.localizedCaseInsensitiveContains(search)
    }

    private var filtered: [GameModel.Collected] {
        model.collected.filter { c in
            (category == nil || c.category == category)
                && (rarity == nil || c.finalRarity == rarity)
                && matches(c)
        }
        .sorted { $0.finalRarity > $1.finalRarity }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            VStack(spacing: 10) {
                TextField(settings.t("名称・文明などで検索", "Search by name, civ…"), text: $search)
                    .textFieldStyle(.roundedBorder)

                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        catChip(settings.t("すべて", "All"), key: nil)
                        ForEach(cats, id: \.self) { catChip(settings.categoryName($0), key: $0) }
                    }
                }
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        rarityChip(settings.t("★すべて", "★All"), value: nil)
                        ForEach(presentRarities, id: \.self) { rarityChip("★\($0)", value: $0) }
                    }
                }

                if filtered.isEmpty {
                    Spacer()
                    VStack(spacing: 6) {
                        Text(model.collected.isEmpty
                             ? settings.t("まだ何も発掘していません", "Nothing excavated yet")
                             : settings.t("該当する遺物がありません", "No matching artifacts"))
                            .foregroundStyle(.secondary)
                        if model.collected.isEmpty {
                            Text(settings.t("「発掘」で遺物を見つけると ここに並びます",
                                            "Artifacts you excavate will appear here"))
                                .font(.caption).foregroundStyle(.secondary).multilineTextAlignment(.center)
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
                Label(settings.t("もどる", "Back"), systemImage: "chevron.left").font(.callout.bold())
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.small)
            Text(settings.t("🏛 展示室", "🏛 Exhibition")).font(.title3.bold())
            Text(settings.t("\(model.collected.count)点", "\(model.collected.count)"))
                .font(.caption).foregroundStyle(.secondary)
            Spacer()
            #if DEBUG
            Button(role: .destructive) { confirmReset = true } label: {
                Label(settings.t("リセット", "Reset"), systemImage: "trash").font(.caption)
            }.buttonStyle(.bordered).tint(.red)
            .confirmationDialog(settings.t("展示室をすべてリセットしますか？", "Reset the whole exhibition?"),
                                isPresented: $confirmReset, titleVisibility: .visible) {
                Button(settings.t("リセットする", "Reset"), role: .destructive) { model.resetCollection() }
                Button(settings.t("やめる", "Cancel"), role: .cancel) {}
            }
            #endif
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    private func catChip(_ label: String, key: String?) -> some View {
        Button(label) { category = key }
            .font(.caption).buttonStyle(.bordered)
            .tint(category == key ? .accentColor : .gray)
    }

    private func rarityChip(_ label: String, value: Int?) -> some View {
        Button(label) { rarity = value }
            .font(.caption).buttonStyle(.bordered)
            .tint(rarity == value ? (value.map { rarityColor($0) } ?? .accentColor) : .gray)
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
                Text(settings.artifactName(civ: c.civ, era: c.era, category: c.category, rarity: c.finalRarity))
                    .font(.caption.bold()).foregroundStyle(.black)
                    .lineLimit(2).multilineTextAlignment(.center)
                    .frame(maxWidth: .infinity, minHeight: 30, alignment: .top)
                Text(settings.categoryName(c.category)).font(.system(size: 10))
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
