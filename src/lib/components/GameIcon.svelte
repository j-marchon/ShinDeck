<script lang="ts" module>
  import { SvelteSet } from "svelte/reactivity";

  /** Games whose icon could not be resolved; shared so we only ask once. */
  const missing = new SvelteSet<string>();

  /** Stable per-game hue for the monogram fallback. */
  function hue(name: string): number {
    let h = 0;
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) % 360;
    return h;
  }

  function monogram(name: string): string {
    const words = name.split(/[\s\-_:]+/).filter((w) => /[\p{L}\p{N}]/u.test(w));
    const letters = words.length > 1 ? words[0][0] + words[1][0] : (words[0] ?? "?").slice(0, 2);
    return letters.toUpperCase();
  }
</script>

<script lang="ts">
  import { api } from "../api";
  import Icon from "./Icon.svelte";

  let { game, name, size = 16 }: { game: string; name: string; size?: number } = $props();

  // ShadowPlay's own folders: desktop captures and loose clips get glyphs.
  const kind = $derived(game === "" ? "unsorted" : game.toLowerCase() === "desktop" ? "desktop" : "game");
</script>

<span class="icon" style:--size="{size}px">
  {#if kind === "desktop"}
    <span class="glyph"><Icon name="monitor" size={size * 0.72} /></span>
  {:else if kind === "unsorted"}
    <span class="glyph"><Icon name="folder" size={size * 0.72} /></span>
  {:else if missing.has(game)}
    <span class="mono" style:--hue={hue(game)} style:font-size="{Math.max(7, size * 0.42)}px">{monogram(name)}</span>
  {:else}
    <img src={api.gameIconUrl(game)} alt="" width={size} height={size} decoding="async" onerror={() => missing.add(game)} />
  {/if}
</span>

<style>
  .icon {
    width: var(--size);
    height: var(--size);
    flex-shrink: 0;
    display: inline-grid;
    place-items: center;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    border-radius: 3px;
  }
  .glyph,
  .mono {
    width: 100%;
    height: 100%;
    border-radius: calc(var(--size) * 0.22);
    display: grid;
    place-items: center;
  }
  .glyph {
    background: var(--surface-3);
    color: var(--text-dim);
  }
  .mono {
    background: hsl(var(--hue) 35% 22%);
    color: hsl(var(--hue) 70% 78%);
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }
</style>
