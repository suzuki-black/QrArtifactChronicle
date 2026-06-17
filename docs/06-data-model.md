# 06. 生成結果・図鑑データモデル

## 6.1 生成結果オブジェクト（Artifact）

生成パイプライン（[00] 0.2）の最終出力。**実行時に組み立てる派生物**で、永続化は最小限（6.3）。

```ts
type Artifact = {
  // --- 同一性 ---
  artifactHash: string        // = hex( SHA-256(seed)[0:16] ) … 先頭16byte=32 hex chars 固定。
                              //   seed = SHA-256(normalizedKey)（[01]1.2）を再ハッシュした上位16byte。
                              //   図鑑の主キー兼キャッシュキー。
  genVersion: number          // 生成ロジック/DB/テンプレの版（6.4）

  // --- レア度 ---
  baseRarity: number          // 1..10（DB選択に使用）
  eraBonus: number            // 0..5
  finalRarity: number         // 1..13（表示・価値）
  isMythic: boolean           // finalRarity >= 11（★11〜13）

  // --- ベース遺物 ---
  baseId: number              // base_artifact.id（フォールバック後の確定ID）
  fellBackFrom?: string       // フォールバック発火時の段階（デバッグ/分析用, [03]3.5）
  civ: string; era: string; category: string
  name: string                // ベース名（テキストで個体差を付ける場合あり）
  baseDescription: string

  // --- 見た目（[04]/[05]） ---
  colorMod: { hShift:number; sMul:number; vMul:number }
  colorName: string           // 表示用の色ラベル
  dirtLayerId: string
  damage: { chip:boolean; crack:boolean; wear:boolean }
  preservationScore: number   // 0..1

  // --- 個体差テキスト ---
  // ⚠️ 実装（現況）: 下記スロット構造ではなく、民明書房調の単一段落を生成（[04]4.5 実装メモ）。
  //    FFI(RenderedImage)では description: String 一本で返す。言語は生成パラメータ（lang: Ja/En）。
  //    描画(画像)は言語非依存。description のみ言語別に生成される（describe_qr(text,lang)）。
  description: string          // 「詳説 世界の遺物（萬象書房 1890年刊）」抜粋体裁の解説文（選択言語）

  // --- 画像（実体ではなく参照, [05]） ---
  // angle は常に 0（アングルは1枚に確定 / [05]5.4）。将来の擬似3D拡張のため枠だけ残す。
  images: { style: "museum"|"dig"|"catalog"; angle: 0; src: string }[]
}
```

## 6.2 QR日時の扱い

`eraBonus` は QR由来の `year` から（[02] 2.2）。`year` の取得可否・フォールバックは [07] 必須論点。
`Artifact` には算出済み `eraBonus` のみ持たせ、`year` 抽出ロジックは `qr/timestamp` に閉じる。

## 6.3 図鑑（コレクション）の永続化 — 軽量主義

> ⚠️ **実装メモ（現況）**: macOSプロトタイプは下記のSQLite軽量モデルを**まだ採用していない**。
> 実際は Swift 側 `GameModel.Collected` を **`collection.json`（Application Support）に保存**する。
> 保存するのは **QR文字列(text) ＋ 一覧用サムネ(最大256px PNG) ＋ 属性**（civ/era/category/各レア度/保存度/汚れ/破損/段）。
> **フル解像度画像(512²)は保存せず**、詳細表示時に保存中の text から `render_qr` で**都度再生成**＋メモリキャッシュ
> する（画像は年に非依存なので text だけで決定論的に同一画像を復元できる）。**解説文(description)も保存せず**、
> `describe_qr(text, lang)` で**言語別に都度再生成**（表示名も civ/era/category＋★ からSwiftが多言語生成）。
> お気に入りは別ファイル `favorites.json`（artifactHash集合）。フル画像を保存しない方向に一歩進めた形で、
> 「キーのみ保存し属性キャッシュも再生成する完全版」への移行は更なるロードマップ（docs/08 8.11）。

**（設計目標）画像も `Artifact` 全体も保存しない。** 再生成可能なので、保存するのは復元に必要な最小キーだけ。

```sql
CREATE TABLE collected (
  artifact_hash TEXT PRIMARY KEY,    -- hex(SHA-256(seed)[0:16])（6.1 と同一）
  normalized_key BLOB NOT NULL,      -- normalizeKey([01]1.1) の出力【生バイト列】をそのまま BLOB 保存。
                                     --   テキスト化/再エンコードせず保存（再生成の唯一の入力）。
  gen_version   INTEGER NOT NULL,    -- 収集時の版
  qr_year       INTEGER,             -- 抽出できた場合のみ（無ければNULL→補正再計算の根拠）
  first_seen_at INTEGER NOT NULL,    -- 収集日時（実時刻。決定論には不使用）
  final_rarity  INTEGER NOT NULL,    -- 【非正規化キャッシュ】収集時の finalRarity を複製保持
  fav           INTEGER DEFAULT 0
);
```

- `normalized_key` は **`normalizeKey` の生バイト列をそのまま BLOB 保存**する（[01]1.1）。
  バイナリQRでも欠落なく往復でき、`SHA-256(normalized_key)` で seed を、さらに `artifact_hash` を再導出できる。
- `final_rarity` は **非正規化キャッシュ**。**図鑑の一覧表示（ソート・フィルタ・星表示）はこの列だけで行い**、
  各遺物を再生成しない（数千〜数万件でも高速）。正本は再生成結果だが、一覧では複製値で十分。
- 詳細を開いたときのみ `normalized_key`＋`gen_version` から `Artifact` を再生成し、画像を合成（[05]キャッシュ）。
- これにより図鑑数千〜数万件でも保存サイズは数MB級に収まる。

## 6.4 バージョニング（genVersion）— 決定論の生命線

ロジック・DB・テンプレートを変更すると同じQRから別遺物が出うる → 「同じQRは同じ遺物」要件違反。

ルール（確定 / Q7）:
- **破壊的変更（タグ名・分布・DB入替・合成順）→ `genVersion` をインクリメント。**
- **旧遺物は完全再現しない。** 旧DB・旧テンプレートは**端末に保持しない**（容量を圧迫しないため）。
- 既収集遺物は**現行ロジック（最新版）で近似再現**する。
- 収集時の値で意味があるもの（`final_rarity` 等）は `collected` に保存済み（6.3）なのでそのまま表示。
- 生成版が現行と異なる遺物には、UIに **「旧バージョン生成」注記**を表示する。

### この方針のトレードオフ（確定済み・記録のため）
- ○ 旧版アセットを抱えないので容量・実装がシンプル。バージョン分岐が `cache` 無効化だけで済む。
- △ ロジック更新で既収集遺物の見た目/一部属性が変わりうる（＝厳密な「同じQR=同じ遺物」は
  **同一`genVersion`内でのみ**保証）。注記でユーザーに明示することで世界観的にも許容する
  （「研究の進展で復元像が更新された」という解釈が成立する）。
- 実装: `collected.gen_version != CURRENT_GEN_VERSION` の遺物は詳細表示時に注記バッジを付与。

## 6.5 不変条件（テストで担保）
- `regenerate(normalized_key, gen_version)` は何度呼んでも同じ `artifact_hash` を返す。
- `finalRarity == clamp(baseRarity + eraBonus, 1, 13)`。
- `isMythic == (finalRarity >= 11)`。
- `colorMod` と `baseRarity` は無相関（[01] 1.5 独立性テスト）。
