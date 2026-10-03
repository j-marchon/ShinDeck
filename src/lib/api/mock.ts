/**
 * Browser-only stand-in for the Tauri backend, used by `npm run dev` when the
 * page is opened in a normal browser. Generates a believable library so the
 * UI can be developed and screenshotted without Windows or real clips.
 * Drop any `sample.webm` into `dev-media/` (git-ignored) to test playback.
 */
import type { Backend, Clip, Game, Library, Settings } from "./types";

const GAMES: [string, number, number][] = [
  // name, clip count, hue
  ["Counter-strike 2", 46, 32],
  ["Valorant", 28, 352],
  ["Apex Legends", 19, 8],
  ["Rocket League", 23, 210],
  ["ELDEN RING", 12, 45],
  ["Cyberpunk 2077", 9, 58],
  ["Fortnite", 15, 270],
  ["Desktop", 4, 200],
];

const pad = (n: number, w = 2) => String(n).padStart(w, "0");
const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

function shadowplayName(game: string, d: Date) {
  return `${game} ${d.getFullYear()}.${pad(d.getMonth() + 1)}.${pad(d.getDate())} - ${pad(d.getHours())}.${pad(
    d.getMinutes(),
  )}.${pad(d.getSeconds())}.${pad(Math.floor(Math.random() * 99))}.DVR`;
}

function buildLibrary(root: string): Library {
  const clips: Clip[] = [];
  let seed = 7;
  const rand = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  for (const [game, count] of GAMES) {
    for (let i = 0; i < count; i++) {
      const date = Date.now() - Math.floor(rand() * 120 * 86400_000);
      const name = shadowplayName(game, new Date(date));
      const id = `${game}/${name}.mp4`;
      clips.push({
        id,
        name,
        path: `${root}\\${game}\\${name}.mp4`,
        game,
        size: Math.floor(30e6 + rand() * 400e6),
        date,
        modified: date,
        durationMs: Math.floor(15_000 + rand() * 285_000),
        favorite: favorites.has(id),
      });
    }
  }
  const games: Game[] = GAMES.map(([name]) => {
    const own = clips.filter((c) => c.game === name);
    return { id: name, name, clipCount: own.length, latest: Math.max(...own.map((c) => c.date)) };
  }).sort((a, b) => a.name.localeCompare(b.name));
  return { root, games, clips };
}

const favorites = new Set<string>();
let settings: Settings = { libraryPath: null, setupComplete: false };

function svgUrl(svg: string) {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

function hueOf(game: string) {
  return GAMES.find(([g]) => g === game)?.[2] ?? 120;
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
    settings = { libraryPath: path, setupComplete: true };
    return settings;
  },
  async scanLibrary() {
    await delay(250);
    return buildLibrary(settings.libraryPath ?? "C:\\Videos");
  },
  async setFavorite(id, favorite) {
    if (favorite) favorites.add(id);
    else favorites.delete(id);
  },
  async revealClip(id) {
    console.info("[mock] reveal", id);
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
  videoUrl: (clip) => `/dev-media/sample.webm?clip=${encodeURIComponent(clip.id)}`,
  thumbnailUrl(clip) {
    const h = hueOf(clip.game);
    const x = (clip.date % 300) + 40;
    return svgUrl(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 480 270"><defs><linearGradient id="g" x2="1" y2="1"><stop offset="0" stop-color="hsl(${h} 45% 30%)"/><stop offset="1" stop-color="hsl(${(h + 40) % 360} 35% 10%)"/></linearGradient></defs><rect width="480" height="270" fill="url(#g)"/><circle cx="${x}" cy="150" r="70" fill="hsl(${h} 60% 55% / .25)"/><rect x="0" y="200" width="480" height="70" fill="hsl(${h} 20% 8% / .5)"/></svg>`,
    );
  },
  gameIconUrl(gameId) {
    // Leave some games without an icon to exercise the monogram fallback.
    if (["Fortnite", "Cyberpunk 2077"].includes(gameId)) return "data:,missing";
    const h = hueOf(gameId);
    return svgUrl(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="14" fill="hsl(${h} 70% 45%)"/><path d="M20 44 32 16 44 44Z" fill="#fff" opacity=".9"/></svg>`,
    );
  },
};
