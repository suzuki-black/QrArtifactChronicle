# 08. 実装設計（Rust共有コア + ネイティブUI）

`docs/00`〜`docs/07` の確定仕様を、実装可能な構成に落とす。
**決定論コアを Rust で1回だけ書き、3プラットフォームで共有**するのが本設計の背骨。

## 8.1 確定スタック

| 層 | 採用 | 根拠 |
|----|------|------|
| 決定論ロジック | **Rust 100%共通** | byte一致を1ソースで保証。`docs/01`の手実装正規化もここに集約 |
| 乱数/ハッシュ | Rust `sha2` クレート（版ピン） | SHA-256はbyte一致。版を `genVersion` に紐付け |
| 画像合成 | **Rust + tiny-skia（CPU）** | 6層合成は単発+キャッシュ。CPUで十分速く、ピクセルもbyte一致。GPU/wgpu不採用 |
| URL正規化 | **手実装**（`docs/01` をRust移植） | 汎用URLライブラリの%エンコード/IDN/ポート差を排除 |
| FFI | **UniFFI** | Swift/Kotlin両バインディング自動生成。手書きグルーのバグ源を排除 |
| DB | SQLite（`rusqlite`） | 同梱の読み取り専用遺物DB＋収集DB |
| アセット | WebP主・AVIF限定（+SQLite） | `docs/05`。AVIFデコード(dav1d)は重いので限定 |
| UI | SwiftUI（macOS/iOS）/ Jetpack Compose（Android） | ネイティブ体験。コアは共有 |
| 移植順 | **macOS → iOS → Android** | 開発ループが軽い順。最も毛色の違うAndroidを最後 |

## 8.2 全体構成図

> 実装の現況を反映（設計当初の単一クレート案から、純粋部とI/O部を別クレートに分離した）。

```
┌─ qrac-core (純粋・決定論, byte一致必須) ────────────────────────────┐
│   normalize / hash(rng) / rarity / timestamp / appearance / flavor   │
│   / derive / constants / types                                       │
└─────────────────────────────────────────────────────────────────────┘
            ▲ 依存
┌─ qrac-render (I/O) ──────────────┐   ┌─ qrac-assetgen (開発ツール) ──┐
│   db.rs(rusqlite) / compose.rs   │   │   ベース画像＋DB生成＋検証     │
│   (tiny-skia) / assets.rs        │   └───────────────────────────────┘
└──────────────────────────────────┘
            ▲ 依存
┌─ qrac-ffi (UniFFI境界) ──────────┐
│   derive_qr / render_qr / …      │  → Swift/Kotlin バインディング自動生成
└──────────────────────────────────┘
            │
   ┌────────▼─────────┐                      ┌──────────────────┐
   │ Swift (SwiftUI)  │  macOS（実装済み）   │ Kotlin (Compose) │  Android（未）
   └──────────────────┘                      └──────────────────┘

参照実装（reference/, TypeScript）= 決定論の「正本オラクル」。
 → vectors/golden.json を生成。Rust側 tests/golden.rs が同じベクタを再現一致検証（8.9）。
 ※ 収集データの永続化は現状 Swift 側（GameModel の collection.json）。docs/06 6.3 のSQLite案は未採用。
```

## 8.3 Rust クレート構成（実装）

ワークスペース `rust/`（`Cargo.toml` の members = qrac-core / qrac-render / qrac-ffi / qrac-assetgen）。

```
qrac-core/  （純粋・決定論。I/Oゼロ。reference/src/*.ts と1:1, golden.rsで検証）
  src/ hash.rs        # substream/uniform/u32/uintBelow/pickWeighted（big-endian, NFC）
       normalize.rs   # docs/01 1.1 手実装
       rarity.rs      # docs/02（整数しきい値）
       timestamp.rs   # docs/02 2.2 extractYear
       appearance.rs  # docs/04 色違い・汚れ・破損・保存状態
       flavor.rs      # docs/04 4.5 民明書房調の解説文（単一段落）
       derive.rs      # docs/00 0.2 (A)〜(F) を束ねる純粋オーケストレータ
       constants.rs   # 重み・しきい値・タグ（reference/src/constants.ts と一致）
       types.rs / lib.rs
  tests/ golden.rs / units.rs

qrac-render/  （I/O。qrac-core に依存）
  src/ db.rs       # docs/03 候補選択・6段フォールバック（rusqlite）
       compose.rs  # docs/05 画像合成（tiny-skia・6層／ブレンド／マスク, 8.6）
       art.rs      # 手続きアート（ベース被写体・汚れ/破損/背景レイヤー生成）
       lib.rs

qrac-ffi/   （UniFFI境界。qrac-core + qrac-render に依存。8.5）
  src/ lib.rs       # uniffi::setup_scaffolding! ＋ #[uniffi::export] 群
       bin/uniffi-bindgen.rs   # 同一版でSwift/Kotlinバインディング生成
  demo/ main.swift / run.sh    # CLI動作デモ

qrac-assetgen/  （開発ツール。ベース画像＋全rarity充足DB＋カバレッジゲート。docs/05 5.6）
```

