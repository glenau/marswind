#!/usr/bin/env bash
#
# Builds a release Marswind.app and packs it into a disk image to hand to
# somebody else.
#
# The order matters: the app is signed **before** the image is made, because
# what gets signed is the app and what gets copied out of the image is the app.
# Signing afterwards would leave the copy inside the image unsigned, and Core
# Audio process taps refuse to work in an unsigned app - the permission record
# is keyed to a signing identity and its usage description.
#
# The image is built with hdiutil rather than Tauri's own dmg bundler. Tauri's
# drives Finder through AppleScript to lay the window out, which needs a desktop
# session and hangs without one; this needs nothing and produces the same thing.
#
# Usage: npm run build:dmg

set -euo pipefail

cd "$(dirname "$0")/.."

APP="src-tauri/target/release/bundle/macos/Marswind.app"
VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")
ARCH=$(uname -m)
STAGE="src-tauri/target/dmg-stage"
DMG="src-tauri/target/Marswind-$VERSION-$ARCH.dmg"

echo "==> Building the translation worker"
bash scripts/build-sidecar.sh

echo "==> Building the release bundle (this takes a few minutes)"
npm run tauri build -- --bundles app

echo "==> Stamping the build number"
bash scripts/stamp-build.sh "$APP"

echo "==> Ad-hoc signing"
# An ad-hoc identity, for want of a Developer ID one. It is stable for a given
# binary, which is what lets a granted Audio Recording permission survive
# relaunches - but it is not notarized, so whoever opens this image has to let
# it through Gatekeeper by hand. See README.md.
codesign --force --deep --sign - --identifier com.marswind.app "$APP"
codesign --verify --verbose=1 "$APP"

echo "==> Staging"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
# The Applications symlink is the whole user interface of a disk image: open it,
# drag the icon onto the folder beside it.
ln -s /Applications "$STAGE/Applications"

echo "==> Packing $DMG"
rm -f "$DMG"
hdiutil create \
	-volname "Marswind $VERSION" \
	-srcfolder "$STAGE" \
	-fs HFS+ \
	-format UDZO \
	-quiet \
	"$DMG"
rm -rf "$STAGE"

# The digest the in-app updater checks the download against. It has to be a
# release asset of its own: GitHub publishes no checksum for an attachment, and
# the updater refuses a release that does not carry one rather than trusting
# whatever arrived. `shasum -c` reads this format directly, so a person can run
# the same check by hand.
echo "==> Writing $DMG.sha256"
(cd "$(dirname "$DMG")" && shasum -a 256 "$(basename "$DMG")" >"$(basename "$DMG").sha256")

echo
echo "Built: $DMG"
ls -lh "$DMG" | awk '{print "       " $5}'
awk '{print "       sha256 " $1}' "$DMG.sha256"
echo
echo "It is ad-hoc signed and NOT notarized. On another Mac, macOS will refuse to"
echo "open it on the first try - the way through is System Settings → Privacy &"
echo "Security → Open Anyway, after letting the launch be blocked once."
echo "(Right-click → Open did the same until macOS 15 removed it.)"
