<script lang="ts">
  import { onMount } from "svelte";
  import { api, type FolderSummary } from "../api";
  import { plural } from "../util/format";
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  let {
    initialPath = null,
    oncomplete,
    oncancel,
  }: {
    /** Current folder when re-opened from the sidebar; null on first launch. */
    initialPath?: string | null;
    oncomplete: () => void;
    oncancel?: () => void;
  } = $props();

  let path = $state("");
  let defaultPath = $state("");
  let summary = $state<FolderSummary | null>(null);
  let checking = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    defaultPath = await api.defaultClipsFolder();
    path = initialPath ?? defaultPath;
  });

  // Re-inspect the folder (debounced) whenever the path changes.
  $effect(() => {
    const current = path.trim();
    summary = null;
    error = null;
    if (!current) return;
    checking = true;
    let cancelled = false;
    const timer = setTimeout(async () => {
      const result = await api.inspectFolder(current);
      if (!cancelled) {
        summary = result;
        checking = false;
      }
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  async function browse() {
    const picked = await api.pickFolder(path || defaultPath);
    if (picked) path = picked;
  }

  async function confirm() {
    saving = true;
    error = null;
    try {
      await api.setLibraryPath(path.trim());
      oncomplete();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="setup">
  <div class="glow" aria-hidden="true"></div>
  <div class="panel">
    <div class="hero">
      <Logo size={64} />
      <h1>{initialPath ? "Clips folder" : "Welcome to ShinDeck"}</h1>
      <p>
        Choose the folder where ShadowPlay saves your recordings. Each game gets its own folder inside it, and
        ShinDeck organises your clips the same way.
      </p>
    </div>

    <form
      onsubmit={(e) => {
        e.preventDefault();
        if (summary?.exists) confirm();
      }}
    >
      <label class="field-label" for="clips-path">Clips folder</label>
      <div class="field">
        <span class="field-icon"><Icon name="folder" size={17} /></span>
        <input id="clips-path" bind:value={path} spellcheck="false" autocomplete="off" />
        <button type="button" class="browse" onclick={browse}>Browse…</button>
      </div>

      <div class="status" aria-live="polite">
        {#if checking}
          <span class="muted">Checking folder…</span>
        {:else if summary && !summary.exists}
          <span class="bad"><Icon name="alert" size={15} /> This folder doesn't exist.</span>
        {:else if summary && summary.clipCount === 0}
          <span class="warn">
            <Icon name="alert" size={15} /> No clips here yet. That's fine: new clips show up automatically.
          </span>
        {:else if summary}
          <span class="good">
            <Icon name="check" size={15} />
            Found {plural(summary.clipCount, "clip")} across {plural(summary.gameCount, "game")}.
          </span>
        {/if}
        {#if defaultPath && path.trim() !== defaultPath}
          <button type="button" class="link" onclick={() => (path = defaultPath)}>Use ShadowPlay default</button>
        {/if}
      </div>

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <div class="actions">
        {#if oncancel}
          <button type="button" class="secondary" onclick={oncancel}>Cancel</button>
        {/if}
        <button type="submit" class="primary" disabled={!summary?.exists || saving}>
          {initialPath ? "Save" : "Continue"}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .setup {
    position: relative;
    height: 100%;
    display: grid;
    place-items: center;
    padding: 32px;
    overflow: hidden;
    background: var(--bg);
  }
  .glow {
    position: absolute;
    width: 900px;
    height: 900px;
    top: -520px;
    left: 50%;
    transform: translateX(-50%);
    background: radial-gradient(circle, rgb(118 185 0 / 0.16), transparent 62%);
    pointer-events: none;
  }

  .panel {
    position: relative;
    width: min(600px, 100%);
    padding: 40px 44px 36px;
    border-radius: 16px;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    box-shadow: 0 30px 80px -30px rgb(0 0 0 / 0.8);
  }
  .panel::before {
    content: "";
    position: absolute;
    top: 0;
    left: 44px;
    right: 44px;
    height: 2px;
    background: var(--accent);
    border-radius: 0 0 2px 2px;
  }

  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
    margin-bottom: 32px;
  }
  h1 {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .hero p {
    max-width: 460px;
    color: var(--text-dim);
    font-size: 14px;
    line-height: 1.55;
  }

  .field-label {
    display: block;
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-faint);
  }
  .field {
    display: flex;
    align-items: center;
    height: 46px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    transition: border-color 0.12s ease;
  }
  .field:focus-within {
    border-color: var(--accent);
  }
  .field-icon {
    padding: 0 10px 0 14px;
    color: var(--accent);
  }
  .field input {
    flex: 1;
    min-width: 0;
    height: 100%;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 14px;
  }
  .browse {
    height: 34px;
    margin-right: 6px;
    padding: 0 14px;
    border-radius: 7px;
    background: var(--surface-3);
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .browse:hover {
    background: #333;
  }

  .status {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 22px;
    margin-top: 12px;
    font-size: 13px;
  }
  .status span {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }
  .muted {
    color: var(--text-faint);
  }
  .good {
    color: var(--accent);
  }
  .warn {
    color: #e0b341;
  }
  .bad {
    color: var(--danger);
  }
  .link {
    color: var(--text-dim);
    font-size: 12.5px;
    text-decoration: underline;
    text-underline-offset: 3px;
    white-space: nowrap;
  }
  .link:hover {
    color: var(--accent);
  }
  .error {
    margin-top: 12px;
    color: var(--danger);
    font-size: 13px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 30px;
  }
  .primary,
  .secondary {
    height: 42px;
    padding: 0 26px;
    border-radius: 9px;
    font-size: 14px;
    font-weight: 700;
  }
  .primary {
    background: var(--accent);
    color: #000;
    transition:
      background 0.12s ease,
      opacity 0.12s ease;
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .primary:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .secondary {
    color: var(--text-dim);
    border: 1px solid var(--border);
  }
  .secondary:hover {
    color: var(--text);
    border-color: #444;
  }
</style>
