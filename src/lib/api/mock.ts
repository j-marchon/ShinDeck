/**
 * Browser-only stand-in for the Tauri backend, used by `npm run dev` when the
 * page is opened in a normal browser. Generates a believable library so the
 * UI can be developed and screenshotted without Windows or real clips.
 * Drop any `sample.webm` into `dev-media/` (git-ignored) to test playback.
 */
import type { Backend, Batch, Clip, Game, Library, Renamed, Settings } from "./types";

const GAMES: [string, number, number][] = [
  // name, clip count, hue
  ["Counter-strike 2", 46, 32],
  ["Valorant", 28, 352],
  ["Apex Legends", 19, 8],
  ["Rocket League", 23, 210],
  ["ELDEN RING", 12, 45],
  ["Cyberpunk 2077", 9, 58],
  ["Fortnite", 15, 270],
  ["Overwatch 2", 5, 28],
  ["Dead by Daylight", 14, 0],
  ["Grand Theft Auto V", 4, 130],
  ["Ready Or Not", 4, 190],
  ["Resident Evil 2 Biohazard RE2", 2, 345],
  ["Slapshot Rebound", 26, 330],
  ["Helldivers 2", 11, 50],
  ["Desktop", 4, 200],
];

const pad = (n: number, w = 2) => String(n).padStart(w, "0");
const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

function shadowplayName(game: string, d: Date) {
  return `${game} ${d.getFullYear()}.${pad(d.getMonth() + 1)}.${pad(d.getDate())} - ${pad(d.getHours())}.${pad(
    d.getMinutes(),
  )}.${pad(d.getSeconds())}.${pad(d.getMilliseconds() % 99)}.DVR`;
}

let root = "C:\\Videos";
const favorites = new Set<string>();
const edited = new Set<string>();
let clips: Clip[] = [];
let settings: Settings = { libraryPath: null, setupComplete: false, saveMode: null, confirmDelete: true, editingAvailable: true };
const progressHandlers = new Set<(p: number) => void>();
let cancelled = false;
const mergeCounts = new Map<string, number>();

function generate() {
  let seed = 7;
  const rand = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  clips = [];
  for (const [game, count] of GAMES) {
    for (let i = 0; i < count; i++) {
      const date = Date.now() - Math.floor(rand() * 500 * 86400_000);
      const name = shadowplayName(game, new Date(date));
      clips.push(makeClip(game, name, date, Math.floor(30e6 + rand() * 400e6), Math.floor(15_000 + rand() * 285_000)));
    }
  }
}

function makeClip(game: string, name: string, date: number, size: number, durationMs: number): Clip {
  const id = `${game}/${name}.mp4`;
  return {
    id,
    name,
    path: `${root}\\${game}\\${name}.mp4`,
    game,
    size,
    date,
    modified: date,
    durationMs,
    favorite: favorites.has(id),
    edited: edited.has(id),
  };
}

function library(): Library {
  const games: Game[] = GAMES.map(([name]) => {
    const own = clips.filter((c) => c.game === name);
    return { id: name, name, clipCount: own.length, latest: Math.max(0, ...own.map((c) => c.date)) };
  })
    .filter((g) => g.clipCount > 0)
    .sort((a, b) => a.name.localeCompare(b.name));
  return { root, games, clips: clips.map((c) => ({ ...c, favorite: favorites.has(c.id), edited: edited.has(c.id) })) };
}

