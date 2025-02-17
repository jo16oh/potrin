<script>
  import { css } from "styled-system/css";
</script>

<div
  data-tauri-drag-region
  class={css({
    position: "fixed",
    zIndex: "global.titlebar",
    height: "[28px]",
    w: "full",
    userSelect: "none",
  })}
>
  <slot />
</div>
