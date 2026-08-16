/**
 * The captions as they arrive from the backend.
 *
 * A row is built from three events: recognition growing it, translation writing
 * into it a word at a time, and translation finishing a piece of it.
 *
 * A row is translated in **segments** - recognition hands them over as its words
 * are committed, without waiting for the row to end - so a row's translation is
 * the segments joined in order, with the one still being generated on the end.
 * Finished segments are authoritative and are never overwritten by a partial:
 * the pieces exist to put words on screen early, and a late one must not undo
 * the answer that followed it.
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SkippedEvent,
  TranscriptEvent,
  TranslationEvent,
  TranslationPartialEvent,
} from "./api";

export type CaptionLine = {
  id: number;
  /** Recognized words that will not change again. */
  text: string;
  /** Recognized words that may still be revised. */
  tentative: string;
  /** Recognition has finished this row. */
  done: boolean;
  /** How many segments this row was cut into. Known once the row is final. */
  expected: number;
  /** Translations of the segments that are complete, by segment index. */
  segments: string[];
  /** Segments the translator was too far behind to take. They are never
   *  coming, and a row waiting for one would say "translating…" forever. */
  skipped: Set<number>;
  /** The segment being generated right now, if any. */
  live: { seq: number; text: string } | null;
};

export type Captions = ReturnType<typeof createCaptions>;

/** Everything translated for a row so far, in order. */
export function translationOf(line: CaptionLine): string {
  const pieces = [...line.segments];
  if (line.live && !pieces[line.live.seq]) pieces[line.live.seq] = line.live.text;
  return pieces.filter(Boolean).join(" ");
}

/** Whether nothing more is coming for this row.
 *
 * A row is settled when recognition has finished it and every segment it was
 * cut into has either come back or been reported skipped. Without the skipped
 * half, a row whose segment the translator dropped kept its spinner for the
 * rest of the session. */
export function isTranslated(line: CaptionLine): boolean {
  if (!line.done || line.live !== null) return false;
  for (let seq = 0; seq < line.expected; seq += 1) {
    if (!line.segments[seq] && !line.skipped.has(seq)) return false;
  }
  return true;
}

/** A row that is finished and came back with holes in it. */
export function isIncomplete(line: CaptionLine): boolean {
  return line.done && line.skipped.size > 0;
}

export function createCaptions(limit = 300) {
  let lines = $state<CaptionLine[]>([]);
  let recognitionMs = $state(0);
  let translationMs = $state(0);
  let firstWordMs = $state(0);
  let unlisten: UnlistenFn[] = [];

  function find(id: number) {
    return lines.find((line) => line.id === id);
  }

  /** A translation can arrive before the row it belongs to has been drawn. */
  function ensure(id: number): CaptionLine {
    const existing = find(id);
    if (existing) return existing;

    const line: CaptionLine = {
      id,
      text: "",
      tentative: "",
      done: false,
      expected: 0,
      segments: [],
      skipped: new Set(),
      live: null,
    };
    lines.push(line);
    return line;
  }

  async function start() {
    unlisten.push(
      await listen<TranscriptEvent>("asr://transcript", (event) => {
        const { line, segments, text, tentative, final, inferenceMs } = event.payload;
        recognitionMs = inferenceMs;
        if (!text && !tentative) return;

        const existing = ensure(line);
        existing.text = text;
        existing.tentative = tentative;
        existing.done = final;
        existing.expected = segments;

        if (lines.length > limit) lines = lines.slice(-limit);
      }),
    );

    unlisten.push(
      await listen<TranslationPartialEvent>("translate://partial", (event) => {
        const { line, seq, text } = event.payload;
        const existing = ensure(line);
        // The finished text for this segment is already in; a partial arriving
        // after it is stale.
        if (existing.segments[seq]) return;
        existing.live = { seq, text };
      }),
    );

    unlisten.push(
      await listen<SkippedEvent>("translate://skipped", (event) => {
        const { line, seq } = event.payload;
        const existing = ensure(line);
        existing.skipped = new Set(existing.skipped).add(seq);
        if (existing.live?.seq === seq) existing.live = null;
      }),
    );

    unlisten.push(
      await listen<TranslationEvent>("translate://line", (event) => {
        const { line, seq, text, translationMs: total, firstWordMs: first } = event.payload;
        translationMs = total;
        firstWordMs = first;

        const existing = ensure(line);
        existing.segments[seq] = text;
        if (existing.live?.seq === seq) existing.live = null;
      }),
    );
  }

  function stop() {
    unlisten.forEach((fn) => fn());
    unlisten = [];
  }

  function clear() {
    lines = [];
  }

  return {
    get lines() {
      return lines;
    },
    get current(): CaptionLine | undefined {
      return lines[lines.length - 1];
    },
    get recognitionMs() {
      return recognitionMs;
    },
    get translationMs() {
      return translationMs;
    },
    get firstWordMs() {
      return firstWordMs;
    },
    start,
    stop,
    clear,
  };
}
