<script lang="ts">
  import { library } from "../state/library.svelte";
  import { numberedPattern, selection } from "../state/selection.svelte";
  import { plural } from "../util/format";
  import Modal from "./Modal.svelte";

  /** Confirms a batch rename or removal of the selected clips. */

  let name = $state("");

  const trimmed = $derived(name.trim());
  // Numbering continues after "{name} #n" names already in the library.
  const first = $derived.by(() => {
    if (!trimmed) return 1;
    const pattern = numberedPattern(trimmed);
    let max = 0;
    for (const clip of library.clips) max = Math.max(max, Number(pattern.exec(clip.name)?.[1] ?? 0));
    return max + 1;
  });
  const last = $derived(first + selection.count - 1);

  function focus(node: HTMLInputElement) {
    node.focus();
  }

  function submit(e: SubmitEvent) {
    e.preventDefault();
    if (trimmed) selection.rename(trimmed);
  }
</script>

{#if selection.dialog === "rename"}
  <Modal title="Rename {plural(selection.count, 'clip')}" onclose={() => selection.closeDialog()} width={440}>
    <form onsubmit={submit}>
      <label class="field">
        <span>New name</span>
        <input
          use:focus
          bind:value={name}
          oninput={() => (selection.error = null)}
          disabled={selection.busy}
          placeholder="e.g. Ace clutches"
          spellcheck="false"
          maxlength="170"
        />
      </label>
      <p class="text">
        {#if trimmed}
          Clips become <b>“{trimmed} #{first}”</b>{#if selection.count > 1}{" "}to <b>“{trimmed} #{last}”</b>{/if}, oldest first.
        {:else}
          Each clip gets the name followed by its number: “Name #1”, “Name #2”…
        {/if}
      </p>
      {#if selection.error}<p class="error">{selection.error}</p>{/if}
      <footer>
        <button type="button" class="btn-ghost" onclick={() => selection.closeDialog()} disabled={selection.busy}>Cancel</button>
        <button type="submit" class="btn-primary" disabled={!trimmed || selection.busy}>
          {selection.busy ? "Renaming…" : "Rename"}
        </button>
      </footer>
    </form>
  </Modal>
{:else if selection.dialog === "delete"}
  <Modal title="Remove {plural(selection.count, 'clip')}?" onclose={() => selection.closeDialog()} width={420}>
    <p class="text">
      {#if selection.count === 1}
        <b>{selection.clips[0].name}</b> will be moved to the Recycle Bin.
      {:else}
        <b>{plural(selection.count, "clip")}</b> will be moved to the Recycle Bin.
      {/if}
      You can restore them from there.
    </p>
    {#if selection.error}<p class="error">{selection.error}</p>{/if}
    <footer>
      <button class="btn-ghost" onclick={() => selection.closeDialog()} disabled={selection.busy}>Cancel</button>
      <button class="btn-danger" onclick={() => selection.remove()} disabled={selection.busy}>
        {selection.busy ? "Removing…" : "Move to Recycle Bin"}
      </button>
    </footer>
  </Modal>
{/if}

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
    font-size: 12px;
    color: var(--text-faint);
  }
  .field input {
    height: 38px;
    padding: 0 12px;
    border-radius: 10px;
    border: none;
    outline: none;
    background: var(--glass-2);
    color: var(--text);
    font-size: 13.5px;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.06);
  }
  .field input:focus {
    box-shadow: inset 0 0 0 1px rgb(118 185 0 / 0.6);
  }
  .text {
    margin-top: 12px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-dim);
    overflow-wrap: anywhere;
  }
  .text:first-child {
    margin-top: 0;
  }
  .text b {
    color: var(--text);
    font-weight: 500;
  }
  .error {
    margin-top: 10px;
    color: var(--danger);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 22px;
  }
  .btn-primary {
    height: 34px;
    padding: 0 16px;
    font-size: 13px;
  }
  .btn-danger {
    height: 34px;
    padding: 0 16px;
    border-radius: 10px;
    background: var(--danger);
    color: #2a0b09;
    font-size: 13px;
    font-weight: 500;
  }
  .btn-danger:hover:not(:disabled) {
    filter: brightness(1.12);
  }
  .btn-danger:disabled {
    opacity: 0.6;
  }
</style>
