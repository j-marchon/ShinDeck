<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { api, type Clip } from "../../api";
  import { editor } from "../../state/editor.svelte";
  import { library } from "../../state/library.svelte";
  import { formatLongDate, formatSize, formatTime } from "../../util/format";
  import { storedValue } from "../../util/storage";
  import { exportSpec, jobLabel, noEdits, renameTo, type PendingEdits } from "../../util/pending";
  import { isUnchanged, keptDuration, newTrim, skipTarget, type TrimState } from "../../util/trim";
  import GameIcon from "../GameIcon.svelte";
  import Icon, { type IconName } from "../Icon.svelte";
  import Slider from "../Slider.svelte";
  import EditPanel from "./EditPanel.svelte";
  import SavePrompt from "./SavePrompt.svelte";
  import TrimBar from "./TrimBar.svelte";

  let {
    playlist,
    startId,
    startEditing = false,
    onclose,
  }: {
    /** Snapshot of the gallery order when the player opened; "next" follows it. */
    playlist: Clip[];
    startId: string;
    /** Open with the Edit panel already showing. */
    startEditing?: boolean;
    /** Called with the clip that was showing, so the gallery can scroll to it. */
    onclose: (lastId: string) => void;
  } = $props();

  const SEEK_STEP = 5;
  const VOLUME_STEP = 0.1;
  const SPEEDS = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 3, 4];
  const HIDE_CONTROLS_AFTER = 2500;
  const prefs = storedValue("player", { volume: 1, muted: false });

  // Local copy: renames and new clips from edits update it in place.
  // svelte-ignore state_referenced_locally
  let list = $state.raw(playlist.slice());
  // svelte-ignore state_referenced_locally
  let index = $state(Math.max(0, playlist.findIndex((c) => c.id === startId)));
  const clip = $derived(list[index]);
  const gameName = $derived(library.gameName(clip.game));
  const favorite = $derived(library.isFavorite(clip.id));

  let video: HTMLVideoElement;
  let paused = $state(true);
  let currentTime = $state(0);
  let duration = $state(0);
  let bufferedEnd = $state(0);
  let volume = $state(prefs.get().volume);
  let muted = $state(prefs.get().muted);
  let speed = $state(1);
  let failed = $state(false);
  let failureDetail = $state("");
  let fullscreen = $state(false);
  let seeking = $state(false);
  let wasPlayingBeforeSeek = false;
  let showHelp = $state(false);
  /** Video detached so an edit can replace the file on disk. */
  let released = $state(false);

  // svelte-ignore state_referenced_locally
  let editing = $state(startEditing);
  let trimming = $state(false);
  /** Working copy while in trim mode; committed to `pending` with Done. */
  let trim = $state<TrimState>(newTrim(0));
  /** Edits staged in the panel; applied only when Save is pressed. */
  // svelte-ignore state_referenced_locally
  let pending = $state<PendingEdits>(noEdits(list[index]));

  let controlsVisible = $state(true);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let osd = $state<{ icon?: IconName; text?: string; id: number } | null>(null);
  let osdTimer: ReturnType<typeof setTimeout> | undefined;

  const hasPrev = $derived(index > 0);
  const hasNext = $derived(index < list.length - 1);
  const progress = $derived(duration > 0 ? currentTime / duration : 0);
  const videoSrc = $derived(released ? undefined : api.videoUrl(clip));
  const kept = $derived(trimming ? keptDuration(trim) : 0);

  // --- lifecycle ---------------------------------------------------------

  onMount(async () => {
    fullscreen = await api.isFullscreen();
    poke();
  });

  onDestroy(() => {
    clearTimeout(hideTimer);
    clearTimeout(osdTimer);
    if (fullscreen) api.setFullscreen(false);
  });

  $effect(() => prefs.set({ volume, muted }));

  // A new clip resets per-clip state (including unsaved edits); speed and
  // volume carry over.
  $effect(() => {
    void clip.id;
    failed = false;
    bufferedEnd = 0;
    trimming = false;
    pending = noEdits(clip);
  });

  $effect(() => {
    if (video) {
      video.defaultPlaybackRate = speed;
      video.playbackRate = speed;
    }
  });

  // While trimming, playback previews the result: it skips cut parts and
  // stops at the end handle.
  $effect(() => {
    if (!trimming || paused) return;
    let frame = requestAnimationFrame(function guard() {
      const t = video.currentTime;
      if (t >= trim.end - 0.03) {
        video.pause();
        video.currentTime = trim.end;
        return;
      }
      const jump = skipTarget(trim, t);
      if (jump !== null) video.currentTime = jump;
      frame = requestAnimationFrame(guard);
    });
    return () => cancelAnimationFrame(frame);
  });

  // --- playback ----------------------------------------------------------

  function flash(icon?: IconName, text?: string) {
    clearTimeout(osdTimer);
    osd = { icon, text, id: (osd?.id ?? 0) + 1 };
    osdTimer = setTimeout(() => (osd = null), 650);
  }

  function poke() {
    controlsVisible = true;
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (!paused && !seeking && !showHelp && !trimming) controlsVisible = false;
    }, HIDE_CONTROLS_AFTER);
  }

  function togglePlay() {
    if (failed || released) return;
    if (video.paused || video.ended) {
      if (trimming && video.currentTime >= trim.end - 0.05) video.currentTime = trim.start;
      video.play().catch(() => {});
      flash("play");
    } else {
      video.pause();
      flash("pause");
    }
  }

  function seekBy(seconds: number) {
    if (!duration) return;
    video.currentTime = Math.min(duration, Math.max(0, video.currentTime + seconds));
    flash(undefined, `${seconds > 0 ? "+" : "−"}${Math.abs(seconds)}s`);
  }

  function seekTo(time: number) {
    if (duration) video.currentTime = Math.min(duration, Math.max(0, time));
  }

  function scrub(active: boolean) {
    if (active && !seeking) {
      seeking = true;
      wasPlayingBeforeSeek = !video.paused;
      video.pause();
    } else if (!active) {
      seeking = false;
      if (wasPlayingBeforeSeek) video.play().catch(() => {});
      poke();
    }
  }

  function changeSpeed(direction: 1 | -1) {
    const current = SPEEDS.indexOf(speed);
    speed = SPEEDS[Math.min(SPEEDS.length - 1, Math.max(0, (current < 0 ? 3 : current) + direction))];
    flash(undefined, `${speed}×`);
  }

  function resetSpeed() {
    speed = 1;
    flash(undefined, "1×");
  }

  function changeVolume(delta: number) {
    volume = Math.round(Math.min(1, Math.max(0, volume + delta)) * 100) / 100;
    muted = volume === 0;
    flash(volume === 0 ? "mute" : "volume", `${Math.round(volume * 100)}%`);
  }

  function toggleMute() {
    muted = !muted;
    if (!muted && volume === 0) volume = 0.5;
    flash(muted ? "mute" : "volume", muted ? "Muted" : `${Math.round(volume * 100)}%`);
  }

  function go(delta: 1 | -1) {
    const target = index + delta;
    if (target < 0 || target >= list.length || trimming) return;
    index = target;
    poke();
  }

  async function toggleFullscreen() {
    fullscreen = await api.toggleFullscreen();
  }

  async function close() {
    if (fullscreen) {
      await api.setFullscreen(false);
      fullscreen = false;
      return;
    }
    onclose(clip.id);
  }

  function updateBuffered() {
    const ranges = video.buffered;
    bufferedEnd = ranges.length ? ranges.end(ranges.length - 1) : 0;
  }

  // --- editing -----------------------------------------------------------

  function startTrim() {
    if (!duration) return;
    trim = pending.trim ? { ...$state.snapshot(pending.trim), pending: null } : newTrim(duration);
    trimming = true;
    editing = true;
    video.pause();
    poke();
  }

  function cancelTrim() {
    trimming = false;
  }

  /** Stages the trim; nothing is written until Save. */
  function doneTrim() {
    pending.trim = isUnchanged(trim, duration) ? null : { ...$state.snapshot(trim), pending: null };
    trimming = false;
  }

  function discard() {
    pending = noEdits(clip);
    editor.error = null;
  }

  /** Applies every staged edit: one export for trim/compress, then the rename. */
  async function save() {
    if (editor.running || editor.prompt) return;
    const source = clip;
    const name = renameTo(pending, source);
    const spec = exportSpec(pending);
    editor.error = null;
    let current = source;
    let created = false;

    if (spec) {
      const destination = await editor.destination();
      if (!destination) return;
      if (destination === "replace") {
        // Let go of the file so Windows allows replacing it.
        video.pause();
        released = true;
        await tick();
        video.load();
      }
      const result = await editor.export(source, spec, destination, jobLabel(pending));
      released = false;
      if (!result) return;
      if (destination === "replace") {
        list = list.map((c) => (c.id === source.id ? result : c));
        library.upsert(result, source.id);
      } else {
        list = [...list.slice(0, index + 1), result, ...list.slice(index + 1)];
        library.upsert(result);
        created = true;
      }
      current = result;
    }

    if (name) {
      try {
        const renamed = await library.rename(current.id, name);
        list = list.map((c) => (c.id === current.id ? renamed : c));
        current = renamed;
        if (!spec) editor.notify("Renamed");
      } catch (e) {
        const message = String(e).replace(/^Error: /, "");
        editor.error = spec ? `Saved, but the rename failed: ${message}` : message;
        return;
      }
    }

    if (created) index = list.findIndex((c) => c.id === current.id);
    pending = noEdits(current);
  }

  // --- keyboard ----------------------------------------------------------

  function onkeydown(e: KeyboardEvent) {
    if (editor.prompt) return;
    if (e.ctrlKey && e.key.toLowerCase() === "s" && editing) {
      e.preventDefault();
      save();
      return;
    }
    if (e.ctrlKey || e.altKey || e.metaKey) return;
    const target = e.target as HTMLElement;
    if (target.matches("input, textarea, select")) return;
    const handled = () => {
      e.preventDefault();
      poke();
    };
    if (showHelp && e.key !== "?") {
      if (e.key === "Escape") {
        showHelp = false;
        handled();
      }
      return;
    }
    switch (e.key) {
      case " ":
      case "k":
        togglePlay();
        return handled();
      case "ArrowLeft":
        seekBy(-SEEK_STEP);
        return handled();
      case "ArrowRight":
        seekBy(SEEK_STEP);
        return handled();
      case "ArrowUp":
        changeVolume(VOLUME_STEP);
        return handled();
      case "ArrowDown":
        changeVolume(-VOLUME_STEP);
        return handled();
      case "Escape":
        if (trimming && trim.pending !== null) trim.pending = null;
        else if (trimming) cancelTrim();
        else close();
        return handled();
      case "Enter":
        if (trimming) doneTrim();
        return handled();
      case "n":
      case "N":
      case "PageDown":
        go(1);
        return handled();
      case "p":
      case "P":
      case "PageUp":
        go(-1);
        return handled();
      case "]":
      case "+":
      case "=":
        changeSpeed(1);
        return handled();
      case "[":
      case "-":
      case "_":
        changeSpeed(-1);
        return handled();
      case "0":
      case "Backspace":
        resetSpeed();
        return handled();
      case "m":
        toggleMute();
        return handled();
      case "f":
        toggleFullscreen();
        return handled();
      case "s":
        library.toggleFavorite(clip.id);
        return handled();
      case "e":
        editing = !editing;
        return handled();
      case "?":
        showHelp = !showHelp;
        return handled();
    }
  }

  const SHORTCUTS: [string, string][] = [
    ["Space", "Play / pause"],
    ["← / →", "Back / forward 5 seconds"],
    ["N / P", "Next / previous clip"],
    ["[ / ]", "Slower / faster"],
    ["0", "Normal speed"],
    ["↑ / ↓", "Volume"],
    ["M", "Mute"],
    ["F", "Fullscreen"],
    ["S", "Favorite"],
    ["E", "Edit panel"],
    ["Enter", "Done (while trimming)"],
    ["Ctrl + S", "Save edits"],
    ["Esc", "Leave trim / back to gallery"],
  ];
