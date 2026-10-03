import { api, type Clip, type Game } from "../api";
import { storedValue } from "../util/storage";

export type SortKey = "date" | "name" | "game" | "favorites";

export type View = { kind: "all" } | { kind: "favorites" } | { kind: "game"; id: string };

export const SORT_OPTIONS: { key: SortKey; label: string }[] = [
  { key: "date", label: "Date" },
  { key: "name", label: "Name" },
  { key: "game", label: "Game" },
  { key: "favorites", label: "Favorites" },
];

/** Natural direction per key: newest first, A→Z, favorites first. */
const DEFAULT_DESCENDING: Record<SortKey, boolean> = {
  date: true,
  name: false,
  game: false,
  favorites: true,
};

const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

const sortPref = storedValue<{ key: SortKey; descending: boolean }>("sort", {
  key: "date",
  descending: true,
});

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

  view = $state<View>({ kind: "all" });
  sortKey = $state<SortKey>(sortPref.get().key);
  descending = $state(sortPref.get().descending);
  query = $state("");

  gameById = $derived(new Map(this.games.map((g) => [g.id, g])));

  favoriteCount = $derived(this.favorites.size);

  /** Clips for the current view, search and sort, in display order. */
  visible = $derived.by(() => this.#computeVisible());

  viewTitle = $derived.by(() => {
    switch (this.view.kind) {
      case "all":
        return "All clips";
      case "favorites":
        return "Favorites";
      case "game":
        return this.gameName(this.view.id);
    }
  });

  gameName(id: string): string {
    return this.gameById.get(id)?.name ?? (id || "Unsorted");
  }

  isFavorite(id: string): boolean {
    return this.favorites.has(id);
  }

  async load({ silent = false } = {}) {
    if (!silent) this.status = "loading";
    try {
      const library = await api.scanLibrary();
      this.root = library.root;
      this.clips = library.clips;
      this.games = library.games;
      this.favorites = new Set(library.clips.filter((c) => c.favorite).map((c) => c.id));
      this.error = null;
      this.status = "ready";
      // The game being viewed may have disappeared (folder deleted/renamed).
      const view = this.view;
      if (view.kind === "game" && !this.gameById.has(view.id)) this.view = { kind: "all" };
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

  setSort(key: SortKey) {
    if (key === this.sortKey) {
      this.descending = !this.descending;
    } else {
      this.sortKey = key;
      this.descending = DEFAULT_DESCENDING[key];
    }
    sortPref.set({ key: this.sortKey, descending: this.descending });
  }

  toggleDirection() {
    this.descending = !this.descending;
    sortPref.set({ key: this.sortKey, descending: this.descending });
  }

  #setFavoriteLocal(id: string, favorite: boolean) {
    const next = new Set(this.favorites);
    if (favorite) next.add(id);
    else next.delete(id);
    this.favorites = next;
  }

  #computeVisible(): Clip[] {
    const { view, favorites } = this;
    const query = this.query.trim().toLowerCase();

    let list = this.clips;
    if (view.kind === "favorites") list = list.filter((c) => favorites.has(c.id));
    else if (view.kind === "game") list = list.filter((c) => c.game === view.id);
    if (query) {
      list = list.filter(
        (c) => c.name.toLowerCase().includes(query) || this.gameName(c.game).toLowerCase().includes(query),
      );
    }

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
      case "favorites":
        primary = (a, b) => Number(favorites.has(a.id)) - Number(favorites.has(b.id));
        break;
    }
    // Ties always fall back to newest first.
    return list.toSorted((a, b) => dir * primary(a, b) || byDateDesc(a, b));
  }
}

export const library = new LibraryState();
