# セッション引き継ぎメモ（QrArtifactChronicle）

> 次セッションの最初に読むための作業状態スナップショット。仕様の正本ではない（正本は docs/00〜08）。
> 最終更新: このメモ作成時点。リポジトリ: `git@github.com:suzuki-black/QrArtifactChronicle.git`（branch `main`）。

## 0. プロジェクト要旨
QRコードを読むと**決定論的に**架空文明の「遺物」を発掘し、民明書房オマージュ（出版社「萬象書房」）の
胡散臭い解説文がつくスマホ向け収集ゲーム。同じQR→必ず同じ遺物。完全オフライン。
スタック: Rust 純粋コア(qrac-core) / DB+合成(qrac-render) / UniFFI(qrac-ffi) / SwiftUI(apple)。
TypeScript 参照実装をオラクルに `TS ≡ golden.json ≡ Rust ≡ Swift` のパリティを維持。バージョン **0.2.0（プロトタイプ）**。

## 1. このセッションで完了し push 済み（main）
直近コミット（新しい順）:
- `4b803a4` docs(readme): 実機キャプチャの Screenshots セクション（英日）追加
- `f0a7de8` feat(ios): iOS ビルド足場（Rustクロスコンパイル＋マルチプラットフォームXCFramework＋Swift移植性）
- `d400d8b` feat(render): 1024²合成＋再生成画像のLRUキャッシュ
- `62a1384` perf(app): 収集の軽量化（サムネ保存＋詳細はフル再生成）
- `be3b209` test(core): extractYear 共有回帰コーパス（TS≡Rust）
- `9e939ec` feat(render): 本格6層合成エンジン＋手続きアート強化
- `8ac23b4` feat(app): お気に入り＋展示室フィルタ

機能面の到達点:
- **画像**: 6層合成（陰影ベース＋汚れ/摩耗/ひび/欠け＋保存グレード＋背景）、**1024²**、手続きアートは
  512デザイン空間を2倍描画（ベクタ高精細）。アプリ詳細はLRU（16枚≈64MB）。
- **収集**: サムネ(≤256px)＋text＋属性のみ保存、フル解像度は text から決定論再生成（画像は年に非依存）。
  `collection.json` 後方互換（移行不要）。お気に入りは `favorites.json` 別管理。
- **i18n**: 英日（解説文も）。設定画面で切替。
- **GitHub**: About 紹介文＋トピック13個設定済み。README は体験重視＋ヒーロー3枚＋スクショ群。

## 2. 未コミットで残っているもの（2026-06-21 セッションで「出土の系譜」を実装）
- **出土の系譜 実装完了（未コミット）**。Q1〜Q8 を全推奨で確定し §9 計画 1〜7 を実装:
  - `rust/qrac-core/src/genealogy.rs`（新）: `RefKind`/`RefMeta`/`type_label`/`describe_pair`(canonical)/image_set コーデック。
  - `rust/qrac-core/src/flavor.rs`: `describe(seed,a,refs,lang)` に拡張（参照文1文付加）。
  - `rust/qrac-render/src/db.rs`: `artifact_reference` テーブル＋`seed_references()`（型付き規則・安定ハッシュ採否・双方向行）＋`references_from()`＋`verify_reference_reachability()`。
  - `rust/qrac-assetgen/src/main.rs`: 参照生成＋到達性CIゲート（NGで exit 1）。**参照 102 行・全到達可能**。
  - `rust/qrac-ffi/src/lib.rs`: `image_set_id` 露出（Artifact/RenderedImage）＋refs解決＋`type_label`/`type_label_for_set`/`references_of`/`describe_pair` を export。
  - `rust/qrac-core/src/constants.rs`: **GEN_VERSION 1→2**。
  - `apple/`: `Collected.imageSetId`（Optional＋読込時バックフィル）、詳細画面「出土の系譜」セクション（ロック/タップ遷移）、両型所持で合本解説。FFI バインディング(`qrac_ffi.swift`)再生成・xcframework 再作成済み。
  - **Q7=(i) を厳密実装**: 型呼称の語彙を qrac-core 単一の出所に集約。`civ_label`/`era_label`/`category_label` を FFI export し、Swift の `civName`/`eraName`/`categoryName` がそれを呼ぶ（表示文字列は従来と同一・drift 構造的に不能）。参照名＝詳細チップの一致を保証。
  - テスト: `genealogy_snapshot.rs`（新, ゴールデン）＋ db 参照テスト。**`cargo test --workspace` 緑／golden(TS≡Rust)・year_corpus 緑／macOS アプリ ビルド緑**。
- `docs/proposals/01-genealogy-references.md`（実装完了ステータスに更新）。`docs/SESSION-HANDOFF.md`（本ファイル）。
- **残**: 系譜ビュー(所持型の参照網グラフ, §9-8) は後続（任意）。
- 注: `git status` に `rust/.../examples/*` 等は既にコミット済み。

