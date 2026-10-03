<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  /** Centered dialog over a dimmed backdrop. Escape or backdrop click closes. */
  let {
    title,
    onclose,
    width = 480,
    children,
  }: { title: string; onclose: () => void; width?: number; children: Snippet } = $props();

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopImmediatePropagation();
      onclose();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose}>
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    style:width="{width}px"
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{title}</h2>
      <button class="x" aria-label="Close" onclick={onclose}><Icon name="close" size={16} /></button>
    </header>
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgb(0 0 0 / 0.72);
    backdrop-filter: blur(4px);
    animation: fade 0.15s ease;
  }
  .dialog {
    max-width: 100%;
    max-height: 100%;
    overflow-y: auto;
    padding: 22px 24px 24px;
    border-radius: 14px;
    background: var(--bg-elev);
    border: 1px solid rgb(118 185 0 / 0.35);
    box-shadow:
      0 0 40px -10px rgb(118 185 0 / 0.35),
      0 30px 60px -20px #000;
    animation: rise 0.2s cubic-bezier(0.2, 0.9, 0.3, 1.1);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 18px;
  }
  h2 {
    font-size: 17px;
    font-weight: 700;
  }
  .x {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-dim);
  }
  .x:hover {
    background: var(--surface-3);
    color: var(--text);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.98);
    }
  }
</style>
