# 05. 画像合成パイプラインと容量設計

> ⚠️ **実装状況**: 6層合成は **実装済み**（`qrac-render/compose.rs`）。ベース被写体／汚れ／破損／背景は
> **透過PNGレイヤー**として `qrac-assetgen` が事前生成し、実行時に **tiny-skia のブレンドモード**
> （Multiply / Screen / SoftLight / DestinationOut）と**被写体シルエットのマスク**で固定順に合成する。
> アセットが無いレイヤーは `qrac-render/art.rs` の**手続き生成にフォールバック**する。
> 現状の差分: キャンバスは **512²**（設計目標は 1024²）、合成結果の **LRUキャッシュは未実装**、
> アートは写真ではなく**質感のあるスタイライズ**（アーティスト製アセットへの差し替え余地を残す）。
> 実装の現況は docs/08 §8.6 を参照。

## 5.1 方針: 事前生成 vs 実行時合成

| 要素 | いつ作る | 理由 |
|------|---------|------|
| ベース画像（各アングル・各写真風） | **事前生成**（開発時） | 有限。LLM/重い生成は端末で動かさない |
| 色補正・汚れ・破損の合成 | **実行時**（端末GPU/Canvas） | 組み合わせ無限。事前生成すると容量爆発 |
| 合成結果 | **キャッシュ**（端末ローカル） | 図鑑再表示の高速化。容量上限でLRU破棄 |

無限の組み合わせを事前に焼くのは不可能なので、**ベース＋透明レイヤーを実行時合成**が唯一現実的。

## 5.2 実行時合成パイプライン（6層・合成順序は固定 ＝ 決定論）

レイヤー構造は **baseImage → HSV → dirt → damage → preservation → background** の6層で固定
（[00] 0.4.3 と一致）。順序固定により、同じ属性なら必ず同じ絵になる（決定論を画像まで貫徹）。

```
compose(base, colorMod, dirt, damage, preservationScore, style):
  # --- 1. baseImage: 確定レコードの被写体（artifactEra/civ/category/baseRarity で確定） ---
  img = load(base.imageSet[style])            # 背景透過・アングル1枚（5.4 / 推奨解像度は下記）
  # --- 2. HSV: 色違い（tag="color"） ---
  img = applyHSV(img, colorMod)               # hShift/sMul/vMul を1パス
  # --- 3. dirt: 汚れ（tag="dirt", ブレンドモードは層別。強度=dirtStrength） ---
  img = blend(img, DIRT_PNG[dirt], mode=DIRT_BLEND[dirt], alpha=dirtStrength(preservationScore))
  # --- 4. damage: 破損（tag="damage", 摩耗→ひび→欠けの順, 強度=damageStrength） ---
  if damage.wear:  img = blend(img, WEAR_PNG,  mode="soft-light", alpha=damageStrength(preservationScore))
  if damage.crack: img = blend(img, CRACK_PNG, mode="multiply",   alpha=damageStrength(preservationScore))
  if damage.chip:  img = blend(img, CHIP_PNG,  mode="mask-out")    # 欠けはアルファ抜き(強度補正なし)
  # --- 5. preservation: 保存状態の最終調整（tag="preserve", 強度=preservationScore） ---
  img = applyPreservation(img, preservationScore)  # 彩度/明度/コントラスト補正（下記）
  # --- 6. background: 写真風背景（被写体の最背面に normal 合成） ---
  img = blend(BACKGROUND[style], img, mode="normal")  # museum/dig/catalog のスタイル別背景
  return img
```

### レイヤー別ブレンドモード（固定）

決定論のため、各レイヤーのブレンドモードは固定する（同属性→同じ絵）。

| 層 | レイヤー | ブレンドモード | 補足 |
|----|---------|---------------|------|
| 3 | dirt: mud（泥） | `multiply` | 暗く沈ませる |
| 3 | dirt: sand（砂） | `overlay` | 明部に砂色を乗せる |
| 3 | dirt: soot（煤） | `multiply` | 黒ずみ |
| 3 | dirt: sea_salt（海塩） | `screen` | 白い析出物を明るく乗せる |
| 3 | dirt: volcanic_ash（火山灰） | `multiply` | 黒色変色（テキスト整合, [04]4.5） |
| 4 | damage: wear（摩耗） | `soft-light` | 表面の艶/質感を弱める |
| 4 | damage: crack（ひび） | `multiply` | 暗い線を刻む |
| 4 | damage: chip（欠け） | `mask-out` | アルファ抜き（被写体の一部を削る） |
| 6 | background | `normal` | 被写体を最前面に通常合成 |

