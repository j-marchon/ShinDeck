<script lang="ts">
  import { api } from "../api";
  import { library } from "../state/library.svelte";
  import { settings } from "../state/settings.svelte";
  import Icon from "./Icon.svelte";
  import DeleteConfirmOptions from "./DeleteConfirmOptions.svelte";
  import MergeModeOptions from "./MergeModeOptions.svelte";
  import Modal from "./Modal.svelte";
  import SaveModeOptions from "./SaveModeOptions.svelte";

  let { onclose }: { onclose: () => void } = $props();
  let error = $state<string | null>(null);

  async function changeFolder() {
    const picked = await api.pickFolder(settings.value?.libraryPath ?? undefined);
    if (!picked) return;
    try {
      settings.value = await api.setLibraryPath(picked);
      error = null;
      library.clearFilters();
      await library.load();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<Modal title="Settings" {onclose} width={500}>
  <section>
    <h3>Clips folder</h3>
    <div class="folder">
      <Icon name="folder" size={17} />
      <span class="path" title={settings.value?.libraryPath ?? ""}><bdi>{settings.value?.libraryPath}</bdi></span>
      <button class="btn" onclick={changeFolder}>Change…</button>
    </div>
    {#if error}<p class="error">{error}</p>{/if}
  </section>

  <section>
    <h3>When saving an edit</h3>
    <SaveModeOptions value={settings.value?.saveMode ?? "ask"} onchange={(mode) => settings.setSaveMode(mode)} />
  </section>

  <section>
    <h3>When removing a clip</h3>
    <DeleteConfirmOptions
      value={settings.value?.confirmDelete ?? true}
      onchange={(confirm) => settings.setConfirmDelete(confirm)}
    />
  </section>

  <section>
    <h3>When merging clips</h3>
    <MergeModeOptions
      value={settings.value?.mergeReplace ?? false}
      onchange={(replace) => settings.setMergeReplace(replace)}
    />
  </section>

  {#if settings.value && !settings.value.editingAvailable}
    <p class="warn"><Icon name="alert" size={15} /> The bundled ffmpeg is missing, so editing is unavailable. Reinstall ShinDeck to restore it.</p>
  {/if}
</Modal>

<style>
  section + section {
    margin-top: 22px;
  }
  h3 {
    margin-bottom: 9px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-faint);
  }
  .folder {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 6px 0 14px;
    border-radius: 12px;
    background: var(--glass);
    color: var(--text-faint);
  }
  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
    color: var(--text);
    font-size: 13px;
  }
  .btn {
    height: 32px;
    padding: 0 14px;
    border-radius: 9px;
    background: var(--glass-2);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
  }
  .btn:hover {
    background: var(--glass-3);
  }
  .error {
    margin-top: 8px;
    color: var(--danger);
    font-size: 13px;
  }
  .warn {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin-top: 20px;
    font-size: 12.5px;
    color: #e0b341;
  }
</style>
