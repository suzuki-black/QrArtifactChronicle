# 提案01. 出土の系譜（解説文の相互参照）— 設計指示書（確定版・レビュー用）

> ✅ **ステータス: 実装完了（プロトタイプ, 2026-06-21）。** 判断A〜E＋Q1〜Q8（全推奨）で確定し、
> §9 段階計画 1〜7 を実装済み（系譜ビュー §9-8 は後続）。Rust 全テスト緑・到達性CIゲート緑・
> macOS アプリ ビルド緑。GEN_VERSION 1→2。正本 docs/00〜08 とは別系統の提案。
> 参照する実体: `flavor.rs:62 describe(seed,a,lang)`／`db.rs base_artifact(id,image_set_id,name)`／
> `ffi RenderedImage`／`Localization.swift artifactName/civName/...`／`constants.rs GEN_VERSION`。

## 0. 目的と前提

**目的**: 単体の民明書房テキストを「相互参照する偽史ネットワーク」へ拡張し、
(a) 解説文の再読性、(b) 特定の遺物“型”を狙って外を歩く動機を、コア体験の延長として生やす。

**守るべきコア不変条件**（docs/00）: 同一QR→同一遺物（決定論）／有限DBで無限感／完全オフライン。

**初稿からの確定差分（レビュー結果）**:
- **A**: 参照・所持キーは `base_artifact.id` ではなく **`image_set_id`（=型, 安定キー）** を使う。
- **B**: フレーバー文は **Rust専用**（TSオラクルに flavor は無い）。テストは **TS≡Rustパリティではなく
  Rust内部の決定論＋ゴールデンスナップショット**で固定する。
- **C**: 参照が挙げる名前は **ユーザーが画面で見る型呼称と一致**させる（`base.name` は使わない）。
  型呼称の語彙は **単一の出所**（qrac-core）に統一し、Swift と一致させる。
- **D**: FFI に **型キー `image_set_id`** を露出（バックフィル・参照照合に必須）。
- **E**: 合本解説は **型ペアで canonical**（全員同一）。個体seedは使わない。

## 1. 確定した設計判断

| # | 判断 | 確定内容 |
|---|------|---------|
| A | 参照の指す先 | **型 = `image_set_id`**（=(civ,era,category)、rarity跨ぎで1つ。db.rs:72-73 で安定と保証） |
| B | flavor とDBの境界 | flavor は純粋関数のまま。参照メタは**呼び出し側(qrac-render/ffi)が解決して引数で渡す** |
| C | 参照の表示名 | **型呼称**（civ/era/category由来）。**語彙の出所は qrac-core 一箇所**、Swift と一致 |
| E | 合本の同一性 | **型ペア (image_set_from, image_set_to) から canonical 生成**（順不同・全員同一） |

> 🔑 なぜ id でなく image_set_id か: `base_artifact.id` は INSERT 順依存（db.rs:50,136）。DBを
> 5,000〜10,000件へ拡張・再生成すると id がズレ、参照も保存済みキーも壊れる。`image_set_id` は
> `ci*100+ei*10+ki` で型から決まり安定。所持判定「いずれかの鐘でOK」とも一致（型=image_set）。

## 2. データモデル

### 2.1 参照グラフ（DB・assetgen が決定論生成）
```sql
CREATE TABLE artifact_reference (
  from_set  INTEGER NOT NULL,   -- 引用元の image_set_id（型）
  to_set    INTEGER NOT NULL,   -- 引用先の image_set_id（型）
  kind      TEXT NOT NULL,      -- 'pair' | 'cites' | 'rival'
  PRIMARY KEY (from_set, to_set, kind)
);
```
- **id ではなく image_set_id で張る**（判断A）。
- 制約: `from_set <> to_set`／重複禁止／`to_set` は §4 到達性条件を満たす型のみ。
- **表示名はキャッシュしない**: 型呼称は (civ,era,category) から実行時に §3.4 の共有語彙で生成
  （初稿の `to_name_ja/en` は廃止。`base.name` と画面表示名の不一致＝判断C を回避）。
- 生成規則（例, §3.1）: 同 civ で weapon↔ritual を pair、inscription→architecture を cites 等。
  採否は `hash("ref", from_set, to_set, kind)` の安定ハッシュ。

