#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
ruby scripts/generate-function-catalog.rb --check
cargo build --release --manifest-path hp41-bridge/Cargo.toml
swift build -c release

APP=".build/HP-41 Calculator.app"
VERSION="${HP41_VERSION:-$(sed -nE 's/^version = "([^"]+)"/\1/p' ../Cargo.toml | head -1)}"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp .build/release/hp41-gui "$APP/Contents/MacOS/hp41-gui"
cp ../app-icon-1024.png "$APP/Contents/Resources/AppIcon.png"
cp Resources/PrivacyInfo.xcprivacy "$APP/Contents/Resources/PrivacyInfo.xcprivacy"
plutil -create xml1 "$APP/Contents/Info.plist"
plutil -insert CFBundleExecutable -string hp41-gui "$APP/Contents/Info.plist"
plutil -insert CFBundleIdentifier -string ch.talent-factory.hp41 "$APP/Contents/Info.plist"
plutil -insert CFBundleName -string "HP-41 Calculator" "$APP/Contents/Info.plist"
plutil -insert CFBundleIconFile -string AppIcon.png "$APP/Contents/Info.plist"
plutil -insert CFBundlePackageType -string APPL "$APP/Contents/Info.plist"
plutil -insert CFBundleShortVersionString -string "$VERSION" "$APP/Contents/Info.plist"
plutil -insert LSMinimumSystemVersion -string 14.0 "$APP/Contents/Info.plist"
plutil -insert NSHighResolutionCapable -bool true "$APP/Contents/Info.plist"

echo "Built $APP"
