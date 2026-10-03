<script lang="ts">
  import { formatTime } from "../../util/format";
  import { MIN_KEEP, mergeCuts, rulerStep, type TrimState } from "../../util/trim";

  /**
   * Film-roll timeline for trimming. Drag the green handles to set the start
   * and end; right-click twice on the film to cut out the part between the
   * two markers. Cut edges can be dragged, and cuts removed with their ×.
   */
  let {
    duration,
    currentTime,
    filmstrip,
    trim = $bindable(),
    onseek,
    onscrub,
  }: {
    duration: number;
    currentTime: number;
    filmstrip: string;
    trim: TrimState;
    onseek: (time: number) => void;
    onscrub?: (active: boolean) => void;
  } = $props();

  type Drag = { kind: "start" } | { kind: "end" } | { kind: "seek" } | { kind: "cut"; index: number; edge: 0 | 1 };

  let film: HTMLDivElement;
  let drag = $state<Drag | null>(null);
  let hover = $state<number | null>(null);
  let stripFailed = $state(false);

  const pct = (t: number) => `${(Math.min(duration, Math.max(0, t)) / duration) * 100}%`;
  const ticks = $derived.by(() => {
    const step = rulerStep(duration);
    return Array.from({ length: Math.floor(duration / step) + 1 }, (_, i) => i * step);
  });

  function timeAt(e: PointerEvent | MouseEvent): number {
    const rect = film.getBoundingClientRect();
    return Math.min(duration, Math.max(0, ((e.clientX - rect.left) / rect.width) * duration));
  }

  function begin(e: PointerEvent, next: Drag) {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    film.setPointerCapture(e.pointerId);
    drag = next;
    if (next.kind === "seek") {
      onscrub?.(true);
      onseek(timeAt(e));
    }
  }

  function onpointermove(e: PointerEvent) {
    const t = timeAt(e);
    hover = t;
    const d = drag;
    if (!d) return;
    if (d.kind === "seek") {
      onseek(t);
    } else if (d.kind === "start") {
      trim.start = Math.min(t, trim.end - MIN_KEEP);
      onseek(trim.start);
    } else if (d.kind === "end") {
      trim.end = Math.max(t, trim.start + MIN_KEEP);
      onseek(trim.end);
    } else {
      const cut = trim.cuts[d.index];
      if (!cut) return;
      const other = cut[1 - d.edge];
      cut[d.edge] = d.edge === 0 ? Math.min(t, other - 0.1) : Math.max(t, other + 0.1);
      onseek(cut[d.edge]);
    }
  }

  function onpointerup(e: PointerEvent) {
    if (!drag) return;
    if (drag.kind === "seek") onscrub?.(false);
    if (drag.kind === "cut") trim.cuts = mergeCuts(trim.cuts);
    drag = null;
    if (film.hasPointerCapture(e.pointerId)) film.releasePointerCapture(e.pointerId);
  }

  /** Right-click: first marker starts a cut, second one completes it. */
  function oncontextmenu(e: MouseEvent) {
    e.preventDefault();
    const t = timeAt(e);
    if (trim.pending === null) {
      trim.pending = t;
    } else {
      const [a, b] = trim.pending < t ? [trim.pending, t] : [t, trim.pending];
      trim.pending = null;
      if (b - a >= 0.1) trim.cuts = mergeCuts([...trim.cuts, [a, b]]);
    }
  }

  function removeCut(index: number) {
    trim.cuts = trim.cuts.filter((_, i) => i !== index);
  }
</script>

