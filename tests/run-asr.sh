#!/usr/bin/env bash
#
# Recognition accuracy against the fixture corpus.
#
# Each clip is played through the system output and captured back through the
# real pipeline - tap, resampler, VAD, whisper - so this exercises the same path
# a user gets, not a private shortcut.
#
# Recognition varies run to run, so each clip is measured several times and
# scored on the median. Thresholds live in fixtures/manifest.json.
#
# Usage: tests/run-asr.sh [--model ID] [--runs N] [name ...]

set -uo pipefail

cd "$(dirname "$0")/.."

MODEL=${MARSWIND_TEST_MODEL:-large-v3-turbo-q5_0}
RUNS=3
names=()

while [ $# -gt 0 ]; do
	case "$1" in
	--model)
		MODEL=$2
		shift 2
		;;
	--runs)
		RUNS=$2
		shift 2
		;;
	*)
		names+=("$1")
		shift
		;;
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

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

printf '%-22s %8s %8s %s\n' "CLIP" "WER" "LIMIT" "RESULT"

for name in "${names[@]}"; do
	limit=$(python3 -c "
import json, sys
entry = next(e for e in json.load(open('tests/fixtures/manifest.json')) if e['name'] == sys.argv[1])
print(entry['maxWer'])
" "$name")

	audio="tests/fixtures/$name.wav"
	reference="tests/fixtures/$name.txt"
	seconds=$(python3 -c "
import wave, sys
with wave.open(sys.argv[1]) as w:
    print(int(w.getnframes() / w.getframerate()) + 8)
" "$audio")

	scores=()
	for run in $(seq 1 "$RUNS"); do
		log="$work/$name-$run.log"
		MARSWIND_SELFTEST="asr:$seconds" \
			MARSWIND_SELFTEST_MODEL="$MODEL" \
			MARSWIND_SELFTEST_PLAY="$audio" \
			"$BIN" >"$log" 2>&1
		score=$(python3 tests/wer.py "$reference" "$log" 2>"$work/$name-$run.detail")
		scores+=("${score:-1.0}")
	done

	median=$(python3 -c "
import sys
values = sorted(float(v) for v in sys.argv[1:])
print(f'{values[len(values) // 2]:.4f}')
" "${scores[@]}")

	if python3 -c "import sys; sys.exit(0 if float(sys.argv[1]) <= float(sys.argv[2]) else 1)" "$median" "$limit"; then
		result="pass"
	else
		result="FAIL"
		failures=$((failures + 1))
	fi

	printf '%-22s %7.1f%% %7.1f%% %s   (runs: %s)\n' \
		"$name" \
		"$(python3 -c "print(float('$median') * 100)")" \
		"$(python3 -c "print(float('$limit') * 100)")" \
		"$result" \
		"$(python3 -c "print(', '.join(f'{float(v) * 100:.1f}%' for v in '${scores[*]}'.split()))")"

	if [ "$result" = "FAIL" ]; then
		cat "$work/$name-1.detail"
	fi
done

if [ "$failures" -gt 0 ]; then
	echo
	echo "$failures clip(s) above threshold"
	exit 1
fi