> 設計当初は単一 `qrac-core` に db/compose/text/collection を含める想定だったが、純粋性を保つため
> db/compose は `qrac-render` に分離。`text.rs`→`flavor.rs`、`rng.rs`→`hash.rs`、`collection.rs` は
> Rustに持たず Swift 側（collection.json）に。UniFFI は `.udl` ではなく proc-macro 方式。

`pure` 群（normalize/rng/rarity/timestamp/appearance/derive）は I/O ゼロ。`reference/src/*.ts` と
**関数単位で対応**させ、golden で同値を担保する。

## 8.4 🔑 整数しきい値による決定論（float依存の排除）

**最重要の移植論点（`docs/01` 1.3 のNOTE / `docs/07` impl論点①の確定方針）。**

`uniform = u64be/2^64` を `f64` 化して比較すると、**離散判定の境界でバケットが飛ぶ**恐れがある
（bigint→f64 / u64→f64 の丸めが言語で最終ビット差を生む）。そこで:

> **規則: 離散決定は整数比較で行う。連続出力のみ f64 を使う。**

| 種別 | 対象 | 方式 |
|------|------|------|
| **離散** | レア度バケット、破損ON/OFF、`pickWeighted` | substream の生整数（u32/u64, big-endian）を**整数しきい値**と直接比較。f64を経由しない |
| **連続** | `colorMod`(hShift/sMul/vMul)、`preservationScore` | `f64 = u64be/2^64` から算出。最終ビットはIEEE754でbyte一致（同一演算順なら言語非依存）、かつ離散境界を持たないので飛ばない |

実装規約:
- レア度: `x = u32be(substream(seed,"rarity"))` を `RARITY_CUTOFFS_U32`（= `floor(CUM * 2^32)` の**ハードコード定数**）と比較。
- 破損: `u32be(substream(seed,"damage.chip"))` を `floor(0.5 * 2^32)` 等と比較。
- `pickWeighted`（整数重み）: `r = uintBelow(seed, tag, Σw)` を累積整数重みで走査（f64の `uniform*total` を使わない）。
- **しきい値定数は TS / Rust に同一の十進リテラルでハードコード**（算出式の丸め差を持ち込まない）。
- 連続値の演算は**両言語で同じ順序**（例: `u64/2^64 * 360 - 180`）に固定。

> ✅ **実装済み（reference/, TS）**: rng(`u32`/整数`pickWeighted`)・rarity(`RARITY_CUTOFFS_U32`)・
> appearance(`DAMAGE_CUTOFFS_U32`/整数`dirt`) を整数しきい値化。連続値(colorMod/preservation)は f64 のまま。
> `golden.json` 再生成済み・全テスト緑。`constants.test.ts` がしきい値リテラルを式と照合してロック。
> **この golden が Rust 実装の一致目標**になる。

## 8.5 FFI 境界（UniFFI）— 実装

`qrac-ffi/src/lib.rs` が公開する実際のAPI（UniFFI proc-macro）。画像は **PNGバイト列**で返す
（ネイティブ側で `NSImage`/`Bitmap` 化）。

```
// #[uniffi::export]
enum Lang { Ja, En }                                          // 解説文の言語（表示専用・決定論に無関係）

fn configure_assets(dir: String)                              // ベース画像のディレクトリ設定
fn derive_qr(text: String) -> Artifact                       // 属性（年は内容から自動抽出）
fn derive_qr_with_year(text: String, year: Int32?) -> Artifact
fn render_qr(text: String, lang: Lang) -> RenderedImage      // derive→DB選択→合成→PNG＋解説
fn render_qr_with_year(text: String, year: Int32?, lang: Lang) -> RenderedImage
fn describe_qr(text: String, lang: Lang) -> String           // 解説文のみ（言語切替時の再生成）

// #[derive(uniffi::Record)]
struct Artifact { artifactHash, baseRarity, eraBonus, finalRarity, isMythic,
                  civ, era, category, color{hShift,sMul,vMul}, dirtLayerId,
                  damage{chip,crack,wear}, preservationScore }
struct RenderedImage { width, height, png: bytes, baseName, matchedStage, description }
```

