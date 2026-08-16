#!/usr/bin/env bash
#
# Regenerates the application icon, from the one script that draws it all the
# way to the .icns and .ico the bundle ships.
#
# Only needed when the icon itself changes - the results are committed, so a
# normal build never runs this.

set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> drawing assets/icon.svg"
python3 scripts/make-icon.py

# There is no SVG rasterizer in the standard toolchain, and adding a package
# manager to a build that already needs Rust, Node and cmake is not worth it for
# one PNG. Chrome is on nearly every machine and renders SVG exactly as the app
# will; anything else that can write a 1024×1024 PNG works just as well.
CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
if [[ ! -x "$CHROME" ]]; then
	echo "error: no rasterizer found at '$CHROME'." >&2
	echo "       set CHROME to a Chromium binary, or convert assets/icon.svg to" >&2
	echo "       a 1024x1024 assets/icon.png yourself and rerun from the step below." >&2
	exit 1
fi

echo "==> rasterizing assets/icon.png"
"$CHROME" --headless --disable-gpu \
	--screenshot=assets/icon.png \
	--window-size=1024,1024 \
	--default-background-color=00000000 \
	--hide-scrollbars \
	"file://$PWD/assets/icon.svg" >/dev/null 2>&1

# Quiet unless it fails: it narrates a mobile icon set this app has no use for.
echo "==> generating src-tauri/icons"
log=$(npx tauri icon assets/icon.png 2>&1) || {
	echo "$log" >&2
	exit 1
}

# The app is macOS and Windows. The mobile sets Tauri writes alongside them are
# several megabytes of files nothing in this repository reads.
rm -rf src-tauri/icons/android src-tauri/icons/ios

# The window's own favicon, which is what shows in a browser during `npm run dev`.
cp src-tauri/icons/128x128.png static/favicon.png

echo "==> done"
ls -1 src-tauri/icons | sed 's/^/    /'
