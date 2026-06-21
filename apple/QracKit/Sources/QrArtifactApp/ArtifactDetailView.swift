import SwiftUI
import QracFFI

/// 遺物の詳細ページ。メイン画面に似ているが、操作系は無く「もどる」だけ。
struct ArtifactDetailView: View {
    @ObservedObject var model: GameModel
    @EnvironmentObject var settings: Settings
    let item: GameModel.Collected
    let onBack: () -> Void
    /// 関連遺物（発掘済み）をタップしたときの遷移。表示中の item を差し替える。
    var onOpen: (GameModel.Collected) -> Void = { _ in }

    @State private var desc: String = ""
    private let detailLabel = Color(white: 0.38)

    var body: some View {
        VStack(spacing: 0) {
            header
            ScrollView { card.padding(14) }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(red: 0.96, green: 0.94, blue: 0.89))
        .onAppear { desc = model.description(for: item) }
        .onChange(of: settings.language) { _ in desc = model.description(for: item) }
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button { onBack() } label: {
                Label(settings.t("もどる", "Back"), systemImage: "chevron.left").font(.callout.bold())
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.small)
            Text(settings.t("遺物詳細", "Artifact detail")).font(.title3.bold())
            Spacer()
            let fav = model.isFavorite(item.id)
            Button { model.toggleFavorite(item.id) } label: {
                Label(settings.t("お気に入り", "Favorite"),
                      systemImage: fav ? "heart.fill" : "heart")
                    .labelStyle(.iconOnly).font(.title3)
            }
            .buttonStyle(.bordered).tint(fav ? .pink : .gray).controlSize(.small)
            .help(settings.t("お気に入り", "Favorite"))
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    private var card: some View {
        VStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 16).fill(Color(white: 0.93))
                // フル解像度は text から再生成（保存はサムネのみ, docs/06 6.3）。無ければ保存サムネにフォールバック。
                if let img = model.fullImage(for: item) ?? PlatformImage(data: item.png) {
                    Image(platformImage: img).resizable().interpolation(.high).scaledToFit().padding(8)
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
                    Text(settings.mythicLabel).font(.caption.bold())
                        .padding(.horizontal, 6).padding(.vertical, 2)
                        .background(Color.pink.opacity(0.2)).foregroundStyle(.pink).clipShape(Capsule())
                }
            }
            Text(settings.artifactName(civ: item.civ, era: item.era, category: item.category,
                                       rarity: item.finalRarity))
                .font(.headline).foregroundStyle(.black).multilineTextAlignment(.center)
            HStack(spacing: 6) {
                chip(settings.civName(item.civ), .brown)
                chip(settings.eraName(item.era), .indigo)
                chip(settings.categoryName(item.category), .teal)
            }
            statGrid
            bookExcerpt
            relatedSection
        }
        .padding(14).background(Color.white)
        .clipShape(RoundedRectangle(cornerRadius: 18))
        .shadow(color: .black.opacity(0.08), radius: 6, y: 3)
    }

    private var statGrid: some View {
        let cols = [GridItem(.flexible()), GridItem(.flexible())]
        let stageVal = item.stage == 7 ? "GLOBAL" : settings.t("段\(item.stage)", "stage \(item.stage)")
        return LazyVGrid(columns: cols, spacing: 6) {
            stat(settings.t("基本レア度", "Base rarity"), "★\(item.baseRarity)")
            stat(settings.t("時代補正", "Era bonus"), "+\(item.eraBonus)")
            stat(settings.t("最終レア度", "Final rarity"), "★\(item.finalRarity)")
            stat(settings.t("保存度", "Preservation"), "\(Int(item.preservation * 100))%")
            stat(settings.t("汚れ", "Dirt"), settings.dirtName(item.dirt))
            stat(settings.t("破損", "Damage"),
                 settings.damageText(chip: item.chip, crack: item.crack, wear: item.wear))
            stat(settings.t("DB段", "DB stage"), stageVal)
            stat("hash", String(item.id.prefix(8)))
        }
        .font(.caption)
    }

    private var bookExcerpt: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Image(systemName: "book.closed.fill").foregroundStyle(.brown)
                Text(settings.bookTitle).font(.subheadline.bold()).foregroundStyle(.brown)
            }
            Text(desc).font(.callout).foregroundStyle(.black.opacity(0.88))
                .fixedSize(horizontal: false, vertical: true).lineSpacing(3)
            Text(settings.bookFooter)
                .font(.caption2).foregroundStyle(detailLabel)
                .frame(maxWidth: .infinity, alignment: .trailing)
        }
        .padding(12)
        .background(Color(red: 0.97, green: 0.95, blue: 0.88))
        .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.brown.opacity(0.35), lineWidth: 1))
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }

    // MARK: - 関連遺物（出土の系譜, 提案01 §5）

    @ViewBuilder private var relatedSection: some View {
        let refs = model.references(for: item)
        if !refs.isEmpty {
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 6) {
                    Image(systemName: "point.3.connected.trianglepath.dotted").foregroundStyle(.brown)
                    Text(settings.t("出土の系譜", "Genealogy")).font(.subheadline.bold()).foregroundStyle(.brown)
                }
                ForEach(refs, id: \.toSet) { ref in
                    relatedRow(ref)
                }
            }
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color(red: 0.95, green: 0.96, blue: 0.93))
            .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.brown.opacity(0.3), lineWidth: 1))
            .clipShape(RoundedRectangle(cornerRadius: 10))
        }
    }

    /// 参照の種別ラベル。
    private func kindLabel(_ kind: String) -> String {
        switch kind {
        case "pair":  return settings.t("対をなす", "Pair")
        case "cites": return settings.t("言及する", "Cites")
        case "rival": return settings.t("好敵手", "Rival")
        default:      return kind
        }
    }

    @ViewBuilder private func relatedRow(_ ref: Reference) -> some View {
        let owned = model.owns(set: ref.toSet)
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                chip(kindLabel(ref.kind), .indigo)
                if owned {
                    Button {
                        if let target = model.collected(ofType: ref.toSet) { onOpen(target) }
                    } label: {
                        HStack(spacing: 4) {
                            Text(ref.toLabel).font(.callout.bold()).foregroundStyle(.blue)
                                .multilineTextAlignment(.leading)
                            Image(systemName: "chevron.right").font(.caption2).foregroundStyle(.blue)
                        }
                    }
                    .buttonStyle(.plain)
                } else {
                    HStack(spacing: 4) {
                        Image(systemName: "lock.fill").font(.caption2)
                        Text(ref.toLabel).font(.callout)
                    }
                    .foregroundStyle(detailLabel)
                }
                Spacer(minLength: 0)
            }
            if owned, let combined = model.combinedDescription(from: item, ref: ref) {
                Text(combined).font(.caption).foregroundStyle(.black.opacity(0.85))
                    .fixedSize(horizontal: false, vertical: true).lineSpacing(2)
                    .padding(.leading, 2)
            } else if !owned {
                Text(settings.t("発掘すると解禁", "Excavate to unlock"))
                    .font(.caption2).foregroundStyle(detailLabel).padding(.leading, 2)
            }
        }
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
