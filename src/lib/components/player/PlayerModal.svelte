<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { api, type Clip, type EditSpec } from "../../api";
  import { editor } from "../../state/editor.svelte";
  import { library } from "../../state/library.svelte";
  import { formatLongDate, formatSize, formatTime } from "../../util/format";
  import { storedValue } from "../../util/storage";
  import {
    isUnchanged,
    keptDuration,
    keptSegments,
    newTrim,
    skipTarget,
    type TrimState,
  } from "../../util/trim";
  import GameIcon from "../GameIcon.svelte";
  import Icon, { type IconName } from "../Icon.svelte";
  import InlineName from "../InlineName.svelte";
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
  let trim = $state<TrimState>(newTrim(0));

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

  // A new clip resets per-clip state; speed and volume carry over.
  $effect(() => {
    void clip.id;
    failed = false;
    bufferedEnd = 0;
    trimming = false;
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
    trim = newTrim(duration);
    trimming = true;
    video.pause();
    poke();
  }

  function cancelTrim() {
    trimming = false;
    trim = newTrim(duration);
  }

  async function saveTrim() {
    if (isUnchanged(trim, duration)) return cancelTrim();
    const spec: EditSpec = { keep: keptSegments(trim), targetBytes: null, label: "trimmed" };
    if (await runEdit(spec, trim.cuts.length ? "Cutting" : "Trimming")) trimming = false;
  }

  function compress(targetBytes: number, label: string) {
    runEdit({ keep: null, targetBytes, label }, `Compressing for ${label}`);
  }

  /** Resolves where to save, runs the export and shows the result. */
  async function runEdit(spec: EditSpec, jobLabel: string): Promise<boolean> {
    const destination = await editor.destination();
    if (!destination) return false;
    const source = clip;
    if (destination === "replace") {
      // Let go of the file so Windows allows replacing it.
      video.pause();
      released = true;
      await tick();
      video.load();
    }
    const result = await editor.export(source, spec, destination, jobLabel);
    released = false;
    if (!result) return false;

    if (destination === "replace") {
      list = list.map((c) => (c.id === source.id ? result : c));
      library.upsert(result, source.id);
    } else {
      list = [...list.slice(0, index + 1), result, ...list.slice(index + 1)];
      library.upsert(result);
      index += 1;
    }
    return true;
  }

  function onrenamed(renamed: Clip, previousId: string) {
    list = list.map((c) => (c.id === previousId ? renamed : c));
  }

  // --- keyboard ----------------------------------------------------------

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey || e.altKey || e.metaKey || editor.prompt) return;
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
        if (trimming && !editor.running) saveTrim();
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
    ["Enter", "Save trim (while trimming)"],
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
      }}><Icon name="chevronLeft" size={30} /></button
    >
    <button
      class="nav next"
      disabled={!hasNext}
      aria-label="Next clip"
      title="Next clip (N)"
      onclick={(e) => {
        e.stopPropagation();
        go(1);
      }}><Icon name="chevronRight" size={30} /></button
    >
  {/if}

  <div
    class="panel"
    class:editing
    class:trimming
    class:idle={!controlsVisible}
    role="dialog"
    aria-modal="true"
    aria-label={clip.name}
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onpointermove={poke}
  >
    <div class="main">
      <header class="head dimmable">
        <button class="round" onclick={close} title="Back to gallery (Esc)" aria-label="Back to gallery">
          <Icon name="back" size={20} />
        </button>
        <div class="info">
          <InlineName {clip} onrenamed={(c) => onrenamed(c, clip.id)} />
          <div class="sub">
            <GameIcon game={clip.game} name={gameName} size={15} />
            <span>{gameName}</span>
            <span class="dot">•</span>
            <span>{formatLongDate(clip.date)}</span>
            <span class="dot">•</span>
            <span>{formatSize(clip.size)}</span>
          </div>
        </div>
        <span class="position">{index + 1} / {list.length}</span>
        <button
          class="round"
          class:starred={favorite}
          onclick={() => library.toggleFavorite(clip.id)}
          title={favorite ? "Remove from favorites (S)" : "Add to favorites (S)"}
          aria-label="Favorite"
          aria-pressed={favorite}
        >
          <Icon name="star" size={18} filled={favorite} />
        </button>
        <button class="round" onclick={() => api.revealClip(clip.id)} title="Show in folder" aria-label="Show in folder">
          <Icon name="folderOpen" size={18} />
        </button>
        <button
          class="edit-btn"
          class:active={editing}
          aria-pressed={editing}
          title="Edit (E)"
          onclick={() => (editing = !editing)}
        >
          <Icon name="pencil" size={15} />
          Edit
        </button>
      </header>

      <div class="stage spotlight">
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
            <Icon name="alert" size={34} />
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
              {#if osd.icon}<Icon name={osd.icon} size={osd.text ? 18 : 30} />{/if}
              {#if osd.text}<span>{osd.text}</span>{/if}
            </div>
          {/key}
        {/if}

        {#if showHelp}
          <div class="help" role="dialog" aria-label="Keyboard shortcuts">
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

      <div class="bar spotlight" onmousedown={(e) => e.preventDefault()} role="toolbar" tabindex="-1">
        {#if trimming}
          <div class="trim-head">
            <span class="trim-title"><Icon name="scissors" size={15} /> Trim & Cut</span>
            <span class="trim-hint">Drag the green handles to trim · Right-click the film twice to cut out a part</span>
            <span class="keep">Keeping <b>{formatTime(kept)}</b> of {formatTime(duration)}</span>
            {#if editor.running}
              <div class="trim-progress">
                <span>{editor.label}… {Math.round(editor.progress * 100)}%</span>
                <div class="mini-track"><div style:width="{editor.progress * 100}%"></div></div>
              </div>
              <button class="ghost" onclick={() => editor.cancel()}>Stop</button>
            {:else}
              {#if editor.error}<span class="trim-error" title={editor.error}>{editor.error}</span>{/if}
              <button class="ghost" onclick={() => (trim = newTrim(duration))}>Reset</button>
              <button class="ghost" onclick={cancelTrim}>Cancel</button>
              <button class="save" disabled={kept < 0.5} onclick={saveTrim}>
                <Icon name="check" size={15} /> Save
              </button>
            {/if}
          </div>
          <TrimBar
            {duration}
            {currentTime}
            filmstrip={api.filmstripUrl(clip)}
            bind:trim
            onseek={seekTo}
            onscrub={scrub}
          />
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
              <Icon name="prev" size={17} />
            </button>
            <button class="ctl play" onclick={togglePlay} title={paused ? "Play (Space)" : "Pause (Space)"} aria-label={paused ? "Play" : "Pause"}>
              <Icon name={paused ? "play" : "pause"} size={19} />
            </button>
            <button class="ctl" onclick={() => go(1)} disabled={!hasNext || trimming} title="Next clip (N)" aria-label="Next clip">
              <Icon name="next" size={17} />
            </button>

            <div class="volume">
              <button class="ctl" onclick={toggleMute} title="Mute (M)" aria-label="Mute">
                <Icon name={muted || volume === 0 ? "mute" : volume < 0.5 ? "volumeLow" : "volume"} size={18} />
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
              <Icon name="keyboard" size={18} />
            </button>
            <button class="ctl" onclick={toggleFullscreen} title="Fullscreen (F)" aria-label="Fullscreen">
              <Icon name={fullscreen ? "minimize" : "maximize"} size={17} />
            </button>
          </div>
        </div>
      </div>
    </div>

    <aside class="side dimmable" aria-hidden={!editing} inert={!editing}>
      <div class="side-inner">
        <EditPanel
          {clip}
          onclose={() => (editing = false)}
          onstarttrim={startTrim}
          oncompress={compress}
          {onrenamed}
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
    inset: 36px 0 0 0;
    z-index: 40;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.86);
    backdrop-filter: blur(8px) saturate(0.6);
    animation: fade-in 0.18s ease;
  }
  .backdrop.fullscreen {
    inset: 0;
    z-index: 60;
    background: #000;
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }

  .panel {
    position: relative;
    width: min(1560px, calc(100vw - 150px));
    height: min(940px, calc(100vh - 36px - 48px));
    display: grid;
    grid-template-columns: minmax(0, 1fr) 0px;
    border-radius: 14px;
    overflow: hidden;
    background: #090909;
    border: 1px solid rgb(118 185 0 / 0.5);
    box-shadow:
      0 0 0 1px rgb(0 0 0 / 0.7),
      0 0 26px rgb(118 185 0 / 0.22),
      0 0 90px -20px rgb(118 185 0 / 0.3),
      0 40px 90px -30px #000;
    outline: none;
    transition: grid-template-columns 0.38s cubic-bezier(0.2, 0.8, 0.2, 1);
    animation: rise 0.24s cubic-bezier(0.2, 0.9, 0.3, 1.05);
  }
  .panel.editing {
    grid-template-columns: minmax(0, 1fr) 340px;
  }
  .fullscreen .panel {
    width: 100vw;
    height: 100vh;
    border: none;
    border-radius: 0;
    box-shadow: none;
  }
  .panel.idle {
    cursor: none;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.98);
    }
  }

  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .side {
    overflow: hidden;
    min-width: 0;
  }
  .side-inner {
    width: 340px;
    height: 100%;
    opacity: 0;
    transform: translateX(40px);
    transition:
      opacity 0.3s ease,
      transform 0.38s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .editing .side-inner {
    opacity: 1;
    transform: none;
  }

  /* Trim mode: dim everything except the video and the film strip. */
  .dimmable {
    transition:
      opacity 0.25s ease,
      filter 0.25s ease;
  }
  .trimming .dimmable {
    opacity: 0.22;
    filter: grayscale(0.7);
    pointer-events: none;
  }
  .spotlight {
    transition: box-shadow 0.25s ease;
  }
  .trimming .stage {
    box-shadow: inset 0 0 0 2px rgb(118 185 0 / 0.55);
  }
  .trimming .bar {
    background: #0f130a;
    box-shadow:
      inset 0 1px 0 rgb(118 185 0 / 0.6),
      0 -10px 40px -10px rgb(118 185 0 / 0.35);
  }

  .nav {
    position: absolute;
    top: 50%;
    width: 52px;
    height: 52px;
    margin-top: -26px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: var(--text-dim);
    background: rgb(255 255 255 / 0.05);
    transition:
      color 0.15s ease,
      background 0.15s ease,
      box-shadow 0.15s ease;
  }
  .nav.prev {
    left: 14px;
  }
  .nav.next {
    right: 14px;
  }
  .nav:hover:not(:disabled) {
    color: var(--accent);
    background: var(--accent-soft);
    box-shadow: 0 0 20px -4px rgb(118 185 0 / 0.6);
  }
  .nav:disabled {
    opacity: 0.2;
    cursor: default;
  }

  /* --- header -------------------------------------------------------- */
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px 12px 12px;
    border-bottom: 1px solid #1a1a1a;
    background: #0c0c0c;
  }
  .info {
    flex: 1;
    min-width: 0;
  }
  .sub {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-top: 2px;
    font-size: 12.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
  }
  .dot {
    color: var(--text-faint);
  }
  .position {
    font-size: 12.5px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .round {
    width: 36px;
    height: 36px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: var(--text);
    background: rgb(255 255 255 / 0.06);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .round:hover {
    background: rgb(255 255 255 / 0.12);
    color: var(--accent);
  }
  .round.starred {
    color: var(--accent);
  }
  .edit-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 36px;
    padding: 0 16px;
    border-radius: 18px;
    background: var(--accent);
    color: #000;
    font-weight: 700;
    font-size: 13px;
    box-shadow: 0 0 18px -4px rgb(118 185 0 / 0.75);
    transition:
      background 0.12s ease,
      box-shadow 0.12s ease,
      transform 0.1s ease;
  }
  .edit-btn:hover {
    background: var(--accent-hover);
    box-shadow: 0 0 24px -2px rgb(118 185 0 / 0.85);
  }
  .edit-btn:active {
    transform: scale(0.96);
  }
  .edit-btn.active {
    background: #142008;
    color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  /* --- video ---------------------------------------------------------- */
  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
    background: #000;
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
    padding: 10px 16px 10px;
    background: #0c0c0c;
    border-top: 1px solid #1a1a1a;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 6px;
  }
  .trimming .row {
    margin-top: 26px;
  }
  .group {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .ctl {
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: #fff;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .ctl:hover:not(:disabled) {
    background: rgb(255 255 255 / 0.08);
    color: var(--accent);
  }
  .ctl:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .ctl.play {
    width: 42px;
    height: 42px;
    margin: 0 2px;
    border-radius: 50%;
    padding-left: 1px;
    background: var(--accent);
    color: #000;
  }
  .ctl.play:hover {
    background: var(--accent-hover);
    color: #000;
  }
  .ctl.small {
    width: 28px;
    height: 28px;
    font-size: 17px;
    font-weight: 600;
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
    font-size: 13.5px;
    font-variant-numeric: tabular-nums;
    color: var(--text-dim);
  }
  .time .current {
    color: #fff;
    font-weight: 600;
  }
  .time .sep {
    margin: 0 4px;
    color: var(--text-faint);
  }
  .speed {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-right: 6px;
    padding: 0 4px;
    height: 34px;
    border-radius: 17px;
    background: rgb(255 255 255 / 0.06);
  }
  .rate {
    min-width: 46px;
    font-size: 12.5px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: #fff;
  }
  .rate.changed {
    color: var(--accent);
  }

  /* --- trim header ---------------------------------------------------- */
  .trim-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
    animation: fade-in 0.2s ease;
  }
  .trim-title {
    display: flex;
    align-items: center;
    gap: 7px;
    font-weight: 700;
    font-size: 13.5px;
    color: var(--accent);
  }
  .trim-hint {
    flex: 1;
    min-width: 0;
    font-size: 12.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .keep {
    font-size: 12.5px;
    color: var(--text-dim);
    white-space: nowrap;
  }
  .keep b {
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .ghost {
    height: 32px;
    padding: 0 12px;
    border-radius: 8px;
    font-size: 13px;
    color: var(--text-dim);
    border: 1px solid #2a2a2a;
  }
  .ghost:hover {
    color: var(--text);
    border-color: #444;
  }
  .save {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 16px;
    border-radius: 8px;
    background: var(--accent);
    color: #000;
    font-weight: 700;
    font-size: 13px;
    box-shadow: 0 0 16px -4px rgb(118 185 0 / 0.8);
  }
  .trim-progress {
    display: flex;
    flex-direction: column;
    gap: 5px;
    width: 180px;
    font-size: 12px;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .mini-track {
    height: 5px;
    border-radius: 5px;
    background: rgb(255 255 255 / 0.1);
    overflow: hidden;
  }
  .mini-track div {
    height: 100%;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
    transition: width 0.2s ease;
  }
  .trim-error {
    max-width: 220px;
    font-size: 12px;
    color: #ff9b95;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .save:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .save:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* --- overlays ------------------------------------------------------- */
  .osd {
    position: absolute;
    top: 50%;
    left: 50%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 18px;
    border-radius: 50%;
    background: rgb(0 0 0 / 0.6);
    color: #fff;
    pointer-events: none;
    transform: translate(-50%, -50%);
    animation: osd 0.65s ease forwards;
  }
  .osd.pill {
    border-radius: 999px;
    padding: 10px 18px;
    font-size: 17px;
    font-weight: 700;
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
    font-size: 14px;
  }
  .spinner {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid rgb(118 185 0 / 0.25);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .failure h2 {
    font-size: 18px;
    color: var(--text);
  }
  .failure p {
    max-width: 440px;
    font-size: 14px;
    line-height: 1.5;
  }
  .failure code {
    font-size: 12px;
    color: var(--text-faint);
  }
  .failure :global(svg) {
    color: var(--accent);
  }
  .help {
    position: absolute;
    right: 16px;
    bottom: 16px;
    width: 300px;
    padding: 18px 20px;
    border-radius: 12px;
    background: rgb(18 18 18 / 0.96);
    border: 1px solid var(--border);
    box-shadow: 0 20px 50px -20px #000;
  }
  .help h3 {
    margin-bottom: 12px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-faint);
  }
  .help dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 7px 14px;
    align-items: center;
    font-size: 13px;
  }
  .help dd {
    color: var(--text-dim);
  }
  kbd {
    display: inline-block;
    min-width: 26px;
    padding: 2px 7px;
    border-radius: 5px;
    background: var(--surface-3);
    border-bottom: 2px solid #000;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    text-align: center;
    color: var(--text);
  }
</style>
