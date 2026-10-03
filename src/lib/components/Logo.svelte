<script lang="ts">
  let { size = 28 }: { size?: number } = $props();
</script>

<!-- Same artwork as src-tauri/icons/icon.svg -->
<svg width={size} height={size} viewBox="0 0 1024 1024" aria-hidden="true">
  <rect width="1024" height="1024" rx="224" fill="#000" />
  <rect x="288" y="196" width="448" height="290" rx="44" fill="#2f4a00" />
  <rect x="236" y="256" width="552" height="350" rx="52" fill="#4f7c00" />
  <rect x="180" y="328" width="664" height="436" rx="60" fill="#76b900" />
  <path d="M458 444 L458 648 L628 546 Z" fill="#0b0b0b" stroke="#0b0b0b" stroke-width="28" stroke-linejoin="round" />
</svg>
