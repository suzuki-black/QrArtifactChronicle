# 01. 決定論の基盤（ハッシュと乱数ストリーム分割）

「同じQR → 同じ遺物」「色違いはレア度と独立」を構造的に保証する中核。

## 1.1 正規化（normalize）

ハッシュ前に生データを正規化し、「実質同じQR」が同じシードになるようにする。
ここを雑にすると、見た目同じQRから別遺物が出て要件違反になる。

> ✅ **正規化強度は確定（Q6）: 軽い正規化のみ。**
> - **URLのみ**: ホスト名を小文字化・既定ポート除去・末尾スラッシュ正規化・**クエリのキー順をソート**。
> - **正規化しない**: パスの大文字小文字、クエリの**値**、フラグメント（別物の可能性があるため）。
> - **テキストQR・バイナリQR**: 生データそのまま（末尾空白の除去のみ）。

> 🔧 **実装方針（確定 / docs/08）: URL正規化は手実装。`url`クレート等の汎用URLライブラリは使わない。**
> 汎用ライブラリは %エンコード正規化・IDN/punycode・既定ポート規則がプラットフォーム/版で異なり、
> 別 `normalizedKey` → 別遺物を招く。**下記の擬似コードをバイト単位でそのまま移植**し、参照実装
> （TS）とRust本実装を**バイト一致**させる。本仕様自体が `genVersion` の一部（変更時はインクリメント）。

```
normalizeKey(rawBytes):
  if not isProbablyText(rawBytes):
    return rawBytes                       # バイナリQRは一切いじらない

  s = decodeUtf8Lenient(rawBytes)
  s = stripTrailingWhitespace(s)          # 末尾の空白・改行・NUL のみ

  if looksLikeUrl(s):
    s = canonicalizeUrl(s)                # 下記のバイト精度仕様
  return utf8Bytes(s)                      # 一般テキストは末尾整形のみ
```

### `canonicalizeUrl` のバイト精度仕様（手実装・移植の正本）

汎用ライブラリに委譲せず、以下を**この順で**適用する。各ステップの「やる/やらない」を厳密に守る。

```
canonicalizeUrl(s):   # s は "http://" or "https://" で始まる（looksLikeUrl 済み）
  # 1. 簡易自前パース: scheme "://" authority [ "/" path ] [ "?" query ] [ "#" fragment ]
  #    authority = [ userinfo "@" ] host [ ":" port ]
  #    最初の "/"(authority終端) / "?" / "#" の位置でフィールドを切る。デコードは一切しない。
  scheme, authority, path, query, fragment = parse(s)

  # 2. scheme: ASCII の A-Z のみ小文字化（http/https）
  scheme = asciiLower(scheme)

  # 3. host: authority 内のホスト部のみ ASCII の A-Z を小文字化。
  #    - 非ASCIIバイトは変更しない（IDN/punycode 変換は【しない】）
  #    - userinfo（"@"より前）は変更しない
  host = asciiLowerAsciiLettersOnly(host)

  # 4. 既定ポート除去: scheme=http かつ port=80、または scheme=https かつ port=443 のときだけ
  #    ":port" を削除。それ以外のポートは保持。
  if (scheme=="http" and port=="80") or (scheme=="https" and port=="443"): drop port

  # 5. path 末尾スラッシュ: path が "/" 単独なら維持。末尾が "/" でそれ以外なら末尾 "/" を1つ除去。
  #    path の大文字小文字・%エンコードは【保持】。
  path = (path == "/" ) ? "/" : rstripOneTrailingSlash(path)
  if path == "": path = "/"

  # 6. query: あれば "&" で分割し、各要素を「最初の '=' まで」をキーとみなす（'=' 無しは値なしキー）。
  #    キーの【UTF-8バイト列】で安定ソート（同一キーの相対順は保持）。'=' と値は逐語保持・デコードしない。
  if query present:
    parts = split(query, "&")
    parts = stableSortBy(parts, key = bytesBeforeFirst('=', part))   # バイト辞書順
    query = join(parts, "&")

  # 7. fragment は逐語保持（変更しない）

  # 8. 再構築（存在するフィールドのみ連結。区切り文字も入力に存在した時のみ付与）
  return scheme + "://" + authority' + path + ("?"+query if present) + ("#"+fragment if present)
```

設計意図とライブラリ非依存の理由:
- **%エンコードに一切触れない** → `%2F` と `/` を同一視しない（情報を壊さない）。汎用ライブラリは
  ここを正規化しがちで、それが最大のプラットフォーム差要因。
- **IDN/punycode変換をしない** → `idna` 実装差に依存しない。非ASCIIホストの2表記は稀なので吸収しない。
- **キーのソートはバイト辞書順**（コードポイント順と一致, UTF-8の性質）で安定。ロケール非依存。
- パス・クエリ値・フラグメントは有意（別ページ＝別遺物）なので保持。ホスト大小・既定ポート・末尾/・
  クエリ順は同一リソースの表記揺れなので吸収する。
- バイナリは解釈の余地がないため無加工が最も安全。

### ヘルパ関数の定義（決定論のため挙動を固定）

正規化の各ヘルパは実装差でブレると別遺物が出るため、判定基準を以下に固定する。

