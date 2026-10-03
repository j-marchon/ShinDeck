import type { Clip, EditSpec } from "../api";
import { formatMegabytes, formatTime } from "./format";
import { keptDuration, keptSegments, type TrimState } from "./trim";

/** A compression target picked in the edit panel. */
export interface CompressChoice {
  id: string;
  label: string;
  bytes: number;
}

/**
 * Edits staged in the panel. Nothing touches the disk until the user presses
 * Save, which applies all of them at once.
 */
export interface PendingEdits {
  name: string;
  /** Committed with "Done" in trim mode; null = keep the whole clip. */
  trim: TrimState | null;
  compress: CompressChoice | null;
}

export function noEdits(clip: Clip): PendingEdits {
  return { name: clip.name, trim: null, compress: null };
}

export function renameTo(pending: PendingEdits, clip: Clip): string | null {
  const name = pending.name.trim();
  return name && name !== clip.name ? name : null;
}

export function hasVideoEdits(pending: PendingEdits): boolean {
  return pending.trim !== null || pending.compress !== null;
}

export function hasEdits(pending: PendingEdits, clip: Clip): boolean {
  return hasVideoEdits(pending) || renameTo(pending, clip) !== null;
}

/** Export request for the staged trim/cut/compression (null if none). */
export function exportSpec(pending: PendingEdits): EditSpec | null {
  if (!hasVideoEdits(pending)) return null;
  const tags = [pending.trim && "trimmed", pending.compress?.label].filter(Boolean);
  return {
    keep: pending.trim ? keptSegments(pending.trim) : null,
    targetBytes: pending.compress?.bytes ?? null,
    label: tags.join(", "),
  };
}

export function jobLabel(pending: PendingEdits): string {
  const trimming = pending.trim ? (pending.trim.cuts.length ? "Cutting" : "Trimming") : null;
  if (trimming && pending.compress) return `${trimming} & compressing`;
  if (pending.compress) return `Compressing for ${pending.compress.label}`;
  return trimming ?? "Saving";
}

/** Human-readable list of what Save will do. */
export function describe(pending: PendingEdits, clip: Clip, duration: number): string[] {
  const lines: string[] = [];
  const name = renameTo(pending, clip);
  if (name) lines.push(`Rename to “${name}”`);
  if (pending.trim) {
    const cuts = pending.trim.cuts.length;
    lines.push(
      `Keep ${formatTime(keptDuration(pending.trim))} of ${formatTime(duration)}` +
        (cuts ? ` · ${cuts} cut${cuts > 1 ? "s" : ""}` : ""),
    );
  }
  if (pending.compress) lines.push(`Compress to ${formatMegabytes(pending.compress.bytes)} (${pending.compress.label})`);
  return lines;
}
