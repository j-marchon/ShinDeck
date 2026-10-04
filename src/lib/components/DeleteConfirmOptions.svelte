<script lang="ts">
  import OptionCards, { type Option } from "./OptionCards.svelte";

  /** Radio cards for "ask before moving a clip to the Recycle Bin". */
  let { value, onchange }: { value: boolean; onchange: (confirm: boolean) => void } = $props();

  const OPTIONS: Option<boolean>[] = [
    { value: true, icon: "alert", title: "Ask me every time", detail: "Confirm before a clip is removed" },
    { value: false, icon: "trash", title: "Remove right away", detail: "Moves the clip to the Recycle Bin without asking" },
  ];
</script>

<OptionCards {value} options={OPTIONS} {onchange} />