</script>

<svelte:window {onkeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" class:fullscreen onclick={() => !trimming && close()}>
  {#if !fullscreen && !trimming}
    <button
      class="nav prev"
      disabled={!hasPrev}
      aria-label="Previous clip"
      title="Previous clip (P)"
      onclick={(e) => {
        e.stopPropagation();
        go(-1);
      }}><Icon name="chevronLeft" size={26} /></button
    >
    <button
      class="nav next"
      disabled={!hasNext}
      aria-label="Next clip"
      title="Next clip (N)"
      onclick={(e) => {
        e.stopPropagation();
        go(1);
      }}><Icon name="chevronRight" size={26} /></button
    >
  {/if}

  <div class="layout" class:editing class:trimming onclick={(e) => e.stopPropagation()}>
    <div
      class="player card"
      class:idle={!controlsVisible}
      role="dialog"
      aria-modal="true"
      aria-label={clip.name}
      tabindex="-1"
      onpointermove={poke}
    >
      <header class="head dimmable">
        <button class="icon-btn" onclick={close} title="Back to gallery (Esc)" aria-label="Back to gallery">
          <Icon name="back" size={18} />
        </button>
        <div class="info">
          <div class="title">
            {#if library.isEdited(clip.id)}<span class="edited" title="Edited in ShinDeck"><Icon name="pencil" size={12} /></span>{/if}
            <span>{clip.name}</span>
          </div>
          <div class="sub">
            <GameIcon game={clip.game} name={gameName} size={14} />
            <span>{gameName}</span>
            <span class="dot">·</span>
            <span>{formatLongDate(clip.date)}</span>
            <span class="dot">·</span>
            <span>{formatSize(clip.size)}</span>
          </div>
        </div>
        <span class="position">{index + 1} / {list.length}</span>
        <button
          class="icon-btn"
          class:starred={favorite}
          onclick={() => library.toggleFavorite(clip.id)}
          title={favorite ? "Remove from favorites (S)" : "Add to favorites (S)"}
          aria-label="Favorite"
          aria-pressed={favorite}
        >
          <Icon name="star" size={17} filled={favorite} />
        </button>
        <button class="icon-btn" onclick={() => api.revealClip(clip.id)} title="Show in folder" aria-label="Show in folder">
          <Icon name="folderOpen" size={17} />
        </button>
        <button class="edit-btn" class:active={editing} aria-pressed={editing} title="Edit (E)" onclick={() => (editing = !editing)}>
          <Icon name="pencil" size={14} />
          Edit
        </button>
      </header>

      <div class="stage">
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
          bind:this={video}
          src={videoSrc}
          autoplay
          preload="auto"
          bind:paused
          bind:currentTime
          bind:duration
          bind:volume
          bind:muted
          onprogress={updateBuffered}
          onloadedmetadata={() => (video.playbackRate = speed)}
          onerror={() => {
            if (released) return;
            failed = true;
            failureDetail = video.error?.message || (video.error ? `Media error ${video.error.code}` : "");
          }}
          onpause={poke}
          onclick={togglePlay}
          ondblclick={toggleFullscreen}
        ></video>

        {#if released}
          <div class="overlay-msg"><span class="spinner"></span> Saving edit…</div>
        {:else if failed}
          <div class="failure">
            <Icon name="alert" size={30} />
            <h2>This clip can't be played</h2>
            <p>
              The file may be damaged or use a codec Windows can't decode. HEVC (H.265) recordings need the
              <b>HEVC Video Extensions</b> from the Microsoft Store.
            </p>
            {#if failureDetail}<code>{failureDetail}</code>{/if}
          </div>
        {/if}

        {#if osd}
          {#key osd.id}
            <div class="osd" class:pill={!!osd.text}>
              {#if osd.icon}<Icon name={osd.icon} size={osd.text ? 18 : 28} />{/if}
              {#if osd.text}<span>{osd.text}</span>{/if}
            </div>
          {/key}
        {/if}

        {#if showHelp}
          <div class="help glass-panel" role="dialog" aria-label="Keyboard shortcuts">
            <h3>Keyboard shortcuts</h3>
            <dl>
              {#each SHORTCUTS as [key, action] (key)}
                <dt><kbd>{key}</kbd></dt>
                <dd>{action}</dd>
              {/each}
            </dl>
          </div>
        {/if}
      </div>

      <div class="bar" onmousedown={(e) => e.preventDefault()} role="toolbar" tabindex="-1">
        {#if trimming}
          <div class="trim-head">
            <span class="trim-title"><Icon name="scissors" size={14} /> Trim & Cut</span>
            <span class="trim-hint">Drag the handles to trim · right-click the film twice to cut out a part</span>
            <span class="keep">Keeping <b>{formatTime(kept)}</b> of {formatTime(duration)}</span>
            <button class="btn-ghost" onclick={() => (trim = newTrim(duration))}>Reset</button>
            <button class="btn-ghost" onclick={cancelTrim}>Cancel</button>
            <button class="btn-primary done" disabled={kept < 0.5} onclick={doneTrim}>
              <Icon name="check" size={14} /> Done
            </button>
          </div>
          <TrimBar {duration} {currentTime} filmstrip={api.filmstripUrl(clip)} bind:trim onseek={seekTo} onscrub={scrub} />
        {:else}
          <Slider
            value={progress}
            buffered={duration ? bufferedEnd / duration : 0}
            label="Seek"
            tooltip={(r) => formatTime(r * duration)}
            oninput={(r) => {
              scrub(true);
              seekTo(r * duration);
            }}
            onrelease={() => scrub(false)}
          />
        {/if}

        <div class="row">
          <div class="group">
            <button class="ctl" onclick={() => go(-1)} disabled={!hasPrev || trimming} title="Previous clip (P)" aria-label="Previous clip">
              <Icon name="prev" size={16} />
            </button>
            <button class="ctl play" onclick={togglePlay} title={paused ? "Play (Space)" : "Pause (Space)"} aria-label={paused ? "Play" : "Pause"}>
              <Icon name={paused ? "play" : "pause"} size={17} />
            </button>
            <button class="ctl" onclick={() => go(1)} disabled={!hasNext || trimming} title="Next clip (N)" aria-label="Next clip">
              <Icon name="next" size={16} />
            </button>

            <div class="volume">
              <button class="ctl" onclick={toggleMute} title="Mute (M)" aria-label="Mute">
                <Icon name={muted || volume === 0 ? "mute" : volume < 0.5 ? "volumeLow" : "volume"} size={17} />
              </button>
              <div class="volume-slider">
                <Slider
                  value={muted ? 0 : volume}
                  label="Volume"
                  oninput={(r) => {
                    volume = r;
                    muted = r === 0;
                  }}
                />
              </div>
            </div>

            <span class="time">
              <span class="current">{formatTime(currentTime)}</span>
              <span class="sep">/</span>
              <span>{formatTime(duration)}</span>
            </span>
          </div>

          <div class="group">
            <div class="speed" role="group" aria-label="Playback speed">
              <button class="ctl small" onclick={() => changeSpeed(-1)} disabled={speed <= SPEEDS[0]} title="Slower ([)" aria-label="Slower">−</button>
              <button class="rate" class:changed={speed !== 1} onclick={resetSpeed} title="Reset speed (0)">{speed}×</button>
              <button
                class="ctl small"
                onclick={() => changeSpeed(1)}
                disabled={speed >= SPEEDS[SPEEDS.length - 1]}
                title="Faster (])"
                aria-label="Faster">+</button
              >
            </div>
            <button class="ctl" onclick={() => (showHelp = !showHelp)} title="Keyboard shortcuts (?)" aria-label="Keyboard shortcuts">
              <Icon name="keyboard" size={17} />
            </button>
            <button class="ctl" onclick={toggleFullscreen} title="Fullscreen (F)" aria-label="Fullscreen">
              <Icon name={fullscreen ? "minimize" : "maximize"} size={16} />
            </button>
          </div>
        </div>
      </div>
    </div>

    <aside class="side" aria-hidden={!editing} inert={!editing}>
      <div class="side-card card dimmable">
        <EditPanel
          {clip}
          bind:pending
          {duration}
          onclose={() => (editing = false)}
          onstarttrim={startTrim}
          onsave={save}
          ondiscard={discard}
        />
      </div>
    </aside>
  </div>
</div>

{#if editor.prompt}
  <SavePrompt />
{/if}

<style>
  /* --- frame --------------------------------------------------------- */
  .backdrop {
    position: fixed;
    inset: 30px 0 0 0;
    z-index: 40;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 18px 84px 28px;
    background: rgb(0 0 0 / 0.72);
    backdrop-filter: blur(16px) saturate(0.6);
    animation: fade-in 0.2s ease;
  }
  .backdrop.fullscreen {
    inset: 0;
    z-index: 60;
    padding: 0;
    background: #000;
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 0px;
    width: min(1680px, 100%);
    height: min(960px, 100%);
    transition: grid-template-columns 0.42s cubic-bezier(0.2, 0.8, 0.2, 1);
    animation: rise 0.28s cubic-bezier(0.2, 0.9, 0.3, 1.05);
  }
  .layout.editing {
    grid-template-columns: minmax(0, 1fr) 362px;
  }
  .fullscreen .layout {
    width: 100%;
    height: 100%;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.985);
    }
  }

  /* Floating cards. */
  .card {
    border-radius: 24px;
    background: var(--float);
    box-shadow: var(--float-shadow);
  }
  .player {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    outline: none;
    transition: box-shadow 0.3s ease;
  }
  .player.idle {
    cursor: none;
  }
  .fullscreen .player {
    border-radius: 0;
    box-shadow: none;
    background: #000;
  }

  .side {
    min-width: 0;
  }
  .side-card {
    width: 346px;
    height: 100%;
    margin-left: 16px;
    overflow: hidden;
    opacity: 0;
    transform: translateX(28px) scale(0.985);
    transition:
      opacity 0.32s ease,
      transform 0.42s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .editing .side-card {
    opacity: 1;
    transform: none;
  }

  /* Trim mode: everything but the video and the film strip steps back. */
  .dimmable {
    transition:
      opacity 0.3s ease,
      filter 0.3s ease,
      transform 0.42s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .trimming .dimmable {
    opacity: 0.25;
    filter: grayscale(0.6) blur(0.5px);
    pointer-events: none;
  }
  .trimming.editing .side-card {
    opacity: 0.25;
  }
  .trimming .player {
    box-shadow:
      0 0 0 1px rgb(118 185 0 / 0.35),
      0 30px 70px -24px rgb(0 0 0 / 0.95);
  }

  .nav {
    position: absolute;
    top: 50%;
    width: 46px;
    height: 46px;
    margin-top: -23px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: var(--text-dim);
    background: var(--glass);
    transition:
      color 0.15s ease,
      background 0.15s ease;
  }
  .nav.prev {
    left: 20px;
  }
  .nav.next {
    right: 20px;
  }
  .nav:hover:not(:disabled) {
    color: #fff;
    background: var(--glass-2);
  }
  .nav:disabled {
    opacity: 0.2;
    cursor: default;
  }

  /* --- header -------------------------------------------------------- */
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px 12px 14px;
  }
  .fullscreen .head {
    display: none;
  }
  .info {
    flex: 1;
    min-width: 0;
    margin-left: 2px;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
  }
  .title span:last-child {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .edited {
    color: var(--accent);
    display: grid;
  }
  .sub {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 3px;
    font-size: 12px;
    color: var(--text-faint);
    white-space: nowrap;
    overflow: hidden;
  }
  .position {
    margin-right: 4px;
    font-size: 12px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .icon-btn {
    width: 34px;
    height: 34px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 11px;
    color: var(--text-dim);
    background: var(--glass);
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }
  .icon-btn:hover {
    background: var(--glass-2);
    color: var(--text);
  }
  .icon-btn.starred {
    color: var(--accent);
  }
  .edit-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 14px;
    border-radius: 11px;
    background: var(--glass-2);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }
  .edit-btn :global(svg) {
    color: var(--accent);
  }
  .edit-btn:hover {
    background: var(--glass-3);
  }
  .edit-btn.active {
    background: rgb(118 185 0 / 0.12);
    color: var(--accent);
  }

  /* --- video ---------------------------------------------------------- */
  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
    margin: 0 12px;
    border-radius: 16px;
    overflow: hidden;
    background: #000;
  }
  .fullscreen .stage {
    margin: 0;
    border-radius: 0;
  }
  video {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #000;
  }

  /* --- controls ------------------------------------------------------- */
  .bar {
    padding: 12px 18px 12px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 6px;
  }
  .trimming .row {
    margin-top: 24px;
  }
  .group {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .ctl {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    color: var(--text-dim);
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }
  .ctl:hover:not(:disabled) {
    background: var(--glass-2);
    color: var(--text);
  }
  .ctl:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .ctl.play {
    width: 40px;
    height: 40px;
    margin: 0 4px;
    border-radius: 50%;
    padding-left: 1px;
    background: var(--accent);
    color: #050505;
  }
  .ctl.play:hover {
    background: var(--accent-hover);
    color: #050505;
  }
  .ctl.small {
    width: 28px;
    height: 28px;
    font-size: 16px;
  }
  .volume {
    display: flex;
    align-items: center;
    margin-left: 6px;
  }
  .volume-slider {
    width: 0;
    overflow: hidden;
    transition:
      width 0.2s ease,
      margin 0.2s ease;
  }
  .volume:hover .volume-slider {
    width: 86px;
    margin: 0 10px 0 4px;
    overflow: visible;
  }
  .time {
    margin-left: 10px;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--text-faint);
  }
  .time .current {
    color: var(--text);
    font-weight: 500;
  }
  .time .sep {
    margin: 0 4px;
  }
  .speed {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-right: 6px;
    padding: 0 3px;
    height: 32px;
    border-radius: 16px;
    background: var(--glass);
  }
  .rate {
    min-width: 42px;
    font-size: 12.5px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    color: var(--text-dim);
  }
  .rate.changed {
    color: var(--accent);
  }

  /* --- trim header ---------------------------------------------------- */
  .trim-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
    animation: fade-in 0.2s ease;
  }
  .trim-title {
    display: flex;
    align-items: center;
    gap: 7px;
    font-weight: 600;
    font-size: 13px;
  }
  .trim-title :global(svg) {
    color: var(--accent);
  }
  .trim-hint {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    color: var(--text-faint);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .keep {
    margin-right: 4px;
    font-size: 12px;
    color: var(--text-faint);
    white-space: nowrap;
  }
  .keep b {
    color: var(--text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .done {
    height: 34px;
    padding: 0 16px;
  }

  /* --- overlays ------------------------------------------------------- */
  .osd {
    position: absolute;
    top: 50%;
    left: 50%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px;
    border-radius: 50%;
    background: rgb(0 0 0 / 0.45);
    backdrop-filter: blur(10px);
    color: #fff;
    pointer-events: none;
    transform: translate(-50%, -50%);
    animation: osd 0.65s ease forwards;
  }
  .osd.pill {
    border-radius: 999px;
    padding: 9px 16px;
    font-size: 15px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  @keyframes osd {
    0% {
      opacity: 0;
      transform: translate(-50%, -50%) scale(0.9);
    }
    15% {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1);
    }
    70% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
  .failure,
  .overlay-msg {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    color: var(--text-dim);
  }
  .overlay-msg {
    flex-direction: row;
    font-size: 13.5px;
  }
  .spinner {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid rgb(255 255 255 / 0.15);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .failure h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }
  .failure p {
    max-width: 440px;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .failure code {
    font-size: 12px;
    color: var(--text-faint);
  }
  .help {
    position: absolute;
    right: 14px;
    bottom: 14px;
    width: 300px;
    padding: 16px 18px;
    border-radius: 16px;
  }
  .help h3 {
    margin-bottom: 12px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-faint);
  }
  .help dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 7px 14px;
    align-items: center;
    font-size: 12.5px;
  }
  .help dd {
    color: var(--text-dim);
  }
  kbd {
    display: inline-block;
    min-width: 24px;
    padding: 2px 7px;
    border-radius: 6px;
    background: var(--glass-2);
    font-family: inherit;
    font-size: 11.5px;
    font-weight: 500;
    text-align: center;
    color: var(--text);
  }
</style>
