<script lang="ts" module>
  // 24×24 stroke icons (Lucide-style). Kept inline: no icon font, no requests.
  const PATHS = {
    play: '<path d="M7 4.5v15a1 1 0 0 0 1.5.86l12.5-7.5a1 1 0 0 0 0-1.72L8.5 3.64A1 1 0 0 0 7 4.5Z" fill="currentColor" stroke="none"/>',
    pause: '<rect x="6" y="4" width="4.2" height="16" rx="1.2" fill="currentColor" stroke="none"/><rect x="13.8" y="4" width="4.2" height="16" rx="1.2" fill="currentColor" stroke="none"/>',
    star: '<path d="M12 2.8l2.85 5.77 6.37.93-4.61 4.49 1.09 6.34L12 17.33l-5.7 3 1.09-6.34L2.78 9.5l6.37-.93L12 2.8Z"/>',
    back: '<path d="M15 18l-6-6 6-6"/>',
    next: '<path d="M5 4.5v15l11-7.5L5 4.5Z" fill="currentColor"/><path d="M19 5v14"/>',
    prev: '<path d="M19 4.5v15L8 12l11-7.5Z" fill="currentColor"/><path d="M5 5v14"/>',
    volume: '<path d="M11 5 6 9H2v6h4l5 4V5Z" fill="currentColor"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07M19.07 4.93a10 10 0 0 1 0 14.14"/>',
    volumeLow: '<path d="M11 5 6 9H2v6h4l5 4V5Z" fill="currentColor"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>',
    mute: '<path d="M11 5 6 9H2v6h4l5 4V5Z" fill="currentColor"/><path d="m22 9-6 6M16 9l6 6"/>',
    maximize: '<path d="M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3"/>',
    minimize: '<path d="M8 3v3a2 2 0 0 1-2 2H3M21 8h-3a2 2 0 0 1-2-2V3M3 16h3a2 2 0 0 1 2 2v3M16 21v-3a2 2 0 0 1 2-2h3"/>',
    folder: '<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/>',
    folderOpen: '<path d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"/>',
    search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
    refresh: '<path d="M21 12a9 9 0 1 1-2.64-6.36L21 8"/><path d="M21 3v5h-5"/>',
    sortDesc: '<path d="M11 5h10M11 9h7M11 13h4M3 17l3 3 3-3M6 18V4"/>',
    sortAsc: '<path d="M11 5h4M11 9h7M11 13h10M3 7l3-3 3 3M6 6v14"/>',
    settings: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
    grid: '<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>',
    monitor: '<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>',
    close: '<path d="M18 6 6 18M6 6l12 12"/>',
    check: '<path d="M20 6 9 17l-5-5"/>',
    alert: '<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><path d="M12 9v4M12 17h.01"/>',
    film: '<rect x="2" y="2" width="20" height="20" rx="2.5"/><path d="M7 2v20M17 2v20M2 12h20M2 7h5M2 17h5M17 17h5M17 7h5"/>',
    keyboard: '<rect x="2" y="5" width="20" height="14" rx="2"/><path d="M6 9h.01M10 9h.01M14 9h.01M18 9h.01M6 13h.01M18 13h.01M8 16h8"/>',
  } as const;

  export type IconName = keyof typeof PATHS;
</script>

<script lang="ts">
  let {
    name,
    size = 18,
    filled = false,
    stroke = 2,
  }: { name: IconName; size?: number; filled?: boolean; stroke?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={filled ? "currentColor" : "none"}
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {@html PATHS[name]}
</svg>

<style>
  svg {
    flex-shrink: 0;
    display: block;
  }
</style>