- `derive_qr*` は純粋コアのみ。`render_qr*` は DB選択(qrac-render) ＋ 合成 ＋ 解説文(`lang`)。
- `Lang` は解説文の言語のみを切り替える。属性・ハッシュ・決定論には影響しない（docs/00 0.4.1 と同じ非対称）。
- 表示名はSwiftが civ/era/category＋★ から多言語生成（`baseName` は参考値）。
- 収集の永続化（collect/list）はFFIに持たず **Swift側 GameModel**（collection.json）で実施。
- 当初案にあった `collect` / `list_collection` / `build_artifact` / `RgbaImage(rgba)` / `Style` は不採用。

## 8.6 画像合成（tiny-skia, CPU）

> ✅ **6層合成は実装済み**（`docs/05` 5.2 準拠）。透過PNGレイヤー＋名前付きブレンドモード＋被写体マスク。
> 残差分はロードマップ（解像度1024²／LRUキャッシュ／写真アート差し替え）。下記は実コード
> `qrac-render/src/compose.rs`＋`art.rs` の挙動。

実装（`render(attr, base, assets_dir: Option<&Path>)`、キャンバス **1024×1024**。`art.rs` は 512 デザイン空間で
記述し描画時に2倍拡大＝ベクタは高解像度ネイティブ描画）:
```
1. baseImage : assets_dir/base/<id>/<style>/0.png をデコード、無ければ art::base_sprite で手続き生成
2. HSV       : apply_hsv（per-pixel、層2。docs通り）
5. preserve  : apply_preservation（彩度/明度/コントラストのカラーグレード、層5。docs通り）
   chip      : DestinationOut で被写体アルファを削る（欠け）
   →ここで被写体シルエットの Mask を作成（以降の汚れ/破損を被写体内に限定）
6. background: assets_dir/bg/<style>.png（無ければ art::background）→ その上に被写体を合成
3. dirt      : assets_dir/layers/dirt/<id>.png（無ければ art::dirt_texture）を
               Multiply（sea_salt のみ Screen）＋マスク＋強度=dirtStrength、hash由来の回転で個体差
4. damage    : wear=SoftLight / crack=Multiply（いずれもマスク＋強度=damageStrength）
```
- レイヤーPNGは `qrac-assetgen` が事前生成（`assets/{base,layers/dirt,layers/damage,bg}`）。
- 出力は `render_png`（PNGバイト列）/ `render_rgba`。手続き生成（`art.rs`）は実行時フォールバック兼ねる。
- ロードマップ: 解像度1024²、LRUキャッシュ、写真アート（外部生成）への差し替え（README参照）。

## 8.7 DB（rusqlite）

- **遺物DB**（読み取り専用・同梱）: `docs/03` 3.2 スキーマ。`idx_base_lookup` で候補抽出。
- 候補選択は `docs/03` 3.4 の `ORDER BY id` ＋ `uintBelow(seed,"pick",len)`。**6段フォールバック**（`docs/03` 3.5 / `docs/00` 0.4.2）を `db.rs` に実装。
- **収集DB**（読み書き・端末ローカル）: `docs/06` 6.3 `collected`。`normalized_key` は BLOB 逐語、一覧は `final_rarity` 非正規化キャッシュで完結。
- 接続は1コネクションを Rust 側で保持。SQLite は単一プロセス前提でOK。

## 8.8 アセット同梱

```
assets/base/<image_set_id>/<style>/0.webp   # 背景透過の被写体（style=museum/dig/catalog, angle="0"固定）
assets/layers/dirt/<id>.webp                # 全遺物共有
assets/layers/damage/{wear,crack,chip}.webp # 共有
assets/data/color_regions.json              # 名前付き色の領域（docs/04 4.1）
artifacts.sqlite                            # 遺物DB（読み取り専用）
```

- 合計 1.2〜1.4GB（`docs/05` 5.4）。プラットフォーム別パッケージング:
  - iOS/macOS: アプリバンドル or On-Demand Resources（容量次第で分割DL検討）。
  - Android: `assets/` 同梱は150MB上限の都合上、**Play Asset Delivery (install-time)** で配信。
- WebPデコードは `image`/`libwebp`。AVIFは限定運用（必要箇所のみ）。

## 8.9 決定論テスト戦略（TSオラクル ⇔ Rust）

