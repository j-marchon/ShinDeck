<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { api, type Clip } from "../api";
  import { library } from "../state/library.svelte";
  import { formatLongDate, formatSize, formatTime } from "../util/format";
  import { storedValue } from "../util/storage";
  import GameIcon from "./GameIcon.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import Slider from "./Slider.svelte";

  let {
    playlist,
    startId,
    onclose,
  }: {
    /** Snapshot of the gallery order when the player opened; "next" follows it. */
    playlist: Clip[];
    startId: string;
    /** Called with the clip that was showing, so the gallery can scroll to it. */
    onclose: (lastId: string) => void;
  } = $props();

  const SEEK_STEP = 5;
  const VOLUME_STEP = 0.1;
  const SPEEDS = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2, 3, 4];
  const HIDE_CONTROLS_AFTER = 2500;
  const prefs = storedValue("player", { volume: 1, muted: false });

  // svelte-ignore state_referenced_locally
  let index = $state(Math.max(0, playlist.findIndex((c) => c.id === startId)));
  const clip = $derived(playlist[index]);
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

  let controlsVisible = $state(true);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  let osd = $state<{ icon?: IconName; text?: string; id: number } | null>(null);
  let osdTimer: ReturnType<typeof setTimeout> | undefined;

  const hasPrev = $derived(index > 0);
  const hasNext = $derived(index < playlist.length - 1);
  const progress = $derived(duration > 0 ? currentTime / duration : 0);

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

  // A new clip resets per-clip state (time and duration come from the media
  // events of the new source); the chosen speed and volume carry over.
  $effect(() => {
    void clip.id;
    failed = false;
    bufferedEnd = 0;
  });

  $effect(() => {
    if (video) {
      video.defaultPlaybackRate = speed;
      video.playbackRate = speed;
    }
  });

  // --- actions -----------------------------------------------------------

  function flash(icon?: IconName, text?: string) {
    clearTimeout(osdTimer);
    osd = { icon, text, id: (osd?.id ?? 0) + 1 };
    osdTimer = setTimeout(() => (osd = null), 650);
  }

  /** Shows the controls and schedules hiding them while playing. */
  function poke() {
    controlsVisible = true;
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (!paused && !seeking && !showHelp) controlsVisible = false;
    }, HIDE_CONTROLS_AFTER);
  }

  function togglePlay() {
    if (failed) return;
    if (video.paused || video.ended) {
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

  function seekTo(ratio: number) {
    if (!duration) return;
    if (!seeking) {
      seeking = true;
      wasPlayingBeforeSeek = !video.paused;
      video.pause();
    }
    video.currentTime = ratio * duration;
  }

  function endSeek() {
    seeking = false;
    if (wasPlayingBeforeSeek) video.play().catch(() => {});
    poke();
  }

  function changeSpeed(direction: 1 | -1) {
    const current = SPEEDS.indexOf(speed);
    const next = SPEEDS[Math.min(SPEEDS.length - 1, Math.max(0, (current < 0 ? 3 : current) + direction))];
    speed = next;
    flash(undefined, `${next}×`);
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
    if (target < 0 || target >= playlist.length) return;
    index = target;
    flash(delta > 0 ? "next" : "prev");
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

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey || e.altKey || e.metaKey) return;
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
        close();
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
      case "?":
        showHelp = !showHelp;
        return handled();
    }
  }

  function updateBuffered() {
    const ranges = video.buffered;
    bufferedEnd = ranges.length ? ranges.end(ranges.length - 1) : 0;
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
    ["Esc", "Back to gallery"],
  ];
</script>

<svelte:window {onkeydown} />

<div
  class="player"
  class:idle={!controlsVisible}
  role="dialog"
  aria-modal="true"
  aria-label={clip.name}
  tabindex="-1"
  onpointermove={poke}
