#!/usr/bin/env bash
# 配布用パッケージ（署名なし）を作る: .zip と .dmg を apple/dist/ に出力。
# 使い方:  bash apple/package-release.sh
# 前提  :  release ビルド（build-app.sh APP_CONFIG=release）。本スクリプトが自動で行う。
#
# ⚠️ 署名・公証なし(ad-hoc 署名のみ)。配布先 macOS では Gatekeeper の警告が出る。
#    解除方法は README の「Download」節を参照。
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DIST="$HERE/dist"
APP_DISPLAY="QrArtifactChronicle"          # 配布時の見た目の名前
SRC_APP="$HERE/${APP_DISPLAY}Release.app"  # build-app.sh release の出力

echo "1) release ビルド"
APP_CONFIG=release bash "$HERE/build-app.sh" >/dev/null
[ -d "$SRC_APP" ] || { echo "❌ $SRC_APP が無い"; exit 1; }

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$SRC_APP/Contents/Info.plist")"
BASENAME="${APP_DISPLAY}-${VERSION}-macos"
echo "   version: $VERSION"

echo "2) ステージング（${APP_DISPLAY}.app に改名・表示名を整える）"
rm -rf "$DIST"
STAGE="$DIST/stage"
mkdir -p "$STAGE"
STAGED_APP="$STAGE/${APP_DISPLAY}.app"
cp -R "$SRC_APP" "$STAGED_APP"
# 配布物では表示名を素直な QrArtifactChronicle に統一（release は …Release だった）。
/usr/libexec/PlistBuddy -c "Set :CFBundleName ${APP_DISPLAY}" "$STAGED_APP/Contents/Info.plist"
# Info.plist 改変で署名が無効化されるため ad-hoc で再署名（署名なし配布・公証なし）。
codesign --force --deep --sign - "$STAGED_APP" 2>/dev/null || true

echo "3) .zip（ditto・macOS 互換）"
ZIP="$DIST/${BASENAME}.zip"
( cd "$STAGE" && ditto -c -k --sequesterRsrc --keepParent "${APP_DISPLAY}.app" "$ZIP" )

echo "4) .dmg（hdiutil・/Applications へのドラッグ用シンボリックリンク同梱）"
DMGROOT="$DIST/dmgroot"
mkdir -p "$DMGROOT"
cp -R "$STAGED_APP" "$DMGROOT/"
ln -s /Applications "$DMGROOT/Applications"
DMG="$DIST/${BASENAME}.dmg"
hdiutil create -volname "$APP_DISPLAY" -srcfolder "$DMGROOT" -ov -format UDZO "$DMG" >/dev/null

echo "5) 後片付け＋チェックサム"
rm -rf "$STAGE" "$DMGROOT"
echo
echo "✅ 完成（apple/dist/）:"
( cd "$DIST" && shasum -a 256 "${BASENAME}.zip" "${BASENAME}.dmg" )
echo
echo "次の手順: GitHub の Releases にこの2ファイルをアップロード（署名・公証なし）。"
