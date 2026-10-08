<script lang="ts">
  import { library } from "../state/library.svelte";
  import { numberedPattern, selection } from "../state/selection.svelte";
  import { plural } from "../util/format";
  import { settings } from "../state/settings.svelte";
  import { formatSize } from "../util/format";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";
  import { PRESETS } from "./player/EditPanel.svelte";

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

  // Compress dialog: a preset or a custom size, and whether to keep the originals.
  let choice = $state<string>(PRESETS[0].id);
  let customMb = $state<number | null>(null);
  // svelte-ignore state_referenced_locally
  let replace = $state(settings.defaultDestination === "replace");

  const target = $derived.by(() => {
    if (choice === "custom") {
      return customMb && customMb > 0
        ? { label: `${customMb} MB`, bytes: Math.round(customMb * 1_000_000) }
        : null;
    }
    const preset = PRESETS.find((p) => p.id === choice);
    return preset ? { label: preset.name, bytes: preset.bytes } : null;
  });
  const needed = $derived(target ? selection.clips.filter((c) => c.size > target.bytes).length : 0);
  const total = $derived(selection.clips.reduce((t, c) => t + c.size, 0));

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
{:else if selection.dialog === "compress"}
  <Modal title="Compress {plural(selection.count, 'clip')}" onclose={() => selection.closeDialog()} width={460}>
    <p class="text">Each clip is re-encoded to fit the size you pick ({formatSize(total)} selected now).</p>
    <div class="tiles" role="radiogroup" aria-label="Compression target">
      {#each PRESETS as preset (preset.id)}
        <button
          class="tile"
          class:on={choice === preset.id}
          role="radio"
          aria-checked={choice === preset.id}
          disabled={selection.busy}
          onclick={() => (choice = preset.id)}
        >
          <b>{preset.name}</b><small>{preset.bytes / 1_000_000} MB</small>
        </button>
      {/each}
      <button class="tile" class:on={choice === "custom"} role="radio" aria-checked={choice === "custom"} disabled={selection.busy} onclick={() => (choice = "custom")}>
        <b>Custom</b>
        {#if choice === "custom"}
          <input type="number" min="1" step="1" placeholder="MB" bind:value={customMb} onclick={(e) => e.stopPropagation()} />
        {:else}<small>Pick a size</small>{/if}
      </button>
    </div>
    {#if target}
      <p class="text">
        {#if needed === 0}
          Every selected clip is already under <b>{formatSize(target.bytes)}</b>.
        {:else}
          <b>{plural(needed, "clip")}</b> will be compressed{#if needed < selection.count}; {selection.count - needed} already fit{/if}.
        {/if}
      </p>
    {/if}
    <label class="replace">
      <input type="checkbox" bind:checked={replace} disabled={selection.busy} />
      <span>Replace the originals {replace ? "(the compressed file takes their place)" : "(saved as new clips)"}</span>
    </label>
    {#if selection.busy}
      <div class="job">
        <div class="job-head">
          <span>Compressing {Math.min(selection.current + 1, needed)} of {needed}…</span>
          <b>{Math.round(selection.progress * 100)}%</b>
        </div>
        <div class="track"><div class="fill" style:width="{selection.progress * 100}%"></div></div>
      </div>
    {/if}
    {#if selection.error}<p class="error">{selection.error}</p>{/if}
    <footer>
      {#if selection.busy}
        <button class="btn-ghost" onclick={() => selection.cancelCompress()}>Cancel</button>
      {:else}
        <button class="btn-ghost" onclick={() => selection.closeDialog()}>Cancel</button>
        <button
          class="btn-compress"
          disabled={!target || needed === 0}
          onclick={() => target && selection.compress(target, replace ? "replace" : "new")}
        >
          <Icon name="compress" size={15} /> Compress
        </button>
      {/if}
    </footer>
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
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    margin-top: 14px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 10px 6px;
    border-radius: 10px;
    background: var(--glass);
    box-shadow: inset 0 0 0 1px transparent;
    font-size: 13px;
  }
  .tile:hover:not(:disabled) {
    background: var(--glass-2);
  }
  .tile.on {
    box-shadow: inset 0 0 0 1px #37b24d;
  }
  .tile small {
    font-size: 12px;
    color: var(--text-faint);
  }
  .tile input {
    width: 100%;
    height: 20px;
    border: none;
    outline: none;
    background: none;
    text-align: center;
    color: var(--text);
    font-size: 12px;
  }
  .replace {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 14px;
    font-size: 13px;
    cursor: pointer;
  }
  .replace input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .job {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .job-head {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    color: var(--text-dim);
  }
  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--glass-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: #37b24d;
    transition: width 0.2s ease;
  }
  .btn-compress {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 34px;
    padding: 0 16px;
    border-radius: 10px;
    background: #2f9e44;
    color: #fff;
    font-size: 13px;
    font-weight: 600;
  }
  .btn-compress:hover:not(:disabled) {
    background: #37b24d;
  }
  .btn-compress:disabled {
    opacity: 0.5;
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
