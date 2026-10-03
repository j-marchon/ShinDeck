<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Clip } from "./lib/api";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import Gallery from "./lib/components/Gallery.svelte";
  import PlayerModal from "./lib/components/player/PlayerModal.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";
  import Setup from "./lib/components/Setup.svelte";
  import Skeleton from "./lib/components/Skeleton.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import Toolbar from "./lib/components/Toolbar.svelte";
  import { library } from "./lib/state/library.svelte";
  import { settings } from "./lib/state/settings.svelte";
  import { windowState } from "./lib/state/window.svelte";

  let showSettings = $state(false);
  let refreshing = $state(false);
  let player = $state<{ playlist: Clip[]; startId: string; editing: boolean } | null>(null);
  let gallery = $state<ReturnType<typeof Gallery>>();

  const screen = $derived(settings.value === null ? "boot" : !settings.value.setupComplete ? "setup" : "library");

  // Scroll to top when what's listed changes in kind (not on favorite toggles).
  const resetKey = $derived(
    JSON.stringify([library.game, library.favoritesOnly, library.dateRange, library.sortKey, library.descending, library.query]),
  );

  onMount(() => windowState.track());

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      const loaded = await settings.load();
      if (loaded.setupComplete) library.load();
      // New recordings, deletions and renames picked up by the folder watcher.
      unlisten = await api.onLibraryChanged(() => library.load({ silent: true }));
    })();
    return () => unlisten?.();
  });

  async function onSetupComplete() {
    await settings.load();
    library.clearFilters();
    await library.load();
  }

  async function refresh() {
    refreshing = true;
    await library.load({ silent: true });
    refreshing = false;
  }

  function openClip(clip: Clip, editing = false) {
    player = { playlist: library.visible, startId: clip.id, editing };
  }

  function closePlayer(lastId: string) {
    player = null;
    gallery?.reveal(lastId);
  }
</script>

<div class="app" class:square={windowState.maximized || windowState.fullscreen}>
  <TitleBar />

  {#if screen === "setup"}
    <div class="body"><Setup oncomplete={onSetupComplete} /></div>
  {:else if screen === "library"}
    <main class="body" inert={player !== null || showSettings}>
      <Toolbar onrefresh={refresh} onsettings={() => (showSettings = true)} {refreshing} shortcuts={player === null} />
      <div class="content">
        {#if library.status === "loading" || library.status === "idle"}
          <Skeleton />
        {:else if library.status === "error"}
          <EmptyState icon="alert" title="Couldn't read your clips folder" action={{ label: "Open settings", run: () => (showSettings = true) }}>
            {library.error}
          </EmptyState>
        {:else if library.clips.length === 0}
          <EmptyState icon="film" title="No clips yet">
            Save an Instant Replay in-game (<kbd>Alt</kbd> + <kbd>F10</kbd> by default) and it will show up here automatically.
          </EmptyState>
        {:else if library.visible.length === 0}
          {#if library.favoritesOnly && library.favoriteCount === 0}
            <EmptyState icon="star" title="No favorites yet" action={{ label: "Show all clips", run: () => library.clearFilters() }}>
              Hover a clip and click its star to keep it here.
            </EmptyState>
          {:else}
            <EmptyState icon="search" title="No clips match these filters" action={{ label: "Clear filters", run: () => library.clearFilters() }} />
          {/if}
        {:else}
          <Gallery
            bind:this={gallery}
            clips={library.visible}
            onopen={(clip) => openClip(clip)}
            onedit={(clip) => openClip(clip, true)}
            {resetKey}
          />
        {/if}
      </div>
    </main>

    {#if player}
      <PlayerModal playlist={player.playlist} startId={player.startId} startEditing={player.editing} onclose={closePlayer} />
    {/if}
    {#if showSettings}
      <SettingsDialog onclose={() => (showSettings = false)} />
    {/if}
  {/if}
</div>

<style>
  /* The window itself is transparent; this is its visible frame. */
  .app {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    border-radius: 14px;
    background: var(--bg);
    /* Makes overlays (player, dialogs) position against and clip to the
       rounded frame instead of the square window. */
    contain: layout paint;
  }
  .app::after {
    content: "";
    position: absolute;
    inset: 0;
    z-index: 100;
    border-radius: inherit;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.07);
    pointer-events: none;
  }
  .app.square {
    border-radius: 0;
  }
  .app.square::after {
    display: none;
  }
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .content {
    flex: 1;
    min-height: 0;
  }
</style>
