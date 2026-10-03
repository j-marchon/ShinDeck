<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon, { type IconName } from "./Icon.svelte";

  let {
    icon,
    title,
    action,
    children,
  }: {
    icon: IconName;
    title: string;
    action?: { label: string; run: () => void };
    children?: Snippet;
  } = $props();
</script>

<div class="empty">
  <div class="badge"><Icon name={icon} size={26} /></div>
  <h2>{title}</h2>
  {#if children}<p>{@render children()}</p>{/if}
  {#if action}<button onclick={action.run}>{action.label}</button>{/if}
</div>

<style>
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 32px;
    text-align: center;
  }
  .badge {
    width: 64px;
    height: 64px;
    margin-bottom: 6px;
    display: grid;
    place-items: center;
    border-radius: 18px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  h2 {
    font-size: 18px;
    font-weight: 700;
  }
  p {
    max-width: 420px;
    color: var(--text-dim);
    line-height: 1.55;
  }
  p :global(kbd) {
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--surface-3);
    font-family: inherit;
    font-size: 12px;
    color: var(--text);
  }
  button {
    margin-top: 10px;
    height: 38px;
    padding: 0 20px;
    border-radius: 8px;
    background: var(--accent);
    color: #000;
    font-weight: 700;
  }
</style>
