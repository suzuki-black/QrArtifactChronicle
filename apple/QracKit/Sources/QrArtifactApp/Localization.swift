import Foundation
import QracFFI

/// アプリ内言語設定。System / English / 日本語。即時切替・永続化。
enum AppLanguage: String, CaseIterable, Identifiable {
    case system, en, ja
    var id: String { rawValue }
}

@MainActor
final class Settings: ObservableObject {
    @Published var language: AppLanguage {
        didSet { UserDefaults.standard.set(language.rawValue, forKey: Self.key) }
    }
    private static let key = "appLanguage"

    init() {
        let raw = UserDefaults.standard.string(forKey: Self.key) ?? ""
        language = AppLanguage(rawValue: raw) ?? .system
    }

    /// system を端末ロケールで解決した実効言語。
    var effective: AppLanguage {
        switch language {
        case .system:
            return (Locale.preferredLanguages.first ?? "en").hasPrefix("ja") ? .ja : .en
        case .en: return .en
        case .ja: return .ja
        }
    }
    var isJa: Bool { effective == .ja }

    /// 日本語/英語の文字列を実効言語で選ぶ。
    func t(_ ja: String, _ en: String) -> String { isJa ? ja : en }

    /// 解説文生成用の FFI 言語。
    func ffiLang() -> Lang { isJa ? .ja : .en }

    // MARK: 語彙テーブル
    // Q7=(i): 型呼称の語彙は qrac-core を単一の出所とし、FFI 経由で取得する。
    // これにより「出土の系譜」の参照名と詳細チップの呼称が構造的に一致する（提案01 §3.4）。

    func civName(_ k: String) -> String { civLabel(civ: k, lang: ffiLang()) }
    func eraName(_ k: String) -> String { eraLabel(era: k, lang: ffiLang()) }
    func categoryName(_ k: String) -> String { categoryLabel(category: k, lang: ffiLang())
    }
    func dirtName(_ k: String) -> String {
        switch k {
        case "none": return t("なし", "none")
        case "mud": return t("泥", "mud")
        case "sand": return t("砂", "sand")
        case "soot": return t("煤", "soot")
        case "sea_salt": return t("海塩", "sea salt")
        case "volcanic_ash": return t("火山灰", "volcanic ash")
        default: return k
        }
    }
    func damageText(chip: Bool, crack: Bool, wear: Bool) -> String {
        let parts = [
            chip ? t("欠け", "chip") : nil,
            crack ? t("ひび", "crack") : nil,
            wear ? t("摩耗", "wear") : nil,
        ].compactMap { $0 }
        return parts.isEmpty ? t("なし", "none") : parts.joined(separator: " / ")
    }

    /// 遺物の表示名（civ/era/category/レア度から多言語生成）。
    func artifactName(civ: String, era: String, category: String, rarity: Int) -> String {
        if isJa {
            return "\(civName(civ))の\(categoryName(category))（\(eraName(era))・★\(rarity)）"
        } else {
            return "\(civName(civ)) \(categoryName(category)) (\(eraName(era)), ★\(rarity))"
        }
    }

    // 書誌（解説の体裁）
    var bookTitle: String { t("詳説 世界の遺物", "A Detailed Account of the World’s Relics") }
    var bookFooter: String {
        t("萬象書房 発行　1890年刊版より抜粋", "from Banshō Shobō, 1890 edition")
    }
    var mythicLabel: String { t("神話級", "Mythic") }
}