```
reference/(TS) ── gen:vectors ──▶ vectors/golden.json ◀── tests/golden.rs (Rust) が再現一致を検証
```

- `golden.json` が**言語非依存の正本**。TSとRustが同じ入力から同じ `DerivedAttributes` を出すことを担保。
- Rust 側 `tests/golden.rs`: JSON を読み、各 `input` で `derive` し `output` と比較（実装済み・緑）。
- 加えて Rust 側でも分布・独立性・正規化のプロパティテストを持つ（`tests/units.rs`, `reference/test/*` と対）。
- **golden が変わる変更＝破壊的変更**。意図的なら `gen:vectors` 再生成＋`GEN_VERSION` 更新（`docs/06` 6.4）。
- ⚠️ **テスト依存の注意**: Rust の golden 比較では `serde_json` の **`float_roundtrip` フィーチャ必須**。
  無効だとデフォルトのfloatパーサが1 ULP誤差を出し、正しいコアでも偽陰性になる（実装中に遭遇・対処済み）。
  これはテスト専用の注意であり、コア本体の決定論とは無関係。

## 8.10 版ピンと genVersion 結合

決定論に影響する依存は**版を固定し、その版自体を `genVersion` の一部**とみなす:

- `sha2`（SHA-256実装）、`unicode-normalization`（NFCタグ。タグはASCIIなので実質no-opだが規約として固定）。
- URL正規化は自前なので外部版に依存しない（手実装の利点）。
- これら or `constants.rs`（重み・しきい値・タグ）を変えたら `GEN_VERSION++`。旧遺物は近似再現＋注記（`docs/06` 6.4）。

## 8.11 移植ロードマップ & 残タスク

**フェーズ順（macOS → iOS → Android）**
1. ✅ **qrac-core 純粋部** を Rust 実装（normalize/rng/rarity/timestamp/appearance/derive）＋ `tests/golden.rs` 緑化。
   → `rust/qrac-core/` に実装済み。golden 10ベクタを **byte一致で再現**、units 9件緑。TS↔Rust 決定論パリティ確認済み。
2. ✅ UniFFI で macOS(Swift) に橋渡し、最小動作（QR文字列→derive→遺物表示）。
   → `rust/qrac-ffi/`（UniFFI境界）＋ `demo/`（Swift CLI ＋ SwiftUI ContentView）。
   Swift 経由でも hash が golden と一致＝FFI越しでも決定論を確認。`bash qrac-ffi/demo/run.sh` で再現。
   ✅ SwiftUI を Xcode アプリ化＋xcframework 埋め込み（`apple/`）。`bash apple/build-app.sh` で
   `QrArtifactChronicle.app` を生成（macOS arm64）。残: iOS/Androidスライス・署名配布。
3. ✅ `db.rs`（候補選択・6段フォールバック / `qrac-render`）＋ `compose.rs`（tiny-skia 6層・PNGレイヤー＋ブレンド＋マスク）。
   FFI に `render_qr` を追加し、Swift で実際の遺物画像(PNG)を表示。残: 写真アート差し替え（下記）。
   ※ db/compose は純粋コアを汚さないよう別クレート `qrac-render` に分離（docs/08 8.3 を更新）。
4. 🟡 iOS（足場実装済み）: cargo の `aarch64-apple-ios`／`aarch64-apple-ios-sim` ビルド＋
   **マルチプラットフォーム XCFramework**（`apple/build-xcframework.sh`、macOS+iOS device+iOS sim の3スライス）。
   Swift層は `Platform.swift`（`PlatformImage`＝NSImage/UIImage、画像縮小・プレビュー View・カメラ探索を
   `#if os(...)` で吸収）により macOS と共有。**残（手元作業）**: Xcode の iOS アプリターゲット作成
   （SwiftPM 実行ファイルは iOS の .app を生成できないため）・`NSCameraUsageDescription` 設定・実機署名/実行。
   詳細は §8.13。
5. Android: cargo の Android ターゲット＋UniFFI(Kotlin)、Compose UI、Play Asset Delivery。

**着手前に片付ける確定タスク**
- [x] `reference/` の離散判定を**整数しきい値**へ更新（rng/rarity/appearance）＋ `golden.json` 再生成（8.4）。
- [x] `RARITY_CUTOFFS_U32` / `DAMAGE_CUTOFFS_U32` を確定し、TS/Rust 同一リテラルで共有（`constants.test.ts` でロック）。
- [x] `extractYear` の対象形式コーパスを整備（誤検出/取りこぼしの回帰テスト, `docs/07` Q1表）。
      共有コーパス `reference/vectors/year_corpus.json`（36ケース）を TS(`rarity.test.ts`)/Rust(`tests/year_corpus.rs`)
      の双方が検証し、検出のTS≡Rustパリティを固定。
