<script lang="ts">
  import { Columns2, Maximize2, PencilLine } from "lucide-svelte";
  import Dialog from "../common/Dialog.svelte";
  import CardsView from "../view/CardsView.svelte";
  import Button from "../common/Button.svelte";
  import { css } from "styled-system/css";
</script>

<Dialog
  triggerStyle={floatingButtonStyle}
  contentStyle={hoverViewContainerStyle}
>
  {#snippet trigger()}
    <PencilLine class={floatingButtonIconStyle} />
  {/snippet}
  {#snippet content()}
    <CardsView />
    <div class={rightSideButtonContainer}>
      <Button style={rightSideButtonStyle}>
        <Maximize2 class={iconInsideRightSideButton} />
      </Button>
      <Button style={rightSideButtonStyle}>
        <Columns2 class={iconInsideRightSideButton} />
      </Button>
      <Button style={rightSideButtonStyle}>
        <PencilLine class={iconInsideRightSideButton} />
      </Button>
    </div>
  {/snippet}
</Dialog>

<script module>
  const floatingButtonStyle = css.raw({
    zIndex: "global.float",
    position: "fixed",
    right: "[16px]",
    bottom: "[16px]",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    p: "0",
    w: "14",
    h: "14",
    bg: "workspace.bg/90",
    rounded: "[100%]",
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

  const hoverViewContainerStyle = css.raw({
    top: "[0.5rem]",
    w: "[80vw]",
    h: "[90vh]",
    maxW: "[37.5rem]",
    p: "0",
  });

  const rightSideButtonContainer = css({
    display: "flex",
    flexDir: "column",
    gap: "4",
    p: "0",
    position: "absolute",
    top: "4",
    left: "[calc(100% + 0.75rem)]",
    w: "fit",
    h: "fit",
  });

  const rightSideButtonStyle = css.raw({
    p: "0",
    w: "8",
    h: "8",
    bg: "view.bg",
    _hover: {
      bg: "view.bg-selected",
    },
    rounded: "circle",
  });

  const iconInsideRightSideButton = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
  });
</script>
