<script lang="ts">
  import { X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import DialogClose from "../common/DialogClose.svelte";
  import { buttonStyle } from "../common/Button.svelte";
</script>

<div class={containerStyle}>
  <div class={headerStyle}>
    <div class={headerLeftButtons}></div>
    <div class={headerRightButtons}>
      <DialogClose class={headerButtonStyle}>
        <X class={headerIconStyle} />
      </DialogClose>
    </div>
  </div>
  <div class={contentContainerStyle}>content</div>
</div>

<script module>
  const containerStyle = css({
    h: "fit",
    w: "full",
  });

  const headerStyle = css({
    display: "flex",
    flexDir: "row",
    justifyContent: "space-between",
    alignItems: "center",
    w: "full",
    h: "8",
    px: "1",
    py: "1",
  });

  const headerLeftButtons = css({
    display: "flex",
    flexDir: "row",
    w: "fit",
  });

  const headerRightButtons = css({
    display: "flex",
    flexDir: "row",
    w: "fit",
  });

  const headerButtonStyle = css({
    ...buttonStyle,
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    shadow: "[none]",
    bg: "transparent",
    _hover: {
      bg: "selected",
    },
  });

  const headerIconStyle = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
  });

  const contentContainerStyle = css({
    px: "2",
    py: "24",
    display: "flex",
    flexDir: "column",
  });
</script>
