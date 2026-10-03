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
    gap: 6px;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 14px;
    border-radius: 12px;
    background: var(--glass);
    text-align: left;
    transition: background 0.15s ease;
  }
  .option:hover {
    background: var(--glass-2);
  }
  .option.selected {
    background: var(--glass-2);
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.1);
  }
  .icon {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: var(--glass);
    color: var(--text-dim);
  }
  .selected .icon {
    color: var(--accent);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  b {
    font-size: 13.5px;
    font-weight: 500;
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
    box-shadow: inset 0 0 0 1.5px rgb(255 255 255 / 0.25);
  }
  .selected .dot {
    box-shadow: inset 0 0 0 1.5px var(--accent);
    background: radial-gradient(circle, var(--accent) 38%, transparent 42%);
  }
</style>
