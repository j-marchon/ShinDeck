/** Mirrors the serialized Rust types in `src-tauri/src`. */

/** What happens to an edited clip. `ask` prompts after every edit. */
export type SaveMode = "ask" | "new" | "replace";
export type Destination = "new" | "replace";

export interface Settings {
  libraryPath: string | null;
  setupComplete: boolean;
  /** null until the user picks one (they're prompted on their first edit). */
  saveMode: SaveMode | null;
  /** Ask before moving a clip to the Recycle Bin. */
  confirmDelete: boolean;
  /** Merges move both originals to the Recycle Bin by default. */
  mergeReplace: boolean;
  /** False when the bundled ffmpeg is missing. */
  editingAvailable: boolean;
}

export interface EditSpec {
  /** Ranges to keep, in seconds. null keeps the whole clip. */
  keep: [number, number][] | null;
  /** Compress to fit this many bytes. */
  targetBytes: number | null;
  /** Tag for the new file's name, e.g. "trimmed" or "Discord". */
  label: string;
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
  /** Produced or modified by ShinDeck's editor. */
  edited: boolean;
}

/** Result of an operation on several clips; one failure doesn't stop the rest. */
export interface Batch<T> {
  done: T[];
  failed: number;
  /** The first failure's reason. */
  error: string | null;
}

export interface Renamed {
  /** The clip's id before the rename. */
  from: string;
  clip: Clip;
}

/** Result of merging clips. */
export interface Merged {
  /** The new "{Game} Merge #n" clip. */
  clip: Clip;
  /** Originals moved to the Recycle Bin (when replacing). */
  removed: string[];
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
  setFavorites(ids: string[], favorite: boolean): Promise<void>;
  revealClip(id: string): Promise<void>;
  renameClip(id: string, name: string): Promise<Clip>;
  /** Renames the clips, in order, to "{name} #n", continuing after numbers in use. */
  renameClips(ids: string[], name: string): Promise<Batch<Renamed>>;
  exportClip(id: string, spec: EditSpec, destination: Destination): Promise<Clip>;
  /** Joins the clips end to end, in this order. Progress arrives via onExportProgress. */
  mergeClips(ids: string[], destination: Destination): Promise<Merged>;
  /** Cancels the running export or merge. */
  cancelExport(): Promise<void>;
  onExportProgress(handler: (progress: number) => void): Promise<() => void>;
  setSaveMode(mode: SaveMode): Promise<Settings>;
  setConfirmDelete(confirm: boolean): Promise<Settings>;
  setMergeReplace(replace: boolean): Promise<Settings>;
  /** Moves the clip file to the Recycle Bin. */
  deleteClip(id: string): Promise<void>;
  /** Moves several clips to the Recycle Bin; `done` lists the removed ids. */
  deleteClips(ids: string[]): Promise<Batch<string>>;
  pickFolder(defaultPath?: string): Promise<string | null>;
  onLibraryChanged(handler: () => void): Promise<() => void>;
  toggleFullscreen(): Promise<boolean>;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  closeWindow(): Promise<void>;
  isMaximized(): Promise<boolean>;
  onWindowResized(handler: () => void): Promise<() => void>;
  isFullscreen(): Promise<boolean>;
  setFullscreen(on: boolean): Promise<void>;
  videoUrl(clip: Clip): string;
  thumbnailUrl(clip: Clip): string;
  gameIconUrl(gameId: string): string;
  /** A strip of 16 evenly spaced frames, for the trim bar. */
  filmstripUrl(clip: Clip): string;
}
