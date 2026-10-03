<script lang="ts">
  import type { Clip } from "../api";
  import ClipCard from "./ClipCard.svelte";

  let {
    clips,
    onopen,
    resetKey,
  }: {
    clips: Clip[];
    onopen: (clip: Clip) => void;
    /** Scroll back to the top whenever this value changes (view/sort/search). */
    resetKey: string;
  } = $props();

  // Layout constants (px). Cards share one row height, which lets us
  // virtualize: only rows near the viewport exist in the DOM, so the gallery
  // stays smooth with tens of thousands of clips.
  const PAD = 24;
  const GAP = 18;
  const MIN_CARD = 250;
  const META_HEIGHT = 56;
  const OVERSCAN_ROWS = 2;

  let scroller: HTMLDivElement;
  let width = $state(0);
  let height = $state(0);
  let scrollTop = $state(0);

  const columns = $derived(Math.max(1, Math.floor((width - PAD * 2 + GAP) / (MIN_CARD + GAP))));
  const cardWidth = $derived(Math.max(0, (width - PAD * 2 - GAP * (columns - 1)) / columns));
  const cardHeight = $derived(Math.round((cardWidth * 9) / 16) + META_HEIGHT);
  const rowHeight = $derived(cardHeight + GAP);
  const rowCount = $derived(Math.ceil(clips.length / columns));
  const totalHeight = $derived(rowCount > 0 ? PAD * 2 + rowCount * rowHeight - GAP : 0);

  const firstRow = $derived(Math.max(0, Math.floor((scrollTop - PAD) / rowHeight) - OVERSCAN_ROWS));
  const lastRow = $derived(Math.min(rowCount, Math.ceil((scrollTop + height - PAD) / rowHeight) + OVERSCAN_ROWS));

  const items = $derived.by(() => {
    if (cardWidth <= 0) return [];
    const out: { clip: Clip; x: number; y: number }[] = [];
    for (let row = firstRow; row < lastRow; row++) {
      for (let col = 0; col < columns; col++) {
        const index = row * columns + col;
        if (index >= clips.length) break;
        out.push({ clip: clips[index], x: PAD + col * (cardWidth + GAP), y: PAD + row * rowHeight });
      }
    }
    return out;
  });

  $effect(() => {
    void resetKey;
    if (scroller) scroller.scrollTop = 0;
  });

  /** Brings a clip into view (used when returning from the player). */
  export function reveal(id: string) {
    const index = clips.findIndex((c) => c.id === id);
    if (index < 0 || !scroller) return;
    const top = PAD + Math.floor(index / columns) * rowHeight;
    if (top < scroller.scrollTop || top + cardHeight > scroller.scrollTop + height) {
      scroller.scrollTop = top - (height - cardHeight) / 2;
    }
  }
</script>

<div
  class="scroller"
  bind:this={scroller}
  bind:clientWidth={width}
  bind:clientHeight={height}
  onscroll={() => (scrollTop = scroller.scrollTop)}
>
  <div class="canvas" style:height="{totalHeight}px">
    {#each items as { clip, x, y } (clip.id)}
      <div class="slot" style:width="{cardWidth}px" style:height="{cardHeight}px" style:transform="translate({x}px, {y}px)">
        <ClipCard {clip} {onopen} />
      </div>
    {/each}
  </div>
</div>

<style>
  .scroller {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    contain: strict;
  }
  .canvas {
    position: relative;
  }
  .slot {
    position: absolute;
    top: 0;
    left: 0;
  }
</style>
