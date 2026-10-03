<script lang="ts">
  import { tick } from "svelte";
  import { library } from "../state/library.svelte";
  import Dropdown from "./Dropdown.svelte";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";

  let open = $state(false);
  let search = $state("");
  let input = $state<HTMLInputElement>();
  let list = $state<HTMLDivElement>();
  /** Keyboard focus: -1 = "All games", 0.. = filtered games. */
  let focused = $state(-1);

  const games = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return q ? library.games.filter((g) => g.name.toLowerCase().includes(q)) : library.games;
  });
  const selected = $derived(library.game === null ? null : library.gameById.get(library.game));

  $effect(() => {
    if (open) {
      search = "";
      focused = -1;
      tick().then(() => input?.focus());
    }
  });

  function pick(id: string | null, close: () => void) {
    library.game = id;
    close();
  }

  function onkeydown(e: KeyboardEvent, close: () => void) {
    if (e.key === "ArrowDown") {
      focused = Math.min(games.length - 1, focused + 1);
    } else if (e.key === "ArrowUp") {
      focused = Math.max(-1, focused - 1);
    } else if (e.key === "Enter") {
      pick(focused < 0 ? null : games[focused].id, close);
    } else {
      return;
    }
    e.preventDefault();
    tick().then(() => list?.querySelector(".focused")?.scrollIntoView({ block: "nearest" }));
  }
</script>

<Dropdown bind:open width={300}>
  {#snippet trigger({ open, toggle })}
    <div class="trigger">
      <button class="chip" class:on={selected === null} class:pressed={open} onclick={toggle} aria-haspopup="menu">
        {#if selected}
          <GameIcon game={selected.id} name={selected.name} size={18} />
          <span class="name">{selected.name}</span>
        {:else}
          <Icon name="grid" size={15} />
          <span>All games</span>
        {/if}
        <Icon name="chevronDown" size={14} />
      </button>
      {#if selected}
        <button class="clear" title="Show all games" aria-label="Show all games" onclick={() => (library.game = null)}>
          <Icon name="close" size={13} />
        </button>
      {/if}
    </div>
  {/snippet}

  {#snippet children(close)}
    <div class="search">
      <Icon name="search" size={14} />
      <input
        bind:this={input}
        bind:value={search}
        placeholder="Find a game"
        spellcheck="false"
        oninput={() => (focused = search ? 0 : -1)}
        onkeydown={(e) => onkeydown(e, close)}
      />
    </div>

    <button
      class="menu-item all"
      class:active={library.game === null}
      class:focused={focused === -1}
      onclick={() => pick(null, close)}
    >
      <span class="lead"><Icon name="grid" size={15} /></span>
      All games
      <span class="menu-count">{library.clips.length}</span>
    </button>
    <div class="menu-sep"></div>

    <div class="list" bind:this={list} role="menu">
      {#each games as game, i (game.id)}
        <button
          class="menu-item"
          role="menuitem"
          class:active={library.game === game.id}
          class:focused={focused === i}
          onclick={() => pick(game.id, close)}
        >
          <GameIcon game={game.id} name={game.name} size={20} />
          <span class="label">{game.name}</span>
          <span class="menu-count">{game.clipCount}</span>
        </button>
      {:else}
        <p class="none">No games match "{search}"</p>
      {/each}
    </div>
  {/snippet}
</Dropdown>

<style>
  .trigger {
    display: flex;
    align-items: center;
    position: relative;
  }
  .chip {
    max-width: 240px;
  }
  .chip.pressed {
    border-color: var(--accent);
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text);
  }
  .clear {
    width: 22px;
    height: 22px;
    margin-left: -30px;
    margin-right: 8px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: var(--text-dim);
    background: var(--surface-3);
  }
  .trigger:has(.clear) .chip {
    padding-right: 36px;
  }
  .clear:hover {
    color: #fff;
    background: #444;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    margin-bottom: 6px;
    padding: 0 10px;
    border-radius: 7px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text-faint);
  }
  .search:focus-within {
    border-color: var(--accent);
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
  .lead {
    width: 20px;
    display: grid;
    place-items: center;
  }
  .all {
    font-weight: 600;
  }
  .all.active {
    box-shadow: inset 0 0 0 1px rgb(118 185 0 / 0.4);
  }
  .list {
    max-height: min(420px, 55vh);
    overflow-y: auto;
  }
  .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .none {
    padding: 10px;
    font-size: 13px;
    color: var(--text-faint);
  }
</style>
