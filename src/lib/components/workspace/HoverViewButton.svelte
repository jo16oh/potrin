<script lang="ts">
  import { PencilLine } from "lucide-svelte";
  import { css } from "styled-system/css";
  import HoverView from "../common/HoverView";

  HoverView.State.init("cards");
</script>

<HoverView.Context createNewButton={true}>
  <HoverView.Trigger
    class={floatingButtonStyle}
    onmousedown={(e) => {
      e.preventDefault();
    }}
  >
    <PencilLine class={floatingButtonIconStyle} />
  </HoverView.Trigger>
</HoverView.Context>

<script module>
  const floatingButtonStyle = css({
    zIndex: "global.float",
    position: "absolute",
    right: "[24px]",
    bottom: "[16px]",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    p: "0",
    w: "14",
    h: "14",
    bg: "workspace.bg/90",
    rounded: "circle",
    transition: "colors",
    _hover: {
      bg: "workspace.bg-selected",
    },
  });

  const floatingButtonIconStyle = css({
    w: "6",
    h: "6",
    color: "[white]",
  });
</script>
