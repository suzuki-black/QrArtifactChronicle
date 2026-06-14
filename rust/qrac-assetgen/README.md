# qrac-assetgen — アセット生成ツール

docs/05 5.6 / docs/08 8.11。ベース画像と「全rarityマス充足」のDBを生成し、カバレッジをCIゲート検証する。

## 実行

```bash
cd rust
cargo run -p qrac-assetgen -- dist     # 出力先 dist/（省略時 dist）
```

## 出力

```
dist/
  artifacts.sqlite                         全 civ×era×category × rarity 1..10（=1500行）+ GLOBAL
  assets/base/<image_set_id>/<style>/0.png ベース画像（被写体・透過・ニュートラル色）
                                           image_set_id = civ×era×category（150）, style=museum/dig/catalog
```

## カバレッジゲート（CI）

全 150 combo (civ×era×category) で ★1..10 が揃っているか検証。**欠けがあれば exit 1**
（docs/05 5.6 の必須制約）。CI に組み込めば、DB生成漏れでビルドを失敗させられる。

## 実行時統合（接続済み）

`qrac_render::compose::render(attr, base, sprite_png)` が本ツールの出力 `assets/base/<id>/<style>/0.png`
を読み込み、per-pixel HSV(層2)＋汚れ/破損/保存/背景を合成する。FFI `configure_assets(dir)` で
assets ディレクトリを設定し、macOS アプリは同梱 `Resources/assets` を使う（無ければ手続き生成にフォールバック）。

## 次

- ベース画像は今は**手続き生成のシルエット**（`qrac_render::render_base_sprite`、civの自然色）。
  アーティスト製の WebP に差し替える（レイアウト・命名はそのまま、HSV で色違いを生む前提）。
