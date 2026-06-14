#!/usr/bin/env bash
# Rust コア → xcframework → SwiftUI アプリ(.app) を一括ビルドする。
# 使い方: apple/ で  bash build-app.sh   → apple/QrArtifactChronicle.app
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
RUST="$ROOT/rust"

# APP_CONFIG=debug(既定) | release
#   debug  : デバッグ入力パネルあり（開発用）→ QrArtifactChronicle.app
#   release: #if DEBUG が外れ手入力等を除外（本番想定）→ QrArtifactChronicleRelease.app
CONFIG="${APP_CONFIG:-debug}"
if [ "$CONFIG" = "release" ]; then
  SWIFT_BUILD_FLAGS="-c release"
  APP_NAME="QrArtifactChronicleRelease"
  BUNDLE_ID="com.example.qrartifactchronicle.release"
else
  SWIFT_BUILD_FLAGS=""
  APP_NAME="QrArtifactChronicle"
  BUNDLE_ID="com.example.qrartifactchronicle"
fi
echo "== build config: $CONFIG → $APP_NAME.app =="

echo "1) Rust release ビルド（cdylib + staticlib）"
( cd "$RUST" && cargo build -p qrac-ffi --release )

echo "2) UniFFI Swift バインディング生成（cdylib から）"
( cd "$RUST" && cargo run -q -p qrac-ffi --bin uniffi-bindgen -- \
    generate --library target/release/libqrac_ffi.dylib --language swift --out-dir target/swift )

echo "3) ヘッダ/Swift を配置し XCFramework を作成（macOS arm64）"
rm -rf "$HERE/headers" "$HERE/QracFFI.xcframework"
mkdir -p "$HERE/headers"
cp "$RUST/target/swift/qrac_ffiFFI.h" "$HERE/headers/"
cp "$RUST/target/swift/qrac_ffiFFI.modulemap" "$HERE/headers/module.modulemap"
cp "$RUST/target/swift/qrac_ffi.swift" "$HERE/QracKit/Sources/QracFFI/qrac_ffi.swift"
xcodebuild -create-xcframework \
    -library "$RUST/target/release/libqrac_ffi.a" -headers "$HERE/headers" \
    -output "$HERE/QracFFI.xcframework" >/dev/null

echo "4) SwiftUI アプリを $CONFIG ビルド"
( cd "$HERE/QracKit" && swift build $SWIFT_BUILD_FLAGS 2>&1 | grep -vE "ld: warning|was built for newer" || true )
BIN="$(cd "$HERE/QracKit" && swift build $SWIFT_BUILD_FLAGS --show-bin-path)/QrArtifactApp"

echo "5) アセット生成（ベース画像＋DB, カバレッジゲート）"
( cd "$RUST" && cargo run -q -p qrac-assetgen -- "$HERE/.assets-build" )

echo "6) .app バンドルを組み立て（アセット同梱）"
APP="$HERE/$APP_NAME.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/QrArtifactApp"
cp -R "$HERE/.assets-build/assets" "$APP/Contents/Resources/assets"
cp -R "$HERE/sounds" "$APP/Contents/Resources/sounds"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundleExecutable</key><string>QrArtifactApp</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.2.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSPrincipalClass</key><string>NSApplication</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSCameraUsageDescription</key><string>QR code scanning to excavate artifacts. / QRコードを読み取って遺物を発掘するためにカメラを使用します。</string>
</dict></plist>
PLIST

plutil -lint "$APP/Contents/Info.plist"

# macOS のカメラ権限(TCC)を確実にするため ad-hoc 署名する。
codesign --force --sign - "$APP/Contents/MacOS/QrArtifactApp" 2>/dev/null || true
codesign --force --sign - "$APP" 2>/dev/null || true

echo "✅ 完成: $APP"
echo "   起動: open '$APP'   （未署名のため初回は右クリック→開く）"
