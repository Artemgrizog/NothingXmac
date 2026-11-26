#!/bin/bash
# Fast dev loop: debug build wrapped in an app bundle (needed for Bluetooth permission).
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."
VER=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
cargo build
APP=target/dev/NothingX.app
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/debug/nothingx "$APP/Contents/MacOS/NothingX"
sed "s/@VERSION@/$VER/g" packaging/Info.plist > "$APP/Contents/Info.plist"
cp -R packaging/en.lproj packaging/ru.lproj "$APP/Contents/Resources/"
[ -f dist/NothingX.app/Contents/Resources/AppIcon.icns ] && cp dist/NothingX.app/Contents/Resources/AppIcon.icns "$APP/Contents/Resources/" || true
# Stable designated requirement (dev only): macOS keeps the Bluetooth permission
# across rebuilds instead of asking again for every new ad-hoc signature.
codesign --force --sign - --requirements '=designated => identifier "dev.nothingx.mac"' "$APP" 2>/dev/null
pkill -x NothingX 2>/dev/null || true
for _ in $(seq 1 50); do pgrep -x NothingX >/dev/null || break; sleep 0.1; done
open "$APP"
