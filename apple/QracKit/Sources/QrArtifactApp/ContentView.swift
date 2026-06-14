import SwiftUI
import QracFFI

private let labelColor = Color(white: 0.38)   // 読みやすいラベル色

/// スマホ縦長前提のゲーム画面（macOS 上で phone フレーム表示）。
/// 上: 遺物カード（スクロール）／下: 操作バー（常に表示）。
struct ContentView: View {
    enum Page { case main, exhibition, detail, camera }
    enum InputMode { case manual, camera }
    @StateObject private var model = GameModel()
    @State private var showBalance = false
    @State private var page: Page = .main
    @State private var selected: GameModel.Collected?
    @State private var inputMode: InputMode = .manual   // デバッグの入力方法トグル
    // 展示室の検索条件（ページをまたいで保持）
    @State private var exSearch = ""
    @State private var exCategory: String?
    @State private var exRarity: Int?

    private let phoneWidth: CGFloat = 370

    var body: some View {
        ZStack {
            LinearGradient(colors: [Color(white: 0.20), Color(white: 0.10)],
                           startPoint: .top, endPoint: .bottom).ignoresSafeArea()

            ZStack {
                if page == .main {
                    mainPage.transition(.move(edge: .leading))
                } else if page == .exhibition {
                    ExhibitionView(
                        model: model,
                        onBack: { withAnimation(.easeInOut(duration: 0.25)) { page = .main } },
                        onSelect: { c in
                            selected = c
                            withAnimation(.easeInOut(duration: 0.25)) { page = .detail }
                        },
                        search: $exSearch,
                        category: $exCategory,
                        rarity: $exRarity)
                        .transition(.move(edge: .trailing))
                } else if page == .detail, let sel = selected {
                    ArtifactDetailView(item: sel,
                        onBack: { withAnimation(.easeInOut(duration: 0.25)) { page = .exhibition } })
                        .transition(.move(edge: .trailing))
                } else if page == .camera {
                    CameraScanView(
                        onScan: { s in
                            model.input = s
                            withAnimation(.easeInOut(duration: 0.25)) { page = .main }
                            model.excavate()                // スキャン文字列で発掘→アニメ→遺物
                        },
                        onCancel: { withAnimation(.easeInOut(duration: 0.25)) { page = .main } })
                        .transition(.move(edge: .trailing))
                }
            }
            .frame(width: phoneWidth)
            .frame(maxHeight: .infinity)                   // ウィンドウ高さに追従
            .clipShape(RoundedRectangle(cornerRadius: 34))
            .overlay(RoundedRectangle(cornerRadius: 34).strokeBorder(.black.opacity(0.85), lineWidth: 10))
            .shadow(color: .black.opacity(0.5), radius: 20, y: 8)
            .padding(.vertical, 18)

            // 発掘演出モーダル（ファミコン風パラパラ・約3秒）
            if model.phase != .idle {
                DigModalView(scolding: model.phase == .scolding) {
                    model.finishDigAnimation()
                }
                .transition(.opacity)
            }
        }
        .animation(.easeInOut(duration: 0.15), value: model.phase)
        .frame(minWidth: phoneWidth + 40, minHeight: 540)
        .onAppear { model.configureAssetsIfBundled() }   // 初期は何も表示しない
        .sheet(isPresented: $showBalance) { BalanceView(model: model) }
    }

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
            Text("まだ何も発掘していません").font(.headline).foregroundStyle(.secondary)
            #if DEBUG
            Text("入力欄に文字列を入れて「発掘」してみよう")
                .font(.caption).foregroundStyle(.secondary).multilineTextAlignment(.center)
            #else
            Text("QRコードを読み取って発掘しよう")
                .font(.caption).foregroundStyle(.secondary)
            #endif
        }
        .frame(maxWidth: .infinity)
    }

    // MARK: Header
    private var header: some View {
        HStack(spacing: 8) {
            Text("🏺 QR考古学").font(.title2.bold())
            Spacer()
            Button { withAnimation(.easeInOut(duration: 0.25)) { page = .exhibition } } label: {
                Label("展示室", systemImage: "building.columns.fill").font(.caption.bold())
            }
            .buttonStyle(.borderedProminent).tint(.brown).controlSize(.small)
            #if DEBUG
            Text("DEBUG").font(.caption2.bold())
                .padding(.horizontal, 8).padding(.vertical, 3)
                .background(Color.orange).foregroundStyle(.white).clipShape(Capsule())
            #endif
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    // MARK: 遺物カード（スクロール領域）
    private var artifactCard: some View {
        VStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 16).fill(Color(white: 0.93))
                if let img = model.image {
                    Image(nsImage: img).resizable().interpolation(.high).scaledToFit().padding(8)
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
                        Text("神話級").font(.caption.bold())
                            .padding(.horizontal, 6).padding(.vertical, 2)
                            .background(Color.pink.opacity(0.2)).foregroundStyle(.pink).clipShape(Capsule())
                    }
                }
                Text(model.baseName).font(.headline).foregroundStyle(.black)
                    .multilineTextAlignment(.center)
                HStack(spacing: 6) {
                    chip(a.civ, .brown); chip(a.era, .indigo); chip(a.category, .teal)
                }
                statGrid(a)
                if !model.descriptionText.isEmpty { bookExcerpt }
            }
        }
        .padding(14).background(Color.white)
        .clipShape(RoundedRectangle(cornerRadius: 18))
        .shadow(color: .black.opacity(0.08), radius: 6, y: 3)
    }

    // 民明書房調の解説（「詳説 世界の遺物」抜粋の体裁）
    private var bookExcerpt: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Image(systemName: "book.closed.fill").foregroundStyle(.brown)
                Text("詳説 世界の遺物").font(.subheadline.bold()).foregroundStyle(.brown)
            }
            Text(model.descriptionText)
                .font(.callout)
                .foregroundStyle(.black.opacity(0.88))
                .fixedSize(horizontal: false, vertical: true)
                .lineSpacing(3)
            Text("萬象書房 発行　1890年刊版より抜粋")
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
        return LazyVGrid(columns: cols, spacing: 6) {
            stat("基本レア度", "★\(a.baseRarity)")
            stat("時代補正", "+\(a.eraBonus)")
            stat("最終レア度", "★\(a.finalRarity)")
            stat("保存度", "\(Int(a.preservationScore * 100))%")
            stat("汚れ", a.dirtLayerId)
            stat("破損", damageText(a.damage))
            stat("DB段", model.stage == 7 ? "GLOBAL" : "段\(model.stage)")
            stat("hash", String(a.artifactHash.prefix(8)))
        }
        .font(.caption)
    }

    // MARK: 操作バー（常に表示）
    // ⚠️ 手入力・ランダム等のデバッグ機能はリリースビルドでは #if DEBUG により除外される。
    private var controlBar: some View {
        VStack(spacing: 8) {
            #if DEBUG
            // 入力方法の切替（デバッグのみ）。ラベル付き＋アイコンで分かりやすく。
            HStack(spacing: 8) {
                Text("入力方法").font(.caption.bold()).foregroundStyle(labelColor)
                Picker("入力方法", selection: $inputMode) {
                    Label("手動", systemImage: "keyboard").tag(InputMode.manual)
                    Label("カメラ", systemImage: "camera.viewfinder").tag(InputMode.camera)
                }.pickerStyle(.segmented).labelStyle(.titleAndIcon)
            }

            // 入力エリア（固定高さ：トグルで下の行がズレないように）
            ZStack {
                if inputMode == .manual {
                    VStack(spacing: 8) {
                        HStack(spacing: 6) {
                            Image(systemName: "keyboard").foregroundStyle(labelColor)
                            TextField("任意の文字列 / URL を手入力", text: $model.input)
                                .textFieldStyle(.roundedBorder).onSubmit { model.dig() }
                        }
                        Button { model.excavate() } label: {
                            Label("発掘", systemImage: "hammer.fill").frame(maxWidth: .infinity)
                        }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.large)
                            .disabled(!model.canExcavate)
                        presetRow
                    }
                    .frame(maxHeight: .infinity, alignment: .top)
                } else {
                    VStack(spacing: 8) {
                        Button { openCamera() } label: {
                            Label("カメラで発掘", systemImage: "camera.viewfinder").frame(maxWidth: .infinity)
                        }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.large)
                        Text("QRコードをかざすと自動で読み取ります")
                            .font(.caption).foregroundStyle(labelColor)
                    }
                }
            }
            .frame(height: 132)

            Divider().padding(.vertical, 1)

            // デバッグ共通ツール
            HStack(spacing: 8) {
                Button { model.randomExcavate() } label: {
                    Label("ランダム", systemImage: "dice.fill").frame(maxWidth: .infinity)
                }.buttonStyle(.borderedProminent).tint(.brown)
                Button { showBalance = true } label: {
                    Label("分布", systemImage: "chart.bar.fill").frame(maxWidth: .infinity)
                }.buttonStyle(.borderedProminent).tint(.teal).help("ゲームバランス確認")
            }
            HStack(spacing: 8) {
                Toggle("年代", isOn: $model.useYear)
                    .toggleStyle(.switch).fixedSize()
                    .onChange(of: model.useYear) { _ in model.dig() }
                if model.useYear {
                    Stepper("\(model.year)年", value: $model.year, in: 1900...2099)
                        .onChange(of: model.year) { _ in model.dig() }
                        .font(.callout)
                } else {
                    Text("（古いほど高レア・神話級）").font(.caption).foregroundStyle(labelColor)
                }
                Spacer()
            }
            #else
            // リリース: 手入力は出さず、カメラ発掘のみ。
            Button { openCamera() } label: {
                Label("カメラで発掘", systemImage: "camera.viewfinder").frame(maxWidth: .infinity)
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.large)
            Text("QRコードを読み取って遺物を発掘します")
                .font(.caption).foregroundStyle(labelColor)
            #endif
        }
        .padding(12)
        .background(Color(white: 0.985))
        .overlay(Rectangle().frame(height: 1).foregroundStyle(.black.opacity(0.08)), alignment: .top)
    }

    private func openCamera() {
        model.input = ""                                   // 入力欄を初期化してからカメラへ
        withAnimation(.easeInOut(duration: 0.25)) { page = .camera }
    }

    private var presetRow: some View {
        let presets: [(String, String)] = [
            ("URL", "https://example.com/welcome"),
            ("和文", "古代の遺物QRコード"),
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

    // MARK: helpers
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
func damageText(_ d: Damage) -> String {
    let parts = [d.chip ? "欠" : nil, d.crack ? "ひび" : nil, d.wear ? "摩耗" : nil].compactMap { $0 }
    return parts.isEmpty ? "なし" : parts.joined(separator: "/")
}