- [x] アセット生成ツール（`docs/05` 5.6）の「全rarityマス充足」CIゲート実装 → `rust/qrac-assetgen`。
      `dist/artifacts.sqlite`（1500行＋GLOBAL）＋`dist/assets/base/<id>/<style>/0.png`（450枚）を生成し、
      150 combo の ★1..10 充足を検証（欠け→exit 1）。
- [x] UniFFI の API（8.5）: `derive_qr` / `derive_qr_with_year` / `render_qr`。
- [x] Rust `qrac-core` 純粋部の実装＋ `tests/golden.rs` で `golden.json` 再現一致（`rust/qrac-core/`）。
- [x] UniFFI 橋渡し → macOS(SwiftUI)アプリ（`apple/`, `QrArtifactChronicle.app`）→ `db.rs`/`compose.rs`（`rust/qrac-render`）。
- [x] **実行時アセット統合＋本格6層合成**: `compose::render(attr, base, assets_dir)` が
      `assets_dir` 配下のベース／汚れ／破損／背景の透過PNGレイヤーを読み込み、tiny-skia の
      ブレンドモード（Multiply/Screen/SoftLight/DestinationOut）＋被写体マスクで層1〜6を固定順合成。
      HSV(層2)・apply_preservation(層5)・hash由来の汚れ回転も実装。レイヤー欠落時は `art.rs` の
      手続き生成にフォールバック。FFI `configure_assets` で assets dir 設定、アプリは同梱
      `Resources/assets` を使用。レイヤーは `qrac-assetgen` が事前生成。
- [🟡] iOS スライス（足場）: Rust の iOS device/sim ビルド＋マルチプラットフォーム XCFramework
      （`apple/build-xcframework.sh`）、Swift の AppKit→UIKit 抽象化（`Platform.swift`／`#if os(...)`）まで実装。
      残: Xcode の iOS アプリターゲット作成・署名/公証（手元作業, §8.13）。
- [ ] Android スライス（cargo-ndk / UniFFI Kotlin / Compose UI）と Play 配信。
- [ ] ベース画像をアーティスト製/写真風 WebP に差し替え（レイアウト・命名はそのまま）。
- [ ] **UniFFI(MPL-2.0) → 手書き C ABI FFI 置換**（依存ツリーを完全に寛容化。商用クローズド時のみ必須）。
- [x] `compose` の本格6層化（透過PNGレイヤー＋ブレンドモード＋マスク）— `compose.rs`/`art.rs`/`qrac-assetgen`。
- [x] 合成のLRUキャッシュ・解像度1024²化。compose は 1024² 出力（`art.rs` は 512 デザイン空間を2倍拡大、
      テクスチャ密度は面積比で補正）。アプリは詳細フル画像をメモリLRU（上限16枚≈64MB）でキャッシュ。
      ※ 端末ローカルの合成結果ディスクLRU（cache/composed, docs/05 5.4）は将来課題として残置。
- [x] 収集の軽量化（docs/06 6.3）。一覧用サムネ（最大256px）のみ保存し、詳細のフル解像度(1024²)は
      保存中の QR text から `renderQr` で都度再生成＋メモリLRUキャッシュ。画像は年に非依存なので text だけで
      決定論的に同一画像を復元できる。`collection.json` のフィールドは不変（png にサムネを入れる）＝移行不要。
      ※完全な「キーのみ保存（属性キャッシュも再生成）」は将来の更なる軽量化として残置。

## 8.12 macOS アプリの実装機能（`apple/QracKit`）

プロトタイプのSwiftUIアプリ（縦長スマホUIをmacOS上で表示）。決定論は全てRust側、Swiftは入力・表示・収集のみ。

- **画面遷移**（`ContentView` の `Page`）: メイン / 展示室 / 詳細 / カメラ / 設定。phoneフレーム内でスライド遷移。
- **多言語（i18n）**（`Localization.swift` / `SettingsView.swift`）: 英語/日本語/システム追従を**設定⚙️**で即時切替・
  永続化（`Settings: ObservableObject`、`@AppStorage`）。UI文言・カテゴリ等の語彙・遺物表示名（`artifactName`）・
  **解説文（`describe_qr(lang)` で再生成）**まで全て言語連動。明色テーマ固定（`preferredColorScheme(.light)`）。
