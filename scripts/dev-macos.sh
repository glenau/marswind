#!/usr/bin/env bash
#
# Builds and launches a debug .app bundle on macOS.
#
# `npm run tauri dev` runs a bare executable with no Info.plist and no code
# signature. Core Audio process taps refuse to work in that shape: the
# permission prompt is keyed to a signing identity and its usage description, so
# capture always fails. This script produces a real bundle, ad-hoc signs it, and
# launches it.
#
# Note that an ad-hoc signature changes on every build, so macOS treats each
# build as a new app and asks for the Audio Recording permission again. A
# Developer ID certificate is what removes that.

set -euo pipefail

cd "$(dirname "$0")/.."

APP="src-tauri/target/debug/bundle/macos/Marswind.app"

echo "==> Building the translation worker"
bash scripts/build-sidecar.sh

echo "==> Building debug bundle"
npm run tauri build -- --debug --bundles app

echo "==> Stamping the build number"
bash scripts/stamp-build.sh "$APP"

echo "==> Ad-hoc signing"
codesign --force --sign - --identifier com.marswind.app "$APP"
codesign --verify --verbose=1 "$APP"

echo "==> Launching"
open "$APP"
echo "Logs: log stream --predicate 'process == \"Marswind\"' --level info"
