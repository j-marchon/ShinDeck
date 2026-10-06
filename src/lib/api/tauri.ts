import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { Backend, Batch, Clip, FolderSummary, Library, Merged, Renamed, Settings } from "./types";

const appWindow = getCurrentWindow();

export const tauriBackend: Backend = {
  getSettings: () => invoke<Settings>("get_settings"),
  defaultClipsFolder: () => invoke<string>("default_clips_folder"),
  inspectFolder: (path) => invoke<FolderSummary>("inspect_folder", { path }),
  setLibraryPath: (path) => invoke<Settings>("set_library_path", { path }),
  scanLibrary: () => invoke<Library>("scan_library"),
  setFavorite: (id, favorite) => invoke("set_favorite", { id, favorite }),
  setFavorites: (ids, favorite) => invoke("set_favorites", { ids, favorite }),
  revealClip: (id) => invoke("reveal_clip", { id }),
  renameClip: (id, name) => invoke<Clip>("rename_clip", { id, name }),
  renameClips: (ids, name) => invoke<Batch<Renamed>>("rename_clips", { ids, name }),
  exportClip: (id, spec, destination) => invoke<Clip>("export_clip", { id, spec, destination }),
  mergeClips: (ids, destination) => invoke<Merged>("merge_clips", { ids, destination }),
  cancelExport: () => invoke("cancel_export"),
  onExportProgress: (handler) => listen<number>("export-progress", (e) => handler(e.payload)),
  setSaveMode: (mode) => invoke<Settings>("set_save_mode", { mode }),
  setConfirmDelete: (confirm) => invoke<Settings>("set_confirm_delete", { confirm }),
  setMergeReplace: (replace) => invoke<Settings>("set_merge_replace", { replace }),
  deleteClip: (id) => invoke("delete_clip", { id }),
  deleteClips: (ids) => invoke<Batch<string>>("delete_clips", { ids }),

  async pickFolder(defaultPath) {
    const picked = await open({ directory: true, defaultPath, title: "Choose your ShadowPlay clips folder" });
    return typeof picked === "string" ? picked : null;
  },

  onLibraryChanged: (handler) => listen("library-changed", handler),

  isFullscreen: () => appWindow.isFullscreen(),
  setFullscreen: (on) => appWindow.setFullscreen(on),
  minimizeWindow: () => appWindow.minimize(),
  toggleMaximizeWindow: () => appWindow.toggleMaximize(),
  closeWindow: () => appWindow.close(),
  isMaximized: () => appWindow.isMaximized(),
  onWindowResized: (handler) => appWindow.onResized(() => handler()),
  async toggleFullscreen() {
    const next = !(await appWindow.isFullscreen());
    await appWindow.setFullscreen(next);
    return next;
  },

  // Videos stream through Tauri's asset protocol (supports HTTP range
  // requests, so seeking never reads the whole file).
  // The query only busts the webview cache after an edit replaces the file.
  videoUrl: (clip: Clip) => `${convertFileSrc(clip.path)}?v=${clip.modified}`,
  // `modified` busts the webview cache when a clip file is replaced.
  thumbnailUrl: (clip: Clip) => `${convertFileSrc(clip.id, "thumb")}?v=${clip.modified}`,
  gameIconUrl: (gameId) => convertFileSrc(gameId, "gameicon"),
  filmstripUrl: (clip: Clip) => `${convertFileSrc(clip.id, "filmstrip")}?v=${clip.modified}`,
};