function svgUrl(svg: string) {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

function hueOf(game: string) {
  return GAMES.find(([g]) => g === game)?.[2] ?? 120;
}

function find(id: string) {
  const clip = clips.find((c) => c.id === id);
  if (!clip) throw new Error("Clip not found");
  return clip;
}

export const mockBackend: Backend = {
  async getSettings() {
    return settings;
  },
  async defaultClipsFolder() {
    return "C:\\Users\\Player\\Videos\\NVIDIA";
  },
  async inspectFolder(path) {
    await delay(150);
    const exists = /^[a-z]:\\/i.test(path);
    return exists
      ? { exists, gameCount: GAMES.length, clipCount: GAMES.reduce((n, g) => n + g[1], 0) }
      : { exists, gameCount: 0, clipCount: 0 };
  },
  async setLibraryPath(path) {
    root = path;
    settings = { ...settings, libraryPath: path, setupComplete: true };
    generate();
    return settings;
  },
  async scanLibrary() {
    await delay(250);
    if (!clips.length) generate();
    return library();
  },
  async setFavorite(id, favorite) {
    if (favorite) favorites.add(id);
    else favorites.delete(id);
  },
  async setFavorites(ids, favorite) {
    for (const id of ids) {
      if (favorite) favorites.add(id);
      else favorites.delete(id);
    }
  },
  async renameClips(ids, name) {
    await delay(200);
    name = name.trim();
    if (!name) throw new Error("The name can't be empty");
    if (/[<>:"/\\|?*]/.test(name)) throw new Error("Names can't contain < > : \" / \\ | ? *");
    const pattern = new RegExp(`^${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")} #(\\d+)$`, "i");
    let next = Math.max(0, ...clips.map((c) => Number(pattern.exec(c.name)?.[1] ?? 0))) + 1;
    const batch: Batch<Renamed> = { done: [], failed: 0, error: null };
    for (const id of ids) {
      const clip = clips.find((c) => c.id === id);
      if (!clip) {
        batch.failed++;
        batch.error ??= "Clip not found; it may have been moved or deleted";
        continue;
      }
      batch.done.push({ from: id, clip: await this.renameClip(id, `${name} #${next++}`) });
    }
    return batch;
  },
  async revealClip(id) {
    console.info("[mock] reveal", id);
  },
  async renameClip(id, name) {
    await delay(120);
    name = name.trim();
    if (!name) throw new Error("The name can't be empty");
    if (/[<>:"/\\|?*]/.test(name)) throw new Error("Names can't contain < > : \" / \\ | ? *");
    const clip = find(id);
    const renamed = makeClip(clip.game, name, clip.date, clip.size, clip.durationMs ?? 0);
    if (renamed.id !== id && clips.some((c) => c.id === renamed.id)) throw new Error("Another clip already has that name");
    for (const set of [favorites, edited]) if (set.delete(id)) set.add(renamed.id);
    clips = clips.map((c) => (c.id === id ? renamed : c));
    return { ...renamed, favorite: favorites.has(renamed.id), edited: edited.has(renamed.id) };
  },
  async exportClip(id, spec, destination) {
    cancelled = false;
    const clip = find(id);
    for (let p = 0; p <= 1.0001; p += 0.05) {
      if (cancelled) throw new Error("Cancelled");
      progressHandlers.forEach((h) => h(Math.min(1, p)));
      await delay(70);
    }
    const kept = spec.keep ? spec.keep.reduce((t, [a, b]) => t + (b - a), 0) * 1000 : (clip.durationMs ?? 0);
    const size = spec.targetBytes ? Math.round(spec.targetBytes * 0.93) : Math.round((clip.size * kept) / (clip.durationMs || 1));
    const now = Date.now();
    if (destination === "replace") {
      const updated = { ...clip, size, durationMs: Math.round(kept), modified: now };
      clips = clips.map((c) => (c.id === id ? updated : c));
      edited.add(id);
      return { ...updated, edited: true, favorite: favorites.has(id) };
    }
    let name = `${clip.name} (${spec.label})`;
    for (let n = 2; clips.some((c) => c.name === name && c.game === clip.game); n++) name = `${clip.name} (${spec.label}) (${n})`;
    const created = { ...makeClip(clip.game, name, clip.date, size, Math.round(kept)), modified: now };
    clips = [...clips, created];
    edited.add(created.id);
    return { ...created, edited: true };
  },
  async mergeClips(ids, destination) {
    cancelled = false;
    const parts = ids.map(find);
    for (let p = 0; p <= 1.0001; p += 0.04) {
      if (cancelled) throw new Error("Cancelled");
      progressHandlers.forEach((h) => h(Math.min(1, p)));
      await delay(70);
    }
    const [first] = parts;
    let n = (mergeCounts.get(first.game) ?? 0) + 1;
    const stem = first.game || "Unsorted";
    while (clips.some((c) => c.game === first.game && c.name === `${stem} Merge #${n}`)) n++;
    mergeCounts.set(first.game, n);
    const durationMs = parts.reduce((t, c) => t + (c.durationMs ?? 0), 0);
    const size = parts.reduce((t, c) => t + c.size, 0);
    const created = { ...makeClip(first.game, `${stem} Merge #${n}`, first.date, size, durationMs), modified: Date.now() };
    const removed = destination === "replace" ? ids : [];
    const favorite = removed.some((id) => favorites.has(id));
    for (const id of removed) {
      favorites.delete(id);
      edited.delete(id);
    }
    clips = [...clips.filter((c) => !removed.includes(c.id)), created];
    edited.add(created.id);
    if (favorite) favorites.add(created.id);
    return { clip: { ...created, edited: true, favorite }, removed };
  },
  async cancelExport() {
    cancelled = true;
  },
  async onExportProgress(handler) {
    progressHandlers.add(handler);
    return () => progressHandlers.delete(handler);
  },
  async setSaveMode(mode) {
    settings = { ...settings, saveMode: mode };
    return settings;
  },
  async setConfirmDelete(confirm) {
    settings = { ...settings, confirmDelete: confirm };
    return settings;
  },
  async deleteClip(id) {
    await delay(120);
    find(id);
    favorites.delete(id);
    edited.delete(id);
    clips = clips.filter((c) => c.id !== id);
  },
  async deleteClips(ids) {
    await delay(200);
    const batch: Batch<string> = { done: [], failed: 0, error: null };
    for (const id of ids) {
      if (!clips.some((c) => c.id === id)) {
        batch.failed++;
        batch.error ??= "Clip not found";
        continue;
      }
      await this.deleteClip(id);
      batch.done.push(id);
    }
    return batch;
  },
  async pickFolder() {
    return "D:\\Captures\\ShadowPlay";
  },
  async onLibraryChanged() {
    return () => {};
  },
  async isFullscreen() {
    return !!document.fullscreenElement;
  },
  async setFullscreen(on) {
    if (on) await document.documentElement.requestFullscreen();
    else if (document.fullscreenElement) await document.exitFullscreen();
  },
  async toggleFullscreen() {
    const next = !document.fullscreenElement;
    await this.setFullscreen(next);
    return next;
  },
  async minimizeWindow() {},
  async toggleMaximizeWindow() {},
  async closeWindow() {
    console.info("[mock] close window");
  },
  async isMaximized() {
    return false;
  },
  async onWindowResized() {
    return () => {};
  },
  videoUrl: (clip) => `/dev-media/sample.webm?clip=${encodeURIComponent(clip.id)}&v=${clip.modified}`,
  thumbnailUrl(clip) {
    const h = hueOf(clip.game);
    const x = (clip.date % 300) + 40;
    return svgUrl(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 480 270"><defs><linearGradient id="g" x2="1" y2="1"><stop offset="0" stop-color="hsl(${h} 45% 30%)"/><stop offset="1" stop-color="hsl(${(h + 40) % 360} 35% 10%)"/></linearGradient></defs><rect width="480" height="270" fill="url(#g)"/><circle cx="${x}" cy="150" r="70" fill="hsl(${h} 60% 55% / .25)"/><rect x="0" y="200" width="480" height="70" fill="hsl(${h} 20% 8% / .5)"/></svg>`,
    );
  },
  gameIconUrl(gameId) {
    // Leave some games without an icon to exercise the monogram fallback.
    if (["Fortnite", "Cyberpunk 2077", "Ready Or Not"].includes(gameId)) return "data:,missing";
    const h = hueOf(gameId);
    return svgUrl(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="14" fill="hsl(${h} 70% 45%)"/><path d="M20 44 32 16 44 44Z" fill="#fff" opacity=".9"/></svg>`,
    );
  },
  filmstripUrl(clip) {
    const h = hueOf(clip.game);
    const frames = Array.from({ length: 16 }, (_, i) => {
      const hue = (h + i * 9) % 360;
      return `<rect x="${i * 192}" width="192" height="108" fill="hsl(${hue} 40% ${18 + (i % 4) * 5}%)"/><circle cx="${i * 192 + 60 + ((i * 37) % 80)}" cy="60" r="26" fill="hsl(${hue} 60% 55% / .35)"/>`;
    }).join("");
    return svgUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 3072 108">${frames}</svg>`);
  },
};
