#!/usr/bin/env bash
#
# Writes the build number into a freshly bundled Marswind.app.
#
# macOS shows the two version keys together in the About window, as
# "Version <CFBundleShortVersionString> (<CFBundleVersion>)". Tauri writes the
# same string into both, so the panel read "Version 0.1.0 (0.1.0)" - the number
# printed twice, in a place meant to tell two builds of one version apart.
#
# The build number is the commit count: monotonic, meaningful, and nobody has to
# remember to bump it. Outside a git checkout it falls back to 1, which is what
# the Info.plist says on its own.
#
# Must run BEFORE codesign - Info.plist is covered by the signature.
#
# Usage: bash scripts/stamp-build.sh <path to Marswind.app>

set -euo pipefail

APP="${1:?usage: stamp-build.sh <path to .app>}"
PLIST="$APP/Contents/Info.plist"

BUILD=$(git rev-list --count HEAD 2>/dev/null || echo 1)

/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD" "$PLIST" 2>/dev/null ||
	/usr/libexec/PlistBuddy -c "Add :CFBundleVersion string $BUILD" "$PLIST"

echo "    build $BUILD"
