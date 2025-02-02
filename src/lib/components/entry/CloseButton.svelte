<script lang="ts">
  import { X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import type { MouseEventHandler } from "svelte/elements";

  let { onclick }: { onclick?: MouseEventHandler<HTMLButtonElement> } =
    $props();
</script>

<button
  class={css({
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    w: "6",
    h: "6",
    rounded: "circle",
    transition: "background",
    _hover: {
      bg: "selected",
    },
  })}
  {onclick}
>
  <X
    class={css({
      color: "view.text-muted",
      w: "4",
      h: "4",
    })}
  />
</button>
