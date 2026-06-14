# 04. バリエーション（色違い・レイヤー・個体差テキスト）

有限のベース遺物を「体感無限」にする層。すべて [01] のタグ付きストリームから決定論的に算出。

## 4.1 色違い（HSV補正・連続値）

レア度と**完全独立**（[01]で構造保証）。`"color"` ストリームからHSV補正を作る。

```
colorMod(seed):
  hShift = (uniform(seed,"color")        * 360) - 180      # 色相 -180..+180 度
  sMul   = 0.6 + uniform(seed,"color.s") * 0.8             # 彩度 ×0.6..1.4
  vMul   = 0.7 + uniform(seed,"color.v") * 0.6             # 明度 ×0.7..1.3
  return { hShift, sMul, vMul }
```

### 適用順序（厳守）
HSV補正は**合成パイプラインの最初に baseImage へ適用**する。その後 dirt → damage → preservation の順に
重ね、**background は最後**に最背面へ合成する（[00] 0.4.3 / [05] 5.2 と一致）。

```
baseImage → [HSV(colorMod)] → dirt → damage → preservation → background
   層1          層2            層3    層4       層5            層6
```

色相シフトを先頭に置くのは、汚れ・破損・退色（preservation）が「色補正後の地の色」の上に乗るべきだから。
順序を変えると同属性でも別の絵になり決定論が崩れるため、この順序は固定。

- 連続値なので色のバリエーション自体は無限。
- 仕様書6章の「名前付き色（砂漠風退色・緑青・黒曜石…）」は、連続HSV空間を**領域に命名**する形で両立:
  ```
  colorName(colorMod) = nearest preset region label   # 表示用ラベルのみ。生成は連続値で行う
  ```
  プリセット例: 通常色 / 砂漠退色 / 緑青(酸化銅) / 黒曜石 / 白金 / 火山灰 / 海水漂白 / 風化淡色 / 良好鮮やか。
- **名前付き色の領域定義**（各プリセットの HSV 中心・半径＝命名境界）は小さなデータ資産として
  同梱し、`assets/data/color_regions.json` に置く（資産配置は [05] 5.5 を参照）。`colorName` は
  この領域テーブルへの最近傍判定のみ行う（生成は常に連続値、ラベルは表示用）。

## 4.2 汚れレイヤー（離散ID）

`"dirt"` ストリームから重み付き選択。透明PNGを合成（[05]）。

```
DIRT_LAYERS = [none, mud, sand, soot, sea_salt, volcanic_ash, ...]   # 各々 weight を持つ
dirtLayerId(seed) = pickWeighted(seed, "dirt", DIRT_LAYERS)
```

- **レイヤーPNGのサイズ上限**: 1枚あたり **1024×1024px・透明WebP/PNG・≤150KB** を上限とする。
  被写体は全スタイル同一座標・同一スケール（[05]5.3）なので、この1枚を全ベース遺物で使い回せる
  （＝枚数は汚れ種ぶんのみ。容量にほぼ効かない, [05]5.4）。

## 4.3 破損レイヤー（離散ID）

`"damage"` ストリームから。欠け / ひび割れ / 摩耗 など。
複数同時付与を許すなら、各破損種ごとに独立ストリーム（`"damage.crack"` 等）でON/OFF判定する。

```
damage(seed):
  chip  = uniform(seed,"damage.chip")  < 0.5
  crack = uniform(seed,"damage.crack") < 0.35
  wear  = uniform(seed,"damage.wear")  < 0.6
  return { chip, crack, wear }          # 合成順序は [05] で固定
```

- 破損レイヤーPNGも汚れと同じ上限: **1024×1024px・透明WebP/PNG・≤150KB**（[05]5.4）。

## 4.3.1 レイヤー強度の preservationScore 補正

汚れ・破損レイヤーは**ON/OFFと種類だけでなく「強度（不透明度）」を `preservationScore` で補正**する。
保存状態が良い個体ほど汚れ・破損が薄く、悪い個体ほど濃く出る。

