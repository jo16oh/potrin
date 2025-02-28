<script lang="ts">
  import { ClockArrowDown, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "../common/Button.svelte";
  import type { View } from "$lib/models/Workspace.svelte";

  type Props = { view: Extract<View, { type: "timeline" }>; pinned?: boolean };
  let { view, pinned }: Props = $props();
</script>

<div class={viewContainer}>
  <div class={headerStyle}>
    <div class={headerLeftButtons}></div>
    <div class={headerTitleContainer}>
      <Button class={headerTitleButtonStyle}>
        <ClockArrowDown class={headerTitleIconStyle} />
      </Button>
      <div class={headerTitleTextStyle}>Timeline</div>
    </div>
    <div class={headerRightButtons}>
      {#if !pinned}
        <Button class={headerButtonStyle}>
          <X class={headerIconStyle} />
        </Button>
      {/if}
    </div>
  </div>
</div>

<script module>
  const headerStyle = css({
    position: "sticky",
    top: "0",
    zIndex: "local.header",
    display: "grid",
    gridTemplateColumns: "[1fr auto 1fr]",
    gap: "4",
    justifyContent: "space-between",
    alignItems: "center",
    w: "full",
    overflow: "hidden",
    h: "8",
    p: "1",
    bg: "view.bg/90",
    roundedTop: "md",
    backdropFilter: "[blur(4px)]",
  });

  const headerLeftButtons = css({
    flexBasis: "[auto]",
    flexGrow: "0",
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    gap: "2",
  });

  const headerRightButtons = css({
    flexBasis: "[auto]",
    flexGrow: "0",
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    justifyContent: "end",
    gap: "2",
  });

  const headerButtonStyle = css({
    justifySelf: "end",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    transition: "colors",
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    shadow: "[none]",
    bg: "transparent",
    _disabled: {
      _hover: {
        bg: "transparent",
      },
    },
    _hover: {
      bg: "view.bg-selected",
    },
  });

  const headerIconStyle = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
    "&[data-disabled=true]": {
      color: "view.text-muted/50",
    },
  });

  const headerTitleContainer = css({
    justifySelf: "center",
    w: "full",
    flex: "[0 1 auto]",
    display: "flex",
    flexDir: "row",
    justifyContent: "center",
    alignItems: "center",
    h: "full",
    minW: "0",
    overflow: "hidden",
    textOverflow: "ellipsis",
  });

  const headerTitleButtonStyle = css({
    flexShrink: "0",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    transition: "colors",
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    bg: "transparent",
    _hover: {
      bg: "view.bg-selected",
    },
  });

  const headerTitleIconStyle = css({
    w: "3",
    h: "3",
    color: "view.text-muted",
  });

  const headerTitleTextStyle = css({
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
    color: "view.text-muted",
    fontSize: "xs",
    cursor: "default",
  });

  const viewContainer = css({
    position: "relative",
    w: "full",
    h: "full",
    bg: "view.bg",
    rounded: "md",
    display: "flex",
    flexDir: "column",
    shadow: "md.around",
    overflow: "hidden",
  });
</script>
