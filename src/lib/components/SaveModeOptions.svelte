<script lang="ts">
  import type { SaveMode } from "../api";
  import OptionCards, { type Option } from "./OptionCards.svelte";

  /** Radio cards for "what happens to an edited clip". */
  let {
    value,
    onchange,
    includeAsk = true,
  }: { value: SaveMode | null; onchange: (mode: SaveMode) => void; includeAsk?: boolean } = $props();

  const OPTIONS: Option<SaveMode>[] = [
    { value: "new", icon: "copy", title: "Save as a new clip", detail: "Keeps the original untouched" },
    { value: "replace", icon: "replace", title: "Replace the original", detail: "Overwrites the clip with the edit" },
    { value: "ask", icon: "settings", title: "Ask me every time", detail: "Choose after each edit" },
  ];
</script>

<OptionCards {value} options={OPTIONS.filter((o) => includeAsk || o.value !== "ask")} {onchange} />