```
# preservationScore（4.4）: 0=劣悪 .. 1=良好
dirtStrength(preservationScore)   = clamp(0.9 - 0.7 * preservationScore, 0.1, 0.9)  # 良好ほど薄い
damageStrength(preservationScore) = clamp(0.9 - 0.6 * preservationScore, 0.15, 0.9)

compose時の不透明度:
  blend(img, DIRT_PNG[dirt],  alpha = dirtStrength(preservationScore))
  blend(img, WEAR_PNG,        alpha = damageStrength(preservationScore))   # crack/chip も同様
```

- 強度はレイヤーの**合成不透明度**として適用する（レイヤーPNG自体は1種類のまま、濃淡だけ変える）。
- これにより「同じ汚れ種でも保存状態で見え方が変わる」連続的な個体差が生まれる（無限化に寄与）。
- 実適用は合成パイプラインの層3（dirt）・層4（damage）で行う（[05] 5.2）。
- ⚠️ `damage` の ON/OFF 判定（4.3）自体は `"damage.*"` ストリームで決まり、強度補正とは独立。
  強度はあくまで「見せ方」であり、決定論の入力（ストリーム）を汚さない。

## 4.4 保存状態とレア度の弱い相関（[02] 2.4の実装）

レア度自体は色と独立のまま、**保存状態のみ**レア度で弱くバイアスする非対称設計。

```
preservation(seed, finalRarity):
  base = uniform(seed, "preserve")               # [0,1) レア度と独立な素の値
  bias = (finalRarity - 1) / 12 * 0.25           # 高レアほど最大+0.25のゲタ
  score = clamp(base + bias, 0, 1)               # 高いほど良好
  return score        # → dirt/damage の強度や良好色の選ばれやすさに反映
```

- `preserve` ストリーム自体はレア度と無相関。バイアスは加算で**弱く**入れるだけ（仕様の「軽い補正」）。
- この `score` を **汚れ・破損レイヤーの強度補正（4.3.1）** と 4.1 の「良好鮮やか色」採用確率に反映する。

## 4.5 個体差テキスト（テンプレート × ハッシュ）

> ⚠️ **実装メモ（現況）**: 実コード `rust/qrac-core/src/flavor.rs` は、下記のスロット分割方式ではなく
> **民明書房調の単一段落**（`describe()`）を生成する。構成は「架空の発掘地名＋出自 → こじつけ語源 →
> 表面状態の解説（汚れ種別と整合）→ 居丈高な締め」、神話級は専用の導入文付き。出典体裁は
> 「詳説 世界の遺物（萬象書房 1890年刊）」。タグは `text:place.*` / `text:desc.etym` / `text:desc.close`、
> 地名サフィックスは `盆地/遺跡/谷/高原/砂海/氷河`、音節数 2..3。FFIでは構造化せず `description: String`
> 一本で返す（docs/06 6.1）。以下のスロット設計は将来の構造化案として残置。
>
> **多言語対応**: `describe(seed, attr, lang)` は `Lang::{Ja,En}` を取り、日英いずれかの単一段落を生成。
> 日英の語句配列は**同じ長さ**に保ち、同一 seed で「同じ選択（index）」を別言語で出す。地名生成も言語別
> （日: カタカナ音節＋和風サフィックス / 英: ローマ字音節＋頭大文字＋英サフィックス）。言語は表示テキスト
> のみに影響し、ハッシュ・属性・決定論には無関係。書名「詳説 世界の遺物」「萬象書房 1890年刊」はUI側で言語別に付与。

`"text:*"` ストリーム群でスロットを埋める。テンプレートは事前用意（端末同梱）。

### テンプレート構造（固定スロット）

個体差テキストは、以下の **5つの固定スロット** を順に組み上げて構成する。
スロットのキー（識別子）は固定。生成ロジック・テンプレートデータ・テストはこのキーで参照する。