- 各 dirt のモードは `DIRT_BLEND[dirtId]` テーブルで定義（汚れ種を増やす時はここに追記）。
- `dirtStrength` / `damageStrength` は preservationScore による不透明度補正（[04] 4.3.1）。
- `chip` の `mask-out` だけは強度補正せず、欠けの形状を確定的に抜く。

### applyPreservation の処理内容

保存状態スコア（0=劣悪 .. 1=良好）に応じ、被写体全体の **彩度・明度・コントラスト**を線形補正する。
1パスのカラーグレーディングで、退色（劣悪）〜鮮やか（良好）を連続表現する。

```
applyPreservation(img, s):   # s = preservationScore ∈ [0,1]
  saturation *= lerp(0.70, 1.10, s)   # 劣悪=退色(×0.70) .. 良好=やや鮮やか(×1.10)
  brightness *= lerp(0.92, 1.05, s)   # 劣悪=くすみ(×0.92) .. 良好=明るい(×1.05)
  contrast   *= lerp(0.90, 1.08, s)   # 劣悪=眠い(×0.90)   .. 良好=締まる(×1.08)
  return applyColorGrade(img, saturation, brightness, contrast)
```

- 補正範囲は上記に固定（決定論）。`lerp(a,b,s) = a + (b-a)*s`。
- HSV補正(層2)が「個体の地の色」を作るのに対し、preservation(層5)は「経年の状態感」を全体に乗せる役割。
  両者は別ストリーム・別目的なので二重適用ではない。

### その他
- 1〜5は被写体への処理、6は背景。被写体は全スタイルで同一座標・同一スケール（レイヤー共有のため）。
- 実装はモバイルGPUシェーダ（Metal/Vulkan）またはCanvas/Skia。HSV・preservationは1パスのシェーダで軽い。
- **baseImage 推奨解像度**: **1024×1024px**（被写体は背景透過。レイヤーPNGと同寸法に揃える＝[04]4.2の上限と一致）。
  Retina表示や図鑑の拡大に耐えつつ、43KB/枚の容量目標（5.4）と両立する実用解像度。

## 5.3 写真風スタイルとアングル

仕様書9章: 博物館展示風(museum) / 発掘現場風(dig) / 図録写真風(catalog)。
- **被写体（baseImage）は背景透過**で持ち、**背景はスタイル別の独立レイヤー**（5.2 の層6 `BACKGROUND[style]`）。
  これにより汚れ・破損レイヤーをスタイル間で共通PNGとして使い回せる。
- スタイル差のうち**ライティング**は、被写体側のスタイル差分（`base.imageSet[style]`）で吸収する。
  純粋な背景だけ差し替えでは光の当たり方が不自然になるため、被写体もスタイルごとに用意する
  （＝ベース画像はスタイル数ぶん持つ。容量試算 5.4 はこれを織り込み済み）。
- 被写体は全スタイルで同一座標・同一スケールに揃える（レイヤー共有と決定論の前提）。

## 5.4 容量設計（2GB） — ✅ 確定（Q3解決済み）

当初の「数十万レコード × 多アングル × 3スタイル」は2GBと両立しないため、以下に確定。

> ✅ **確定構成**
> - レコード数: **5,000〜10,000**
> - アングル: **1枚**（被写体は同一座標・同一スケール → レイヤー共有が効く）
> - 写真風スタイル: 3（museum / dig / catalog）は**ベース画像差分**として保持
> - 色違い・汚れ・破損は**実行時レイヤー合成で無限化**（事前生成しない）
> - 着地: **総容量 1.2〜1.4GB**（2GB枠内、更新・キャッシュの余白を確保）

