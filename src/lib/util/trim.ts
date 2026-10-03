/** Trim & cut model: keep [start, end] minus every cut range. Seconds. */
export interface TrimState {
  start: number;
  end: number;
  cuts: [number, number][];
  /** First marker of a cut being placed (right-click), awaiting the second. */
  pending: number | null;
}

export const MIN_KEEP = 0.5;
const SLIVER = 0.05;

export function newTrim(duration: number): TrimState {
  return { start: 0, end: duration, cuts: [], pending: null };
}

/** Sorted, merged cut ranges. */
export function mergeCuts(cuts: [number, number][]): [number, number][] {
  const sorted = cuts
    .map(([a, b]) => (a <= b ? [a, b] : [b, a]) as [number, number])
    .filter(([a, b]) => b - a >= SLIVER)
    .sort((x, y) => x[0] - y[0]);
  const out: [number, number][] = [];
  for (const [a, b] of sorted) {
    const last = out[out.length - 1];
    if (last && a <= last[1]) last[1] = Math.max(last[1], b);
    else out.push([a, b]);
  }
  return out;
}

/** The ranges that end up in the exported clip. */
export function keptSegments(t: TrimState): [number, number][] {
  const segments: [number, number][] = [];
  let cursor = t.start;
  for (const [a, b] of mergeCuts(t.cuts)) {
    if (b <= t.start || a >= t.end) continue;
    if (a > cursor) segments.push([cursor, a]);
    cursor = Math.max(cursor, b);
  }
  if (t.end > cursor) segments.push([cursor, t.end]);
  return segments.filter(([a, b]) => b - a >= SLIVER);
}

export function keptDuration(t: TrimState): number {
  return keptSegments(t).reduce((sum, [a, b]) => sum + (b - a), 0);
}

export function isUnchanged(t: TrimState, duration: number): boolean {
  return t.start <= SLIVER && t.end >= duration - SLIVER && keptSegments(t).length === 1;
}

/** Where playback should jump to skip a removed part (or null to continue). */
export function skipTarget(t: TrimState, time: number): number | null {
  if (time < t.start - SLIVER) return t.start;
  for (const [a, b] of mergeCuts(t.cuts)) if (time >= a && time < b - SLIVER) return b;
  return null;
}

/** A tick spacing that gives roughly 6–12 labels across the ruler. */
export function rulerStep(duration: number): number {
  const steps = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600];
  return steps.find((s) => duration / s <= 12) ?? 1200;
}
