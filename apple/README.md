# apple/ — macOS アプリ（SwiftUI ＋ Rust コア）

Rust 決定論コア(`qrac-core`)＋I/O層(`qrac-render`)を UniFFI 経由で SwiftUI から呼ぶ macOS アプリ。

## 構成

```
apple/
  QracFFI.xcframework/      Rust 静的ライブラリ(libqrac_ffi.a)を XCFramework 化（macOS arm64 slice）
  headers/                 生成ヘッダ + module.modulemap（xcframework 生成材料）
  QracKit/                 SwiftPM パッケージ
    Package.swift          binaryTarget(xcframework) + QracFFI(生成Swift) + QrArtifactApp(SwiftUI)
    Sources/QracFFI/       UniFFI 生成 Swift API（qrac_ffi.swift）
    Sources/QrArtifactApp/ App / ContentView / GameModel / DigModalView /
                           ExhibitionView / ArtifactDetailView / CameraScanView /
                           Scanner / SoundPlayer
  sounds/                  効果音(オリジナル合成) dig/reveal/scold/error .wav
  build-app.sh             Rust→xcframework→アセット生成→.app を一括ビルド
  QrArtifactChronicle.app  生成物（double-click 起動可）
```

## ビルド & 起動

```bash
cd apple
bash build-app.sh                       # debug   → QrArtifactChronicle.app（デバッグUIあり）
APP_CONFIG=release bash build-app.sh    # release → QrArtifactChronicleRelease.app（#if DEBUG除外）
open QrArtifactChronicle.app            # 未署名のため初回は右クリック→開く
```

`build-app.sh` の流れ: Rust release(cdylib+staticlib) → UniFFIバインディング生成 →
XCFramework作成 → `swift build`（`APP_CONFIG`で debug/release 切替） → アセット生成(qrac-assetgen) →
`.app` 組み立て（assets/sounds 同梱・`NSCameraUsageDescription` 付与・ad-hoc署名）。
リリースでは `#if DEBUG` のデバッグUIが除外され、別バンドルID＋別名 `QrArtifactChronicleRelease.app` で出力。

## Xcode で開く場合

`QracKit/Package.swift` を Xcode で開き、`QrArtifactApp` スキームを Run。
（Xcode はパッケージ実行ターゲットを macOS アプリとして起動する。）

## 画面（スマホ縦長前提のゲーム画面）

macOS 上で **phone フレーム（360×800）** のゲーム画面を表示（iOS 展開前のバランス確認用）。

- **遺物カード**: 合成画像（レア度色の枠）＋ ★レア度 ＋ 名称 ＋ 文明/時代/カテゴリ チップ ＋
  バランスに効くステータス（基本★ / 時代補正 / 最終★ / 保存度 / 汚れ / 破損 / DBフォールバック段 / hash）。
- **ページ遷移**: メイン / 展示室 / 遺物詳細 / カメラ / 設定（phoneフレーム内スライド）。
- **多言語（i18n）**: ヘッダーの**設定⚙️**で 英語 / 日本語 / システム追従 を即時切替・永続化。UI文言・
  カテゴリ語彙・遺物表示名・**解説文**まで言語連動（明色テーマ固定）。
- **カメラQRスキャナ**（リリースの主入力）: Vision `VNDetectBarcodesRequest` でライブ検出（撮影不要・
  リンク自動表示なし）。複数カメラ一覧/切替（内蔵 / iPhone連係 / 外付け）。
- **発掘演出**: ファミコン風パラパラ＋効果音。新規=女の子採掘→「新発見！」、既出=師匠が叱る→「もちだしちゃった…」。
- **展示室**: 発掘済み一覧（カテゴリ/レア度/文字で検索）。`collection.json` に**自動保存**（再起動で復元）。
- **解説文**: 民明書房調（「詳説 世界の遺物（萬象書房 1890年刊）」抜粋体裁）。

### デバッグ専用（`#if DEBUG`・リリースでは非表示）
- **手入力/カメラ切替**、任意文字列の手入力＋プリセット、**ランダム**（アニメ無し連続発掘）、
  **年代指定**（時代補正の確認）、**ゲームバランス確認**（固定シード大量サンプリングで分布可視化）、
  **展示室リセット**。

実測例（n=5000, 補正なし）: ★1=40.8%(目標40.49) … ★8=0.36%(目標0.40)、
文明/時代 ≈ 各20%、カテゴリ ≈ 各16.7% → 設計どおりの優しい分布＋均等出現。

ロジックは全て Rust 側。Swift は入力・表示・集計・収集のみ。**同じQRは必ず同じ遺物**（FFI越しでも決定論）。

## 📷 カメラ読み取りの注意

MacBook内蔵カメラは固定焦点でQRが苦手なことがあります。読めない場合は **iPhoneを連係カメラ**として使うと
解決します（スキャン画面のカメラメニューから選択。一覧に出なければ「カメラを再検索」）。詳細はルートREADME参照。

## 次（iOS/Android・実機配布）

- iOS スライス: `cargo build --target aarch64-apple-ios`（＋ simulator）を追加し、
  `xcodebuild -create-xcframework` に複数 `-library` を渡してマルチプラットフォーム xcframework 化。
- Android: 同じ `qrac-ffi` から UniFFI Kotlin バインディング＋JNI（`cargo-ndk`）。
- 署名/配布: Developer ID 署名 + notarize。
