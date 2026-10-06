<script lang="ts">
  import { tick } from "svelte";
  import type { Clip } from "../api";
  import { library } from "../state/library.svelte";
  import Icon from "./Icon.svelte";

  /**
   * A clip title that turns into a text box on click and renames the file
   * on disk when confirmed (Enter or clicking away). Escape cancels.
   */
  let { clip, onrenamed }: { clip: Clip; onrenamed?: (clip: Clip) => void } = $props();

  let editing = $state(false);
  let saving = $state(false);
  let draft = $state("");
  let error = $state<string | null>(null);
  let input = $state<HTMLInputElement>();

  async function start(e: MouseEvent) {
    // Shift-click belongs to the card (it starts multi-select).
    if (e.shiftKey) return;
    e.stopPropagation();
    draft = clip.name;
    error = null;
    editing = true;
    await tick();
    input?.focus();
    input?.select();
  }

  async function commit() {
    if (!editing || saving) return;
    const name = draft.trim();
    if (name === clip.name || !name) {
      editing = false;
      error = null;
      return;
    }
    saving = true;
    try {
      const renamed = await library.rename(clip.id, name);
      editing = false;
      error = null;
      onrenamed?.(renamed);
    } catch (e) {
      error = String(e).replace(/^Error: /, "");
    } finally {
      saving = false;
    }
    if (error) {
      // Re-enabled now; let the user fix the name right away.
      await tick();
      input?.focus();
    }
  }

  function cancel() {
    editing = false;
    error = null;
  }

  /** Clicking away after a failed attempt reverts instead of retrying. */
  function onblur() {
    if (error) cancel();
    else commit();
  }

  function onkeydown(e: KeyboardEvent) {
    // Keep typing from triggering card or global shortcuts.
    e.stopPropagation();
    if (e.key === "Enter") commit();
    if (e.key === "Escape") cancel();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="inline-name" class:editing class:invalid={error} onclick={(e) => !e.shiftKey && e.stopPropagation()}>
  {#if editing}
    <input
      bind:this={input}
      bind:value={draft}
      disabled={saving}
      spellcheck="false"
      aria-label="Clip name"
      {onblur}
      {onkeydown}
      oninput={() => (error = null)}
    />
    {#if error}<div class="error" role="alert">{error}</div>{/if}
  {:else}
    <button class="title" title="Click to rename" onclick={start}>
      <span class="text">{clip.name}</span>
      <span class="hint"><Icon name="pencil" size={12} /></span>
    </button>
  {/if}
</div>

<style>
  .inline-name {
    position: relative;
    min-width: 0;
  }
  .title,
  input {
    width: calc(100% + 6px);
    height: 24px;
    padding: 0 6px;
    margin-left: -6px;
    border-radius: 7px;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    text-align: left;
    cursor: text;
    transition: background 0.15s ease;
  }
  .title:hover {
    background: var(--glass-2);
  }
  .text {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hint {
    color: var(--text-faint);
    opacity: 0;
    transition: opacity 0.15s ease;
  }
  .title:hover .hint {
    opacity: 1;
  }
  input {
    background: var(--glass-2);
    border: none;
    outline: none;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.18);
  }
  .invalid input {
    box-shadow: inset 0 0 0 1px var(--danger);
    animation: shake 0.3s;
  }
  .error {
    position: absolute;
    top: calc(100% + 4px);
    left: -6px;
    z-index: 5;
    padding: 5px 9px;
    border-radius: 8px;
    background: rgb(40 14 14 / 0.9);
    backdrop-filter: blur(10px);
    color: #ffb4b0;
    font-size: 12px;
    white-space: nowrap;
  }
  @keyframes shake {
    25% {
      transform: translateX(-3px);
    }
    75% {
      transform: translateX(3px);
    }
  }
</style>
