<script lang="ts">
  /**
   * Pointer-driven horizontal slider (seek bar, volume). Custom rather than
   * <input type=range> so it never takes keyboard focus away from the player
   * shortcuts and can show buffered ranges and a hover tooltip.
   */
  let {
    value,
    buffered = 0,
    label,
    tooltip,
    oninput,
    onrelease,
  }: {
    /** 0..1 */
    value: number;
    /** 0..1, drawn behind the progress. */
    buffered?: number;
    label: string;
    tooltip?: (ratio: number) => string;
    oninput: (ratio: number) => void;
    onrelease?: () => void;
  } = $props();

  let track: HTMLDivElement;
  let dragging = $state(false);
  let hover = $state<number | null>(null);

  const clamp = (v: number) => Math.min(1, Math.max(0, v));
  const ratioAt = (e: PointerEvent) => {
    const rect = track.getBoundingClientRect();
    return clamp((e.clientX - rect.left) / rect.width);
  };

  function onpointerdown(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    track.setPointerCapture(e.pointerId);
    dragging = true;
    oninput(ratioAt(e));
  }

  function onpointermove(e: PointerEvent) {
    const ratio = ratioAt(e);
    hover = ratio;
    if (dragging) oninput(ratio);
  }

  function onpointerup(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    track.releasePointerCapture(e.pointerId);
    onrelease?.();
  }
</script>

<div
  class="slider"
  class:dragging
  bind:this={track}
  role="slider"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={Math.round(value * 100)}
  tabindex="-1"
  {onpointerdown}
  {onpointermove}
  {onpointerup}
  onpointercancel={onpointerup}
  onpointerleave={() => (hover = null)}
>
  <div class="rail">
    <div class="buffered" style:width="{clamp(buffered) * 100}%"></div>
    {#if hover !== null && tooltip}
      <div class="hover" style:width="{hover * 100}%"></div>
    {/if}
    <div class="fill" style:width="{clamp(value) * 100}%"></div>
  </div>
  <div class="knob" style:left="{clamp(value) * 100}%"></div>
  {#if hover !== null && tooltip}
    <div class="tooltip" style:left="{hover * 100}%">{tooltip(hover)}</div>
  {/if}
</div>

<style>
  .slider {
    position: relative;
    height: 18px;
    display: flex;
    align-items: center;
    cursor: pointer;
    touch-action: none;
    outline: none;
  }
  .rail {
    position: relative;
    width: 100%;
    height: 4px;
    border-radius: 4px;
    background: rgb(255 255 255 / 0.18);
    overflow: hidden;
    transition: height 0.12s ease;
  }
  .slider:hover .rail,
  .dragging .rail {
    height: 6px;
  }
  .buffered,
  .hover,
  .fill {
    position: absolute;
    inset: 0 auto 0 0;
  }
  .buffered {
    background: rgb(255 255 255 / 0.22);
  }
  .hover {
    background: rgb(255 255 255 / 0.18);
  }
  .fill {
    background: var(--accent);
  }
  .knob {
    position: absolute;
    top: 50%;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 0 4px rgb(118 185 0 / 0.25);
    transform: translate(-50%, -50%) scale(0);
    transition: transform 0.12s ease;
    pointer-events: none;
  }
  .slider:hover .knob,
  .dragging .knob {
    transform: translate(-50%, -50%) scale(1);
  }
  .tooltip {
    position: absolute;
    bottom: 22px;
    transform: translateX(-50%);
    padding: 3px 7px;
    border-radius: 5px;
    background: rgb(0 0 0 / 0.85);
    border: 1px solid var(--border);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    pointer-events: none;
  }
</style>
