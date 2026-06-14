# 03. ベース遺物DBと候補選択

## 3.1 ストレージ

- **SQLite**（端末同梱・読み取り専用）。メタデータ・テキストはここ。
- 画像実体はDBに入れず**ファイル/アセットとして別管理**（[05]）。DBには画像IDのみ。
- **想定レコード数: 5,000〜10,000（確定 / Q3）。** 当初の「数十万」は容量2GBと両立しないため見直し。
  無限感は色違い・レイヤー・個体差テキストで担保する（[04] 4.6 で総バリエーション 10^12 超）。

## 3.2 スキーマ

```sql
-- 軸の値は文字列直書きでなくIDで正規化（索引が小さく・表記揺れ防止）
CREATE TABLE civilization (id INTEGER PRIMARY KEY, key TEXT UNIQUE, name TEXT);
CREATE TABLE era          (id INTEGER PRIMARY KEY, key TEXT UNIQUE, name TEXT, sort INT);
  -- era.sort: 時代の【表示・並び順】専用（古代→中世→…→未来）。図鑑のソートやUI表示にのみ使う。
  -- ★RNGによる era 決定には一切影響しない（era選択は tag="era" の重みのみ。[03]3.3 / 0.4.4）。
CREATE TABLE category     (id INTEGER PRIMARY KEY, key TEXT UNIQUE, name TEXT);

CREATE TABLE base_artifact (
  id            INTEGER PRIMARY KEY,
  civ_id        INTEGER NOT NULL REFERENCES civilization(id),
  era_id        INTEGER NOT NULL REFERENCES era(id),         -- ★遺物時代(架空)。QR年代とは無関係
  category_id   INTEGER NOT NULL REFERENCES category(id),
  base_rarity   INTEGER NOT NULL CHECK (base_rarity BETWEEN 1 AND 10),
  name          TEXT NOT NULL,
  description   TEXT NOT NULL,                               -- 固定のベース説明文
  image_set_id  INTEGER NOT NULL                             -- 画像セットへの参照。
                                                             -- assets/base/<image_set_id>/<style>/0.webp を解決。
                                                             -- スタイル=museum/dig/catalog の3種、アングルは"0"固定([05]5.5)
);

-- 候補抽出キー。等値4軸 → 完全一致索引で O(log n)
CREATE INDEX idx_base_lookup
  ON base_artifact (civ_id, era_id, category_id, base_rarity, id);
```

`id` を索引末尾に含めるのは、候補の**順序を安定**させ決定論的選択を再現可能にするため。

## 3.3 属性の決定（civ / era / category）

仕様書フロー7は「文明×時代×カテゴリ×ベースレア度に合致する遺物を選択」とある。
**3軸の決定元は仕様書に明記がなかったが、ハッシュの異なるビット列から独立に決める方針で確定（Q4）。**

> ✅ **確定（Q4）: civ / era / category は、それぞれ独立したハッシュストリームから決定。**
> 決まった3軸＋`baseRarity` の4キーでDBから候補を引き、1件を選ぶ（3.4）。

```
civ      = pickWeighted(seed, "civ",      CIV_WEIGHTS)      # ストリーム"civ"
era      = pickWeighted(seed, "era",      ERA_WEIGHTS)      # ストリーム"era"  遺物時代(架空)
category = pickWeighted(seed, "category", CATEGORY_WEIGHTS) # ストリーム"category"
```

- 各軸は[01]の別タグなので互いに無相関。重み `*_WEIGHTS` はバランス用定数（均等でも可）。
- `era` はここで決まる**架空の遺物時代**。[02]のQR年代（時代補正）とは別物（README参照）。
- 4キーは「決定論的に決まった属性」→「DBから合致レコードを引く」という一方向。
  合致0件時はフォールバック（3.5）。

### `pickWeighted` の仕様（決定論・挙動を固定）

```
pickWeighted(seed, tag, weights):   # weights = [(value, w), ...] w>=0 の整数または実数
  # 1. 候補は value の昇順で【順序固定】に並べる（定義順依存を排除＝決定論）
  items = sortByValueAsc(weights)
  # 2. 累積分布を作る
  total = Σ w           ;  assert total > 0
  r = uniform(seed, tag) * total          # [0, total)
  acc = 0
  for (value, w) in items:
    acc += w
    if r < acc: return value
  return items.last.value                 # 浮動小数誤差の保険
```

