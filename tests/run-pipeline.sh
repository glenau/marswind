#!/usr/bin/env bash
#
# The whole chain against the fixture corpus: capture, recognition, translation.
#
# Each clip is played through the system output and comes back through the real
# pipeline, so this measures what a user gets. Recognition is scored with word
# error rate, translation with chrF against a reference Russian translation, and
# both are timed against an estimate of when each line was spoken.
#
# Recognition varies run to run - see tests/README.md - so a single run is a
# smoke test and a moved median across several runs is a result.
#
# Usage: tests/run-pipeline.sh [--model ID] [--mt-model ID] [--runs N]
#                              [--label NAME] [--out DIR] [name ...]

set -uo pipefail

cd "$(dirname "$0")/.."

MODEL=${MARSWIND_TEST_MODEL:-large-v3-turbo-q5_0}
MT_MODEL=${MARSWIND_TEST_MT_MODEL:-qwen3-4b-instruct-q4}
RUNS=1
LABEL=run
OUT=""
names=()

while [ $# -gt 0 ]; do
	case "$1" in
	--model) MODEL=$2; shift 2 ;;
	--mt-model) MT_MODEL=$2; shift 2 ;;
	--runs) RUNS=$2; shift 2 ;;
	--label) LABEL=$2; shift 2 ;;
	--out) OUT=$2; shift 2 ;;
	*) names+=("$1"); shift ;;
	esac
done

BIN=${MARSWIND_BIN:-src-tauri/target/debug/bundle/macos/Marswind.app/Contents/MacOS/marswind}
if [ ! -x "$BIN" ]; then
	echo "No app bundle at $BIN"
	echo "Build and sign one first: npm run dev:macos"
	exit 1
fi

if [ ${#names[@]} -eq 0 ]; then
	names=($(python3 -c "
import json
print(' '.join(e['name'] for e in json.load(open('tests/fixtures/manifest.json'))))
"))
fi

OUT=${OUT:-$(mktemp -d)}
mkdir -p "$OUT"
echo "logs in $OUT"

for name in "${names[@]}"; do
	audio="tests/fixtures/$name.wav"
	seconds=$(python3 -c "
import wave, sys
with wave.open(sys.argv[1]) as w:
    print(int(w.getnframes() / w.getframerate()) + 8)
" "$audio")

	for run in $(seq 1 "$RUNS"); do
		log="$OUT/$LABEL-$name-$run.log"
		MARSWIND_SELFTEST="pipeline:$seconds" \
			MARSWIND_SELFTEST_MODEL="$MODEL" \
			MARSWIND_SELFTEST_MT_MODEL="$MT_MODEL" \
			MARSWIND_SELFTEST_PLAY="$audio" \
			"$BIN" >"$log" 2>&1
		python3 tests/score.py "$name" "$log" \
			>"$OUT/$LABEL-$name-$run.json" 2>"$OUT/$LABEL-$name-$run.detail"
		cat "$OUT/$LABEL-$name-$run.detail" >&2
	done
done

python3 tests/report.py "$OUT"
