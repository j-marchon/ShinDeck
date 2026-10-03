<script lang="ts">
  import type { SaveMode } from "../api";
  import Icon, { type IconName } from "./Icon.svelte";

  /** Radio cards for "what happens to an edited clip". */
  let {
    value,
    onchange,
    includeAsk = true,
  }: { value: SaveMode | null; onchange: (mode: SaveMode) => void; includeAsk?: boolean } = $props();

  const OPTIONS: { mode: SaveMode; icon: IconName; title: string; detail: string }[] = [
    { mode: "new", icon: "copy", title: "Save as a new clip", detail: "Keeps the original untouched" },
    { mode: "replace", icon: "replace", title: "Replace the original", detail: "Overwrites the clip with the edit" },
    { mode: "ask", icon: "settings", title: "Ask me every time", detail: "Choose after each edit" },
  ];
</script>

<div class="options" role="radiogroup">
  {#each OPTIONS.filter((o) => includeAsk || o.mode !== "ask") as option (option.mode)}
    <button
      class="option"
      class:selected={value === option.mode}
      role="radio"
      aria-checked={value === option.mode}
      onclick={() => onchange(option.mode)}
    >
      <span class="icon"><Icon name={option.icon} size={18} /></span>
      <span class="text">
        <b>{option.title}</b>
        <small>{option.detail}</small>
      </span>
      <span class="dot"></span>
    </button>
  {/each}
</div>

<style>
  .options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--surface);
    text-align: left;
    transition:
      border-color 0.12s ease,
      background 0.12s ease;
  }
  .option:hover {
    border-color: #3a3a3a;
  }
  .option.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .icon {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: var(--surface-3);
    color: var(--text-dim);
  }
  .selected .icon {
    background: var(--accent);
    color: #000;
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  b {
    font-size: 13.5px;
    color: var(--text);
  }
  small {
    font-size: 12px;
    color: var(--text-faint);
  }
  .dot {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid #444;
  }
  .selected .dot {
    border-color: var(--accent);
    background: radial-gradient(circle, var(--accent) 45%, transparent 50%);
  }
</style>
