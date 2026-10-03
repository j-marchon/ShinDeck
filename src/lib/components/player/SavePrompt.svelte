<script lang="ts">
  import type { Destination } from "../../api";
  import { editor } from "../../state/editor.svelte";
  import Modal from "../Modal.svelte";
  import SaveModeOptions from "../SaveModeOptions.svelte";

  // svelte-ignore state_referenced_locally
  let remember = $state(editor.prompt?.remember ?? true);
  let choice = $state<Destination>("new");
</script>

<Modal title="How should edits be saved?" onclose={() => editor.answer(null, false)} width={460}>
  <SaveModeOptions value={choice} includeAsk={false} onchange={(mode) => (choice = mode === "replace" ? "replace" : "new")} />

  <label class="remember">
    <input type="checkbox" bind:checked={remember} />
    <span>
      Remember my choice
      <small>{remember ? "You won't be asked again. Change it anytime in Settings." : "You'll be asked after every edit."}</small>
    </span>
  </label>

  <div class="actions">
    <button class="secondary" onclick={() => editor.answer(null, false)}>Cancel</button>
    <button class="primary" onclick={() => editor.answer(choice, remember)}>Save</button>
  </div>
</Modal>

<style>
  .remember {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin-top: 16px;
    font-size: 13px;
    color: var(--text);
    cursor: pointer;
  }
  .remember input {
    margin-top: 2px;
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .remember span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  small {
    font-size: 12px;
    color: var(--text-faint);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 22px;
  }
  .primary,
  .secondary {
    height: 38px;
    padding: 0 22px;
    border-radius: 9px;
    font-weight: 700;
    font-size: 13.5px;
  }
  .primary {
    background: var(--accent);
    color: #000;
  }
  .primary:hover {
    background: var(--accent-hover);
  }
  .secondary {
    color: var(--text-dim);
    border: 1px solid var(--border);
  }
  .secondary:hover {
    color: var(--text);
  }
</style>
