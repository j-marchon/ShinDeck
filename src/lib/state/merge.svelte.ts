import { api, type Clip } from "../api";
import { library } from "./library.svelte";
import { selection } from "./selection.svelte";
import { settings } from "./settings.svelte";
import { toast } from "./toast.svelte";

/**
 * Merging two clips: pick them in the gallery (`selecting`), then confirm the
 * order and options in a dialog (`reviewing`) and run the merge.
 */
class MergeState {
  selecting = $state(false);
  /** Picked clips, in merge order. */
  picked = $state.raw<Clip[]>([]);
  reviewing = $state(false);
  /** Move both originals to the Recycle Bin once the merge is saved. */
  replace = $state(false);
  running = $state(false);
  progress = $state(0);
  error = $state<string | null>(null);

  start() {
    selection.stop();
    this.selecting = true;
    this.picked = [];
    this.reviewing = false;
    this.error = null;
    this.replace = settings.value?.saveMode === "replace";
  }

  stop() {
    if (this.running) return;
    this.selecting = false;
    this.picked = [];
    this.reviewing = false;
    this.error = null;
  }

  /** 1-based position in the merge order, or 0 when not picked. */
  position(id: string): number {
    return this.picked.findIndex((c) => c.id === id) + 1;
  }

  toggle(clip: Clip) {
    if (this.position(clip.id)) {
      this.picked = this.picked.filter((c) => c.id !== clip.id);
    } else if (this.picked.length < 2) {
      this.picked = [...this.picked, clip];
      if (this.picked.length === 2) this.review();
    }
  }

  review() {
    this.error = null;
    this.reviewing = this.picked.length === 2;
  }

  /** Back to the gallery to change the selection. */
  back() {
    if (this.running) return;
    this.reviewing = false;
    this.error = null;
  }

  swap() {
    this.picked = this.picked.toReversed();
  }

  async run() {
    if (this.running || this.picked.length !== 2) return;
    this.running = true;
    this.progress = 0;
    this.error = null;
    const unlisten = await api.onExportProgress((p) => (this.progress = p));
    try {
      const ids = this.picked.map((c) => c.id);
      const result = await api.mergeClips(ids, this.replace ? "replace" : "new");
      for (const id of result.removed) library.forget(id);
      library.upsert(result.clip);
      this.running = false;
      this.stop();
      toast.show(`Saved as “${result.clip.name}”`);
    } catch (e) {
      const message = String(e).replace(/^Error: /, "");
      if (message !== "Cancelled") this.error = message;
    } finally {
      unlisten();
      this.running = false;
    }
  }

  cancel() {
    api.cancelExport();
  }
}

export const merge = new MergeState();
