// macOS 最小動作デモ: Rust 決定論コア(qrac-core) を UniFFI 経由で Swift から呼ぶ。
// 「実際にQRから遺物が出る」ことを確認する CLI。SwiftUI 版は同じ deriveQr() を使うだけ。
import Foundation

func stars(_ n: UInt32) -> String { String(repeating: "★", count: Int(n)) }

func show(_ label: String, _ text: String, year: Int32? = nil, useYear: Bool = false) {
    let a = useYear ? deriveQrWithYear(text: text, year: year) : deriveQr(text: text)
    print("───────────────────────────────────────────────")
    print("QR入力: \(label) 「\(text.replacingOccurrences(of: "\n", with: "\\n"))」")
    print("  hash : \(a.artifactHash)")
    let mythic = a.isMythic ? "   ✨神話級✨" : ""
    print("  レア : \(stars(a.finalRarity)) (★\(a.finalRarity) = base★\(a.baseRarity) + 補正\(a.eraBonus))\(mythic)")
    print("  遺物 : \(a.civ)文明 / \(a.era) / \(a.category)")
    print(String(format: "  色   : H%+.1f° S×%.2f V×%.2f / 汚れ:%@",
                 a.color.hShift, a.color.sMul, a.color.vMul, a.dirtLayerId))
    let dmg = [a.damage.chip ? "欠け" : nil, a.damage.crack ? "ひび" : nil, a.damage.wear ? "摩耗" : nil]
        .compactMap { $0 }.joined(separator: ",")
    print(String(format: "  状態 : 破損[%@] 保存度%.0f%%",
                 dmg.isEmpty ? "なし" : dmg, a.preservationScore * 100))
}

print("\n🏺 QrArtifactChronicle — Rust(qrac-core) × Swift via UniFFI\n")

show("URL", "https://example.com/welcome")
show("テキスト", "古代の遺物QRコード")
show("vCard(2014年)", "BEGIN:VCARD\nREV:2014-03-10T19:00:00Z\nEND:VCARD")
show("古い日付(1985)", "log 1985-06-15 backup")
show("超古いQRと仮定(1979指定)", "treasure map", year: 1979, useYear: true)

print("\n=== 実際に遺物画像を合成（Rust tiny-skia → PNG）===")
// qrac-assetgen が出力したベース画像を使う（無ければ手続き生成にフォールバック）。
configureAssets(dir: "dist/assets")
let outDir = "target/artifacts"
try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)
func renderAndSave(_ name: String, _ text: String) {
    let img = renderQr(text: text, lang: .ja)
    let path = "\(outDir)/\(name).png"
    do {
        try img.png.write(to: URL(fileURLWithPath: path))
        print(String(format: "  🖼 %@.png  %dx%d  %dB  base「%@」 (段%d)",
                     name, img.width, img.height, img.png.count, img.baseName, img.matchedStage))
    } catch {
        print("  ⚠️ 書き込み失敗 \(name): \(error)")
    }
}
renderAndSave("url", "https://example.com/welcome")
renderAndSave("japanese", "古代の遺物QRコード")
renderAndSave("vcard2014", "BEGIN:VCARD\nREV:2014-03-10T19:00:00Z\nEND:VCARD")
renderAndSave("iso1985", "log 1985-06-15 backup")
print("  → open \(outDir)/  で画像を確認できます")

print("\n=== 必須要件の確認 ===")
let a1 = deriveQr(text: "https://example.com/welcome")
let a2 = deriveQr(text: "https://example.com/welcome")
print("・同じQRは必ず同じ遺物 : \(a1.artifactHash == a2.artifactHash ? "✅ 一致" : "❌ 不一致")")
let canon = deriveQr(text: "https://EXAMPLE.com:443/welcome").artifactHash
print("・表記揺れ吸収(host大小/既定ポート) : \(a1.artifactHash == canon ? "✅ 同一遺物" : "❌ 別")")
let diff = deriveQr(text: "https://example.com/other").artifactHash
print("・異なるQRは異なる遺物 : \(a1.artifactHash != diff ? "✅ 別遺物" : "❌ 衝突")")
print("")