- **カメラQRスキャナ**（`Scanner.swift` / `CameraScanView.swift`）: **Vision `VNDetectBarcodesRequest`** で
  ライブ検出（撮影不要・リンク自動表示なし）。複数カメラの一覧/切替（内蔵/iPhone連係/外付け）、接続/切断の
  自動更新。`AVCaptureMetadataOutput` はiPhone連係で配信されない事例があり Vision方式を採用。**リリースの主入力**。
- **発掘演出**（`DigModalView.swift`）: ファミコン風パラパラ（Canvasピクセル描画）。新規=女の子の採掘→「新発見！」、
  既出=師匠が叱る→「もちだしちゃった…」。`GameModel.DigPhase{idle,digging,scolding}`。
- **効果音**（`SoundPlayer.swift` ＋ `apple/sounds/*.wav`、オリジナル合成）: dig/reveal/scold/error の4種。
- **展示室**（`ExhibitionView.swift`）: 発掘済み一覧。カテゴリ・レア度・文字検索。`GameModel.collected` を
  `collection.json` に自動保存（再起動で復元）。デバッグのみリセット可。
- **遺物詳細**（`ArtifactDetailView.swift`）: メイン類似だが操作系なし（戻るのみ）。
- **ゲームバランス確認**（`BalanceView.swift`）: 固定シードで大量サンプリング→分布を可視化（デバッグ）。
- **デバッグ専用UI**（`#if DEBUG`）: 手入力/カメラ切替・プリセット・ランダム・年代補正・分布・展示室リセット。
  リリースでは除外（手入力欄は出ず、カメラ発掘のみ）。文字列スキャンで除外を検証済み。
- **ビルド構成**: `bash build-app.sh`＝debug（`QrArtifactChronicle.app`、デバッグUIあり）。
  `APP_CONFIG=release bash build-app.sh`＝release（`QrArtifactChronicleRelease.app`、`#if DEBUG`除外・別バンドルID）。

## 8.13 iOS スライス（足場・現況）

iOS 版に向けた共有基盤まで実装済み。**実機で動く .app の生成は Xcode 側の手作業が残る**（SwiftPM の
実行可能ターゲットは iOS の .app バンドルを生成できないため）。

**実装済み（自動・検証可能）**
- **Rust コアの iOS クロスビルド**: `aarch64-apple-ios`（実機）/ `aarch64-apple-ios-sim`（Apple Silicon シミュレータ）。
  bundled SQLite(C) も iOS SDK 向けにコンパイルされることを確認済み。
- **マルチプラットフォーム XCFramework**: `apple/build-xcframework.sh` が macOS+iOS device+iOS sim の
  **3スライス**で `QracFFI.xcframework` を生成（`xcodebuild -create-xcframework`）。
- **Swift の移植性**: `Platform.swift` に `PlatformImage`（NSImage/UIImage）・`Image(platformImage:)`・
  画像縮小（`PlatformGfx.downscalePNG`）を集約。カメラのプレビュー View（`NSViewRepresentable`↔
  `UIViewRepresentable`）、カメラ探索（macOS=連係/外付け、iOS=背面）、ウィンドウ系 Scene 修飾子は
  `#if os(...)` で分岐。`Package.swift` は `.iOS(.v16)` を宣言。**macOS ビルドは回帰なし**（`swift build` 緑）。

> ⚠️ **ツールチェーン注意**: iOS の std は **rustup** 管理 toolchain にある。PATH 先頭の `cargo` が Homebrew 版
> だと iOS std を持たず "can't find crate for core/std" になる。`build-xcframework.sh` は iOS ビルドだけ
> `rustup which --toolchain stable cargo` の絶対パスを使って回避している。

**残（手元作業）**
1. Xcode で iOS アプリターゲットを作成し、`QracKit` の Swift ソース群と `QracFFI.xcframework` を取り込む
   （または iOS 用の `.xcodeproj` を追加）。
2. `Info.plist` に `NSCameraUsageDescription` を設定（macOS の `build-app.sh` が出すものと同文でよい）。
3. Apple Developer 署名でシミュレータ／実機実行・配布（カメラは実機必須）。
4. iOS 分岐コード（UIKit プレビュー・`UIGraphicsImageRenderer` 縮小・背面カメラ探索）は標準APIで記述済みだが、
   **iOS 実機/シミュレータでのコンパイル・動作確認は未実施**（上記ターゲット作成後に確認する）。
