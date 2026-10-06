<script lang="ts">
  import { merge } from "../state/merge.svelte";
  import Icon from "./Icon.svelte";

  /** Shown under the toolbar while the user picks the two clips to merge. */

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !merge.reviewing) merge.stop();
  }
</script>

<svelte:window {onkeydown} />

<div class="bar" role="status">
  <span class="icon"><Icon name="merge" size={16} /></span>
  <span class="text">
    {#if merge.picked.length === 0}
      Select the <b>first</b> clip to merge
    {:else if merge.picked.length === 1}
      Now select the <b>second</b> clip
    {:else}
      Two clips selected
    {/if}
  </span>
  <span class="count">{merge.picked.length} of 2</span>

  <div class="actions">
    {#if merge.picked.length === 2}
      <button class="btn-primary small" onclick={() => merge.review()}>Continue</button>
    {/if}
    <button class="btn-ghost" onclick={() => merge.stop()}>Cancel</button>
  </div>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 22px 6px;
    padding: 7px 8px 7px 12px;
    border-radius: 12px;
    background: rgb(118 185 0 / 0.08);
    box-shadow: inset 0 0 0 1px rgb(118 185 0 / 0.22);
    font-size: 13px;
    color: var(--text-dim);
    animation: drop 0.18s ease;
  }
  .icon {
    color: var(--accent);
    display: grid;
  }
  b {
    color: var(--text);
    font-weight: 600;
  }
  .count {
    font-size: 12px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }
  .actions {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .small {
    height: 34px;
    padding: 0 16px;
    font-size: 13px;
  }
  @keyframes drop {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }
</style>
