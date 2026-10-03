<script lang="ts" module>
  /** Size limits for sharing. Change these if a platform's limit changes. */
  export const PRESETS = [
    { id: "discord", name: "Discord", bytes: 10_000_000 },
    { id: "whatsapp", name: "WhatsApp", bytes: 16_000_000 },
    { id: "instagram", name: "Instagram", bytes: 50_000_000 },
  ] as const;
</script>

<script lang="ts">
  import type { Clip, SaveMode } from "../../api";
  import { editor } from "../../state/editor.svelte";
  import { settings } from "../../state/settings.svelte";
  import { formatMegabytes, formatSize, formatTime } from "../../util/format";
  import { describe, hasEdits, type PendingEdits } from "../../util/pending";
  import { keptDuration } from "../../util/trim";
  import Icon from "../Icon.svelte";

  let {
    clip,
    pending = $bindable(),
    duration,
    onclose,
    onstarttrim,
    onsave,
    ondiscard,
  }: {
    clip: Clip;
    pending: PendingEdits;
    duration: number;
    onclose: () => void;
    onstarttrim: () => void;
    onsave: () => void;
    ondiscard: () => void;
  } = $props();

  type ChoiceId = (typeof PRESETS)[number]["id"] | "custom";

  // svelte-ignore state_referenced_locally
  let choice = $state<ChoiceId | null>((pending.compress?.id as ChoiceId) ?? null);
  let customMb = $state<number | null>(null);
  let customInput = $state<HTMLInputElement>();

  // A new clip (or discarded edits) clears the selection too.
  $effect(() => {
    if (pending.compress === null) {
      void clip.id;
      choice = null;
    }
  });

  const available = $derived(settings.value?.editingAvailable ?? true);
  const busy = $derived(editor.running);
  const changes = $derived(describe(pending, clip, duration));
  const dirty = $derived(hasEdits(pending, clip));

  const target = $derived.by(() => {
    if (choice === "custom") {
      return customMb && customMb > 0
        ? { id: "custom", label: formatMegabytes(customMb * 1_000_000), bytes: Math.round(customMb * 1_000_000) }
        : null;
    }
    const preset = PRESETS.find((p) => p.id === choice);
    return preset ? { id: preset.id, label: preset.name, bytes: preset.bytes } : null;
  });
  const staged = $derived(
    pending.compress !== null && target !== null && pending.compress.id === target.id && pending.compress.bytes === target.bytes,
  );

  const SAVE_MODES: { mode: SaveMode; label: string }[] = [
    { mode: "new", label: "New clip" },
    { mode: "replace", label: "Replace" },
    { mode: "ask", label: "Ask" },
  ];

  function pick(id: ChoiceId) {
    choice = choice === id && id !== "custom" ? null : id;
    if (id === "custom") queueMicrotask(() => customInput?.focus());
  }

  function stageCompression() {
    if (staged) pending.compress = null;
    else if (target) pending.compress = target;
  }

  function stop(e: KeyboardEvent) {
    // Typing in the panel never triggers player shortcuts.
    e.stopPropagation();
    if (e.key === "Escape") (e.currentTarget as HTMLElement).blur();
  }
</script>

