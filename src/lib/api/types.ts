/** Mirrors the serialized Rust types in `src-tauri/src`. */

export interface Settings {
  libraryPath: string | null;
  setupComplete: boolean;
}

export interface FolderSummary {
  exists: boolean;
  gameCount: number;
  clipCount: number;
}

export interface Clip {
  /** Path relative to the library root ("Game/clip.mp4"). Stable key. */
  id: string;
  /** File name without extension. */
  name: string;
  /** Absolute path on disk. */
  path: string;
  /** Game id (folder name). Empty string = clip in the library root. */
  game: string;
  size: number;
  /** Capture time, ms since epoch. */
  date: number;
  /** Modification time, ms since epoch (thumbnail cache buster). */
  modified: number;
  durationMs: number | null;
  favorite: boolean;
}

export interface Game {
  id: string;
  name: string;
  clipCount: number;
  latest: number;
}

export interface Library {
  root: string;
  games: Game[];
  clips: Clip[];
}

/** Everything the UI needs from the host. Implemented by Tauri (and a mock for browser dev). */
export interface Backend {
  getSettings(): Promise<Settings>;
  defaultClipsFolder(): Promise<string>;
  inspectFolder(path: string): Promise<FolderSummary>;
  setLibraryPath(path: string): Promise<Settings>;
  scanLibrary(): Promise<Library>;
  setFavorite(id: string, favorite: boolean): Promise<void>;
  revealClip(id: string): Promise<void>;
  pickFolder(defaultPath?: string): Promise<string | null>;
  onLibraryChanged(handler: () => void): Promise<() => void>;
  toggleFullscreen(): Promise<boolean>;
  isFullscreen(): Promise<boolean>;
  setFullscreen(on: boolean): Promise<void>;
  videoUrl(clip: Clip): string;
  thumbnailUrl(clip: Clip): string;
  gameIconUrl(gameId: string): string;
}
