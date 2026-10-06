import type { Batch, Clip } from "../api";
import { plural } from "../util/format";
import { library } from "./library.svelte";
import { merge } from "./merge.svelte";
import { toast } from "./toast.svelte";

/** "Ace" -> /^Ace #(\d+)$/i, the batch rename pattern. */
export function numberedPattern(name: string): RegExp {
  return new RegExp(`^${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")} #(\\d+)$`, "i");
}

/**
 * Multi-select in the gallery, like a phone's photo picker: started from the
 * toolbar's Select button or by Shift-clicking a clip. The selected clips can
 * then be favorited, renamed or removed together.
 */
class SelectionState {
  active = $state(false);
  ids = $state.raw<ReadonlySet<string>>(new Set());
  /** Which batch dialog is open. */
  dialog = $state<"rename" | "delete" | null>(null);
  busy = $state(false);
  error = $state<string | null>(null);

  /** Last clip clicked, where a Shift-click range starts. */
  #anchor: string | null = null;

  /** Selected clips still in the library, oldest first. */
  clips = $derived(library.clips.filter((c) => this.ids.has(c.id)).toSorted((a, b) => a.date - b.date));
  count = $derived(this.clips.length);
  allFavorite = $derived(this.count > 0 && this.clips.every((c) => library.isFavorite(c.id)));
  allVisibleSelected = $derived(library.visible.length > 0 && library.visible.every((c) => this.ids.has(c.id)));

  start() {
    merge.stop();
    this.active = true;
    this.ids = new Set();
    this.#anchor = null;
    this.error = null;
  }

  stop() {
    if (this.busy) return;
    this.active = false;
    this.ids = new Set();
    this.dialog = null;
    this.error = null;
    this.#anchor = null;
  }

  has(id: string): boolean {
    return this.ids.has(id);
  }

  /**
   * A click on a clip while selecting, or a Shift-click at any time (which
   * starts selecting). Shift-click while selecting adds the whole range from
   * the previously clicked clip.
   */
  click(clip: Clip, shift: boolean) {
    if (!this.active) {
      this.start();
      this.#set([clip.id], true);
    } else if (shift && this.#anchor) {
      const visible = library.visible;
      const from = visible.findIndex((c) => c.id === this.#anchor);
      const to = visible.findIndex((c) => c.id === clip.id);
      if (from < 0 || to < 0) this.#set([clip.id], !this.has(clip.id));
      else this.#set(visible.slice(Math.min(from, to), Math.max(from, to) + 1).map((c) => c.id), true);
    } else {
      this.#set([clip.id], !this.has(clip.id));
    }
    this.#anchor = clip.id;
  }

  /** Selects every clip shown, or clears the selection when all already are. */
  toggleAll() {
    if (this.allVisibleSelected) this.ids = new Set();
    else this.#set(library.visible.map((c) => c.id), true);
  }

  async favorite() {
    if (!this.count || this.busy) return;
    const favorite = !this.allFavorite;
    try {
      await library.setFavorites(this.clips.map((c) => c.id), favorite);
      toast.show(`${plural(this.count, "clip")} ${favorite ? "added to" : "removed from"} favorites`);
    } catch (e) {
      toast.show(`Couldn't update favorites: ${String(e).replace(/^Error: /, "")}`);
    }
  }

  open(dialog: "rename" | "delete") {
    if (!this.count || this.busy) return;
    this.error = null;
    this.dialog = dialog;
  }

  closeDialog() {
    if (this.busy) return;
    this.dialog = null;
    this.error = null;
  }

  /** Renames the selection, oldest first, to "{name} #n". */
  async rename(name: string) {
    await this.#run(
      () => library.renameMany(this.clips.map((c) => c.id), name),
      (batch) => batch.done.map((r) => r.from),
      (n) => `Renamed ${plural(n, "clip")}`,
    );
  }

  async remove() {
    await this.#run(
      () => library.removeMany(this.clips.map((c) => c.id)),
      (batch) => batch.done,
      (n) => `Moved ${plural(n, "clip")} to the Recycle Bin`,
    );
  }

  /** Runs a batch; clips that failed stay selected with the reason shown. */
  async #run<T>(job: () => Promise<Batch<T>>, doneIds: (batch: Batch<T>) => string[], message: (n: number) => string) {
    if (this.busy) return;
    this.busy = true;
    this.error = null;
    try {
      const batch = await job();
      if (batch.failed === 0) {
        this.busy = false;
        this.stop();
        toast.show(message(batch.done.length));
        return;
      }
      const done = new Set(doneIds(batch));
      this.ids = new Set([...this.ids].filter((id) => !done.has(id)));
      const what = batch.done.length ? `${message(batch.done.length)}, but ` : "";
      this.error = `${what}${plural(batch.failed, "clip")} failed: ${batch.error}`;
    } catch (e) {
      this.error = String(e).replace(/^Error: /, "");
    } finally {
      this.busy = false;
    }
  }

  #set(ids: string[], selected: boolean) {
    const next = new Set(this.ids);
    for (const id of ids) {
      if (selected) next.add(id);
      else next.delete(id);
    }
    this.ids = next;
  }
}

export const selection = new SelectionState();
