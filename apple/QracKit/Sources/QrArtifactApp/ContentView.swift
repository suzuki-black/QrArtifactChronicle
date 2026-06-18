import SwiftUI
import QracFFI

private let labelColor = Color(white: 0.38)   // 読みやすいラベル色

/// スマホ縦長前提のゲーム画面（macOS 上で phone フレーム表示）。
/// 上: 遺物カード（スクロール）／下: 操作バー（常に表示）。
struct ContentView: View {
    enum Page { case main, exhibition, detail, camera, settings }
    enum InputMode { case manual, camera }
    @StateObject private var model = GameModel()
    @StateObject private var settings = Settings()
    @State private var showBalance = false
    @State private var page: Page = .main
    @State private var selected: GameModel.Collected?
    @State private var inputMode: InputMode = .manual   // デバッグの入力方法トグル
    // 展示室の検索条件（ページをまたいで保持）
    @State private var exSearch = ""
    @State private var exCategory: String?
    @State private var exRarity: Int?
    @State private var exFavoritesOnly = false

    private let phoneWidth: CGFloat = 370

    var body: some View {
        ZStack {
            LinearGradient(colors: [Color(white: 0.20), Color(white: 0.10)],
                           startPoint: .top, endPoint: .bottom).ignoresSafeArea()

            ZStack {
                switch page {
                case .main:
                    mainPage.transition(.move(edge: .leading))
                case .exhibition:
                    ExhibitionView(
                        model: model,
                        onBack: { go(.main) },
                        onSelect: { c in selected = c; go(.detail) },
                        search: $exSearch, category: $exCategory, rarity: $exRarity,
                        favoritesOnly: $exFavoritesOnly)
                        .transition(.move(edge: .trailing))
                case .detail:
                    if let sel = selected {
                        ArtifactDetailView(model: model, item: sel, onBack: { go(.exhibition) })
                            .transition(.move(edge: .trailing))
                    }
                case .camera:
                    CameraScanView(
                        onScan: { s in model.input = s; go(.main); model.excavate() },
                        onCancel: { go(.main) })
                        .transition(.move(edge: .trailing))
                case .settings:
                    SettingsView(settings: settings, onBack: { go(.main) })
                        .transition(.move(edge: .trailing))
                }
            }
            .frame(width: phoneWidth)
            .frame(maxHeight: .infinity)
            .clipShape(RoundedRectangle(cornerRadius: 34))
            .overlay(RoundedRectangle(cornerRadius: 34).strokeBorder(.black.opacity(0.85), lineWidth: 10))
            .shadow(color: .black.opacity(0.5), radius: 20, y: 8)
            .padding(.vertical, 18)

            if model.phase != .idle {
                DigModalView(scolding: model.phase == .scolding) { model.finishDigAnimation() }
                    .transition(.opacity)
            }
        }
        .environmentObject(settings)
        .animation(.easeInOut(duration: 0.15), value: model.phase)
        .frame(minWidth: phoneWidth + 40, minHeight: 540)
        .onAppear { model.configureAssetsIfBundled(); model.lang = settings.ffiLang() }
        .onChange(of: settings.language) { _ in
            model.lang = settings.ffiLang()
            model.relocalize()
        }
        .sheet(isPresented: $showBalance) { BalanceView(model: model, settings: settings) }
    }

    private func go(_ p: Page) { withAnimation(.easeInOut(duration: 0.25)) { page = p } }

