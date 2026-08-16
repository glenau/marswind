#!/usr/bin/env bash
#
# Builds a release Marswind.app and installs it into /Applications.
#
# Two things this does that `npm run tauri build` on its own does not:
#
#   - builds the translation worker first, because Tauri bundles it as a sidecar
#     and the build fails without it;
#   - ad-hoc signs the finished bundle. Core Audio process taps refuse to work
#     in an unsigned app: the permission record is keyed to a signing identity
#     and its usage description.
#
# An ad-hoc signature is stable as long as the binary is, so the Audio Recording
# permission granted to this build survives launches - but a rebuild is a new
# identity and macOS asks again. A Developer ID certificate is what removes that.
#
# Usage: npm run install:macos

set -euo pipefail

cd "$(dirname "$0")/.."

APP="src-tauri/target/release/bundle/macos/Marswind.app"
DESTINATION="/Applications/Marswind.app"

echo "==> Building the translation worker"
bash scripts/build-sidecar.sh

echo "==> Building the release bundle (this takes a few minutes)"
npm run tauri build -- --bundles app

echo "==> Stamping the build number"
bash scripts/stamp-build.sh "$APP"

echo "==> Ad-hoc signing"
codesign --force --deep --sign - --identifier com.marswind.app "$APP"
codesign --verify --verbose=1 "$APP"

if [ -d "$DESTINATION" ]; then
	echo "==> Replacing $DESTINATION"
	rm -rf "$DESTINATION"
fi

echo "==> Installing to $DESTINATION"
cp -R "$APP" "$DESTINATION"

echo
echo "Installed. Open it with:"
echo "    open $DESTINATION"
echo
echo "On first launch macOS asks for the Audio Recording permission - say yes,"
echo "or nothing will be captured. Then open Settings in the app and download a"
echo "recognition model and a translation model."