>
  <!-- svelte-ignore a11y_media_has_caption -->
  <video
    bind:this={video}
    src={api.videoUrl(clip)}
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
      failed = true;
      failureDetail = video.error?.message || (video.error ? `Media error ${video.error.code}` : "");
    }}
    onpause={poke}
    onclick={togglePlay}
    ondblclick={toggleFullscreen}
  ></video>

  {#if failed}
    <div class="failure">
      <Icon name="alert" size={36} />
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

  <div class="top" onmousedown={(e) => e.preventDefault()} role="toolbar" tabindex="-1">
    <button class="round" onclick={close} title="Back to gallery (Esc)" aria-label="Back to gallery">
      <Icon name="back" size={22} />
    </button>
    <div class="info">
      <div class="name" title={clip.name}>{clip.name}</div>
      <div class="sub">
        <GameIcon game={clip.game} name={gameName} size={16} />
        <span>{gameName}</span>
        <span class="dot">•</span>
        <span>{formatLongDate(clip.date)}</span>
        <span class="dot">•</span>
        <span>{formatSize(clip.size)}</span>
      </div>
    </div>
    <span class="position">{index + 1} / {playlist.length}</span>
    <button
      class="round"
      class:starred={favorite}
      onclick={() => library.toggleFavorite(clip.id)}
      title={favorite ? "Remove from favorites (S)" : "Add to favorites (S)"}
      aria-label="Favorite"
      aria-pressed={favorite}
    >
      <Icon name="star" size={20} filled={favorite} />
    </button>
    <button class="round" onclick={() => api.revealClip(clip.id)} title="Show in folder" aria-label="Show in folder">
      <Icon name="folderOpen" size={19} />
    </button>
  </div>

  <div class="controls" onmousedown={(e) => e.preventDefault()} role="toolbar" tabindex="-1">
    <Slider
      value={progress}
      buffered={duration ? bufferedEnd / duration : 0}
      label="Seek"
      tooltip={(r) => formatTime(r * duration)}
      oninput={seekTo}
      onrelease={endSeek}
    />

    <div class="row">
      <div class="group">
        <button class="ctl" onclick={() => go(-1)} disabled={!hasPrev} title="Previous clip (P)" aria-label="Previous clip">
          <Icon name="prev" size={18} />
        </button>
        <button class="ctl play" onclick={togglePlay} title={paused ? "Play (Space)" : "Pause (Space)"} aria-label={paused ? "Play" : "Pause"}>
          <Icon name={paused ? "play" : "pause"} size={20} />
        </button>
        <button class="ctl" onclick={() => go(1)} disabled={!hasNext} title="Next clip (N)" aria-label="Next clip">
          <Icon name="next" size={18} />
        </button>

        <div class="volume">
          <button class="ctl" onclick={toggleMute} title="Mute (M)" aria-label="Mute">
            <Icon name={muted || volume === 0 ? "mute" : volume < 0.5 ? "volumeLow" : "volume"} size={19} />
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
          <button class="ctl small" onclick={() => changeSpeed(-1)} disabled={speed <= SPEEDS[0]} title="Slower ([)" aria-label="Slower">
            −
          </button>
          <button class="rate" class:changed={speed !== 1} onclick={resetSpeed} title="Reset speed (0)">{speed}×</button>
          <button
            class="ctl small"
            onclick={() => changeSpeed(1)}
            disabled={speed >= SPEEDS[SPEEDS.length - 1]}
            title="Faster (])"
            aria-label="Faster"
          >
            +
          </button>
        </div>
        <button class="ctl" onclick={() => (showHelp = !showHelp)} title="Keyboard shortcuts (?)" aria-label="Keyboard shortcuts">
          <Icon name="keyboard" size={19} />
        </button>
        <button class="ctl" onclick={toggleFullscreen} title="Fullscreen (F)" aria-label="Fullscreen">
          <Icon name={fullscreen ? "minimize" : "maximize"} size={18} />
        </button>
      </div>
    </div>
  </div>

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

<style>
  .player {
    position: fixed;
    inset: 0;
    z-index: 10;
    background: #000;
    outline: none;
    animation: fade-in 0.16s ease;
  }
  .player.idle {
    cursor: none;
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }

  video {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #000;
  }

  /* --- top bar --------------------------------------------------------- */
  .top,
  .controls {
    position: absolute;
    left: 0;
    right: 0;
    transition:
      opacity 0.25s ease,
      transform 0.25s ease;
  }
  .idle .top,
  .idle .controls {
    opacity: 0;
    pointer-events: none;
  }
  .idle .top {
    transform: translateY(-8px);
  }
  .idle .controls {
    transform: translateY(8px);
  }

  .top {
    top: 0;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px 20px 40px;
    background: linear-gradient(to bottom, rgb(0 0 0 / 0.85), transparent);
  }
  .info {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-size: 16px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-top: 4px;
    font-size: 13px;
    color: var(--text-dim);
    white-space: nowrap;
  }
  .dot {
    color: var(--text-faint);
  }
  .position {
    font-size: 13px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  .round {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: #fff;
    background: rgb(255 255 255 / 0.08);
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .round:hover {
    background: rgb(255 255 255 / 0.16);
  }
  .round.starred {
    color: var(--accent);
  }

  /* --- bottom controls ------------------------------------------------- */
  .controls {
    bottom: 0;
    padding: 48px 20px 14px;
    background: linear-gradient(to top, rgb(0 0 0 / 0.9), rgb(0 0 0 / 0.5) 55%, transparent);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 6px;
  }
  .group {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .ctl {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: #fff;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .ctl:hover:not(:disabled) {
    background: rgb(255 255 255 / 0.1);
    color: var(--accent);
  }
  .ctl:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .ctl.play {
    width: 46px;
    height: 46px;
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
    width: 30px;
    height: 30px;
    font-size: 18px;
    font-weight: 600;
  }

  .volume {
    display: flex;
    align-items: center;
    margin-left: 8px;
  }
  .volume-slider {
    width: 0;
    overflow: hidden;
    transition:
      width 0.2s ease,
      margin 0.2s ease;
  }
  .volume:hover .volume-slider {
    width: 90px;
    margin: 0 10px 0 4px;
    overflow: visible;
  }

  .time {
    margin-left: 12px;
    font-size: 14px;
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
    height: 36px;
    border-radius: 18px;
    background: rgb(255 255 255 / 0.06);
  }
  .rate {
    min-width: 50px;
    font-size: 13px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: #fff;
  }
  .rate.changed {
    color: var(--accent);
  }

  /* --- overlays -------------------------------------------------------- */
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

  .failure {
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
    right: 20px;
    bottom: 110px;
    width: 300px;
    padding: 18px 20px;
    border-radius: 12px;
    background: rgb(18 18 18 / 0.96);
    border: 1px solid var(--border);
    box-shadow: 0 20px 50px -20px #000;
  }
  .help h3 {
    margin-bottom: 12px;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-faint);
  }
  .help dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 14px;
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
