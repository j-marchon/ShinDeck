<script lang="ts">
  import OptionCards, { type Option } from "./OptionCards.svelte";

  /** Radio cards for "what happens to the originals after a merge". */
  let { value, onchange }: { value: boolean; onchange: (replace: boolean) => void } = $props();

  const OPTIONS: Option<boolean>[] = [
    { value: false, icon: "copy", title: "Keep the originals", detail: "Both clips stay in your library" },
    { value: true, icon: "replace", title: "Replace the originals", detail: "Both clips move to the Recycle Bin once merged" },
  ];
</script>

<OptionCards {value} options={OPTIONS} {onchange} />
