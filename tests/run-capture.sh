#!/usr/bin/env bash
#
# Audio capture fidelity.
#
# Plays a reference tone through the system output, captures it back through the
# tap and the resampler, and checks that what came out is what went in: the same
# amplitude, the same frequency, and no aliasing from the rate conversion.
#
# Usage: tests/run-capture.sh

set -uo pipefail

cd "$(dirname "$0")/.."

BIN=${MARSWIND_BIN:-src-tauri/target/debug/bundle/macos/Marswind.app/Contents/MacOS/marswind}
if [ ! -x "$BIN" ]; then
	echo "No app bundle at $BIN"
	echo "Build and sign one first: npm run dev:macos"
	exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "==> capturing the reference tone"
MARSWIND_SELFTEST=capture:4 \
	MARSWIND_SELFTEST_PLAY=tests/fixtures/tone-440.wav \
	MARSWIND_SELFTEST_OUT="$work/captured.wav" \
	"$BIN" 2>&1 | grep -E "SELFTEST (format|result|FAIL)"

python3 tests/analyze-tone.py "$work/captured.wav"
