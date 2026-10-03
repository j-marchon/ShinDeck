<script lang="ts">
  import { api, type Clip } from "../api";
  import { library } from "../state/library.svelte";
  import { formatDuration, formatRelativeDate } from "../util/format";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";

  let { clip, onopen }: { clip: Clip; onopen: (clip: Clip) => void } = $props();

  // State is tracked per URL: when the file changes (e.g. ShadowPlay finished
  // writing it) the URL changes and the thumbnail is retried automatically.
  const thumbUrl = $derived(api.thumbnailUrl(clip));
  let loadedUrl = $state<string | null>(null);
  let failedUrl = $state<string | null>(null);
  const thumbFailed = $derived(failedUrl === thumbUrl);
  const favorite = $derived(library.isFavorite(clip.id));
  const gameName = $derived(library.gameName(clip.game));

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onopen(clip);
    }
  }

  function toggleFavorite(e: MouseEvent) {
    e.stopPropagation();
    library.toggleFavorite(clip.id);
  }
</script>

<div class="card" role="button" tabindex="0" title={clip.name} onclick={() => onopen(clip)} {onkeydown}>
  <div class="thumb">
    {#if thumbFailed}
      <div class="placeholder"><GameIcon game={clip.game} name={gameName} size={40} /></div>
    {:else}
      <img
        src={thumbUrl}
        alt=""
        loading="lazy"
        decoding="async"
        class:visible={loadedUrl === thumbUrl}
        onload={(e) => (loadedUrl = (e.currentTarget as HTMLImageElement).getAttribute("src"))}
        onerror={(e) => (failedUrl = (e.currentTarget as HTMLImageElement).getAttribute("src"))}
      />
    {/if}

    <div class="hover-play"><span><Icon name="play" size={22} /></span></div>

    <button
      class="star"
      class:active={favorite}
      aria-label={favorite ? "Remove from favorites" : "Add to favorites"}
      aria-pressed={favorite}
      onclick={toggleFavorite}
    >
      <Icon name="star" size={17} filled={favorite} />
    </button>

    {#if clip.durationMs}
      <span class="duration">{formatDuration(clip.durationMs)}</span>
    {/if}
  </div>

  <div class="meta">
    <div class="name">{clip.name}</div>
    <div class="sub">
      <GameIcon game={clip.game} name={gameName} size={16} />
      <span class="game">{gameName}</span>
      <span class="date">{formatRelativeDate(clip.date)}</span>
    </div>
  </div>
</div>

<style>
  .card {
    height: 100%;
    display: flex;
    flex-direction: column;
    border-radius: var(--radius);
    cursor: pointer;
    outline: none;
  }

  .thumb {
    position: relative;
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--surface-2);
    transition: box-shadow 0.15s ease;
  }
  /* Ring drawn inside the thumbnail so it is never clipped by neighbours. */
  .thumb::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.06);
    pointer-events: none;
    transition: box-shadow 0.15s ease;
  }
  .card:hover .thumb,
  .card:focus-visible .thumb {
    box-shadow: 0 10px 28px -12px rgb(118 185 0 / 0.45);
  }
  .card:hover .thumb::after,
  .card:focus-visible .thumb::after {
    box-shadow: inset 0 0 0 2px var(--accent);
  }

  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition:
      opacity 0.2s ease,
      transform 0.3s ease;
  }
  img.visible {
    opacity: 1;
  }
  .card:hover img {
    transform: scale(1.03);
  }

  .placeholder {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: radial-gradient(circle at 50% 40%, var(--surface-3), var(--surface-2));
    opacity: 0.8;
  }

  .hover-play {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: linear-gradient(to top, rgb(0 0 0 / 0.45), transparent 60%);
    opacity: 0;
    transition: opacity 0.15s ease;
  }
  .hover-play span {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    padding-left: 3px;
    background: var(--accent);
    color: #000;
    transform: scale(0.85);
    transition: transform 0.15s ease;
  }
  .card:hover .hover-play {
    opacity: 1;
  }
  .card:hover .hover-play span {
    transform: scale(1);
  }

  .star {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: #fff;
    background: rgb(0 0 0 / 0.55);
    backdrop-filter: blur(6px);
    opacity: 0;
    transition:
      opacity 0.15s ease,
      color 0.15s ease,
      transform 0.1s ease;
  }
  .card:hover .star,
  .card:focus-visible .star,
  .star.active {
    opacity: 1;
  }
  .star:hover {
    color: var(--accent);
  }
  .star:active {
    transform: scale(0.88);
  }
  .star.active {
    color: var(--accent);
    filter: drop-shadow(0 0 6px rgb(118 185 0 / 0.6));
  }

  .duration {
    position: absolute;
    right: 8px;
    bottom: 8px;
    padding: 2px 6px;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    background: rgb(0 0 0 / 0.72);
    color: #fff;
  }

  .meta {
    padding: 10px 2px 0;
    min-width: 0;
  }
  .name {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .card:hover .name {
    color: #fff;
  }
  .sub {
    margin-top: 5px;
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
    color: var(--text-dim);
    min-width: 0;
  }
  .game {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .date {
    margin-left: auto;
    flex-shrink: 0;
    color: var(--text-faint);
    font-size: 12px;
  }
</style>
