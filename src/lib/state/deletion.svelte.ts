import type { Clip } from "../api";
import { library } from "./library.svelte";
import { settings } from "./settings.svelte";

/**
 * Removing a clip (to the Recycle Bin). `pending` is the clip awaiting the
 * user's confirmation, or the one whose removal failed; the dialog shows it.
 */
class DeletionState {
  pending = $state<Clip | null>(null);
  busy = $state(false);
  error = $state<string | null>(null);

  /** Entry point for the remove button: asks first unless the user opted out. */
  request(clip: Clip) {
    this.error = null;
    this.pending = clip;
    if (settings.value?.confirmDelete === false) void this.confirm(false);
  }

  /** `askAgain: false` also turns the confirmation prompt off for next time. */
  async confirm(askAgain = true) {
    const clip = this.pending;
    if (!clip || this.busy) return;
    this.busy = true;
    this.error = null;
    try {
      await library.remove(clip.id);
      this.pending = null;
      if (!askAgain && settings.value?.confirmDelete !== false) await settings.setConfirmDelete(false);
    } catch (e) {
      // Stays open so the reason is visible.
      this.error = String(e);
    } finally {
      this.busy = false;
    }
  }

  cancel() {
    if (this.busy) return;
    this.pending = null;
    this.error = null;
  }
}

export const deletion = new DeletionState();