```
fullText = render([
  place,                # 1. 発掘地・出土層（架空地名＋層番号）
  layer,                # 2. 出土層・年・発掘隊などの発掘メタ情報
  condition_sentence,   # 3. 状態の説明文（汚れ/破損/保存状態と整合する一文）
  usage_sentence,       # 4. 推定用途の一文（category 由来）
  researcher_comment,   # 5. 研究者コメント（レア度/保存状態に応じた一文）
])
```

| キー（固定） | 内容 | 供給源 | 条件付け | 例 |
|-------------|------|--------|---------|-----|
| `place` | 発掘地名 | 音節ジェネレータ（下記） | — | アラ＝サル盆地 |
| `layer` | 出土層・発掘年・発掘隊 | 層番号(uintBelow)＋年表＋隊名表 | — | 第7層 / 19XX年 / 第三次中央調査隊 |
| `condition_sentence` | 状態の説明文 | 状況フレーズ表 | `dirtLayerId`・`damage`・`preservationScore` で表を切替 | 表面の黒色変色は火山灰によるものと考えられる |
| `usage_sentence` | 推定用途の一文 | category別の用途表 | `category` で表を切替 | 祭礼用と推定される |
| `researcher_comment` | 研究者コメント | コメント表 | `finalRarity`・`preservationScore` で表を切替 | 保存状態は驚くほど良い |

- 各スロットは別の `"text:<key>.*"` ストリームから引く（[01] 1.3 / [00] 0.4.4 のタグ命名規則）。
  例: `place` は `text:place.*`、`condition_sentence` は `text:cond.*`。
- スロットの**並び順・キー名は固定**（テンプレ差し替えで文面は変わっても構造は不変）。
  破壊的にキーや順序を変える場合は `genVersion` を上げる（[06] 6.4）。
- 破損率の数値（例「推定破損率38%」）は `damage`＋`preservationScore` から算出し、
  `condition_sentence` 内に埋め込む（独立スロットにはしない）。

### 架空地名の音節ジェネレータ（無限・決定論）
```
SYLL = ["ア","サル","ラ","ネブ","カ","トゥ","メ","ゾ", ...]
placeName(seed):
  n = 2 + uintBelow(seed,"text:place.len", 3)            # 2..4音節
  parts = [ SYLL[ uintBelow(seed, "text:place."+i, len(SYLL)) ] for i in 0..n ]
  joiner = pick(seed,"text:place.join", ["＝","・",""])   # アラ＝サル 等
  suffix = pick(seed,"text:place.suf", ["盆地","遺跡","谷","高原","海溝"])
  return join(parts, joiner) + suffix
```

### 整合性（重要）
テキストは画像と矛盾してはいけない。例: 汚れが `volcanic_ash` なら状況フレーズも火山灰系に寄せる。
→ スロット供給テーブルを `dirtLayerId` / `damage` / `finalRarity` で**条件付け**る
（独立ストリームで引きつつ、引く表を属性で切り替える）。これで「黒色変色は火山灰による」等が自然に整う。

### 仕様書の例文の再現
> 「この個体は“アラ＝サル盆地”の第7層から発掘された。表面の黒色変色は火山灰によるものと考えられる。」

= `placeName` + `第{N}層`(uintBelowで層番号) + 状況フレーズ（dirt=ash条件下の一文）で生成可能。

## 4.6 総バリエーション数（体感無限の根拠）

離散部分だけでも:
```
ベース遺物数 × 汚れ × 破損(2^3) × 名前付き色領域(≒9) × テキスト離散組合せ
≈ 10^4(5,000〜10,000) × 6 × 8 × 9 × (地名・隊名・年・コメントの積 >10^6)
≈ 10^12 以上
```
これに連続HSV（hShift/sMul/vMul）と、汚れ・破損の preservationScore 連続強度（4.3.1）が乗る＝実質無限。
衝突は事実上起きない＝「異なるQRは異なる遺物」を満たす。
