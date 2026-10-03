<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Clip, type Settings } from "./lib/api";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import Gallery from "./lib/components/Gallery.svelte";
  import Player from "./lib/components/Player.svelte";
  import Setup from "./lib/components/Setup.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Skeleton from "./lib/components/Skeleton.svelte";
  import Toolbar from "./lib/components/Toolbar.svelte";
  import { library } from "./lib/state/library.svelte";

  let settings = $state<Settings | null>(null);
  let editingFolder = $state(false);
  let refreshing = $state(false);
  let player = $state<{ playlist: Clip[]; startId: string } | null>(null);
  let gallery = $state<ReturnType<typeof Gallery>>();

  const screen = $derived(
    settings === null ? "boot" : !settings.setupComplete || editingFolder ? "setup" : "library",
  );

  // Scroll to top when what's listed changes in kind (not on favorite toggles).
  const resetKey = $derived(JSON.stringify([library.view, library.sortKey, library.descending, library.query]));

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      settings = await api.getSettings();
      if (settings.setupComplete) library.load();
      // New recordings, deletions and renames picked up by the folder watcher.
      unlisten = await api.onLibraryChanged(() => library.load({ silent: true }));
    })();
    return () => unlisten?.();
  });

  async function onSetupComplete() {
    editingFolder = false;
    settings = await api.getSettings();
    library.view = { kind: "all" };
    await library.load();
  }

  async function refresh() {
    refreshing = true;
    await library.load({ silent: true });
    refreshing = false;
  }

  function openClip(clip: Clip) {
    player = { playlist: library.visible, startId: clip.id };
  }

  function closePlayer(lastId: string) {
    player = null;
    gallery?.reveal(lastId);
  }
</script>

{#if screen === "setup"}
  <Setup
    initialPath={settings?.setupComplete ? settings.libraryPath : null}
    oncomplete={onSetupComplete}
    oncancel={settings?.setupComplete ? () => (editingFolder = false) : undefined}
  />
{:else if screen === "library"}
  <div class="shell" inert={player !== null}>
    <Sidebar onsettings={() => (editingFolder = true)} />
    <main>
      <Toolbar onrefresh={refresh} {refreshing} shortcuts={player === null} />
      <div class="content">
        {#if library.status === "loading" || library.status === "idle"}
          <Skeleton />
        {:else if library.status === "error"}
          <EmptyState icon="alert" title="Couldn't read your clips folder" action={{ label: "Change folder", run: () => (editingFolder = true) }}>
            {library.error}
          </EmptyState>
        {:else if library.clips.length === 0}
          <EmptyState icon="film" title="No clips yet">
            Save an Instant Replay in-game (<kbd>Alt</kbd> + <kbd>F10</kbd> by default) and it will show up here automatically.
          </EmptyState>
        {:else if library.visible.length === 0}
          {#if library.query}
            <EmptyState icon="search" title="No matches">Nothing matches "{library.query}".</EmptyState>
          {:else if library.view.kind === "favorites"}
            <EmptyState icon="star" title="No favorites yet">Hover a clip and click its star to keep it here.</EmptyState>
          {:else}
            <EmptyState icon="film" title="Nothing here" />
          {/if}
        {:else}
          <Gallery bind:this={gallery} clips={library.visible} onopen={openClip} {resetKey} />
        {/if}
      </div>
    </main>
  </div>

  {#if player}
    <Player playlist={player.playlist} startId={player.startId} onclose={closePlayer} />
  {/if}
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: 248px 1fr;
    height: 100%;
  }
  main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .content {
    flex: 1;
    min-height: 0;
  }
</style>
