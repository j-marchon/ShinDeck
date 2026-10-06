import type { Clip } from "../api";
import { library } from "./library.svelte";
import { settings } from "./settings.svelte";

/** Lets the caller (the player) take part in a removal. */
export interface RemovalHooks {
  /** Lets go of the file first (Windows won't recycle a file that's open). */
  release?: () => Promise<void>;
  /** The removal failed: take the file back. */
  restore?: () => void;
  /** The clip is gone. */
  removed?: () => void;
}

/**
 * Removing a clip (to the Recycle Bin). `pending` is the clip awaiting the
 * user's confirmation, or the one whose removal failed; the dialog shows it.
 */
class DeletionState {
  pending = $state<Clip | null>(null);
  busy = $state(false);
  error = $state<string | null>(null);
  #hooks: RemovalHooks = {};

  /** Entry point for the remove buttons: asks first unless the user opted out. */
  request(clip: Clip, hooks: RemovalHooks = {}) {
    this.error = null;
    this.pending = clip;
    this.#hooks = hooks;
    if (settings.value?.confirmDelete === false) void this.confirm(false);
  }

  /** `askAgain: false` also turns the confirmation prompt off for next time. */
  async confirm(askAgain = true) {
    const clip = this.pending;
    if (!clip || this.busy) return;
    this.busy = true;
    this.error = null;
    const hooks = this.#hooks;
    try {
      await hooks.release?.();
      await library.remove(clip.id);
      this.pending = null;
      this.#hooks = {};
      hooks.removed?.();
      if (!askAgain && settings.value?.confirmDelete !== false) await settings.setConfirmDelete(false);
    } catch (e) {
      // Stays open so the reason is visible.
      this.error = String(e).replace(/^Error: /, "");
      hooks.restore?.();
    } finally {
      this.busy = false;
    }
  }

  cancel() {
    if (this.busy) return;
    this.pending = null;
    this.error = null;
    this.#hooks = {};
  }
}

export const deletion = new DeletionState();
