<script lang="ts">
  import { UserRound } from "lucide-svelte";
  import { css, type Styles } from "styled-system/css";
  import type { Snippet } from "svelte";
  let { style }: { style?: Styles; children?: Snippet } = $props();
</script>

<div
  class={css(
    {
      rounded: "circle",
      bg: "selected",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      w: "6",
      h: "6",
    },
    style,
  )}
>
  <UserRound class={css({ w: "4", h: "4", color: "button.text-muted" })} />
</div>
