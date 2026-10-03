/** 75.4 -> "1:15", 3725 -> "1:02:05" */
export function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) seconds = 0;
  const s = Math.floor(seconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = String(s % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
}

export function formatDuration(ms: number | null): string {
  return ms == null ? "" : formatTime(ms / 1000);
}

const timeFmt = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
const dayFmt = new Intl.DateTimeFormat(undefined, { weekday: "long" });
const shortDateFmt = new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" });
const fullDateFmt = new Intl.DateTimeFormat(undefined, { year: "numeric", month: "short", day: "numeric" });
const longFmt = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });

function startOfDay(t: number) {
  const d = new Date(t);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

/** Compact, human date for cards: "Today, 21:04", "Yesterday", "Friday", "Mar 4", "Mar 4, 2024". */
export function formatRelativeDate(ms: number, now = Date.now()): string {
  const days = Math.round((startOfDay(now) - startOfDay(ms)) / 86_400_000);
  if (days <= 0) return `Today, ${timeFmt.format(ms)}`;
  if (days === 1) return "Yesterday";
  if (days < 7) return dayFmt.format(ms);
  if (new Date(ms).getFullYear() === new Date(now).getFullYear()) return shortDateFmt.format(ms);
  return fullDateFmt.format(ms);
}

export function formatLongDate(ms: number): string {
  return longFmt.format(ms);
}

export function formatSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024;
    i++;
  }
  return `${bytes.toFixed(i >= 3 ? 2 : i === 0 ? 0 : 1)} ${units[i]}`;
}

export function plural(n: number, word: string): string {
  return `${n.toLocaleString()} ${word}${n === 1 ? "" : "s"}`;
}
