# qrac-ffi (UniFFI 境界)

純粋コア `qrac-core` ＋ I/O層 `qrac-render` を Swift / Kotlin に公開する薄い橋渡し層（docs/08 8.5）。
**決定論ロジックは持たない**（コア呼び出しと型変換のみ）。

## 公開API

```rust
#[uniffi::export] fn configure_assets(dir: String)                           // ベース画像のdir設定
#[uniffi::export] fn derive_qr(text: String) -> Artifact                     // 属性（年は自動抽出）
#[uniffi::export] fn derive_qr_with_year(text: String, year: Option<i32>) -> Artifact
#[uniffi::export] fn render_qr(text: String) -> RenderedImage                // derive→DB選択→合成→PNG
#[uniffi::export] fn render_qr_with_year(text: String, year: Option<i32>) -> RenderedImage
```

- `Artifact` = 属性（hash / レア度 / 文明・時代・カテゴリ / 色 / 汚れ / 破損 / 保存度）。
- `RenderedImage` = `{ width, height, png(バイト列), base_name, matched_stage, description }`。
  画像は **PNGバイト列**で返す（ネイティブで `NSImage`/`Bitmap` 化）。`description` は民明書房調の解説文。
- 収集の保存/一覧はFFIに持たず、Swift側（`collection.json`）で実施。

## macOS 最小動作デモ

```bash
cd rust
bash qrac-ffi/demo/run.sh
```

`run.sh` の流れ:
1. `cargo build -p qrac-ffi` → `libqrac_ffi.dylib`
2. `uniffi-bindgen generate --library …` → `target/swift/{qrac_ffi.swift, *.h, *.modulemap}`
3. `swiftc` で `demo/main.swift` ＋ 生成Swift をコンパイル・dylibリンク
4. 実行 → QR文字列から遺物を生成して表示

**検証済み**: Swift 経由で得た hash が `golden.json` / Rust `tests/golden.rs` と**一致**
（例: vCard2014 → `b52575a326330068a263b3d74259c405`）。FFI境界を越えても決定論が保たれる。

## 実 GUI（SwiftUI）

本番の macOS アプリは **`apple/`**（`QracKit` / SwiftUI）にあり、同じ `deriveQr()` / `renderQr()` を呼ぶ。
`apple/build-app.sh` で XCFramework 化＋`.app` を生成。詳細は `apple/README.md`。
（`demo/` は CLI 動作確認用の最小デモ。`demo/run.sh` が `main.swift` をビルド・実行する。）

## 実機統合の注意（次フェーズ）

- 本デモは debug `dylib` を `DYLD_LIBRARY_PATH` でロードする簡易形。
- 実アプリでは各ターゲット（arm64 macOS / arm64 iOS / iOS simulator）向けに**静的ライブラリを
  xcframework 化**して埋め込む。Android は同じ `qrac-ffi` から UniFFI Kotlin バインディング＋JNI。
- `uniffi` の版はピン留めし、生成バインディングと scaffolding を同一版に保つ（docs/08 8.10）。
