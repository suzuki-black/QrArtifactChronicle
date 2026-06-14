# QrArtifactChronicle — 決定論コア 参照実装

`docs/` の確定仕様（特に [00]・[01]・[02]）に対応する**純粋モジュール**の参照実装と決定論テスト。
最終的なモバイル実装（RN / Flutter / ネイティブ）の言語に依存しない**正本**として使う。

- 依存ゼロ（Node 23.6+ の TypeScript ネイティブ実行を利用）。ビルド不要。
- `vectors/golden.json` が**言語非依存のゴールデンベクタ**。移植先はこれを再現できなければならない。

## 実行

```bash
cd reference
npm test            # node --test（全テスト）
npm run gen:vectors # ロジック変更時のみ：ゴールデン再生成→差分レビュー
```

## 収録範囲（純粋＝決定論テスト可能な部分のみ）

| モジュール | 対応docs | 内容 |
|-----------|---------|------|
| `src/hash.ts` | 01 1.2/1.3 | SHA-256・`substream`/`uniform`/`uintBelow`/`pickWeighted`（big-endian, NFCタグ） |
| `src/normalize.ts` | 01 1.1 | `normalizeKey`・URL正規化・テキスト/バイナリ判定・U+FFFD置換 |
| `src/timestamp.ts` | 02 2.2 / 07 Q1 | `extractYear`（vCard/iCal/UNIX/ISO、推測しない） |
| `src/rarity.ts` | 02 | 基本レア度・時代補正・clamp・神話級判定 |
| `src/appearance.ts` | 04 | 色違いHSV・汚れ・破損・保存状態・レイヤー強度 |
| `src/derive.ts` | 00 0.2 (A)〜(F) | 上記を束ねた純粋オーケストレータ `deriveAttributes` |

## テストが保証する不変条件

- **決定論**: 同じ入力 → 同一出力（`determinism.test.ts`、golden回帰）。
- **独立性**: `baseRarity` と色は無相関（`independence.test.ts`、Pearson≈0）。
- **分布**: 30万サンプルで優しい分布に収束（`distribution.test.ts`、★1実効40.49%）。
- **境界**: 時代補正の年代境界・clamp・神話級は eraBonus≥1 必須（`rarity.test.ts`）。
- **正規化**: 表記揺れは吸収・有意差は保持・バイナリ無加工（`normalize.test.ts`）。

## 範囲外（I/O / 別フェーズ — 実装設計で扱う）

純粋でないため本スキャフォールドには含めない。実装設計ドキュメントが必要:

- **ベース遺物DB選択(G)**: SQLite問い合わせ・6段フォールバック（docs/03）。
- **画像合成(H)**: 6層レイヤー・ブレンドモード・GPU/Canvas（docs/05）。
- **個体差テキスト本文(I)**: テンプレート・音節ジェネレータの語彙（docs/04 4.5）。
- **図鑑永続化**: `collected` テーブル・再生成（docs/06）。

## 決定論の要点（移植時の前提）

1. **整数しきい値（実装済み）**: 離散判定（レア度・破損・`pickWeighted`）は `u32` 生整数を
   `RARITY_CUTOFFS_U32`/`DAMAGE_CUTOFFS_U32` と直接比較し、**float を経由しない**（docs/08 8.4）。
   連続値（colorMod/preservationScore）のみ f64。しきい値定数は `constants.test.ts` でロック。
   → Rust 実装も同じ整数リテラルを直書きすること。
2. **URL正規化は手実装**（`src/normalize.ts`）。汎用URLライブラリ非依存・%エンコード不可触。
   Rust も同じバイト精度仕様（docs/01 1.1）を移植する。
3. **`extractYear` の誤検出/取りこぼし**: 現状は保守的ヒューリスティック。対象形式の拡充と
   テストコーパスの整備が残タスク（docs/08 8.11）。
4. **SHA-256実装**: 参照は `node:crypto`。端末側は各プラットフォーム実装だが、golden で byte一致を担保。