### 2.2 収集モデル拡張（後方互換・移行不要）
`GameModel.Collected`（docs/06 6.3）に **型キーを追加**:
- 新フィールド: **`imageSetId: Int`**（`base_id` ではない）。
- 欠落時バックフィル: 保存中 `text` → `deriveQr`＋FFIが返す `image_set_id`（§5/判断D）で補完。
- 所持判定: 「型T を所持か」= `collected.contains { $0.imageSetId == T }`（Setで O(1)化可）。

### 2.3 合本（結合解説）
- **保存しない**。型ペアから都度再生成（解説文非保存の現方針と整合）。
- 解禁条件: `from_set` と `to_set` の**両型を所持**したとき。

## 3. 生成ロジック

### 3.1 assetgen: 参照グラフ生成（`qrac-assetgen::references`）
1. `base_artifact` を image_set（型）に畳み、civ/era/category でグルーピング。
2. 型付き規則で候補ペアを作る（weapon↔ritual=pair 等）。
3. `h = stable_hash("ref", from_set, to_set, kind)` で採否（確率は調整可）。
4. 自己参照・重複・到達性NG（§4）を除外して `artifact_reference` に書く。
5. 最終工程で**到達性CIゲート**（§4）。欠けたら exit 1。

### 3.2 core: 参照メタの注入（判断B 維持）
- シグネチャ変更: `describe(seed, attr, refs, lang) -> String`
  - `refs: &[RefMeta]`、`RefMeta { kind, to_set, to_label }`（`to_label` は §3.4 で解決済みの型呼称文字列）。
- 仕様:
  - `refs` 空 → 従来どおり（参照文なし）。
  - 1件以上 → `seed + kind` で1件を決定論選択し、本文末尾に1文付加。
  - 文面例(JA): pair「これは〈to_label〉と対をなすと伝わる。」/ cites「銘文に〈to_label〉への言及がある。」
- **DBアクセスは flavor の外**（qrac-render/ffi が refs を組み立てて渡す）。flavor は純粋なまま。

### 3.3 合本解説（型ペア・canonical, 判断E）
- 新関数: `describe_pair(set_a, set_b, kind, lang) -> String`
- 入力は**型キー2つ**。順不同を保証するため内部で `(min,max)` に正規化し、そこから pair-seed を導出
  → **全ユーザーで同一テキスト**（個体seedは使わない）。
- 文体は kind で分岐。完全決定論。

### 3.4 型呼称の単一出所（判断C の要）
- 型呼称 `type_label(civ, era, category, lang)` を **qrac-core に1つだけ定義**（civ/era/category→表示語彙）。
  - 例(JA): 「大洋文明の祭祀遺物」/(EN): "ritual relic of the Ocean civilization"。
- qrac-render/ffi はこれで `RefMeta.to_label` を解決して describe に渡す。
- **Swift は同じ語彙で表示**する。実装は次のどちらか（§5/§8で固定）:
  - (i) FFI `type_label(...)` を Swift が呼ぶ（出所は Rust 1箇所）。**推奨**。
  - (ii) Swift の `civName/eraName/categoryName` を据え置き、**Rust語彙と一致する parity テスト**で縛る。
- いずれにせよ、**参照が挙げる名前＝参照先詳細画面が見せる civ/era/category 呼称**を保証する
  （詳細画面は既に civName/eraName/categoryName のチップを表示。ユーザーはこれで照合できる）。

### 3.5 決定論・版管理（GEN_VERSION）
- 参照網・参照差し込み・合本は出力テキストを変える → 導入時に **GEN_VERSION を bump**（docs/07 Q7）。
- 旧版収集物: 「研究の進展により解釈が更新された」体で**現行ロジックで再解釈**（docs/06 6.4）。
  参照キーは image_set_id なので旧収集物にも後付けで参照が張れる（id方式なら不可能だった）。

## 4. 到達性（findability）
- `to_set` は **image_set の出現確率が一定以上**の型に限定（レア型を参照先にしない）。
- 必要なら**複数充足**（「(civ,category) のいずれかの型でOK」）。初期は image_set 単位で実装し拡張余地を残す。
- **CIゲート**: ランダム normalized_key を多数生成→ base選択を通し、全 `to_set` が現実的試行回数で
  出現しうるか検証（docs/05 5.6 のカバレッジゲートと同方式）。NG で assetgen 失敗。

## 5. UI / UX（apple/QracKit）
- 詳細画面に **「関連遺物」セクション**:
  - 参照先の **型呼称**（§3.4・詳細チップと同語彙）。
  - 状態: 未発掘＝グレー＋ロック＋「発掘すると解禁」／発掘済み＝タップで該当遺物へ。
