<script lang="ts">
  import { api, type Clip } from "../api";
  import { deletion } from "../state/deletion.svelte";
  import { library } from "../state/library.svelte";
  import { merge } from "../state/merge.svelte";
  import { formatDuration, formatRelativeDate, formatSize } from "../util/format";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";
  import InlineName from "./InlineName.svelte";

  let {
    clip,
    onopen,
    onedit,
  }: {
    clip: Clip;
    onopen: (clip: Clip) => void;
    /** Opens the player with the editor already showing. */
    onedit: (clip: Clip) => void;
  } = $props();

  // State is tracked per URL: when the file changes (e.g. ShadowPlay finished
  // writing it) the URL changes and the thumbnail is retried automatically.
  const thumbUrl = $derived(api.thumbnailUrl(clip));
  let loadedUrl = $state<string | null>(null);
  let failedUrl = $state<string | null>(null);
  const thumbFailed = $derived(failedUrl === thumbUrl);
  const favorite = $derived(library.isFavorite(clip.id));
  const gameName = $derived(library.gameName(clip.game));
  // While picking clips to merge, a click selects instead of playing.
  const selecting = $derived(merge.selecting);
  const pickedAt = $derived(selecting ? merge.position(clip.id) : 0);

  function activate() {
    if (selecting) merge.toggle(clip);
    else onopen(clip);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      activate();
    }
  }

  function toggleFavorite(e: MouseEvent) {
    e.stopPropagation();
    library.toggleFavorite(clip.id);
  }

  function remove(e: MouseEvent) {
    e.stopPropagation();
    deletion.request(clip);
  }

  function edit(e: MouseEvent) {
    e.stopPropagation();
    onedit(clip);
  }
</script>

<div
  class="card"
  class:selecting
  class:picked={pickedAt > 0}
  role="button"
  tabindex="0"
  aria-pressed={selecting ? pickedAt > 0 : undefined}
  onclick={activate}
  {onkeydown}
>
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

    {#if selecting}
      <span class="pick" class:on={pickedAt > 0}>
        {#if pickedAt}{pickedAt}{/if}
      </span>
    {:else}
      <div class="hover-play"><span><Icon name="play" size={22} /></span></div>
    {/if}

    {#if clip.edited}
      <span class="edited-tag" title="Edited in ShinDeck"><Icon name="pencil" size={12} stroke={2.4} /></span>
    {/if}

    {#if !selecting}
      <div class="actions">
        <button class="action edit" title="Edit clip" aria-label="Edit clip" onclick={edit}>
          <Icon name="pencil" size={15} />
        </button>
        <button
          class="action star"
          class:active={favorite}
          title={favorite ? "Remove from favorites" : "Add to favorites"}
          aria-label={favorite ? "Remove from favorites" : "Add to favorites"}
          aria-pressed={favorite}
          onclick={toggleFavorite}
        >
          <Icon name="star" size={17} filled={favorite} />
        </button>
      </div>

      <button class="action remove" title="Move to Recycle Bin" aria-label="Remove clip" onclick={remove}>
        <Icon name="trash" size={15} />
      </button>
    {/if}

    {#if clip.durationMs}
      <span class="duration">{formatDuration(clip.durationMs)}</span>
    {/if}
  </div>

  <div class="meta">
    <InlineName {clip} />
    <div class="sub">
      <GameIcon game={clip.game} name={gameName} size={16} />
      <span class="game">{gameName}</span>
      <span class="facts">{formatSize(clip.size)} · {formatRelativeDate(clip.date)}</span>
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
    background: var(--glass);
    transition:
      box-shadow 0.2s ease,
      transform 0.2s ease;
  }
  /* Hairline drawn inside the thumbnail so neighbours never clip it. */
  .thumb::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.05);
    pointer-events: none;
    transition: box-shadow 0.2s ease;
  }
  .card:hover .thumb,
  .card:focus-visible .thumb {
    transform: translateY(-2px);
    box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.9);
  }
  .card:hover .thumb::after,
  .card:focus-visible .thumb::after {
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.22);
  }

  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition:
      opacity 0.25s ease,
      transform 0.4s ease;
  }
  img.visible {
    opacity: 1;
  }
  .card:hover img {
    transform: scale(1.025);
  }

  .placeholder {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: radial-gradient(circle at 50% 40%, var(--glass-2), transparent 70%);
    opacity: 0.8;
  }

  .hover-play {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: linear-gradient(to top, rgb(0 0 0 / 0.35), transparent 55%);
    opacity: 0;
    transition: opacity 0.2s ease;
  }
  .hover-play span {
    width: 46px;
    height: 46px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.35);
    backdrop-filter: blur(10px);
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.18);
    color: #fff;
    transform: scale(0.88);
    transition:
      transform 0.2s ease,
      background 0.2s ease,
      color 0.2s ease;
  }
  .card:hover .hover-play {
    opacity: 1;
  }
  .card:hover .hover-play span {
    transform: scale(1);
  }
  .hover-play span:hover {
    color: var(--accent);
  }

  .actions {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 5px;
  }
  .action {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    color: rgb(255 255 255 / 0.9);
    background: rgb(0 0 0 / 0.28);
    backdrop-filter: blur(10px);
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.1);
    opacity: 0;
    transform: translateY(-2px);
    transition:
      opacity 0.18s ease,
      transform 0.18s ease,
      color 0.15s ease,
      background 0.15s ease;
  }
  .card:hover .action,
  .card:focus-visible .action,
  .star.active {
    opacity: 1;
    transform: none;
  }
  .action:hover {
    color: #fff;
    background: rgb(0 0 0 / 0.5);
  }
  .action:active {
    transform: scale(0.9);
  }
  .edit:hover {
    color: var(--accent);
  }
  .star.active {
    color: var(--accent);
  }
  .remove {
    position: absolute;
    left: 8px;
    bottom: 8px;
    transform: translateY(2px);
  }
  .remove:hover {
    color: var(--danger);
  }

  /* Merge selection. */
  .card.picked .thumb {
    box-shadow: 0 0 0 2px var(--accent);
  }
  .pick {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    background: rgb(0 0 0 / 0.35);
    backdrop-filter: blur(8px);
    box-shadow: inset 0 0 0 1.5px rgb(255 255 255 / 0.7);
    color: #050505;
    font-size: 13px;
    font-weight: 700;
    transition:
      background 0.15s ease,
      box-shadow 0.15s ease;
  }
  .card.selecting:hover .pick:not(.on) {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }
  .pick.on {
    background: var(--accent);
    box-shadow: none;
  }

  .edited-tag {
    position: absolute;
    top: 8px;
    left: 8px;
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--accent);
    background: rgb(0 0 0 / 0.42);
    backdrop-filter: blur(8px);
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.1);
  }

  .duration {
    position: absolute;
    right: 8px;
    bottom: 8px;
    padding: 2px 7px;
    border-radius: 6px;
    font-size: 11.5px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    background: rgb(0 0 0 / 0.42);
    backdrop-filter: blur(8px);
    color: rgb(255 255 255 / 0.92);
  }

  .meta {
    padding: 9px 2px 0;
    min-width: 0;
  }
  .sub {
    margin-top: 3px;
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
    color: var(--text-faint);
    min-width: 0;
  }
  .game {
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .facts {
    margin-left: auto;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }
</style>
