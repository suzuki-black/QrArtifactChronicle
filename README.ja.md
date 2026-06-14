# QrArtifactChronicle 🏺

> QRコードを読み取ると、架空の古代文明の遺物を「決定論的に」発掘するアプリ。
> 同じQRからは必ず同じ遺物。有限のDBなのに無限に感じる設計です。

**🇬🇧 English: [README.md](README.md)**

> ⚠️ **プロトタイプ版です。** 決定論コアは堅牢でテスト済みですが、アプリ・アート・アセット
> パイプラインは反復前提の暫定実装です。API・データ形式・見た目は今後変わります。

---

## これは何？

現実のQRコードを読む → 内容をハッシュ化 → 架空の古代文明の遺物を生成します。レア度・文明・
時代・カテゴリ・色・汚れ・破損、そして胡散臭い解説文まで、すべてQRから**決定論的**に決まります。
発掘した遺物は「展示室」に集まります。

- **同じQR → 必ず同じ遺物**（乱数なし・スキャン時刻にも依存しない）
- **有限ベースDB × 無限合成** — 色変え・汚れ/破損レイヤー・個体差テキストで、数千件のベースが
  無限に感じる（10¹²通り超）
- **優しいガチャ的レア度**（★1 ≈ 40%、★10 ≈ 0.01%）
- **時代補正** — QR内容に実日付が含まれると、古いQRほど神話級 ★11〜★13 に到達
- **オフライン** — 実行時にLLMもサーバも使わず、すべて事前生成・同梱

## 仕組み（決定論）

```
QRバイト列 → 正規化 → SHA-256(シード) → タグ別サブストリーム → 各属性
                                              ├ レア度（整数しきい値・float非依存）
                                              ├ 文明 / 時代 / カテゴリ
                                              ├ 色(HSV) / 汚れ / 破損 / 保存状態
                                              └ 時代補正（実日付が取れた時のみ）
        → DB選択（文明×時代×カテゴリ×レア度、6段フォールバック）
        → 画像合成（ベース画像 → HSV → 汚れ → 破損 → 保存状態 → 背景）
        → 民明書房調の解説文
```

全工程がバイト単位で再現可能です。TypeScript参照実装が**言語非依存のオラクル**となり、Rustコアは
そのゴールデンベクタを完全一致で再現します:

```
TS参照  ≡  vectors/golden.json  ≡  Rust qrac-core  ≡  Swift（FFI経由）
```

## リポジトリ構成

```
docs/                 設計仕様書（00〜08）＋意思決定ログ
reference/            TypeScript 参照コア（オラクル）＋ゴールデンベクタ＋テスト
rust/
  qrac-core/          純粋な決定論コア（hash/normalize/rarity/timestamp/appearance/flavor/derive）
  qrac-render/        I/O層: ベース遺物DB(rusqlite) ＋ 画像合成(tiny-skia)
  qrac-ffi/           UniFFI境界（コアを Swift/Kotlin へ公開）
  qrac-assetgen/      アセット生成: ベース画像＋全rarity充足DB＋カバレッジゲート
apple/                macOS SwiftUI アプリ（QrArtifactChronicle.app）＋ XCFramework 化
```

## ビルド & 実行

**Rustコアのテスト**（決定論・分布・独立性・DBフォールバック・合成）:
```bash
cd rust && cargo test --workspace
```

**TypeScriptオラクル**（ゴールデンベクタの生成/検証）:
```bash
cd reference && npm test
```

**アセット生成**（ベース画像＋`artifacts.sqlite`、カバレッジゲート付き）:
```bash
cd rust && cargo run -p qrac-assetgen -- dist
```

**macOSアプリ**（Rustコアのビルド→XCFramework化→.app組み立て）:
```bash
cd apple && bash build-app.sh
open QrArtifactChronicle.app    # 未署名のため初回は右クリック→開く
```

必要環境: Rust(stable)、Node 23.6以上（参照TSのネイティブ実行用）、Xcode / Swift 6（macOSアプリ用）。

## 📷 カメラ読み取りについて

アプリはQRをライブ検出します（Vision `VNDetectBarcodesRequest`。シャッター不要・リンク自動表示なし）。
**MacBookの内蔵カメラは固定焦点で、QR読み取りが苦手なことが多いです。**

読み取れない場合:
- **iPhoneを連係カメラとして使う** のが最も確実です（オートフォーカスでQRがすぐ読めます）。Macの近くに
  置き（同一Apple ID・Wi-Fi/Bluetooth ON）、静止・ロックした状態でスキャン画面のカメラメニューから選択
  （一覧に出ないときは「カメラを再検索」）。
- 外付けUSBウェブカメラでもOK。
- 内蔵カメラで頑張る場合: QRを **30〜50cm** 離して大きく・明るく映し、反射を避ける。

## ロードマップ / TODO

- [ ] **UniFFI(MPL-2.0) を手書き C ABI FFI に置換**し、依存ツリーを完全に寛容ライセンス化
      （クローズド/商用を厳格に行う場合のみ必要。現在のMIT/OSS公開ではMPLのままで問題なし）
- [ ] iOS / Android 対応（マルチプラットフォーム XCFramework、cargo-ndk で Kotlin バインディング）
- [ ] 手続き生成のプレースホルダ画像を、アーティスト製アセット(WebP)に差し替え（配置・命名は差替前提済み）
- [ ] 実行時の6層合成（汚れ/破損の透過PNGレイヤー＋ブレンドモード。現状は手続き描画の暫定）
- [ ] 収集データの軽量化（画像を保存せずキーから再生成するモデルへ）
- [ ] 配布用のコード署名・公証

## ライセンス

**MIT © 2026 suzuki-black** — [LICENSE](LICENSE) 参照。

サードパーティ（すべてMIT互換。UniFFIはMPL-2.0で未改変利用）は
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) に記載。

> 「民明書房」風の解説文は『魁!!男塾』へのオマージュです。本プロジェクトの出版社名・解説文は
> すべてオリジナルの生成テキストです。
