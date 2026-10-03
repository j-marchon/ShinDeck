import { api } from "../api";

/** Tracks maximized/fullscreen so the app frame can drop its rounded corners. */
class WindowState {
  maximized = $state(false);
  fullscreen = $state(false);

  /** Starts tracking; returns a cleanup function. */
  track(): () => void {
    let unlisten: (() => void) | undefined;
    const sync = async () => {
      [this.maximized, this.fullscreen] = await Promise.all([api.isMaximized(), api.isFullscreen()]);
    };
    sync();
    api.onWindowResized(sync).then((u) => (unlisten = u));
    return () => unlisten?.();
  }
}

export const windowState = new WindowState();
