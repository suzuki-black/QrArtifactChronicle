# QrArtifactChronicle 🏺

> Scan any QR code and excavate a fictional ancient artifact — deterministically.
> The same QR always yields the same artifact; a finite database feels infinite.

> ⚠️ **Prototype.** Early proof-of-concept. The deterministic core is solid and fully tested,
> but the app, artwork and asset pipeline are placeholders meant for iteration.
> The in-app UI is currently **Japanese only** (English localization is on the roadmap; a
> translation glossary is provided below).

**[English](#english) · [日本語](#日本語)**

---

# English

## What is this?

Read a real-world QR code → hash its contents → generate a fictional artifact of a made-up
ancient civilization. Rarity, civilization, era, category, colour, dirt, damage and a
tongue-in-cheek encyclopaedia blurb are all derived **deterministically** from the QR.
Collect them into an exhibition room.

- **Same QR → same artifact, always** (no randomness, no timestamps).
- **Finite base DB × infinite composition** — colour shifts, dirt/damage layers and flavour
  text make a few thousand base records feel limitless (>10¹² combinations).
- **Gentle gacha-style rarity** (★1 ≈ 40%, ★10 ≈ 0.01%).
- **Era bonus** — if the QR content embeds a real date, older QRs reach the mythic ★11–★13.
- **Offline** — no LLM/server at runtime; everything is precomputed/bundled.

## How to play

1. Tap **📷 Excavate with camera**. (In a debug build you can instead type any text and tap
   **Excavate**, or hit **Random**.)
2. Point the camera at any QR code. It is read **automatically** — no shutter, and scanned
   links are never auto-opened.
3. Enjoy the retro dig animation:
   - **New find** → a girl digs it up → **"New discovery!"** → the artifact is revealed and
     saved to your collection.
   - **Already owned** → your stern master scolds you ("no two artifacts are alike — you can't
     take it from the exhibition!"). The same artifact is shown but not re-added.
4. Open the **🏛 Exhibition** (top-right) to browse everything you've found. Search by text and
   filter by **category** and **rarity**.
5. Tap any artifact card to open its **detail page** (image, stats, and the encyclopaedia blurb).
6. Mythic ★11–★13 only appear from QR codes that contain a genuinely old real-world date — a
   long-term collection goal.

> **Camera trouble?** The MacBook built-in camera is fixed-focus and often struggles with QR
> codes. Use your **iPhone as a Continuity Camera** (autofocus reads QR easily): keep it near
> the Mac (same Apple ID, Wi-Fi + Bluetooth on), still & locked, then pick it from the camera
> menu in the scan screen (*Re-scan cameras* if it isn't listed). An external USB webcam also
> works. For the built-in camera, hold the QR ~30–50 cm away, large and bright, avoid glare.

## In-game text glossary (JP → EN)

The UI is Japanese for now. Key strings:

| Japanese | English |
|----------|---------|
| QR考古学 | QR Archaeology (app title) |
| 発掘 / カメラで発掘 | Excavate / Excavate with camera |
| 展示室 | Exhibition Room |
| 遺物詳細 | Artifact detail |
| もどる / とじる | Back / Close |
| 入力方法: 手動 / カメラ | Input method: Manual / Camera *(debug)* |
| ランダム / 分布 / リセット | Random / Distribution / Reset *(debug)* |
| 年代（古いほど高レア・神話級） | Era *(older = rarer; mythic)* |
| すべて / カメラを再検索 | All / Re-scan cameras |
| まだ何も発掘していません | Nothing excavated yet |
| QRコードを枠内に収めてください | Fit the QR code in the frame |
| はっくつ ちゅう… / …ザクッ | Excavating… / *(dig SFX)* |
| ✨ しんはっけん！ ✨ / あたらしい いぶつ を てに いれた！ | ✨ New discovery! ✨ / You got a new artifact! |
| めっ！ もちだしきんし！ | No! Don't take it out! |
| おなじ いぶつは ２つと ない！ | No two artifacts are alike! |
| てんじしつ から かってに もちだしちゃった…！ | You took it from the exhibition without asking…! |
| 神話級 | Mythic |
| **Stats:** 基本レア度 / 時代補正 / 最終レア度 / 保存度 / 汚れ / 破損 / DB段 | Base rarity / Era bonus / Final rarity / Preservation / Dirt / Damage / DB fallback stage |
| **Damage:** 欠 / ひび / 摩耗 / なし | Chip / Crack / Wear / None |
| **Categories:** 武器 / 祭具 / 生活用品 / 建築断片 / 碑文 / 機械部品 | Weapon / Ritual / Daily-use / Architecture fragment / Inscription / Machine part |

Civilizations (`desert/ocean/mountain/machine/organic`), eras
(`ancient/medieval/early_modern/modern/future`) and dirt types are shown by their English keys.
The artifact **description** is atmospheric flavour text in the deliberately pompous, dubious
style of the fictional publisher *"萬象書房" (Banshō Shobō, "est. 1890")* — an homage to the
*Minmei Shobō* gag from the manga *Sakigake!! Otokojuku*. All blurbs are original generated text.

## How it works (determinism)

```
QR bytes → normalize → SHA-256 (seed) → tagged sub-streams → attributes
                                                  ├ rarity (integer thresholds, float-free)
                                                  ├ civilization / era / category
                                                  ├ colour (HSV), dirt, damage, preservation
                                                  └ era bonus (only if a real date is found)
        → DB select (civ × era × category × rarity, 6-stage fallback)
        → compose image (base sprite → HSV → dirt → damage → preservation → background)
        → flavour text
```

Byte-for-byte reproducible. A TypeScript reference implementation is the language-neutral
**oracle**; the Rust core must reproduce its golden vectors exactly:

```
TS reference  ≡  vectors/golden.json  ≡  Rust qrac-core  ≡  Swift (via FFI)
```

## Repository layout

```
docs/                 Design spec (00–08) + open-questions log
reference/            TypeScript reference core (oracle) + golden vectors + tests
rust/
  qrac-core/          Pure deterministic core (hash, normalize, rarity, timestamp,
                      appearance, flavor, derive) — verified against golden.json
  qrac-render/        I/O: SQLite base-artifact DB (rusqlite) + image compose (tiny-skia)
  qrac-ffi/           UniFFI boundary exposing the core to Swift/Kotlin
  qrac-assetgen/      Asset generator: base images + full-coverage DB + CI coverage gate
apple/                macOS SwiftUI app (QrArtifactChronicle.app) + XCFramework packaging
```

## Build & run

```bash
# Rust core tests (determinism, distribution, independence, DB fallback, compose)
cd rust && cargo test --workspace

# TypeScript oracle (regenerate / verify golden vectors)
cd reference && npm test

# Generate assets (base images + artifacts.sqlite, with coverage gate)
cd rust && cargo run -p qrac-assetgen -- dist

# macOS app (builds the Rust core, packages an XCFramework, assembles the .app)
cd apple && bash build-app.sh
open QrArtifactChronicle.app   # unsigned: first launch may need right-click → Open
```

Requirements: Rust (stable), Node ≥ 23.6 (reference's native TS execution), Xcode / Swift 6.

## Roadmap / TODO

- [ ] **In-app English localization (i18n)** — the UI is Japanese-only for now.
- [ ] **Replace UniFFI (MPL-2.0) with a hand-written C ABI FFI** for a fully permissive
      dependency tree (only needed for a strict closed-source/commercial posture; MPL is fine
      for the current MIT/OSS release).
- [ ] iOS & Android targets (multi-platform XCFramework; Kotlin bindings via cargo-ndk).
- [ ] Replace procedural placeholder art with real artist assets (WebP); layout is swap-ready.
- [ ] Real 6-layer compositing (dirt/damage PNG layers + blend modes); currently procedural.
- [ ] Lightweight collection (regenerate from key) instead of storing rendered PNGs.
- [ ] Code signing & notarization for distribution.

## License

**MIT © 2026 suzuki-black** — see [LICENSE](LICENSE).
Third-party components (all MIT-compatible; UniFFI is MPL-2.0, used unmodified) are listed in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

---

# 日本語

> QRコードを読み取ると、架空の古代文明の遺物を「決定論的に」発掘するアプリ。
> 同じQRからは必ず同じ遺物。有限のDBなのに無限に感じる設計です。
> ※ 現在アプリ内UIは**日本語のみ**（英語化はロードマップ。上記に英訳glossaryあり）。

## これは何？

現実のQRコードを読む → 内容をハッシュ化 → 架空の古代文明の遺物を生成します。レア度・文明・
時代・カテゴリ・色・汚れ・破損、そして胡散臭い解説文まで、すべてQRから**決定論的**に決まります。
発掘した遺物は「展示室」に集まります。

- **同じQR → 必ず同じ遺物**（乱数なし・スキャン時刻にも依存しない）
- **有限ベースDB × 無限合成**（色変え・汚れ/破損・個体差テキストで 10¹²通り超）
- **優しいガチャ的レア度**（★1 ≈ 40%、★10 ≈ 0.01%）
- **時代補正** — QR内容に実日付が含まれると、古いQRほど神話級 ★11〜★13 に到達
- **オフライン** — 実行時にLLMもサーバも使わず、すべて事前生成・同梱

## 遊び方

1. **📷 カメラで発掘** をタップ（デバッグ版では文字を手入力して「発掘」、または「ランダム」も可）。
2. カメラをQRコードに向けると **自動で読み取り**（シャッター不要・リンク自動表示なし）。
3. ファミコン風の発掘演出:
   - **新規** → 女の子が掘り当て → **「しんはっけん！」** → 遺物を表示し展示室に保存。
   - **発掘済み** → お師匠様に叱られる（「同じ遺物は2つとない＝展示室から持ち出しちゃダメ！」）。
     遺物は表示されるが再登録はされない。
4. 右上の **🏛 展示室** で発掘済みを閲覧。**文字検索＋カテゴリ／レア度フィルタ**。
5. 遺物カードをタップで **詳細ページ**（画像・ステータス・解説文）。
6. 神話級 ★11〜★13 は「現実に古い日付を含むQR」からのみ出現する到達目標。

> **カメラが読みにくい？** MacBook内蔵カメラは固定焦点でQRが苦手なことがあります。**iPhoneを連係
> カメラ**として使うと解決します（Macの近くに置き・静止・ロック、スキャン画面のカメラメニューで選択。
> 出なければ「カメラを再検索」）。外付けUSBカメラも可。内蔵で頑張る場合は 30〜50cm 離して大きく明るく。

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

全工程がバイト単位で再現可能。TypeScript参照実装が**言語非依存のオラクル**で、Rustコアはその
ゴールデンベクタを完全一致で再現します（`TS ≡ golden.json ≡ Rust ≡ Swift`）。

## リポジトリ構成

```
docs/         設計仕様（00〜08）＋意思決定ログ
reference/    TypeScript 参照コア（オラクル）＋ゴールデンベクタ＋テスト
rust/qrac-core      純粋な決定論コア（hash/normalize/rarity/timestamp/appearance/flavor/derive）
rust/qrac-render    I/O: ベース遺物DB(rusqlite) ＋ 画像合成(tiny-skia)
rust/qrac-ffi       UniFFI境界（Swift/Kotlinへ公開）
rust/qrac-assetgen  アセット生成（ベース画像＋全rarity充足DB＋カバレッジゲート）
apple/        macOS SwiftUI アプリ＋XCFramework化
```

## ビルド & 実行

```bash
cd rust && cargo test --workspace                 # Rustコアのテスト
cd reference && npm test                           # TSオラクル（Node 23.6+）
cd rust && cargo run -p qrac-assetgen -- dist      # アセット生成
cd apple && bash build-app.sh && open QrArtifactChronicle.app   # macOSアプリ
```

## ロードマップ / TODO

- [ ] **アプリ内の英語化（i18n）** — 現状UIは日本語のみ
- [ ] **UniFFI(MPL-2.0) → 手書き C ABI FFI 置換**（依存ツリーを完全に寛容化。商用クローズド時のみ必須）
- [ ] iOS / Android 対応（マルチプラットフォーム XCFramework、cargo-ndk）
- [ ] 手続き生成のプレースホルダ画像をアーティスト製アセット(WebP)に差し替え（配置は差替前提済み）
- [ ] 実行時の本格6層合成（透過PNGレイヤー＋ブレンドモード。現状は手続き描画）
- [ ] 収集データの軽量化（画像保存をやめキーから再生成）
- [ ] 配布用のコード署名・公証

## ライセンス

**MIT © 2026 suzuki-black** — [LICENSE](LICENSE) 参照。サードパーティ（すべてMIT互換、UniFFIは
MPL-2.0で未改変利用）は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) に記載。

> 「民明書房」風の解説文は『魁!!男塾』へのオマージュです。出版社名・解説文はすべてオリジナルの生成テキストです。
