<script lang="ts">
  /** The app mark. Below 40px a simplified variant keeps it legible. */
  let { size = 28 }: { size?: number } = $props();
</script>

<!-- Same artwork as src-tauri/icons/icon.svg and icon-small.svg -->
<svg width={size} height={size} viewBox="0 0 1024 1024" aria-hidden="true">
  {#if size < 40}
    <defs>
    <linearGradient id="ls-g" x1="0.15" y1="0" x2="0.85" y2="1">
      <stop offset="0" stop-color="#a8dc2c"/>
      <stop offset="1" stop-color="#5f9e00"/>
    </linearGradient>
    <linearGradient id="ls-t" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#b2e23a"/>
      <stop offset="1" stop-color="#6aa800"/>
    </linearGradient>
  </defs>
  <g fill="none" stroke="url(#ls-g)" stroke-width="86" stroke-linecap="round"><path d="M165.6 712.0 A400 400 0 0 1 661.8 141.1"/><path d="M858.4 312.0 A400 400 0 0 1 362.2 882.9"/></g>
  <g fill="url(#ls-g)" stroke="url(#ls-g)" stroke-width="10" stroke-linejoin="round"><path d="M693.3 63.2 L747.1 175.6 L630.4 219.0 Z"/><path d="M330.7 960.8 L276.9 848.4 L393.6 805.0 Z"/></g>
  
  <path d="M385.8 326.0 L757.8 512.0 L385.8 698.0 Z" fill="url(#ls-t)" stroke="url(#ls-t)" stroke-width="64" stroke-linejoin="round"/>
  <path d="M452.7 426.4 L623.8 512.0 L452.7 597.6 Z" fill="none" stroke="#25292c" stroke-width="44" stroke-linejoin="round"/>
  {:else}
    <defs>
    <linearGradient id="lf-g" x1="0.15" y1="0" x2="0.85" y2="1">
      <stop offset="0" stop-color="#a8dc2c"/>
      <stop offset="1" stop-color="#5f9e00"/>
    </linearGradient>
    <linearGradient id="lf-t" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#b2e23a"/>
      <stop offset="1" stop-color="#6aa800"/>
    </linearGradient>
  </defs>
  <g fill="none" stroke="url(#lf-g)" stroke-width="62" stroke-linecap="round"><path d="M162.1 714.0 A404 404 0 0 1 663.3 137.4"/><path d="M861.9 310.0 A404 404 0 0 1 360.7 886.6"/></g>
  <g fill="url(#lf-g)" stroke="url(#lf-g)" stroke-width="10" stroke-linejoin="round"><path d="M688.1 76.2 L732.0 165.1 L638.6 198.6 Z"/><path d="M335.9 947.8 L292.0 858.9 L385.4 825.4 Z"/></g>
  <circle cx="512" cy="512" r="312" fill="none" stroke="#76b900" stroke-opacity="0.38" stroke-width="20"/>
  <path d="M385.8 326.0 L757.8 512.0 L385.8 698.0 Z" fill="url(#lf-t)" stroke="url(#lf-t)" stroke-width="64" stroke-linejoin="round"/>
  <path d="M447.8 419.0 L633.8 512.0 L447.8 605.0 Z" fill="none" stroke="#25292c" stroke-width="30" stroke-linejoin="round"/>
  {/if}
</svg>
