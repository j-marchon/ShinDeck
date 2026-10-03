<script lang="ts" module>
  /** Size limits for sharing. Change these if a platform's limit changes. */
  export const PRESETS = [
    { id: "discord", name: "Discord", bytes: 10_000_000, note: "Free upload limit" },
    { id: "whatsapp", name: "WhatsApp", bytes: 16_000_000, note: "Video message limit" },
    { id: "instagram", name: "Instagram", bytes: 50_000_000, note: "Stories & DMs" },
  ] as const;
</script>

<script lang="ts">
  import type { Clip, SaveMode } from "../../api";
  import { editor } from "../../state/editor.svelte";
  import { library } from "../../state/library.svelte";
  import { settings } from "../../state/settings.svelte";
  import { formatMegabytes, formatSize } from "../../util/format";
  import Icon from "../Icon.svelte";

  let {
    clip,
    onclose,
    onstarttrim,
    oncompress,
    onrenamed,
  }: {
    clip: Clip;
    onclose: () => void;
    onstarttrim: () => void;
    oncompress: (targetBytes: number, label: string) => void;
    onrenamed: (clip: Clip, previousId: string) => void;
  } = $props();

  let name = $state("");
  let renameError = $state<string | null>(null);
  let customMb = $state<number | null>(null);

  // Reset the name field whenever another clip is shown (or renamed).
  $effect(() => {
    name = clip.name;
    renameError = null;
  });

  const available = $derived(settings.value?.editingAvailable ?? true);
  const busy = $derived(editor.running);
  const customBytes = $derived(customMb && customMb > 0 ? Math.round(customMb * 1_000_000) : null);

  const SAVE_MODES: { mode: SaveMode; label: string }[] = [
    { mode: "new", label: "New clip" },
    { mode: "replace", label: "Replace" },
    { mode: "ask", label: "Ask" },
  ];

  async function rename() {
    const next = name.trim();
    if (!next || next === clip.name) {
      name = clip.name;
      return;
    }
    try {
      const previousId = clip.id;
      const renamed = await library.rename(previousId, next);
      renameError = null;
      onrenamed(renamed, previousId);
    } catch (e) {
      renameError = String(e).replace(/^Error: /, "");
    }
  }

  function onnamekeydown(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
    if (e.key === "Escape") {
      name = clip.name;
      renameError = null;
      (e.currentTarget as HTMLInputElement).blur();
    }
  }
</script>

