#!/bin/bash
# Build NothingX.app and NothingX.dmg (Apple Silicon, ad-hoc signed).
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."
ROOT=$PWD
OUT="$ROOT/dist"
APP="$OUT/NothingX.app"
VER=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)

cargo build --release --target aarch64-apple-darwin
BIN="$ROOT/target/aarch64-apple-darwin/release/nothingx"

rm -rf "$OUT" && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/NothingX"

# icon (also used as the DMG volume icon)
ICONSET="$OUT/AppIcon.iconset"; mkdir -p "$ICONSET"
"$BIN" --render-icon "$OUT/icon1024.png"
for s in 16 32 128 256 512; do
  sips -z $s $s "$OUT/icon1024.png" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z $((s*2)) $((s*2)) "$OUT/icon1024.png" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$ICONSET" "$OUT/icon1024.png"

sed "s/@VERSION@/$VER/g" packaging/Info.plist > "$APP/Contents/Info.plist"

codesign --force --deep --sign - "$APP"

# Drag-and-drop DMG: background with an arrow, app on the left, Applications on the right.
DMG="$OUT/NothingX-$VER.dmg"
if [ ! -x .venv-dmg/bin/dmgbuild ]; then
  python3 -m venv .venv-dmg
  .venv-dmg/bin/pip install -q dmgbuild
fi
"$BIN" --render-dmg-background "$OUT/dmg-bg.png" "$OUT/dmg-bg@2x.png"
tiffutil -cathidpicheck "$OUT/dmg-bg.png" "$OUT/dmg-bg@2x.png" -out "$OUT/dmg-bg.tiff" >/dev/null
rm -f "$DMG"
.venv-dmg/bin/dmgbuild -s packaging/dmg_settings.py \
  -D app="$APP" -D background="$OUT/dmg-bg.tiff" -D icon="$APP/Contents/Resources/AppIcon.icns" \
  "NothingX" "$DMG"
rm -f "$OUT/dmg-bg.png" "$OUT/dmg-bg@2x.png" "$OUT/dmg-bg.tiff"
echo "Built: $APP"
echo "Built: $DMG"
