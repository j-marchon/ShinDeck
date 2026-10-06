<script lang="ts">
  import { api } from "../api";
  import { library } from "../state/library.svelte";
  import { merge } from "../state/merge.svelte";
  import { formatDuration, formatSize } from "../util/format";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  const total = $derived(merge.picked.reduce((t, c) => t + (c.durationMs ?? 0), 0));
  const firstGame = $derived(library.gameName(merge.picked[0]?.game ?? ""));
  const mixedGames = $derived(new Set(merge.picked.map((c) => c.game)).size > 1);
</script>

<Modal title="Merge clips" onclose={() => merge.back()} width={520}>
  <p class="hint">The clips play back to back in this order.</p>

  <ol class="parts">
    {#each merge.picked as clip, i (clip.id)}
      <li class="part">
        <span class="num">{i + 1}</span>
        <img src={api.thumbnailUrl(clip)} alt="" />
        <span class="info">
          <b>{clip.name}</b>
          <small>
            {library.gameName(clip.game)}
            {#if clip.durationMs}· {formatDuration(clip.durationMs)}{/if}
            · {formatSize(clip.size)}
          </small>
        </span>
      </li>
      {#if i === 0}
        <li class="swap-row">
          <button class="swap" onclick={() => merge.swap()} disabled={merge.running} title="Swap order">
            <Icon name="swap" size={14} /> Swap order
          </button>
        </li>
      {/if}
    {/each}
  </ol>

  <p class="result">
    Saved as <b>“{firstGame} Merge #…”</b>{#if total}, {formatDuration(total)} long{/if}.
    {#if mixedGames}<br />The clips are from different games; the merge goes into {firstGame}.{/if}
  </p>

  <label class="replace">
    <input type="checkbox" bind:checked={merge.replace} disabled={merge.running} />
    <span>
      Replace the originals
      <small>
        {merge.replace
          ? "Both clips move to the Recycle Bin once the merge is saved."
          : "Both clips stay in your library."}
      </small>
    </span>
  </label>

  {#if merge.running}
    <div class="job">
      <div class="job-head">
        <span>Merging…</span>
        <b>{Math.round(merge.progress * 100)}%</b>
      </div>
      <div class="track"><div class="fill" style:width="{merge.progress * 100}%"></div></div>
      <button class="link" onclick={() => merge.cancel()}>Cancel</button>
    </div>
  {:else}
    {#if merge.error}
      <p class="error"><Icon name="alert" size={14} /> {merge.error}</p>
    {/if}
    <div class="actions">
      <button class="btn-ghost" onclick={() => merge.back()}>Change selection</button>
      <button class="btn-primary" onclick={() => merge.run()}><Icon name="merge" size={15} /> Merge</button>
    </div>
  {/if}
</Modal>

<style>
  .hint {
    margin-top: -8px;
    margin-bottom: 14px;
    font-size: 13px;
    color: var(--text-faint);
  }
  .parts {
    list-style: none;
    display: flex;
    flex-direction: column;
  }
  .part {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 12px 9px 10px;
    border-radius: 12px;
    background: var(--glass);
    min-width: 0;
  }
  .num {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 50%;
    background: var(--accent);
    color: #050505;
    font-size: 12px;
    font-weight: 700;
  }
  img {
    width: 88px;
    aspect-ratio: 16 / 9;
    flex-shrink: 0;
    object-fit: cover;
    border-radius: 7px;
    background: var(--glass-2);
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .info b {
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  small {
    font-size: 12px;
    color: var(--text-faint);
  }
  .swap-row {
    display: flex;
    justify-content: center;
    padding: 5px 0;
  }
  .swap {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 12px;
    border-radius: 9px;
    font-size: 12.5px;
    color: var(--text-dim);
    background: var(--glass);
  }
  .swap:hover:not(:disabled) {
    color: var(--accent);
    background: var(--glass-2);
  }
  .swap:disabled {
    opacity: 0.4;
  }
  .result {
    margin-top: 14px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-dim);
    overflow-wrap: anywhere;
  }
  .result b {
    color: var(--text);
    font-weight: 500;
  }
  .replace {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin-top: 14px;
    font-size: 13px;
    color: var(--text);
    cursor: pointer;
  }
  .replace input {
    margin-top: 2px;
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
    cursor: pointer;
  }
  .replace span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .error {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-top: 14px;
    font-size: 13px;
    color: var(--danger);
    overflow-wrap: anywhere;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 22px;
  }
  .job {
    margin-top: 20px;
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
  .job-head b {
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }
  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--glass-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }
  .link {
    align-self: flex-end;
    font-size: 12.5px;
    color: var(--text-faint);
  }
  .link:hover {
    color: var(--text);
  }
</style>