<div class="panel">
  <header>
    <h2><Icon name="pencil" size={15} /> Edit</h2>
    <button class="x" aria-label="Close editor" title="Close editor (E)" onclick={onclose}><Icon name="close" size={15} /></button>
  </header>

  <div class="body">
    <section>
      <h3>Name</h3>
      <label class="name">
        <input bind:value={pending.name} spellcheck="false" aria-label="Clip name" onkeydown={stop} />
        <span class="ext">.{clip.path.split(".").pop()}</span>
      </label>
    </section>

    <section>
      <h3>Trim & cut</h3>
      {#if pending.trim}
        <div class="staged-trim">
          <Icon name="scissors" size={15} />
          <span>Keeping <b>{formatTime(keptDuration(pending.trim))}</b> of {formatTime(duration)}</span>
          <button class="link" disabled={busy} onclick={onstarttrim}>Adjust</button>
          <button class="icon-btn" aria-label="Remove trim" title="Remove trim" disabled={busy} onclick={() => (pending.trim = null)}>
            <Icon name="close" size={13} />
          </button>
        </div>
      {:else}
        <button class="tool" disabled={!available || busy || !duration} onclick={onstarttrim}>
          <Icon name="scissors" size={16} />
          <span class="tool-text">
            <b>Trim & Cut</b>
            <small>Drag the ends to trim · right-click the film to cut parts out</small>
          </span>
          <Icon name="chevronRight" size={15} />
        </button>
      {/if}
    </section>

    <section>
      <h3>Compress <span class="now">{formatSize(clip.size)} now</span></h3>
      <div class="tiles" role="radiogroup" aria-label="Compression target">
        {#each PRESETS as preset (preset.id)}
          {@const fits = clip.size <= preset.bytes && !pending.trim}
          <button
            class="tile {preset.id}"
            class:checked={choice === preset.id}
            role="radio"
            aria-checked={choice === preset.id}
            disabled={!available || busy || fits}
            onclick={() => pick(preset.id)}
          >
            <span class="check"><Icon name="check" size={12} stroke={3} /></span>
            <span class="tile-name">{preset.name}</span>
            <span class="tile-size">{fits ? "Already fits" : formatMegabytes(preset.bytes)}</span>
          </button>
        {/each}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="tile custom"
          class:checked={choice === "custom"}
          role="radio"
          tabindex="0"
          aria-checked={choice === "custom"}
          class:disabled={!available || busy}
          onclick={() => pick("custom")}
        >
          <span class="check"><Icon name="check" size={12} stroke={3} /></span>
          <span class="tile-name">Custom</span>
          <span class="custom-size">
            <input
              bind:this={customInput}
              type="number"
              min="1"
              step="any"
              placeholder="25"
              bind:value={customMb}
              onfocus={() => (choice = "custom")}
              onkeydown={stop}
            />
            MB
          </span>
        </div>
      </div>
      <button class="btn-ghost compress" class:staged disabled={!available || busy || (!target && !staged)} onclick={stageCompression}>
        {#if staged}
          <Icon name="check" size={14} /> Compression added · Undo
        {:else if pending.compress}
          Change compression
        {:else}
          Compress
        {/if}
      </button>
    </section>

    {#if !available}
      <p class="warn"><Icon name="alert" size={14} /> Editing needs the bundled ffmpeg, which is missing. Reinstall ShinDeck.</p>
    {/if}
  </div>

  <footer>
    {#if editor.running}
      <div class="job">
        <div class="job-head">
          <span>{editor.label}…</span>
          <b>{Math.round(editor.progress * 100)}%</b>
        </div>
        <div class="track"><div class="fill" style:width="{editor.progress * 100}%"></div></div>
        <button class="link" onclick={() => editor.cancel()}>Cancel</button>
      </div>
    {:else}
      {#if editor.error}
        <div class="message bad">
          <Icon name="alert" size={14} />
          <span>{editor.error}</span>
          <button aria-label="Dismiss" onclick={() => (editor.error = null)}><Icon name="close" size={12} /></button>
        </div>
      {:else if editor.notice && !dirty}
        <div class="message good"><Icon name="check" size={14} /> {editor.notice}</div>
      {/if}

      {#if changes.length}
        <ul class="changes">
          {#each changes as change (change)}
            <li>{change}</li>
          {/each}
        </ul>
      {/if}

      <div class="save-as">
        <span>Save as</span>
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
      </div>

      <div class="actions">
        <button class="btn-ghost" disabled={!dirty} onclick={ondiscard}>Discard</button>
        <button class="btn-primary save" disabled={!dirty} onclick={onsave}>Save</button>
      </div>
    {/if}
  </footer>
</div>

<style>
  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 18px 6px 20px;
  }
  h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 600;
  }
  h2 :global(svg) {
    color: var(--accent);
  }
  .x {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    color: var(--text-faint);
  }
  .x:hover {
    background: var(--glass-2);
    color: var(--text);
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 0 20px 12px;
  }
  section {
    margin-top: 18px;
  }
  h3 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-faint);
  }
  .now {
    font-variant-numeric: tabular-nums;
  }

  .name {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 40px;
    padding: 0 12px;
    border-radius: 11px;
    background: var(--glass);
    transition:
      background 0.15s ease,
      box-shadow 0.15s ease;
  }
  .name:focus-within {
    background: var(--glass-2);
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.12);
  }
  .name input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 13px;
  }
  .ext {
    font-size: 12px;
    color: var(--text-faint);
  }

  .tool {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--glass);
    color: var(--text-dim);
    text-align: left;
    transition: background 0.15s ease;
  }
  .tool:hover:not(:disabled) {
    background: var(--glass-2);
    color: var(--text);
  }
  .tool:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .tool-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tool-text b {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }
  .tool-text small {
    font-size: 11.5px;
    line-height: 1.35;
    color: var(--text-faint);
  }
  .staged-trim {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 8px 0 14px;
    border-radius: 12px;
    background: var(--glass-2);
    font-size: 12.5px;
    color: var(--text-dim);
  }
  .staged-trim :global(svg) {
    color: var(--accent);
  }
  .staged-trim span {
    flex: 1;
  }
  .staged-trim b {
    color: var(--text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .icon-btn {
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-faint);
  }
  .icon-btn:hover {
    background: var(--glass-2);
    color: var(--text);
  }
  .link {
    font-size: 12.5px;
    color: var(--text-dim);
  }
  .link:hover:not(:disabled) {
    color: var(--text);
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  /* --- compression tiles: colour only on the outline and the box ------ */
  .tiles {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .tile {
    --c: rgb(255 255 255 / 0.4);
    --edge: rgb(255 255 255 / 0.05);
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid transparent;
    /* Two backgrounds: the tile fill, and the outline showing through the
       transparent border (lets Instagram's outline be a gradient). */
    background:
      linear-gradient(var(--float), var(--float)) padding-box,
      linear-gradient(var(--edge), var(--edge)) border-box;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
    transition:
      color 0.15s ease,
      opacity 0.15s ease;
  }
  .discord {
    --c: #5865f2;
  }
  .whatsapp {
    --c: #25d366;
  }
  .instagram {
    --c: linear-gradient(135deg, #f58529, #dd2a7b 50%, #8134af);
  }
  .custom {
    --c: #b8b8b8;
  }
  .tile:hover:not(:disabled):not(.disabled) {
    color: var(--text);
    --edge: rgb(255 255 255 / 0.12);
  }
  .tile.checked {
    color: var(--text);
    background:
      linear-gradient(var(--float), var(--float)) padding-box,
      var(--fill, linear-gradient(var(--c), var(--c))) border-box;
  }
  .instagram.checked {
    --fill: var(--c);
  }
  .tile:disabled,
  .tile.disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  .check {
    position: relative;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 6px;
    color: transparent;
    /* Outline in the platform colour, filled when checked. */
    background:
      linear-gradient(var(--float), var(--float)) padding-box,
      var(--fill, linear-gradient(var(--c), var(--c))) border-box;
    border: 1.5px solid transparent;
    transition: color 0.15s ease;
  }
  .instagram .check {
    --fill: var(--c);
  }
  .checked .check {
    color: #fff;
    background: var(--fill, linear-gradient(var(--c), var(--c))) border-box;
  }
  .custom.checked .check {
    color: #111;
  }
  .tile-name {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
  }
  .tile-size,
  .custom-size {
    font-size: 12px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }
  .custom-size {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .custom-size input {
    width: 56px;
    height: 26px;
    padding: 0 8px;
    border-radius: 7px;
    border: none;
    outline: none;
    background: var(--glass-2);
    color: var(--text);
    font-size: 12.5px;
    text-align: right;
    appearance: textfield;
  }
  .custom-size input::-webkit-inner-spin-button {
    -webkit-appearance: none;
  }
  .compress {
    width: 100%;
    height: 38px;
    margin-top: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }
  .compress.staged:not(:disabled) {
    color: var(--accent);
    background: rgb(118 185 0 / 0.08);
  }

  .warn {
    display: flex;
    gap: 7px;
    margin-top: 18px;
    font-size: 12px;
    color: #e0b341;
  }

  /* --- footer --------------------------------------------------------- */
  footer {
    padding: 14px 20px 18px;
    background: linear-gradient(to bottom, transparent, rgb(255 255 255 / 0.015));
  }
  .changes {
    list-style: none;
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 12.5px;
    color: var(--text-dim);
  }
  .changes li {
    display: flex;
    gap: 8px;
    align-items: baseline;
  }
  .changes li::before {
    content: "";
    width: 5px;
    height: 5px;
    flex-shrink: 0;
    border-radius: 50%;
    background: var(--accent);
    transform: translateY(-2px);
  }
  .save-as {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 12px;
    font-size: 12px;
    color: var(--text-faint);
  }
  .modes {
    display: flex;
    padding: 2px;
    border-radius: 9px;
    background: var(--glass);
  }
  .modes button {
    padding: 4px 10px;
    border-radius: 7px;
    font-size: 12px;
    color: var(--text-faint);
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }
  .modes button:hover {
    color: var(--text);
  }
  .modes button.on {
    background: var(--glass-3);
    color: var(--text);
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .actions .btn-ghost {
    height: 40px;
  }
  .save {
    flex: 1;
    height: 40px;
  }

  .job {
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
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .track {
    height: 4px;
    border-radius: 4px;
    background: var(--glass-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 4px;
    background: var(--accent);
    transition: width 0.2s ease;
  }
  .job .link {
    align-self: flex-start;
  }
  .message {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 12px;
    padding: 9px 11px;
    border-radius: 10px;
    font-size: 12.5px;
    line-height: 1.4;
    background: var(--glass);
  }
  .message span {
    flex: 1;
  }
  .message.good {
    color: var(--accent);
  }
  .message.bad {
    color: #ff9b95;
  }
</style>
