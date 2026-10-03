<script lang="ts">
  import { library, SORT_OPTIONS } from "../state/library.svelte";
  import { plural } from "../util/format";
  import Icon from "./Icon.svelte";

  let {
    onrefresh,
    refreshing,
    shortcuts = true,
  }: { onrefresh: () => void; refreshing: boolean; /** Disabled while the player is open. */ shortcuts?: boolean } =
    $props();

  let search: HTMLInputElement;

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
  <div class="title">
    <h1>{library.viewTitle}</h1>
    <span class="count">{plural(library.visible.length, "clip")}</span>
  </div>

  <label class="search">
    <Icon name="search" size={15} />
    <input
      bind:this={search}
      bind:value={library.query}
      onkeydown={onsearchkeydown}
      type="text"
      placeholder="Search clips"
      spellcheck="false"
    />
    {#if library.query}
      <button class="clear" aria-label="Clear search" onclick={() => (library.query = "")}>
        <Icon name="close" size={14} />
      </button>
    {/if}
  </label>

  <div class="sort" role="group" aria-label="Sort by">
    {#each SORT_OPTIONS as option (option.key)}
      <button
        class:active={library.sortKey === option.key}
        aria-pressed={library.sortKey === option.key}
        onclick={() => library.setSort(option.key)}
      >
        {option.label}
      </button>
    {/each}
  </div>

  <button
    class="icon-btn"
    title={library.descending ? "Descending" : "Ascending"}
    aria-label="Toggle sort direction"
    onclick={() => library.toggleDirection()}
  >
    <Icon name={library.descending ? "sortDesc" : "sortAsc"} size={17} />
  </button>

  <button class="icon-btn" class:spinning={refreshing} title="Rescan folder" aria-label="Rescan folder" onclick={onrefresh}>
    <Icon name="refresh" size={16} />
  </button>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 68px;
    padding: 0 24px;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
  }

  .title {
    display: flex;
    align-items: baseline;
    gap: 12px;
    min-width: 0;
    margin-right: auto;
  }
  h1 {
    font-size: 20px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .count {
    font-size: 13px;
    color: var(--text-faint);
    white-space: nowrap;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 240px;
    height: 36px;
    padding: 0 10px 0 12px;
    border-radius: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-faint);
    transition: border-color 0.12s ease;
  }
  .search:focus-within {
    border-color: var(--accent);
    color: var(--text-dim);
  }
  .search input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 13.5px;
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

  .sort {
    display: flex;
    padding: 3px;
    border-radius: 9px;
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .sort button {
    height: 28px;
    padding: 0 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-dim);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .sort button:hover {
    color: var(--text);
  }
  .sort button.active {
    background: var(--accent);
    color: #000;
    font-weight: 600;
  }

  .icon-btn {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-dim);
    border: 1px solid var(--border);
    background: var(--surface);
  }
  .icon-btn:hover {
    color: var(--accent);
    border-color: var(--accent);
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