### 容量試算（確定構成）
```
ベース画像 = レコード数 × スタイル3 × アングル1
  上限ケース: 10,000 × 3 × 1 = 30,000枚
  1.3GB を 30,000枚で割る ≒ 約43KB/枚（WebP高品質で現実的）
  下限ケース:  5,000 × 3 × 1 = 15,000枚 → 1枚あたり余裕（高解像も可）
共有レイヤー（汚れ・破損）= 全遺物共有のため数十枚規模 → 容量にほぼ eff 0
SQLite（メタ＋テキスト＋テンプレ） = 数十MB規模
合成キャッシュ = 端末ローカル、LRUで上限管理（同梱容量に含めない / 上限は下記）
```
→ 写真1枚あたり ~43KB を確保でき、**1.2〜1.4GBに収まる**見込み。

### この構成が「無限感」を失わない理由
- アングルを1枚に絞っても、見た目の個体差は色違い（連続HSV）＋汚れ＋破損＋スタイル3で作る。
- ベース5,000〜10,000でも、合成・テキストを含む総バリエーションは 10^12 超（[04] 4.6）。
- 将来アングルを増やしたくなった場合は「擬似3D（法線マップ簡易ライティング）」で
  実体枚数を増やさず疑似的に角度を出す余地を残す。

### 圧縮
- フォーマットは **WebP/AVIF**（透明対応・高圧縮）。レイヤーPNGもWebP可逆/非可逆を使い分け。
- ベース被写体は非可逆高品質、レイヤー（汚れ/破損のアルファ）はエッジ重視で設定を分ける。
- **background は非可逆WebPで良い**（質感のある写真風の背景で、エッジ精度より圧縮率を優先できる。
  被写体のアルファ境界に関与しないため非可逆の劣化が目立たない）。品質係数の目安は q≈75。

### 合成キャッシュの上限
- 合成結果（`cache/composed/...`）は端末ローカルに **LRU で上限 256MB** を設けて保持する。
- 上限超過時は最終アクセスが古いものから破棄。キャッシュは再合成で必ず復元できる（決定論）ので
  破棄は安全。同梱2GB枠には**含めない**（端末空き容量側で管理）。
- 端末の空き容量が逼迫した場合は上限を動的に縮小（最低 64MB）してよい。

## 5.5 アセット配置とDB参照
```
assets/
  base/<image_set_id>/<style>/<angle>.webp     # ベース画像。<angle> は常に "0"（[06]images.angle）
  layers/dirt/<id>.webp                         # 汚れ（全遺物共有）
  layers/damage/{wear,crack,chip}.webp          # 破損（共有）
  data/color_regions.json                       # 名前付き色の領域定義（[04]4.1）
cache/composed/<artifactHash>/<style>_<angle>.webp   # 実行時合成キャッシュ(LRU)
```
- DB(`base_artifact.image_set_id`)→ `assets/base/<image_set_id>/<style>/0.webp` を解決
  （`<style>` ∈ museum/dig/catalog、`<angle>` は現状 "0" 固定。擬似3D拡張時のみ 0 以外を使う）。
- レイヤー（dirt/damage）と data/ は全遺物共有なので容量に効かない。

## 5.6 DB/アセット生成ツール（開発時・別パイプライン）
- 事前生成側は本アプリと別のオフラインツール。ベース遺物の網羅、画像レンダ、WebP変換、
  SQLite構築、整合チェックまでを担う。
- ここでの生成は決定論不要（人手キュレーション可）。**アプリ実行時の決定論はDB確定後に効く。**

### 🔴 必須カバレッジ制約（最重要）
> **各 (civ × era × category) の組み合わせについて、baseRarity 1..10 を最低1件ずつ埋めること。**
- これを満たすと、実行時のフォールバック（[03] 3.5）は本命（段1）か、悪くても段2（rarity緩和）で
  必ず解決し、段3以降の「軸を捨てる」フォールバックがほぼ発火しない＝遺物の質が安定する。
- 生成ツールは最終工程で**このカバレッジを検証**し、欠けマスがあればビルドを失敗させる（CIゲート）。
- 噛み合わせ上わざと存在させない (civ×era×category) は、そもそも候補軸から外す（重み0, [03]3.3/3.6）。
  「存在させる組み合わせ」については上記の全rarity充足を厳守する。
