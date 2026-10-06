<script lang="ts">
  import { DATE_RANGES, library, SORT_OPTIONS } from "../state/library.svelte";
  import { merge } from "../state/merge.svelte";
  import { settings } from "../state/settings.svelte";
  import { plural } from "../util/format";
  import Dropdown from "./Dropdown.svelte";
  import GameFilter from "./GameFilter.svelte";
  import Icon from "./Icon.svelte";

  let {
    onrefresh,
    onsettings,
    refreshing,
    shortcuts = true,
  }: {
    onrefresh: () => void;
    onsettings: () => void;
    refreshing: boolean;
    /** Disabled while the player is open. */
    shortcuts?: boolean;
  } = $props();

  let search: HTMLInputElement;

  const canMerge = $derived(settings.value?.editingAvailable !== false);

  const dateLabel = $derived(DATE_RANGES.find((r) => r.key === library.dateRange)?.label ?? "Any time");

  function onwindowkeydown(e: KeyboardEvent) {
    if (!shortcuts) return;
    if ((e.ctrlKey && e.key.toLowerCase() === "f") || (e.key === "/" && document.activeElement === document.body)) {
      e.preventDefault();
      search.focus();
      search.select();
    }
  }

  function onsearchkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      library.query = "";
      search.blur();
    }
  }
</script>

<svelte:window onkeydown={onwindowkeydown} />

<header class="toolbar">
  <div class="filters">
    <GameFilter />

    <Dropdown width={180}>
      {#snippet trigger({ open, toggle })}
        <button class="chip" class:on={library.dateRange !== "any"} class:pressed={open} onclick={toggle}>
          <Icon name="calendar" size={15} />
          {dateLabel}
          <Icon name="chevronDown" size={14} />
        </button>
      {/snippet}
      {#snippet children(close)}
        {#each DATE_RANGES as range (range.key)}
          <button
            class="menu-item"
            class:active={library.dateRange === range.key}
            onclick={() => {
              library.dateRange = range.key;
              close();
            }}
          >
            {range.label}
          </button>
        {/each}
      {/snippet}
    </Dropdown>

    <button
      class="chip"
      class:on={library.favoritesOnly}
      aria-pressed={library.favoritesOnly}
      title="Only show favorites"
      onclick={() => (library.favoritesOnly = !library.favoritesOnly)}
    >
      <Icon name="star" size={15} filled={library.favoritesOnly} />
      Favorites
      <span class="badge">{library.favoriteCount}</span>
    </button>

    <label class="search" class:on={library.query}>
      <Icon name="search" size={15} />
      <input
        bind:this={search}
        bind:value={library.query}
        onkeydown={onsearchkeydown}
        type="text"
        placeholder="Search by name"
        spellcheck="false"
      />
      {#if library.query}
        <button class="clear" aria-label="Clear search" onclick={() => (library.query = "")}>
          <Icon name="close" size={14} />
        </button>
      {/if}
    </label>

    {#if library.isFiltered}
      <button class="reset" onclick={() => library.clearFilters()}>Clear filters</button>
    {/if}
  </div>

  <div class="right">
    <span class="count">{plural(library.visible.length, "clip")}</span>

    <button
      class="chip"
      class:on={merge.selecting}
      aria-pressed={merge.selecting}
      disabled={!canMerge}
      title={canMerge ? "Join two clips into one" : "Merging needs the bundled ffmpeg, which is missing"}
      onclick={() => (merge.selecting ? merge.stop() : merge.start())}
    >
      <Icon name="merge" size={15} />
      Merge
    </button>

    <Dropdown align="right" width={190}>
      {#snippet trigger({ open, toggle })}
        <button class="chip" class:pressed={open} onclick={toggle} title="Sort order">
          <Icon name={library.descending ? "sortDesc" : "sortAsc"} size={15} />
          {library.sortLabel}
          <Icon name="chevronDown" size={14} />
        </button>
      {/snippet}
      {#snippet children(close)}
        {#each SORT_OPTIONS as option (option.label)}
          <button
            class="menu-item"
            class:active={library.sortKey === option.key && library.descending === option.descending}
            onclick={() => {
              library.setSort(option);
              close();
            }}
          >
            {option.label}
          </button>
        {/each}
      {/snippet}
    </Dropdown>

    <button class="chip square" class:spinning={refreshing} title="Rescan folder" aria-label="Rescan folder" onclick={onrefresh}>
      <Icon name="refresh" size={16} />
    </button>
    <button class="chip square" title="Settings" aria-label="Settings" onclick={onsettings}>
      <Icon name="settings" size={16} />
    </button>
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 6px 22px 14px;
  }
  .filters,
  .right {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .right {
    flex-shrink: 0;
  }
  .chip:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .chip.square {
    width: 34px;
    padding: 0;
    justify-content: center;
  }
  .badge {
    min-width: 18px;
    padding: 0 5px;
    border-radius: 9px;
    background: var(--glass-2);
    font-size: 11px;
    line-height: 18px;
    font-variant-numeric: tabular-nums;
    text-align: center;
  }
  .chip.on .badge {
    background: rgb(118 185 0 / 0.16);
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 230px;
    min-width: 120px;
    flex-shrink: 1;
    height: 34px;
    padding: 0 10px 0 12px;
    border-radius: 10px;
    background: var(--glass);
    color: var(--text-faint);
    transition:
      background 0.15s ease,
      box-shadow 0.15s ease;
  }
  .search:hover {
    background: var(--glass-2);
  }
  .search:focus-within,
  .search.on {
    background: var(--glass-2);
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.1);
    color: var(--text-dim);
  }
  .search input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 13px;
  }
  .search input::placeholder {
    color: var(--text-faint);
  }
  .clear {
    color: var(--text-faint);
    display: grid;
    place-items: center;
  }
  .clear:hover {
    color: var(--text);
  }
  .reset {
    padding: 0 6px;
    font-size: 12.5px;
    color: var(--text-faint);
    white-space: nowrap;
  }
  .reset:hover {
    color: var(--text);
  }

  .count {
    margin-right: 8px;
    font-size: 12.5px;
    color: var(--text-faint);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .spinning :global(svg) {
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