- 両型所持で **合本解説**を通常解説の下に表示（初回フェードイン等は任意）。
- 系譜ビュー（所持型の参照網グラフ）は**後続**。
- 判断D: 詳細/収集は型キーを使うため、FFI が `image_set_id` を返すこと（§下記）。

## 6. FFI 変更（判断D）
- `RenderedImage` に **`image_set_id: i64`** を追加（現状 base_name/matched_stage のみ, ffi:108-159）。
- `derive_qr`/`derive_qr_with_year` の戻りにも型キーを露出（バックフィル用）。
- 解説生成は `describe_qr(text, lang)` を、参照解決込みに拡張するか別口 `describe_qr_with_refs` を足す
  （refs はアプリの所持集合に依存しないため、FFI内でDBから全 `from_set` の参照を引いて渡す）。
- 新規: `type_label(civ, era, category, lang)`（§3.4 (i) 採用時）／`describe_pair(...)`。

## 7. マイグレーションと互換性
- DB(同梱・読取専用): `artifact_reference` を追加（assetgen が生成）。既存表は不変。
- 収集: `Collected.imageSetId` を追加。既存 collection.json は欠落時バックフィル（§2.2）。
- GEN_VERSION を bump（§3.5）。旧版収集物は現行ロジックで再解釈（docs/06 6.4）。

## 8. テスト戦略（B を反映）
- **決定論（Rust内部）**:
  - `describe(seed,attr,refs,lang)` が同入力→同テキスト。
  - `describe_pair(set_a,set_b,kind,lang)` が順不同で同テキスト（canonical）。
  - **ゴールデンスナップショット**（代表入力→期待テキスト）を Rust テストに固定。
  - ※ **TS≡Rustパリティは課さない**（TSオラクルに flavor 無し）。将来TSへ移植する判断は別途。
- **語彙一致（判断C）**: §3.4 (i)なら不要、(ii)なら **Rust `type_label` ≡ Swift `civName/...` の parity テスト**。
- **到達性**: assetgen CI のサンプリング検証（§4）。自己参照・重複なし。
- **後方互換**: 既存 collection.json 読込でクラッシュせず、`imageSetId` バックフィルが正しい。

## 9. 段階的実装計画
1. 判断A〜E を承認（本書）＋ §11 のQを確定。
2. `type_label`（qrac-core）＋ FFI 露出（§3.4, §6）。
3. `artifact_reference`（image_set_id基準）＋ assetgen 生成＋到達性CIゲート（§2.1, §4）。
4. `describe(…, refs, …)` 拡張＋ゴールデンスナップショット（§3.2, §8）。
5. `describe_pair`（canonical）＋ FFI（§3.3, §6）。
6. `Collected.imageSetId` 追加＋所持判定＋詳細UI（関連遺物/ロック/合本）（§2.2, §5）。
7. GEN_VERSION bump＋旧版再解釈（§3.5）。
8. 系譜ビュー（任意・後続）。

## 10. 受け入れ基準
- 同一QR→同一の参照・合本テキスト（Rust決定論テスト緑）。
- 参照が挙げる名前＝参照先詳細の表示呼称が一致（語彙テスト or FFI共有）。
- 全 `to_set` が到達可能（CIゲート緑）。自己参照・重複なし。
- 既存収集が読め、`imageSetId` バックフィル成功。GEN_VERSION 上昇で旧版表示が壊れない。

## 11. 未決事項 → 確定（2026-06-21, すべて推奨で決定）
- **Q1 = 初期 0〜1本・最大2本**（本文末尾に1文付加。seed+kind で1件を決定論選択）。
- **Q2 = pair:双方向／cites:片方向／rival:双方向**。双方向 kind は assetgen が両向き行を明示生成。
- **Q3 = 3種（pair / cites / rival）でスタート**。
- **Q4 = 初期 image_set 単位**。(civ,category) 拡張は余地のみ残す。
- **Q5 = 初期はテキストのみ**（演出は後続）。
- **Q6 = 到達性CIゲートの実データで確定**（今は「出現確率が一定以上の型のみ to_set」の制約だけ確定）。
- **Q7 = (i) FFI `type_label(...)` を Swift が呼ぶ**（語彙の出所を qrac-core 1箇所に一元化）。
- **Q8 = 型ペアで canonical（確定E）**。personal 案は不採用。
