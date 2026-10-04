<script lang="ts">
  import { deletion } from "../state/deletion.svelte";
  import { settings } from "../state/settings.svelte";
  import Modal from "./Modal.svelte";

  let dontAsk = $state(false);

  const clip = $derived(deletion.pending);
  // With confirmation turned off the dialog only appears to report a failure.
  const confirming = $derived(settings.value?.confirmDelete !== false);
</script>

{#if clip && (confirming || deletion.error)}
  <Modal title={confirming ? "Remove clip?" : "Couldn't remove clip"} onclose={() => deletion.cancel()} width={420}>
    <p class="text">
      {#if confirming}
        <b>{clip.name}</b> will be moved to the Recycle Bin. You can restore it from there.
      {:else}
        <b>{clip.name}</b> is still in your library.
      {/if}
    </p>
    {#if deletion.error}<p class="error">{deletion.error}</p>{/if}

    {#if confirming}
      <label class="check">
        <input type="checkbox" bind:checked={dontAsk} />
        <span>Don't ask me again</span>
      </label>
    {/if}

    <footer>
      <button class="btn" onclick={() => deletion.cancel()} disabled={deletion.busy}>
        {confirming ? "Cancel" : "Close"}
      </button>
      {#if confirming}
        <button class="btn danger" onclick={() => deletion.confirm(!dontAsk)} disabled={deletion.busy}>
          {deletion.busy ? "Removing…" : "Move to Recycle Bin"}
        </button>
      {/if}
    </footer>
  </Modal>
{/if}

<style>
  .text {
    font-size: 13.5px;
    line-height: 1.5;
    color: var(--text-dim);
    overflow-wrap: anywhere;
  }
  .text b {
    color: var(--text);
    font-weight: 500;
  }
  .error {
    margin-top: 10px;
    color: var(--danger);
    font-size: 13px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 16px;
    font-size: 13px;
    color: var(--text-dim);
    cursor: pointer;
    user-select: none;
  }
  .check input {
    width: 15px;
    height: 15px;
    margin: 0;
    accent-color: var(--accent);
    cursor: pointer;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 22px;
  }
  .btn {
    height: 34px;
    padding: 0 16px;
    border-radius: 10px;
    background: var(--glass-2);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
  }
  .btn:hover:not(:disabled) {
    background: var(--glass-3);
  }
  .btn:disabled {
    opacity: 0.6;
  }
  .btn.danger {
    background: var(--danger);
    color: #2a0b09;
  }
  .btn.danger:hover:not(:disabled) {
    background: var(--danger);
    filter: brightness(1.12);
  }
</style>
