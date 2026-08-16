#!/usr/bin/env bash
#
# Regenerates the audio fixtures from their scripts using the macOS speech
# synthesiser.
#
# The generated .wav files are committed, so running the tests needs neither
# macOS nor a particular set of installed voices. This script exists so the
# corpus can be extended or rebuilt, not because the tests call it.
#
# Usage: tests/generate-fixtures.sh [name ...]

set -euo pipefail

cd "$(dirname "$0")/fixtures"

names=("$@")
if [ ${#names[@]} -eq 0 ]; then
	names=($(python3 -c "
import json
print(' '.join(entry['name'] for entry in json.load(open('manifest.json'))))
"))
fi

for name in "${names[@]}"; do
	# One field per line: an empty secondVoice must stay empty rather than
	# letting the next field slide into its place.
	{
		read -r voice
		read -r second_voice
		read -r rate
	} < <(python3 -c "
import json, sys
name = sys.argv[1]
entry = next(e for e in json.load(open('manifest.json')) if e['name'] == name)
print(entry['voice'])
print(entry.get('secondVoice', ''))
print(entry['rate'])
" "$name")

	echo "==> $name (${voice}${second_voice:+ + $second_voice} at ${rate} wpm)"
	work=$(mktemp -d)

	if [ -n "$second_voice" ]; then
		# Alternate voices sentence by sentence, then stitch the pieces back
		# together so the clip changes speaker mid-stream.
		python3 - "$name.txt" "$work" <<-'PY'
			import re, sys
			text = open(sys.argv[1]).read()
			sentences = [s.strip() for s in re.split(r"(?<=[.!?])\s+", text) if s.strip()]
			for index, sentence in enumerate(sentences):
			    open(f"{sys.argv[2]}/part-{index:03d}.txt", "w").write(sentence)
		PY

		for part in "$work"/part-*.txt; do
			index=$(basename "$part" .txt | cut -d- -f2)
			speaker=$voice
			if [ $((10#$index % 2)) -eq 1 ]; then speaker=$second_voice; fi
			say -v "$speaker" -r "$rate" -o "$work/$index.aiff" -f "$part"
			afconvert -f WAVE -d LEI16@16000 -c 1 "$work/$index.aiff" "$work/$index.wav"
		done

		python3 - "$work" "$name.wav" <<-'PY'
			import glob, sys, wave
			parts = sorted(glob.glob(f"{sys.argv[1]}/*.wav"))
			with wave.open(sys.argv[2], "wb") as out:
			    for index, path in enumerate(parts):
			        with wave.open(path) as part:
			            if index == 0:
			                out.setparams(part.getparams())
			            out.writeframes(part.readframes(part.getnframes()))
		PY
	else
		say -v "$voice" -r "$rate" -o "$work/clip.aiff" -f "$name.txt"
		afconvert -f WAVE -d LEI16@16000 -c 1 "$work/clip.aiff" "$name.wav"
	fi

	rm -rf "$work"
	python3 -c "
import sys, wave
with wave.open('$name.wav') as w:
    print(f'    {w.getnframes() / w.getframerate():.1f}s, {w.getframerate()} Hz, {w.getnchannels()} ch')
"
done

echo "==> reference tone"
python3 - <<'PY'
import math, struct, wave

RATE, SECONDS, FREQ, LEVEL = 48000, 5, 440.0, 0.35
with wave.open("tone-440.wav", "wb") as out:
    out.setnchannels(2)
    out.setsampwidth(2)
    out.setframerate(RATE)
    frames = bytearray()
    for i in range(RATE * SECONDS):
        value = int(LEVEL * 32767 * math.sin(2 * math.pi * FREQ * i / RATE))
        frames += struct.pack("<hh", value, value)
    out.writeframes(bytes(frames))
print("    5.0s, 48000 Hz, 2 ch")
PY
