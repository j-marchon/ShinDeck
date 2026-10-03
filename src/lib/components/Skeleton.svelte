<!-- Placeholder grid shown during the first scan. -->
<div class="grid" aria-busy="true" aria-label="Loading clips">
  {#each { length: 12 } as _, i (i)}
    <div class="card" style:animation-delay="{i * 60}ms">
      <div class="thumb"></div>
      <div class="line"></div>
      <div class="line short"></div>
    </div>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 18px;
    padding: 24px;
    overflow: hidden;
    height: 100%;
  }
  .card {
    animation: pulse 1.4s ease-in-out infinite;
  }
  .thumb {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    background: var(--surface-2);
  }
  .line {
    height: 12px;
    margin-top: 12px;
    border-radius: 4px;
    background: var(--surface-2);
  }
  .line.short {
    width: 55%;
    margin-top: 8px;
  }
  @keyframes pulse {
    50% {
      opacity: 0.45;
    }
  }
</style>
