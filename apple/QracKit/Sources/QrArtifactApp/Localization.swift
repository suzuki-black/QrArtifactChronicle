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

    func civName(_ k: String) -> String {
        switch k {
        case "desert": return t("砂漠文明", "Desert")
        case "ocean": return t("海洋文明", "Ocean")
        case "mountain": return t("山岳文明", "Mountain")
        case "machine": return t("機械文明", "Machine")
        case "organic": return t("有機文明", "Organic")
        default: return k
        }
    }
    func eraName(_ k: String) -> String {
        switch k {
        case "ancient": return t("古代", "Ancient")
        case "medieval": return t("中世", "Medieval")
        case "early_modern": return t("近世", "Early modern")
        case "modern": return t("近代", "Modern")
        case "future": return t("未来", "Future")
        default: return k
        }
    }
    func categoryName(_ k: String) -> String {
        switch k {
        case "weapon": return t("武器", "Weapon")
        case "ritual": return t("祭具", "Ritual")
        case "daily": return t("生活用品", "Daily-use")
        case "architecture": return t("建築断片", "Architecture")
        case "inscription": return t("碑文", "Inscription")
        case "machine_part": return t("機械部品", "Machine part")
        default: return k
        }
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
