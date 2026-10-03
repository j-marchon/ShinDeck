import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { Backend, Clip, FolderSummary, Library, Settings } from "./types";

const appWindow = getCurrentWindow();

export const tauriBackend: Backend = {
  getSettings: () => invoke<Settings>("get_settings"),
  defaultClipsFolder: () => invoke<string>("default_clips_folder"),
  inspectFolder: (path) => invoke<FolderSummary>("inspect_folder", { path }),
  setLibraryPath: (path) => invoke<Settings>("set_library_path", { path }),
  scanLibrary: () => invoke<Library>("scan_library"),
  setFavorite: (id, favorite) => invoke("set_favorite", { id, favorite }),
  revealClip: (id) => invoke("reveal_clip", { id }),

  async pickFolder(defaultPath) {
    const picked = await open({ directory: true, defaultPath, title: "Choose your ShadowPlay clips folder" });
    return typeof picked === "string" ? picked : null;
  },

  onLibraryChanged: (handler) => listen("library-changed", handler),

  isFullscreen: () => appWindow.isFullscreen(),
  setFullscreen: (on) => appWindow.setFullscreen(on),
  async toggleFullscreen() {
    const next = !(await appWindow.isFullscreen());
    await appWindow.setFullscreen(next);
    return next;
  },

  // Videos stream through Tauri's asset protocol (supports HTTP range
  // requests, so seeking never reads the whole file).
  videoUrl: (clip: Clip) => convertFileSrc(clip.path),
  // `modified` busts the webview cache when a clip file is replaced.
  thumbnailUrl: (clip: Clip) => `${convertFileSrc(clip.id, "thumb")}?v=${clip.modified}`,
  gameIconUrl: (gameId) => convertFileSrc(gameId, "gameicon"),
};
