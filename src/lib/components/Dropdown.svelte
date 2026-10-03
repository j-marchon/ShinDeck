<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * A button that opens a floating panel below it. Closes on outside click,
   * Escape, or when the content calls `close()`.
   */
  let {
    trigger,
    children,
    align = "left",
    width = 240,
    open = $bindable(false),
  }: {
    trigger: Snippet<[{ open: boolean; toggle: () => void }]>;
    children: Snippet<[() => void]>;
    align?: "left" | "right";
    width?: number;
    open?: boolean;
  } = $props();

  let root: HTMLDivElement;

  const close = () => (open = false);
  const toggle = () => (open = !open);

  function onpointerdown(e: PointerEvent) {
    if (open && !root.contains(e.target as Node)) close();
  }

  function onkeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") {
      e.stopPropagation();
      close();
    }
  }
</script>

<svelte:window {onpointerdown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="dropdown" bind:this={root} {onkeydown}>
  {@render trigger({ open, toggle })}
  {#if open}
    <div class="panel" class:right={align === "right"} style:width="{width}px">
      {@render children(close)}
    </div>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
  }
  .panel {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 30;
    padding: 6px;
    border-radius: 10px;
    background: #141414;
    border: 1px solid #2c2c2c;
    box-shadow:
      0 16px 40px -12px rgb(0 0 0 / 0.9),
      0 0 0 1px rgb(0 0 0 / 0.4);
    transform-origin: top left;
    animation: pop 0.14s cubic-bezier(0.2, 0.9, 0.3, 1.2);
  }
  .panel.right {
    left: auto;
    right: 0;
    transform-origin: top right;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.97);
    }
  }
</style>