    private var mainPage: some View {
        VStack(spacing: 0) {
            header
            ScrollView {
                if model.artifact == nil {
                    emptyState.padding(.top, 60)
                } else {
                    artifactCard.padding(14)
                }
            }
            controlBar
        }
        .background(Color(red: 0.96, green: 0.94, blue: 0.89))
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Image(systemName: "magnifyingglass.circle")
                .font(.system(size: 72)).foregroundStyle(.brown.opacity(0.45))
            Text(settings.t("まだ何も発掘していません", "Nothing excavated yet"))
                .font(.headline).foregroundStyle(.secondary)
            #if DEBUG
            Text(settings.t("入力欄に文字列を入れて「発掘」してみよう",
                            "Type some text and tap Excavate"))
                .font(.caption).foregroundStyle(.secondary).multilineTextAlignment(.center)
            #else
            Text(settings.t("QRコードを読み取って発掘しよう", "Scan a QR code to excavate"))
                .font(.caption).foregroundStyle(.secondary)
            #endif
        }
        .frame(maxWidth: .infinity)
    }

    // MARK: Header
    private var header: some View {
        HStack(spacing: 8) {
            Text(settings.t("🏺 QR考古学", "🏺 QR Archaeology"))
                .font(.title2.bold())
                .lineLimit(1).minimumScaleFactor(0.6).layoutPriority(1)
            Spacer(minLength: 4)
            Button { go(.exhibition) } label: {
                Label(settings.t("展示室", "Exhibition"), systemImage: "building.columns.fill")
                    .font(.caption.bold())
            }.buttonStyle(.borderedProminent).tint(.brown).controlSize(.small)
            Button { go(.settings) } label: {
                Image(systemName: "gearshape.fill")
            }.buttonStyle(.bordered).controlSize(.small)
            #if DEBUG
            Text("DEBUG").font(.caption2.bold())
                .padding(.horizontal, 8).padding(.vertical, 3)
                .background(Color.orange).foregroundStyle(.white).clipShape(Capsule())
            #endif
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    // MARK: 遺物カード
    private var artifactCard: some View {
        VStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 16).fill(Color(white: 0.93))
                if let img = model.image {
                    Image(platformImage: img).resizable().interpolation(.high).scaledToFit().padding(8)
                }
            }
            .frame(height: 260)
            .overlay(RoundedRectangle(cornerRadius: 16)
                .strokeBorder(rarityColor(Int(model.artifact?.finalRarity ?? 1)), lineWidth: 4))

            if let a = model.artifact {
                HStack(spacing: 6) {
                    Text(stars(a.finalRarity)).font(.title3)
                        .foregroundStyle(rarityColor(Int(a.finalRarity)))
                        .lineLimit(1).minimumScaleFactor(0.5)
                    if a.isMythic {
                        Text(settings.mythicLabel).font(.caption.bold())
                            .padding(.horizontal, 6).padding(.vertical, 2)
                            .background(Color.pink.opacity(0.2)).foregroundStyle(.pink).clipShape(Capsule())
                    }
                }
                Text(settings.artifactName(civ: a.civ, era: a.era, category: a.category,
                                           rarity: Int(a.finalRarity)))
                    .font(.headline).foregroundStyle(.black).multilineTextAlignment(.center)
                HStack(spacing: 6) {
                    chip(settings.civName(a.civ), .brown)
                    chip(settings.eraName(a.era), .indigo)
                    chip(settings.categoryName(a.category), .teal)
                }
                statGrid(a)
                if !model.descriptionText.isEmpty { bookExcerpt }
            }
        }
        .padding(14).background(Color.white)
        .clipShape(RoundedRectangle(cornerRadius: 18))
        .shadow(color: .black.opacity(0.08), radius: 6, y: 3)
    }

    private var bookExcerpt: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Image(systemName: "book.closed.fill").foregroundStyle(.brown)
                Text(settings.bookTitle).font(.subheadline.bold()).foregroundStyle(.brown)
            }
            Text(model.descriptionText)
                .font(.callout).foregroundStyle(.black.opacity(0.88))
                .fixedSize(horizontal: false, vertical: true).lineSpacing(3)
            Text(settings.bookFooter)
                .font(.caption2).foregroundStyle(labelColor)
                .frame(maxWidth: .infinity, alignment: .trailing)
        }
        .padding(12)
        .background(Color(red: 0.97, green: 0.95, blue: 0.88))
        .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.brown.opacity(0.35), lineWidth: 1))
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }

    private func statGrid(_ a: Artifact) -> some View {
        let cols = [GridItem(.flexible()), GridItem(.flexible())]
        let stageVal = model.stage == 7 ? "GLOBAL" : settings.t("段\(model.stage)", "stage \(model.stage)")
        return LazyVGrid(columns: cols, spacing: 6) {
            stat(settings.t("基本レア度", "Base rarity"), "★\(a.baseRarity)")
            stat(settings.t("時代補正", "Era bonus"), "+\(a.eraBonus)")
            stat(settings.t("最終レア度", "Final rarity"), "★\(a.finalRarity)")
            stat(settings.t("保存度", "Preservation"), "\(Int(a.preservationScore * 100))%")
            stat(settings.t("汚れ", "Dirt"), settings.dirtName(a.dirtLayerId))
            stat(settings.t("破損", "Damage"),
                 settings.damageText(chip: a.damage.chip, crack: a.damage.crack, wear: a.damage.wear))
            stat(settings.t("DB段", "DB stage"), stageVal)
            stat("hash", String(a.artifactHash.prefix(8)))
        }
        .font(.caption)
    }

    // MARK: 操作バー
    // ⚠️ 手入力・ランダム等のデバッグ機能はリリースビルドでは #if DEBUG により除外される。
    private var controlBar: some View {
        VStack(spacing: 8) {
            #if DEBUG
            HStack(spacing: 8) {
                Text(settings.t("入力方法", "Input")).font(.caption.bold()).foregroundStyle(labelColor)
                Picker("", selection: $inputMode) {
                    Label(settings.t("手動", "Manual"), systemImage: "keyboard").tag(InputMode.manual)
                    Label(settings.t("カメラ", "Camera"), systemImage: "camera.viewfinder").tag(InputMode.camera)
                }.pickerStyle(.segmented).labelStyle(.titleAndIcon).labelsHidden()
            }
            ZStack {
                if inputMode == .manual {
                    VStack(spacing: 8) {
                        HStack(spacing: 6) {
                            Image(systemName: "keyboard").foregroundStyle(labelColor)
                            TextField(settings.t("任意の文字列 / URL を手入力", "Enter any text / URL"),
                                      text: $model.input)
                                .textFieldStyle(.roundedBorder).onSubmit { model.dig() }
                        }
                        Button { model.excavate() } label: {
                            Label(settings.t("発掘", "Excavate"), systemImage: "hammer.fill")
                                .frame(maxWidth: .infinity)
                        }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.large)
                            .disabled(!model.canExcavate)
                        presetRow
                    }
                    .frame(maxHeight: .infinity, alignment: .top)
                } else {
                    VStack(spacing: 8) {
                        Button { openCamera() } label: {
                            Label(settings.t("カメラで発掘", "Excavate with camera"),
                                  systemImage: "camera.viewfinder").frame(maxWidth: .infinity)
                        }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.large)
                        Text(settings.t("QRコードをかざすと自動で読み取ります", "Hold up a QR code to scan automatically"))
                            .font(.caption).foregroundStyle(labelColor)
                    }
                }
            }
            .frame(height: 132)
            Divider().padding(.vertical, 1)
            HStack(spacing: 8) {
                Button { model.randomExcavate() } label: {
                    Label(settings.t("ランダム", "Random"), systemImage: "dice.fill").frame(maxWidth: .infinity)
                }.buttonStyle(.borderedProminent).tint(.brown)
                Button { showBalance = true } label: {
                    Label(settings.t("分布", "Stats"), systemImage: "chart.bar.fill").frame(maxWidth: .infinity)
                }.buttonStyle(.borderedProminent).tint(.teal).help(settings.t("ゲームバランス確認", "Game balance"))
            }
            HStack(spacing: 8) {
                Toggle(settings.t("年代", "Era"), isOn: $model.useYear)
                    .toggleStyle(.switch).fixedSize()
                    .onChange(of: model.useYear) { _ in model.dig() }
                if model.useYear {
                    Stepper(settings.t("\(model.year)年", "\(model.year)"), value: $model.year, in: 1900...2099)
                        .onChange(of: model.year) { _ in model.dig() }.font(.callout)
                } else {
                    Text(settings.t("（古いほど高レア・神話級）", "(older = rarer / mythic)"))
                        .font(.caption).foregroundStyle(labelColor)
                }
                Spacer()
            }
            #else
            // リリース: カメラ発掘のみ。
            Button { openCamera() } label: {
                Label(settings.t("カメラで発掘", "Excavate with camera"), systemImage: "camera.viewfinder")
                    .frame(maxWidth: .infinity)
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.large)
            Text(settings.t("QRコードを読み取って遺物を発掘します", "Scan a QR code to excavate an artifact"))
                .font(.caption).foregroundStyle(labelColor)
            #endif
        }
        .padding(12)
        .background(Color(white: 0.985))
        .overlay(Rectangle().frame(height: 1).foregroundStyle(.black.opacity(0.08)), alignment: .top)
    }

    private func openCamera() {
        model.input = ""
        go(.camera)
    }

    private var presetRow: some View {
        let presets: [(String, String)] = [
            ("URL", "https://example.com/welcome"),
            (settings.t("和文", "JP text"), "古代の遺物QRコード"),
            ("vCard2014", "BEGIN:VCARD\nREV:2014-03-10T00:00:00Z\nEND:VCARD"),
            ("ISO1985", "log 1985-06-15 backup"),
        ]
        return ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 6) {
                ForEach(presets, id: \.0) { p in
                    Button(p.0) { model.input = p.1; model.useYear = false; model.dig() }
                        .font(.caption).buttonStyle(.bordered)
                }
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
            Text(label).foregroundStyle(labelColor)
            Spacer()
            Text(value).bold().foregroundStyle(.black).lineLimit(1).minimumScaleFactor(0.6)
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(Color(white: 0.95)).clipShape(RoundedRectangle(cornerRadius: 6))
    }
}

func stars(_ n: UInt32) -> String { String(repeating: "★", count: Int(n)) }
