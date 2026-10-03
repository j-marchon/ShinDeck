import { api, type Clip, type Game } from "../api";
import { storedValue } from "../util/storage";

export type SortKey = "date" | "name" | "game" | "size";

export interface SortOption {
  key: SortKey;
  descending: boolean;
  label: string;
}

export const SORT_OPTIONS: SortOption[] = [
  { key: "date", descending: true, label: "Newest first" },
  { key: "date", descending: false, label: "Oldest first" },
  { key: "name", descending: false, label: "Name A–Z" },
  { key: "name", descending: true, label: "Name Z–A" },
  { key: "game", descending: false, label: "Game A–Z" },
  { key: "game", descending: true, label: "Game Z–A" },
  { key: "size", descending: true, label: "Largest first" },
  { key: "size", descending: false, label: "Smallest first" },
];

export type DateRange = "any" | "today" | "week" | "month" | "year";

export const DATE_RANGES: { key: DateRange; label: string }[] = [
  { key: "any", label: "Any time" },
  { key: "today", label: "Today" },
  { key: "week", label: "Past 7 days" },
  { key: "month", label: "Past 30 days" },
  { key: "year", label: "Past year" },
];

function rangeStart(range: DateRange, now = Date.now()): number {
  const day = 86_400_000;
  switch (range) {
    case "any":
      return 0;
    case "today": {
      const d = new Date(now);
      d.setHours(0, 0, 0, 0);
      return d.getTime();
    }
    case "week":
      return now - 7 * day;
    case "month":
      return now - 30 * day;
    case "year":
      return now - 365 * day;
  }
}

const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

const sortPref = storedValue<{ key: SortKey; descending: boolean }>("sort", { key: "date", descending: true });

function withoutId(set: ReadonlySet<string>, id: string): Set<string> {
  const next = new Set(set);
  next.delete(id);
  return next;
}

/**
 * Single source of truth for the library. Arrays are `$state.raw` (no deep
 * proxies): they are replaced, never mutated, which keeps thousands of clips
 * cheap to hold and to diff.
 */
class LibraryState {
  status = $state<"idle" | "loading" | "ready" | "error">("idle");
  error = $state<string | null>(null);
  root = $state("");
  clips = $state.raw<Clip[]>([]);
  games = $state.raw<Game[]>([]);
  favorites = $state.raw<ReadonlySet<string>>(new Set());
  edited = $state.raw<ReadonlySet<string>>(new Set());

  // Filters: all of them combine.
  game = $state<string | null>(null);
  favoritesOnly = $state(false);
  dateRange = $state<DateRange>("any");
  query = $state("");

  sortKey = $state<SortKey>(sortPref.get().key);
  descending = $state(sortPref.get().descending);

  gameById = $derived(new Map(this.games.map((g) => [g.id, g])));
  favoriteCount = $derived(this.favorites.size);
  isFiltered = $derived(this.game !== null || this.favoritesOnly || this.dateRange !== "any" || this.query.trim() !== "");
  sortLabel = $derived(
    SORT_OPTIONS.find((o) => o.key === this.sortKey && o.descending === this.descending)?.label ?? "Sort",
  );

  /** Clips matching every active filter, in display order. */
  visible = $derived.by(() => this.#computeVisible());

  gameName(id: string): string {
    return this.gameById.get(id)?.name ?? (id || "Unsorted");
  }

  isFavorite(id: string): boolean {
    return this.favorites.has(id);
  }

  isEdited(id: string): boolean {
    return this.edited.has(id);
  }

  clearFilters() {
    this.game = null;
    this.favoritesOnly = false;
    this.dateRange = "any";
    this.query = "";
  }

  setSort(option: Pick<SortOption, "key" | "descending">) {
    this.sortKey = option.key;
    this.descending = option.descending;
    sortPref.set({ key: option.key, descending: option.descending });
  }

  async load({ silent = false } = {}) {
    if (!silent) this.status = "loading";
    try {
      const library = await api.scanLibrary();
      this.root = library.root;
      this.#setClips(library.clips);
      this.games = library.games;
      this.error = null;
      this.status = "ready";
      // The selected game may have disappeared (folder deleted/renamed).
      if (this.game !== null && !this.gameById.has(this.game)) this.game = null;
    } catch (e) {
      this.error = String(e);
      this.status = "error";
    }
  }

  async toggleFavorite(id: string) {
    const favorite = !this.favorites.has(id);
    this.#setFavoriteLocal(id, favorite); // optimistic
    try {
      await api.setFavorite(id, favorite);
    } catch (e) {
      console.error("Could not save favorite", e);
      this.#setFavoriteLocal(id, !favorite);
    }
  }

  /** Renames on disk; returns the updated clip. Throws a readable message on failure. */
  async rename(id: string, name: string): Promise<Clip> {
    const clip = await api.renameClip(id, name);
    this.upsert(clip, id);
    return clip;
  }

  /**
   * Adds or updates a clip returned by the backend (rename, export). Pass
   * `previousId` when the clip's id changed.
   */
  upsert(clip: Clip, previousId?: string) {
    const replacing = previousId ?? clip.id;
    const exists = this.clips.some((c) => c.id === replacing);
    this.#setClips(exists ? this.clips.map((c) => (c.id === replacing ? clip : c)) : [...this.clips, clip]);
    if (!exists) {
      this.games = this.games.map((g) => (g.id === clip.game ? { ...g, clipCount: g.clipCount + 1 } : g));
    }
  }

  #setClips(clips: Clip[]) {
    this.clips = clips;
    this.favorites = new Set(clips.filter((c) => c.favorite).map((c) => c.id));
    this.edited = new Set(clips.filter((c) => c.edited).map((c) => c.id));
  }

  #setFavoriteLocal(id: string, favorite: boolean) {
    const next = favorite ? new Set(this.favorites).add(id) : withoutId(this.favorites, id);
    this.favorites = next;
    // Keep the clip objects in sync so later upserts don't revert it.
    this.clips = this.clips.map((c) => (c.id === id ? { ...c, favorite } : c));
  }

  #computeVisible(): Clip[] {
    const { favorites, game, favoritesOnly } = this;
    const query = this.query.trim().toLowerCase();
    const since = rangeStart(this.dateRange);

    const list = this.clips.filter(
      (c) =>
        (game === null || c.game === game) &&
        (!favoritesOnly || favorites.has(c.id)) &&
        c.date >= since &&
        (!query || c.name.toLowerCase().includes(query)),
    );

    const dir = this.descending ? -1 : 1;
    const byDateDesc = (a: Clip, b: Clip) => b.date - a.date;
    let primary: (a: Clip, b: Clip) => number;
    switch (this.sortKey) {
      case "date":
        primary = (a, b) => a.date - b.date;
        break;
      case "name":
        primary = (a, b) => collator.compare(a.name, b.name);
        break;
      case "game":
        primary = (a, b) => collator.compare(this.gameName(a.game), this.gameName(b.game));
        break;
      case "size":
        primary = (a, b) => a.size - b.size;
        break;
    }
    // Ties always fall back to newest first.
    return list.toSorted((a, b) => dir * primary(a, b) || byDateDesc(a, b));
  }
}

export const library = new LibraryState();