<div class="trimbar">
  <div class="ruler" aria-hidden="true">
    {#each ticks as t (t)}
      <span class="tick" style:left={pct(t)}><span>{formatTime(t)}</span></span>
    {/each}
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="film"
    class:dragging={drag}
    bind:this={film}
    onpointerdown={(e) => begin(e, { kind: "seek" })}
    {onpointermove}
    {onpointerup}
    onpointercancel={onpointerup}
    onpointerleave={() => (hover = null)}
    {oncontextmenu}
  >
    <div class="sprockets top"></div>
    <div class="frames" style:background-image={stripFailed ? "none" : `url("${filmstrip}")`}>
      <img src={filmstrip} alt="" hidden onerror={() => (stripFailed = true)} />
    </div>
    <div class="sprockets bottom"></div>

    <!-- Outside the trim range -->
    <div class="shade" style:left="0" style:width={pct(trim.start)}></div>
    <div class="shade" style:left={pct(trim.end)} style:right="0"></div>

    <!-- Cut ranges -->
    {#each trim.cuts as [a, b], i (i)}
      <div class="cut" style:left={pct(a)} style:width="calc({pct(b)} - {pct(a)})">
        <span
          class="cut-edge left"
          role="slider"
          tabindex="-1"
          aria-label="Cut start"
          aria-valuenow={a}
          onpointerdown={(e) => begin(e, { kind: "cut", index: i, edge: 0 })}
        ></span>
        <span
          class="cut-edge right"
          role="slider"
          tabindex="-1"
          aria-label="Cut end"
          aria-valuenow={b}
          onpointerdown={(e) => begin(e, { kind: "cut", index: i, edge: 1 })}
        ></span>
        <button
          class="cut-remove"
          title="Keep this part"
          aria-label="Remove cut"
          onpointerdown={(e) => e.stopPropagation()}
          onclick={() => removeCut(i)}>×</button
        >
      </div>
    {/each}

    {#if trim.pending !== null}
      <div class="pending" style:left={pct(trim.pending)}>
        <span>Right-click again to end the cut</span>
      </div>
    {/if}

    <!-- Kept range frame + handles -->
    <div class="frame" style:left={pct(trim.start)} style:width="calc({pct(trim.end)} - {pct(trim.start)})"></div>
    <span
      class="handle start"
      class:active={drag?.kind === "start"}
      style:left={pct(trim.start)}
      role="slider"
      tabindex="-1"
      aria-label="Trim start"
      aria-valuenow={trim.start}
      onpointerdown={(e) => begin(e, { kind: "start" })}
    >
      <i></i>
      <b class="label">{formatTime(trim.start)}</b>
    </span>
    <span
      class="handle end"
      class:active={drag?.kind === "end"}
      style:left={pct(trim.end)}
      role="slider"
      tabindex="-1"
      aria-label="Trim end"
      aria-valuenow={trim.end}
      onpointerdown={(e) => begin(e, { kind: "end" })}
    >
      <i></i>
      <b class="label">{formatTime(trim.end)}</b>
    </span>

    <div class="playhead" style:left={pct(currentTime)}></div>
    {#if hover !== null && !drag}
      <div class="hover" style:left={pct(hover)}><span>{formatTime(hover)}</span></div>
    {/if}
  </div>
</div>

<style>
  .trimbar {
    user-select: none;
    padding: 0 8px;
  }

  .ruler {
    position: relative;
    height: 18px;
    margin-bottom: 4px;
  }
  .tick {
    position: absolute;
    bottom: 0;
    height: 5px;
    border-left: 1px solid rgb(255 255 255 / 0.2);
  }
  .tick span {
    position: absolute;
    bottom: 7px;
    transform: translateX(-50%);
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  .film {
    position: relative;
    height: 78px;
    border-radius: 10px;
    background: #050505;
    cursor: pointer;
    touch-action: none;
  }
  .sprockets {
    position: absolute;
    left: 0;
    right: 0;
    height: 11px;
    background:
      repeating-linear-gradient(90deg, transparent 0 5px, #2c2c2c 5px 13px, transparent 13px 18px) center / 100% 5px no-repeat,
      #0b0b0b;
  }
  .sprockets.top {
    top: 0;
    border-radius: 6px 6px 0 0;
  }
  .sprockets.bottom {
    bottom: 0;
    border-radius: 0 0 6px 6px;
  }
  .frames {
    position: absolute;
    inset: 11px 0;
    background-color: #1a1a1a;
    background-size: 100% 100%;
    background-repeat: no-repeat;
    box-shadow: inset 0 0 0 1px #000;
  }

  .shade {
    position: absolute;
    top: 0;
    bottom: 0;
    background: rgb(0 0 0 / 0.72);
    pointer-events: none;
  }

  .frame {
    position: absolute;
    top: 0;
    bottom: 0;
    border-top: 3px solid var(--accent);
    border-bottom: 3px solid var(--accent);
    box-shadow: 0 0 14px -4px rgb(118 185 0 / 0.4);
    pointer-events: none;
  }

  .handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 14px;
    background: var(--accent);
    cursor: ew-resize;
    display: grid;
    place-items: center;
    z-index: 3;
    transition: background 0.12s ease;
  }
  .handle.start {
    transform: translateX(-100%);
    border-radius: 6px 0 0 6px;
  }
  .handle.end {
    border-radius: 0 6px 6px 0;
  }
  .handle:hover,
  .handle.active {
    background: var(--accent-hover);
  }
  .handle i {
    width: 2px;
    height: 22px;
    border-radius: 2px;
    background: rgb(0 0 0 / 0.55);
    box-shadow:
      -3px 0 0 rgb(0 0 0 / 0.35),
      3px 0 0 rgb(0 0 0 / 0.35);
  }
  .label {
    position: absolute;
    top: -22px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent);
    color: #000;
    font-size: 11px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    opacity: 0;
    transition: opacity 0.12s ease;
    pointer-events: none;
  }
  .handle:hover .label,
  .handle.active .label {
    opacity: 1;
  }

  .cut {
    position: absolute;
    top: 0;
    bottom: 0;
    background: repeating-linear-gradient(-45deg, rgb(255 70 70 / 0.5) 0 6px, rgb(120 0 0 / 0.55) 6px 12px);
    border-left: 2px solid #ff5050;
    border-right: 2px solid #ff5050;
    z-index: 2;
  }
  .cut-edge {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 10px;
    cursor: ew-resize;
  }
  .cut-edge.left {
    left: -6px;
  }
  .cut-edge.right {
    right: -6px;
  }
  .cut-remove {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 22px;
    height: 22px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    background: #ff5050;
    color: #fff;
    font-size: 16px;
    font-weight: 700;
    line-height: 1;
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.6);
  }
  .cut-remove:hover {
    background: #ff7070;
    transform: translate(-50%, -50%) scale(1.1);
  }

  .pending {
    position: absolute;
    top: -4px;
    bottom: -4px;
    width: 0;
    border-left: 2px dashed #ff5050;
    z-index: 4;
    pointer-events: none;
  }
  .pending span {
    position: absolute;
    bottom: calc(100% + 4px);
    left: -1px;
    padding: 2px 7px;
    border-radius: 4px;
    background: #ff5050;
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    animation: blink 1.2s ease-in-out infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.6;
    }
  }

  .playhead {
    position: absolute;
    top: -6px;
    bottom: -6px;
    width: 2px;
    margin-left: -1px;
    background: #fff;
    box-shadow: 0 0 6px rgb(0 0 0 / 0.8);
    z-index: 5;
    pointer-events: none;
  }
  .playhead::before {
    content: "";
    position: absolute;
    top: -2px;
    left: -5px;
    border: 6px solid transparent;
    border-top-color: #fff;
  }
  .hover {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgb(255 255 255 / 0.45);
    z-index: 4;
    pointer-events: none;
  }
  .hover span {
    position: absolute;
    top: calc(100% + 6px);
    transform: translateX(-50%);
    padding: 2px 6px;
    border-radius: 4px;
    background: rgb(0 0 0 / 0.85);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
</style>