<div class="panel">
  <header>
    <span class="badge"><Icon name="pencil" size={15} /></span>
    <h2>Edit</h2>
    <button class="x" aria-label="Close editor" title="Close editor" onclick={onclose}><Icon name="close" size={16} /></button>
  </header>

  <div class="body">
    {#if editor.running}
      <div class="job">
        <div class="job-head">
          <span>{editor.label}…</span>
          <b>{Math.round(editor.progress * 100)}%</b>
        </div>
        <div class="track"><div class="fill" style:width="{editor.progress * 100}%"></div></div>
        <button class="cancel" onclick={() => editor.cancel()}>Cancel</button>
      </div>
    {:else if editor.error}
      <div class="message bad">
        <Icon name="alert" size={15} />
        <span>{editor.error}</span>
        <button aria-label="Dismiss" onclick={() => (editor.error = null)}><Icon name="close" size={13} /></button>
      </div>
    {:else if editor.notice}
      <div class="message good"><Icon name="check" size={15} /> {editor.notice}</div>
    {/if}

    <section>
      <h3>Name</h3>
      <div class="name" class:invalid={renameError}>
        <input
          bind:value={name}
          spellcheck="false"
          aria-label="Clip name"
          onblur={rename}
          onkeydown={onnamekeydown}
          oninput={() => (renameError = null)}
        />
        <span class="ext">.{clip.path.split(".").pop()}</span>
      </div>
      {#if renameError}<p class="error">{renameError}</p>{/if}
    </section>

    <section>
      <h3>Trim & Cut</h3>
      <button class="tool" disabled={!available || busy} onclick={onstarttrim}>
        <span class="tool-icon"><Icon name="scissors" size={18} /></span>
        <span class="tool-text">
          <b>Trim & Cut</b>
          <small>Drag the ends to trim · right-click the film to cut parts out</small>
        </span>
        <Icon name="chevronRight" size={16} />
      </button>
    </section>

    <section>
      <h3>Compress <span class="current">Now {formatSize(clip.size)}</span></h3>
      <div class="presets">
        {#each PRESETS as preset (preset.id)}
          {@const fits = clip.size <= preset.bytes}
          <button
            class="preset {preset.id}"
            disabled={!available || busy || fits}
            title={fits ? `Already under ${formatMegabytes(preset.bytes)}` : `${preset.note}: ${formatMegabytes(preset.bytes)}`}
            onclick={() => oncompress(preset.bytes, preset.name)}
          >
            <b>{preset.name}</b>
            <small>{fits ? "Already fits" : formatMegabytes(preset.bytes)}</small>
          </button>
        {/each}
      </div>
      <form
        class="custom"
        onsubmit={(e) => {
          e.preventDefault();
          if (customBytes) oncompress(customBytes, formatMegabytes(customBytes));
        }}
      >
        <label>
          <span>Custom</span>
          <input
            type="number"
            min="1"
            step="any"
            placeholder="25"
            bind:value={customMb}
            onkeydown={(e) => e.stopPropagation()}
          />
          <span class="unit">MB</span>
        </label>
        <button type="submit" disabled={!available || busy || !customBytes || customBytes >= clip.size}>Compress</button>
      </form>
    </section>

    {#if !available}
      <p class="warn"><Icon name="alert" size={14} /> Editing needs the bundled ffmpeg, which is missing. Reinstall ShinDeck.</p>
    {/if}
  </div>

  <footer>
    <span>Save edits as</span>
    <div class="modes" role="radiogroup" aria-label="Save edits as">
      {#each SAVE_MODES as option (option.mode)}
        <button
          role="radio"
          aria-checked={settings.value?.saveMode === option.mode}
          class:on={settings.value?.saveMode === option.mode}
          onclick={() => settings.setSaveMode(option.mode)}>{option.label}</button
        >
      {/each}
    </div>
  </footer>
</div>

<style>
  .panel {
    width: 340px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #0e0e0e;
    border-left: 1px solid #1f1f1f;
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 18px 18px 14px;
    border-bottom: 1px solid #1c1c1c;
  }
  .badge {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    background: var(--accent);
    color: #000;
    box-shadow: 0 0 14px rgb(118 185 0 / 0.5);
  }
  h2 {
    flex: 1;
    font-size: 16px;
    font-weight: 700;
  }
  .x {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-dim);
  }
  .x:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 6px 18px 18px;
  }
  section {
    margin-top: 18px;
  }
  h3 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 9px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-faint);
  }
  .current {
    letter-spacing: 0;
    text-transform: none;
    font-weight: 500;
    font-size: 12px;
    color: var(--text-dim);
  }

  .name {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 40px;
    padding: 0 10px 0 12px;
    border-radius: 9px;
    background: var(--surface);
    border: 1px solid var(--border);
    transition: border-color 0.12s ease;
  }
  .name:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgb(118 185 0 / 0.12);
  }
  .name.invalid {
    border-color: var(--danger);
  }
  .name input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .ext {
    font-size: 12px;
    color: var(--text-faint);
  }
  .error {
    margin-top: 6px;
    font-size: 12px;
    color: var(--danger);
  }

  .tool {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-dim);
    text-align: left;
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }
  .tool:hover:not(:disabled) {
    border-color: var(--accent);
    box-shadow: 0 0 18px -6px rgb(118 185 0 / 0.6);
    color: var(--accent);
  }
  .tool-icon {
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .tool-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .tool-text b {
    font-size: 13.5px;
    color: var(--text);
  }
  .tool-text small {
    font-size: 11.5px;
    line-height: 1.35;
    color: var(--text-faint);
  }

  .presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .preset {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 11px 4px;
    border-radius: 10px;
    color: #fff;
    transition:
      transform 0.12s ease,
      box-shadow 0.15s ease,
      filter 0.15s ease;
  }
  .preset b {
    font-size: 13px;
  }
  .preset small {
    font-size: 11.5px;
    opacity: 0.85;
  }
  .preset:hover:not(:disabled) {
    transform: translateY(-1px);
    filter: brightness(1.1);
  }
  .preset:active:not(:disabled) {
    transform: scale(0.97);
  }
  .preset:disabled {
    filter: grayscale(0.8) brightness(0.55);
    cursor: not-allowed;
  }
  .discord {
    background: #5865f2;
    box-shadow: 0 6px 18px -8px #5865f2;
  }
  .whatsapp {
    background: #1fae55;
    box-shadow: 0 6px 18px -8px #25d366;
  }
  .instagram {
    background: linear-gradient(135deg, #f58529, #dd2a7b 50%, #8134af);
    box-shadow: 0 6px 18px -8px #dd2a7b;
  }

  .custom {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
  .custom label {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 10px;
    border-radius: 9px;
    background: var(--surface);
    border: 1px solid var(--border);
    font-size: 12.5px;
    color: var(--text-faint);
  }
  .custom label:focus-within {
    border-color: var(--accent);
  }
  .custom input {
    flex: 1;
    width: 40px;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 13px;
    text-align: right;
    appearance: textfield;
  }
  .custom input::-webkit-inner-spin-button {
    -webkit-appearance: none;
  }
  .custom button {
    flex-shrink: 0;
    padding: 0 14px;
    border-radius: 9px;
    background: var(--surface-3);
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .custom button:hover:not(:disabled) {
    background: var(--accent);
    color: #000;
  }
  .custom button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .job {
    margin-top: 14px;
    padding: 12px;
    border-radius: 10px;
    background: var(--accent-soft);
    border: 1px solid rgb(118 185 0 / 0.4);
  }
  .job-head {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    margin-bottom: 8px;
  }
  .job-head b {
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .track {
    height: 6px;
    border-radius: 6px;
    background: rgb(255 255 255 / 0.08);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 6px;
    background: repeating-linear-gradient(-45deg, var(--accent) 0 8px, var(--accent-hover) 8px 16px);
    background-size: 22px 22px;
    animation: stripes 0.6s linear infinite;
    transition: width 0.2s ease;
  }
  @keyframes stripes {
    to {
      background-position: 22px 0;
    }
  }
  .cancel {
    margin-top: 10px;
    font-size: 12.5px;
    color: var(--text-dim);
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .cancel:hover {
    color: var(--danger);
  }
  .message {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-top: 14px;
    padding: 10px 12px;
    border-radius: 9px;
    font-size: 12.5px;
    line-height: 1.4;
  }
  .message span {
    flex: 1;
  }
  .message.good {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .message.bad {
    background: rgb(255 95 87 / 0.1);
    color: #ff9b95;
  }
  .warn {
    display: flex;
    gap: 7px;
    margin-top: 18px;
    font-size: 12px;
    color: #e0b341;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 12px 18px 16px;
    border-top: 1px solid #1c1c1c;
    font-size: 12px;
    color: var(--text-faint);
  }
  .modes {
    display: flex;
    padding: 2px;
    border-radius: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .modes button {
    padding: 4px 9px;
    border-radius: 6px;
    font-size: 12px;
    color: var(--text-dim);
  }
  .modes button.on {
    background: var(--accent);
    color: #000;
    font-weight: 600;
  }
</style>
