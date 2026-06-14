import SwiftUI

/// 遺物の詳細ページ。メイン画面に似ているが、操作系は無く「もどる」だけ。
struct ArtifactDetailView: View {
    let item: GameModel.Collected
    let onBack: () -> Void

    private let detailLabel = Color(white: 0.38)

    var body: some View {
        VStack(spacing: 0) {
            header
            ScrollView { card.padding(14) }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(red: 0.96, green: 0.94, blue: 0.89))
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button { onBack() } label: {
                Label("もどる", systemImage: "chevron.left").font(.callout.bold())
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.small)
            Text("遺物詳細").font(.title3.bold())
            Spacer()
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    private var card: some View {
        VStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 16).fill(Color(white: 0.93))
                if let img = NSImage(data: item.png) {
                    Image(nsImage: img).resizable().interpolation(.high).scaledToFit().padding(8)
                }
            }
            .frame(height: 300)
            .overlay(RoundedRectangle(cornerRadius: 16)
                .strokeBorder(rarityColor(item.finalRarity), lineWidth: 4))

            HStack(spacing: 6) {
                Text(String(repeating: "★", count: item.finalRarity)).font(.title3)
                    .foregroundStyle(rarityColor(item.finalRarity))
                    .lineLimit(1).minimumScaleFactor(0.5)
                if item.isMythic {
                    Text("神話級").font(.caption.bold())
                        .padding(.horizontal, 6).padding(.vertical, 2)
                        .background(Color.pink.opacity(0.2)).foregroundStyle(.pink).clipShape(Capsule())
                }
            }
            Text(item.name).font(.headline).foregroundStyle(.black).multilineTextAlignment(.center)
            HStack(spacing: 6) {
                chip(categoryJP(item.category), .teal)
                chip(item.civ, .brown)
                chip(item.era, .indigo)
            }
            statGrid
            bookExcerpt
        }
        .padding(14).background(Color.white)
        .clipShape(RoundedRectangle(cornerRadius: 18))
        .shadow(color: .black.opacity(0.08), radius: 6, y: 3)
    }

    private var statGrid: some View {
        let cols = [GridItem(.flexible()), GridItem(.flexible())]
        let dmg = [item.chip ? "欠" : nil, item.crack ? "ひび" : nil, item.wear ? "摩耗" : nil]
            .compactMap { $0 }.joined(separator: "/")
        return LazyVGrid(columns: cols, spacing: 6) {
            stat("基本レア度", "★\(item.baseRarity)")
            stat("時代補正", "+\(item.eraBonus)")
            stat("最終レア度", "★\(item.finalRarity)")
            stat("保存度", "\(Int(item.preservation * 100))%")
            stat("汚れ", item.dirt)
            stat("破損", dmg.isEmpty ? "なし" : dmg)
            stat("DB段", item.stage == 7 ? "GLOBAL" : "段\(item.stage)")
            stat("hash", String(item.id.prefix(8)))
        }
        .font(.caption)
    }

    private var bookExcerpt: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Image(systemName: "book.closed.fill").foregroundStyle(.brown)
                Text("詳説 世界の遺物").font(.subheadline.bold()).foregroundStyle(.brown)
            }
            Text(item.description).font(.callout).foregroundStyle(.black.opacity(0.88))
                .fixedSize(horizontal: false, vertical: true).lineSpacing(3)
            Text("萬象書房 発行　1890年刊版より抜粋")
                .font(.caption2).foregroundStyle(detailLabel)
                .frame(maxWidth: .infinity, alignment: .trailing)
        }
        .padding(12)
        .background(Color(red: 0.97, green: 0.95, blue: 0.88))
        .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.brown.opacity(0.35), lineWidth: 1))
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }

    private func chip(_ text: String, _ color: Color) -> some View {
        Text(text).font(.caption.bold())
            .padding(.horizontal, 8).padding(.vertical, 3)
            .background(color.opacity(0.18)).foregroundStyle(color).clipShape(Capsule())
    }
    private func stat(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label).foregroundStyle(detailLabel)
            Spacer()
            Text(value).bold().foregroundStyle(.black).lineLimit(1).minimumScaleFactor(0.6)
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(Color(white: 0.95)).clipShape(RoundedRectangle(cornerRadius: 6))
    }
}
