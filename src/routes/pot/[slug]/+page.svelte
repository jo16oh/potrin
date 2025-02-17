<script lang="ts">
  import Dialog from "$lib/components/common/Dialog.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import CardsView from "$lib/components/view/CardsView.svelte";
  import { PencilLine } from "lucide-svelte";
  import { css } from "styled-system/css";
</script>

<div class={containerStyle}>
  <Sidebar />
  <div
    class={css({
      bg: "view.bg",
      w: "full",
      h: "full",
      rounded: "md",
      shadow: "md.around",
    })}
  ></div>
  <div
    class={css({
      bg: "view.bg",
      w: "full",
      h: "full",
      rounded: "md",
      shadow: "md.around",
    })}
  ></div>
  <Dialog
    triggerStyle={floatingButtonStyle}
    contentStyle={hoverViewContainerStyle}
  >
    {#snippet trigger()}
      <PencilLine class={floatingButtonIconStyle} />
    {/snippet}
    {#snippet content()}
      <CardsView />
    {/snippet}
  </Dialog>
</div>

<script module>
  const containerStyle = css({
    display: "flex",
    flexDir: "row",
    gap: "2",
    bg: "accent.bg",
    w: "screen",
    h: "screen",
    pt: "[28px]",
    pb: "[8px]",
    px: "[8px]",
  });

  const floatingButtonStyle = css.raw({
    position: "fixed",
    right: "[16px]",
    bottom: "[16px]",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    p: "0",
    w: "14",
    h: "14",
    bg: "accent.bg/90",
    rounded: "[100%]",
    transition: "colors",
    _hover: {
      bg: "accent.bg-selected",
    },
  });

  const floatingButtonIconStyle = css({
    w: "6",
    h: "6",
    color: "[white]",
  });

  const hoverViewContainerStyle = css.raw({
    w: "[90vw]",
    h: "[95vh]",
    maxW: "[37.5rem]",
    p: "0",
    overflow: "hidden",
  });
</script>
