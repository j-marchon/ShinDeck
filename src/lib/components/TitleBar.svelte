<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../api";
  import Logo from "./Logo.svelte";

  let maximized = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    const sync = async () => (maximized = await api.isMaximized());
    sync();
    api.onWindowResized(sync).then((u) => (unlisten = u));
    return () => unlisten?.();
  });
</script>

<!-- The native title bar is hidden (decorations: false); this replaces it.
     `data-tauri-drag-region` makes the bar move the window and double-click
     maximize, exactly like a native one. -->
<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <Logo size={18} />
    <span>Shin<b>Deck</b></span>
  </div>

  <div class="controls">
    <button class="ctl" aria-label="Minimize" title="Minimize" onclick={() => api.minimizeWindow()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M0 5h10" stroke="currentColor" stroke-width="1" /></svg>
    </button>
    <button
      class="ctl"
      aria-label={maximized ? "Restore" : "Maximize"}
      title={maximized ? "Restore" : "Maximize"}
      onclick={() => api.toggleMaximizeWindow()}
    >
      {#if maximized}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="0.5" y="2.5" width="7" height="7" rx="1" /><path d="M2.5 2.5v-1a1 1 0 0 1 1-1h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-1" />
        </svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="0.5" y="0.5" width="9" height="9" rx="1.5" />
        </svg>
      {/if}
    </button>
    <button class="ctl close" aria-label="Close" title="Close" onclick={() => api.closeWindow()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M0.5 0.5l9 9M9.5 0.5l-9 9" stroke="currentColor" stroke-width="1.1" /></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    flex-shrink: 0;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 12px;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-dim);
  }
  .brand :global(svg),
  .brand span {
    pointer-events: none;
  }
  .brand b {
    color: var(--accent);
    font-weight: 800;
  }

  .controls {
    display: flex;
    height: 100%;
  }
  .ctl {
    width: 46px;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--text-dim);
    transition:
      background 0.12s ease,
      color 0.12s ease,
      box-shadow 0.12s ease;
  }
  .ctl:hover {
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: inset 0 -2px 0 var(--accent);
  }
  .ctl:hover svg {
    filter: drop-shadow(0 0 4px rgb(118 185 0 / 0.8));
  }
  .ctl.close:hover {
    background: #e81123;
    color: #fff;
    box-shadow: none;
  }
  .ctl.close:hover svg {
    filter: none;
  }
  .ctl:active {
    opacity: 0.8;
  }
</style>
