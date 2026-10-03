<script lang="ts">
  import { api } from "../api";
  import { windowState } from "../state/window.svelte";
  import Logo from "./Logo.svelte";
</script>

<!-- Replaces the native title bar (decorations: false). It shares the app's
     background, so the window reads as one surface. `data-tauri-drag-region`
     moves the window and double-click maximizes, like a native bar. -->
<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <Logo size={15} />
    <span>Shin<b>Deck</b></span>
  </div>

  <div class="controls">
    <button class="ctl" aria-label="Minimize" title="Minimize" onclick={() => api.minimizeWindow()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 5h8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /></svg>
    </button>
    <button
      class="ctl"
      aria-label={windowState.maximized ? "Restore" : "Maximize"}
      title={windowState.maximized ? "Restore" : "Maximize"}
      onclick={() => api.toggleMaximizeWindow()}
    >
      {#if windowState.maximized}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.1">
          <rect x="1" y="3" width="6" height="6" rx="1.4" /><path d="M3 3V2.4A1.4 1.4 0 0 1 4.4 1h3.2A1.4 1.4 0 0 1 9 2.4v3.2A1.4 1.4 0 0 1 7.6 7H7" />
        </svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.1">
          <rect x="1" y="1" width="8" height="8" rx="1.8" />
        </svg>
      {/if}
    </button>
    <button class="ctl close" aria-label="Close" title="Close" onclick={() => api.closeWindow()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" /></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 30px;
    flex-shrink: 0;
    user-select: none;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 7px;
    padding-left: 14px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-faint);
  }
  .brand :global(svg),
  .brand span {
    pointer-events: none;
  }
  .brand b {
    color: var(--text-dim);
    font-weight: 600;
  }

  .controls {
    display: flex;
    gap: 2px;
    height: 100%;
    padding: 4px 6px 0 0;
  }
  .ctl {
    width: 36px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 7px;
    color: var(--text-faint);
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }
  .ctl:hover {
    background: rgb(118 185 0 / 0.12);
    color: var(--accent);
  }
  .ctl.close:hover {
    background: #e5383b;
    color: #fff;
  }
</style>
