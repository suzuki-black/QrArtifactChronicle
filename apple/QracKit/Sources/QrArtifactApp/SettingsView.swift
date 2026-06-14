import SwiftUI

/// 設定ページ: 言語切替 ＋ About（1画面セクション構成）。
struct SettingsView: View {
    @ObservedObject var settings: Settings
    let onBack: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            header
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    languageSection
                    aboutSection
                }
                .padding(16)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(red: 0.96, green: 0.94, blue: 0.89))
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button { onBack() } label: {
                Label(settings.t("もどる", "Back"), systemImage: "chevron.left").font(.callout.bold())
            }.buttonStyle(.borderedProminent).tint(.blue).controlSize(.small)
            Text(settings.t("設定", "Settings")).font(.title3.bold())
            Spacer()
        }
        .padding(.horizontal, 16).padding(.top, 16).padding(.bottom, 10)
        .background(Color(red: 0.91, green: 0.87, blue: 0.79))
    }

    private func section<Content: View>(_ title: String, @ViewBuilder _ content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title).font(.subheadline.bold()).foregroundStyle(.secondary)
            content()
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.white)
                .clipShape(RoundedRectangle(cornerRadius: 12))
        }
    }

    private var languageSection: some View {
        section(settings.t("言語", "Language")) {
            Picker("", selection: $settings.language) {
                Text(settings.t("システムに従う", "System")).tag(AppLanguage.system)
                Text("English").tag(AppLanguage.en)
                Text("日本語").tag(AppLanguage.ja)
            }
            .pickerStyle(.segmented)
            .labelsHidden()
        }
    }

    private var aboutSection: some View {
        section(settings.t("このアプリについて", "About")) {
            VStack(alignment: .leading, spacing: 8) {
                Text("🏺 QrArtifactChronicle").font(.headline)
                row(settings.t("バージョン", "Version"), "0.2.0 (prototype)")
                Text(settings.t(
                    "QRコードから架空の古代遺物を決定論的に発掘する実験的アプリ。",
                    "An experimental app that deterministically excavates fictional ancient artifacts from QR codes."))
                    .font(.callout).foregroundStyle(.secondary)
                row(settings.t("ライセンス", "License"), "MIT © 2026 suzuki-black")
                Text(settings.t(
                    "解説文は漫画『魁!!男塾』の民明書房ネタへのオマージュです（出版社名・本文はすべてオリジナル）。",
                    "Flavour text is an homage to the “Minmei Shobō” gag from the manga Sakigake!! Otokojuku (all original)."))
                    .font(.caption).foregroundStyle(.secondary)
                Text("github.com/suzuki-black/QrArtifactChronicle")
                    .font(.caption.monospaced()).foregroundStyle(.blue)
            }
        }
    }

    private func row(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label).foregroundStyle(.secondary)
            Spacer()
            Text(value).bold()
        }.font(.callout)
    }
}
