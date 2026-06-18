#!/usr/bin/env bash
# Rust コア(qrac-ffi) → マルチプラットフォーム XCFramework を作る。
#   スライス: macOS(arm64) + iOS device(arm64) + iOS simulator(arm64)
# UniFFI の Swift バインディングも生成して QracKit に配置する。
# 使い方: apple/ で  bash build-xcframework.sh
#
# 注意: iOS ターゲットの std は rustup 管理の toolchain にある。一方 PATH 先頭の cargo は
#       Homebrew 版のことがあり iOS std を持たない。そこで iOS ビルドだけ rustup の
#       toolchain を絶対パスで使う（macOS スライスは既存どおり PATH の cargo）。
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
RUST="$ROOT/rust"

IOS_TARGETS=("aarch64-apple-ios" "aarch64-apple-ios-sim")

# iOS 用 rustup toolchain（無ければ案内して終了）。
RUSTUP_CARGO="$(rustup which --toolchain stable cargo 2>/dev/null || true)"
RUSTUP_RUSTC="$(rustup which --toolchain stable rustc 2>/dev/null || true)"
if [ -z "$RUSTUP_CARGO" ]; then
  echo "❌ rustup の stable toolchain が見つかりません。"
  echo "   iOS ビルドには rustup が必要です: https://rustup.rs"
  echo "   導入後: rustup target add ${IOS_TARGETS[*]}"
  exit 1
fi

echo "1) UniFFI Swift バインディング生成（macOS cdylib から）"
( cd "$RUST" && cargo build -p qrac-ffi --release )
( cd "$RUST" && cargo run -q -p qrac-ffi --bin uniffi-bindgen -- \
    generate --library target/release/libqrac_ffi.dylib --language swift --out-dir target/swift )

echo "2) Rust 静的ライブラリをビルド"
echo "   - macOS (host, PATH の cargo)"
( cd "$RUST" && cargo build -p qrac-ffi --release )
for T in "${IOS_TARGETS[@]}"; do
  # iOS ターゲットが未導入なら自動で追加。
  rustup target list --toolchain stable --installed 2>/dev/null | grep -qx "$T" \
    || rustup target add --toolchain stable "$T"
  echo "   - $T (rustup toolchain)"
  ( cd "$RUST" && RUSTC="$RUSTUP_RUSTC" "$RUSTUP_CARGO" build -p qrac-ffi --release --target "$T" )
done

echo "3) ヘッダ/Swift を配置"
rm -rf "$HERE/headers" "$HERE/QracFFI.xcframework"
mkdir -p "$HERE/headers"
cp "$RUST/target/swift/qrac_ffiFFI.h" "$HERE/headers/"
cp "$RUST/target/swift/qrac_ffiFFI.modulemap" "$HERE/headers/module.modulemap"
cp "$RUST/target/swift/qrac_ffi.swift" "$HERE/QracKit/Sources/QracFFI/qrac_ffi.swift"

echo "4) XCFramework を作成（macOS + iOS device + iOS sim）"
xcodebuild -create-xcframework \
    -library "$RUST/target/release/libqrac_ffi.a" -headers "$HERE/headers" \
    -library "$RUST/target/aarch64-apple-ios/release/libqrac_ffi.a" -headers "$HERE/headers" \
    -library "$RUST/target/aarch64-apple-ios-sim/release/libqrac_ffi.a" -headers "$HERE/headers" \
    -output "$HERE/QracFFI.xcframework" >/dev/null

echo "✅ 完成: $HERE/QracFFI.xcframework"
xcodebuild -version >/dev/null 2>&1 && ls "$HERE/QracFFI.xcframework"
