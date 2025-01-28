<script>
  import { css } from "styled-system/css";
</script>

<div
  data-tauri-drag-region="button"
  class={css({
    position: "fixed",
    zIndex: "0",
    height: "7",
    w: "full",
    userSelect: "none",
  })}
>
  <slot />
</div>
