<script lang="ts">
  import { library, type View } from "../state/library.svelte";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  let { onsettings }: { onsettings: () => void } = $props();

  function isActive(view: View): boolean {
    const current = library.view;
    if (view.kind !== current.kind) return false;
    return view.kind !== "game" || (current.kind === "game" && current.id === view.id);
  }

  function select(view: View) {
    library.view = view;
  }
</script>

<aside class="sidebar">
  <div class="brand">
    <Logo size={30} />
    <span>Shin<b>Deck</b></span>
  </div>

  <nav class="primary">
    <button class="item" class:active={isActive({ kind: "all" })} onclick={() => select({ kind: "all" })}>
      <span class="lead"><Icon name="grid" size={16} /></span>
      <span class="label">All clips</span>
      <span class="count">{library.clips.length}</span>
    </button>
    <button
      class="item"
      class:active={isActive({ kind: "favorites" })}
      onclick={() => select({ kind: "favorites" })}
    >
      <span class="lead"><Icon name="star" size={16} /></span>
      <span class="label">Favorites</span>
      <span class="count">{library.favoriteCount}</span>
    </button>
  </nav>

  <div class="section-title">Games</div>
  <nav class="games">
    {#each library.games as game (game.id)}
      <button
        class="item"
        class:active={isActive({ kind: "game", id: game.id })}
        title={game.name}
        onclick={() => select({ kind: "game", id: game.id })}
      >
        <span class="lead"><GameIcon game={game.id} name={game.name} size={20} /></span>
        <span class="label">{game.name}</span>
        <span class="count">{game.clipCount}</span>
      </button>
    {:else}
      {#if library.status === "ready"}
        <p class="empty">No games yet</p>
      {/if}
    {/each}
  </nav>

  <button class="folder" onclick={onsettings} title="Change clips folder">
    <Icon name="folderOpen" size={16} />
    <span class="path"><bdi>{library.root}</bdi></span>
    <Icon name="settings" size={16} />
  </button>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--bg-elev);
    border-right: 1px solid var(--border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 20px 20px 18px;
    font-size: 18px;
    font-weight: 500;
    letter-spacing: 0.01em;
    color: var(--text);
  }
  .brand b {
    color: var(--accent);
    font-weight: 800;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 10px;
  }
  .games {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 12px;
  }

  .section-title {
    padding: 22px 22px 8px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 38px;
    padding: 6px 10px 6px 12px;
    border-radius: 8px;
    color: var(--text-dim);
    text-align: left;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .item:hover {
    background: var(--surface-2);
    color: var(--text);
  }
  .item.active {
    background: var(--accent-soft);
    color: #fff;
  }
  .item.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 3px;
    background: var(--accent);
  }
  .item.active .lead {
    color: var(--accent);
  }
  .lead {
    width: 20px;
    display: grid;
    place-items: center;
  }
  .label {
    flex: 1;
    min-width: 0;
    font-size: 13.5px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .count {
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    color: var(--text-faint);
  }
  .item.active .count {
    color: var(--accent);
  }

  .empty {
    padding: 6px 12px;
    font-size: 13px;
    color: var(--text-faint);
  }

  .folder {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 10px;
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-size: 12px;
    text-align: left;
  }
  .folder:hover {
    border-color: var(--accent);
    color: var(--text);
  }
  .path {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl; /* keep the end of long paths visible */
    text-align: left;
  }
</style>