- **重み 0 の候補は絶対に選ばれない**（累積が増えないため `r < acc` が成立しない）。
  存在させたくない軸値は重み0で無効化できる（[03]3.6 の噛み合わせ除外と同じ仕組み）。
- **累積分布方式**: 区間 `[acc, acc+w)` に `r` が落ちた候補を選ぶ。
- **順序固定**: 候補は value 昇順に並べてから累積するので、定義順や辞書実装に依存しない。
- 重みの総和が0（全候補が重み0）は不正設定（assert）。最低1つは正の重みが必要。

## 3.4 候補選択（決定論）

```
selectBase(seed, civ, era, category, baseRarity):
  candidates = SELECT id FROM base_artifact
               WHERE civ_id=? AND era_id=? AND category_id=? AND base_rarity=?
               ORDER BY id                       -- 安定順序（必須）
  if candidates is not empty:
    idx = uintBelow(seed, "pick", len(candidates))
    return candidates[idx]
  else:
    return fallback(seed, civ, era, category, baseRarity)   # 3.5
```

`ORDER BY id` を必ず付ける。DBの物理順序に依存すると端末/版で順序が変わり決定論が壊れる。

## 3.5 候補0件フォールバック（重要）

4軸×レア度の組み合わせ（例: 文明5 × 時代5 × カテゴリ6 × レア度10 = 1,500通り）に対し
全マスを埋めるのは現実的でない。**必ず候補0件のマスが出る**。仕様書はここを未定義。

フォールバックは「軸を段階的に緩める6段の梯子」を決め打ち順で適用（決定論維持）。
6段すべて空なら**終端の GLOBAL_FALLBACK_ID** を返す（[00] 0.4.2 と一致）。

```
6段フォールバック（上から順に試し、最初に非空の集合を採用）:
  段1. (civ, era, category, baseRarity)   ← 本命(3.4で空だった)
  段2. (civ, era, category, *        )   ← rarity緩和、最近傍rarityを優先
  段3. (civ, era, *,        baseRarity)   ← category緩和
  段4. (civ, *,   category, baseRarity)   ← era緩和
  段5. (civ, *,   *,        *        )   ← 文明だけ維持
  段6. (*,   *,   *,        baseRarity)   ← 文明・時代・カテゴリを完全に無視し、
                                            baseRarity のみ一致する全レコードから選ぶ
  ─────────────────────────────────────────────────────────────────
  終端. GLOBAL_FALLBACK_ID                ← 6段すべて空のときのみ。下記参照
```

- **段6の意味（明確化）**: civ/era/category を一切問わず、`base_rarity = baseRarity` の
  **全レコード**を候補集合とし、その中から選ぶ。レア度だけは必ず一致させる最終手段。
- **終端 GLOBAL_FALLBACK_ID（明確化）**: **文明・時代・カテゴリのいずれにも属さない
  ダミー遺物（DB全体で1件のみ）**。段6でも候補が0になることは現実には起きない
  （baseRarity 1..10 のどれかに必ずレコードがあるため）が、DB不整合への保険として必ず1件用意する。
- どの段で当たっても、その集合内の選択は `uintBelow(seed,"pick",len)` で決定論
  （集合は `ORDER BY id` で安定化してから選ぶ）。
- 「rarity緩和（段2）の最近傍優先」は、候補のrarityと`baseRarity`の差で安定ソートしてから選ぶ。
- **設計推奨（重要）**: フォールバックの発火自体を抑えるため、DB生成時に
  **「各 (civ × era × category) の組について baseRarity 1..10 を最低1件ずつ埋める」**制約を満たす
  （[05] 5.6 のDB生成ツールで保証）。これを守れば実運用上のフォールバックは段2止まりになる。
  ⇒ このルールは事前生成の必須要件として [05] 側でも強調する。

## 3.6 軸の値（初期セット案）

仕様書の例を採用。確定値はバランス調整で変わりうる。

- civilization: 砂漠 / 海洋 / 山岳 / 機械 / 有機
- era（遺物時代）: 古代 / 中世 / 近世 / 近代 / 未来
- category: 武器 / 祭具 / 生活用品 / 建築断片 / 碑文 / 機械部品

「機械文明 × 古代」「有機文明 × 機械部品」のような噛み合わせの良し悪しは、
DB生成時に重み0で除外できる（存在しない組み合わせはそもそもレコードを作らない）。
ただしその場合フォールバック（3.5）の発火が前提になる点に注意。