## 3. 次の候補タスク（ロードマップ）
私（Claude）だけで完結できるもの / 外的要因が要るもの を区別:
1. **出土の系譜（継続率メカ#1）** … 設計指示書 `docs/proposals/01-genealogy-references.md` が確定版。
   実装着手前に Q1〜Q8 を決めるだけ。**私だけで実装可能・最有力の次タスク**。
2. **写真風アート差し替え** … 外部素材が必要（私は環境上、写真生成不可）。OSS/MIT段階は手続きアート維持。
   商用安全 or CC0 素材の調達次第。CC0候補: The Met / Smithsonian / Cleveland / Art Institute / Rijksmuseum。
   ※ みつあ(Mitsua Likes)は**非営利ライセンス**でOSS同梱に不向き、と結論済み。
3. **iOS アプリ実機化** … Rust側＋XCFramework は完了。残: Xcodeで**iOSアプリターゲット作成**＋署名（自分の
   iPhoneは**無料**のPersonal Teamで可、7日ごと再ビルド）。**前提: このMacに iOSプラットフォーム未インストール**
   → `xcodebuild -downloadPlatform iOS`（数GB）を実行しないと Swift の iOS コンパイル検証ができない。
4. **Android スライス** … Kotlin/Compose 全面書き直し＋cargo-ndk＋UniFFI(Kotlin)。大きめ。後回し推奨。
5. **UniFFI(MPL-2.0)→ C-FFI 置換** … クローズド商用時のみ必須。MIT/OSS のままなら不要。

## 4. 系譜メカ設計の確定差分（A〜E, 初稿レビュー結果）
- **A**: 参照キーは `base_artifact.id`（INSERT順で不安定）ではなく **`image_set_id`（型・安定）**。
- **B**: フレーバー文は **Rust専用**（TSオラクルに flavor 無し）→ テストは **Rust内部決定論＋ゴールデンスナップショット**
  （TS≡Rustパリティは課さない）。
- **C**: 参照が挙げる名前＝**ユーザーが画面で見る型呼称**に統一（`base.name` は使わない）。語彙の出所を一箇所に。
- **D**: FFI に **`image_set_id` を露出**（バックフィル・照合用）。
- **E**: 合本解説は **型ペアで canonical**（個体seed不使用・全員同一）。
- 未決: Q1参照本数 / Q2双方向性 / Q3 kind種別 / Q4複数充足単位 / Q5合本演出 / Q6参照先レア上限 /
  **Q7型呼称の実装(i)FFI共有 vs (ii)parityテスト** / **Q8 canonical確認**。

## 5. 環境・運用メモ（重要）
- **Rust toolchain 二重化**: PATH先頭は Homebrew rust（iOS std無し）。iOSクロスビルドは rustup を絶対パスで:
  `~/.rustup/toolchains/stable-x86_64-apple-darwin/bin/cargo`（`build-xcframework.sh` が自動処理）。
  iOSターゲット `aarch64-apple-ios` / `aarch64-apple-ios-sim` 導入済み。
- **ビルド**: `bash apple/build-app.sh`（debug＝デバッグUIあり）/ `APP_CONFIG=release bash apple/build-app.sh`。
  マルチプラットフォームXCFramEWork は `bash apple/build-xcframework.sh`。
- **テスト**: `cd rust && cargo test --workspace` / `cd reference && node --test`。
- **スクショ撮影（自動・はみ出しなし）**: `screencapture -o -x -l <windowid> out.png`。
  windowid は `swift /tmp/winid.swift`（CoreGraphics でアプリ窓を探す小スクリプト。要再作成可）で取得。
  UIクリックの自動化は macOS に弾かれる（System Events `click at` がタイムアウト）→ 操作はユーザー、撮影はClaude。
- **iOS検証の壁**: iOSシミュレータ ランタイム未インストール。`xcodebuild -downloadPlatform iOS` が必要。

## 6. 不変条件・厳守事項
- **決定論が核**: RNGタグ/しきい値/normalize規則/生成テキストを変えたら **GEN_VERSION bump**（docs/07 Q7）。
  qrac-core を変えたら golden.json と year_corpus の緑を必ず確認。
- **個人情報をリポジトリに入れない**: 開発機のローカルのメールアドレス / OSユーザー名は厳禁（本ファイルにも書かない）。
  git author は `suzuki-black <suzuki-black@users.noreply.github.com>`。
- **ライセンス**: MIT © 2026 suzuki-black。萬象書房は『魁!!男塾』民明書房ネタのオリジナル・パロディ（本文・社名はオリジナル）。
- **プッシュはユーザーが明示したときのみ**。コミットメッセージ末尾に Co-Authored-By を付与。
- **アートは手続き生成のスタイライズ（写真ではない）**と README で正直に明示し続ける。