```
isProbablyText(rawBytes):
  # テキストQRかバイナリQRかの判定。判定基準を固定:
  #  (a) 全バイトが妥当な UTF-8 として解釈できる、または
  #  (b) printable ASCII（0x20..0x7E ＋ \t\r\n）が全体の 95% 以上
  # (a) または (b) を満たせばテキスト扱い、いずれも満たさなければバイナリ扱い。
  return isValidUtf8(rawBytes) or printableAsciiRatio(rawBytes) >= 0.95

decodeUtf8Lenient(rawBytes):
  # 不正な UTF-8 シーケンスは U+FFFD（REPLACEMENT CHARACTER）に置換して復号する。
  # ※ ここに到達する時点で isProbablyText は真。(b)経由で不正バイトが残る場合のみ置換が発生。

looksLikeUrl(s):
  # 判定は厳格に限定: 文字列が "http://" または "https://" で始まる場合のみ URL とみなす。
  # （mailto:/tel:/ftp: 等の他スキームは URL 正規化の対象外＝一般テキスト扱い）
  return s.startsWith("http://") or s.startsWith("https://")

stableSortQueryByKey(parts):
  # クエリ要素をキーの【UTF-8バイト辞書順】で【安定ソート】（同一キーの相対順は保持・値は不変）。
  # キー = 各要素の最初の '=' より前のバイト列（'=' 無しは要素全体）。デコードしない。
  # 例: a=2&a=1&b=3 → a=2&a=1&b=3（a同士の順は変えない）／ b=3&a=1 → a=1&b=3
```

## 1.2 シード

```
seed = SHA-256(normalizedKey)   # 32 bytes
```

## 1.3 タグ付き派生 RNG（採用案）

256bitを固定スライスで切り分ける方式（byte0-3=レア度…）は、属性が増えるとビットが枯渇し、
将来追加で割り当てがズレて互換性が壊れる。代わりに**ドメイン分離した無限ストリーム**を使う。

```
# 各決定ごとに独立な 256bit を生成
#  tag は UTF-8 NFC 正規化してから連結する（環境差で合成済み/分解済みが混在しても同一バイト列になるため）
substream(seed, tag, counter=0) = SHA-256( seed || 0x00 || utf8(nfc(tag)) || u32be(counter) )

# 一様実数 [0,1)  … 先頭8バイトを big-endian 固定で u64 化（u64be）
uniform(seed, tag) = u64be( substream(seed, tag)[0:8] ) / 2^64

# 一様整数 [0, n)  … 剰余バイアスを避ける棄却法。4バイト抽出も big-endian 固定（u32be）
uintBelow(seed, tag, n):
  for c in 0,1,2,...:
    x = u32be( substream(seed, tag, c)[0:4] )
    limit = floor(2^32 / n) * n
    if x < limit: return x % n
```

> バイトオーダは **big-endian に固定**（`u64be` / `u32be` / `u32be(counter)` すべて）。
> 端末のネイティブエンディアンに依存させると決定論が壊れるため、実装で明示すること。

### なぜこの方式か
- **独立性**: タグが違えば出力は暗号学的に無相関。`uniform(seed,"rarity")` と
  `colorMod(seed,"color")` は完全独立 → 「色違いはレア度と独立」を**設計で保証**。
- **拡張性**: 新属性は新タグを足すだけ。既存属性の出力は1ビットも変わらない。
- **無尽蔵**: counterで同一タグから何ビットでも取り出せる（テキスト生成など多ビット消費に有効）。

### タグ一覧（予約）
| tag | 用途 | 参照 |
|-----|------|------|
| `"rarity"` | 基本レア度 | [02](02-rarity-and-era.md) |
| `"civ"` | 文明 | [03](03-base-artifact-db.md) |
| `"era"` | 遺物時代 | [03](03-base-artifact-db.md) |
| `"category"` | カテゴリ | [03](03-base-artifact-db.md) |
| `"pick"` | 候補集合からのベース遺物選択 | [03](03-base-artifact-db.md) |
| `"color"` | HSV補正 | [04](04-variations.md) |
| `"dirt"` | 汚れレイヤー | [04](04-variations.md) |
| `"damage"` | 破損レイヤー | [04](04-variations.md) |
| `"preserve"` | 保存状態（色/レイヤーの軽い相関に使用） | [04](04-variations.md) |
| `"text:*"` | 個体差テキストの各スロット | [04](04-variations.md) |

タグ文字列は**一度決めたら変えない**（変えると全遺物が変わる）。変更時は `genVersion` を上げる（[06]）。

## 1.4 「異なるQRは高確率で異なる遺物」について

SHA-256のため、異なる正規化キー同士はシードが衝突しない（事実上）。
ただし**最終的な見た目の衝突**は別問題: 属性空間の総数が有限なら鳩の巣で重複は起きる。
体感無限を支えるのは個体差テキストとHSV連続値（[04]）。総バリエーション数の試算は [04] 参照。

## 1.5 テスト指針（決定論）
- ゴールデンテスト: 代表的な生データ集合 → 期待 `Artifact` のスナップショット。
  `genVersion` 据え置きでこれが変わったら**リグレッション**。
- 独立性テスト: 大量シードで `baseRarity` と `colorMod` の相関係数 ≈ 0 を確認。
- 分布テスト: 100万シードで基本レア度のヒストグラムが目標分布に収束（[02]）。
