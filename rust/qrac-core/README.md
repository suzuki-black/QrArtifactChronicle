# qrac-core (Rust)

QrArtifactChronicle の決定論コア（Rust 本実装）。`docs/08` の共有コア。
`../../reference/`（TS オラクル）と関数単位で対応し、golden vectors で **byte一致**を検証する。

## 実行

```bash
cd rust/qrac-core
cargo test          # golden（TSベクタ再現）＋ units（境界/正規化/分布/独立性）
```

## 収録（純粋部のみ）

| ファイル | docs | reference(TS) 対応 |
|----------|------|--------------------|
| `hash.rs` | 01 | `hash.ts`（substream/uniform/u32/uint_below/pick_weighted） |
| `normalize.rs` | 01 | `normalize.ts`（手実装URL正規化・%エンコード不可触） |
| `timestamp.rs` | 02 | `timestamp.ts`（extract_year・正規表現不使用） |
| `rarity.rs` | 02 | `rarity.ts`（整数しきい値 `RARITY_CUTOFFS_U32`） |
| `appearance.rs` | 04 | `appearance.ts`（離散=整数 / 連続=f64） |
| `flavor.rs` | 04 | 民明書房調の解説文生成（TS未対応・Rust専用） |
| `derive.rs` | 00 | `derive.ts` |

（他に `constants.rs` / `types.rs` / `lib.rs`）

## 決定論の要点

- **big-endian 固定・タグ NFC・離散は整数しきい値**（docs/08 8.4）。連続値のみ f64。
- 定数（`RARITY_CUTOFFS_U32` 等）は TS と**同一リテラル**。`reference/test/constants.test.ts` が出典式と照合。
- URL正規化は手実装（汎用URLライブラリ非依存）。バイト精度で TS と一致。
- **依存版はピン留め**（`sha2`/`unicode-normalization`）し genVersion の一部とみなす（docs/08 8.10）。

## 範囲（他クレート）

`qrac-core` は純粋部のみ。I/Oは別クレートに実装済み:
- **DB選択・6段フォールバック** → `qrac-render/src/db.rs`（rusqlite）
- **画像合成** → `qrac-render/src/compose.rs`（tiny-skia, 現状は手続き描画の暫定）
- **UniFFI境界** → `qrac-ffi`（`derive_qr`/`render_qr` 等）
- **アセット生成** → `qrac-assetgen`

（収集データの永続化は macOSアプリ側の `collection.json`。Rustコアには持たない。）
