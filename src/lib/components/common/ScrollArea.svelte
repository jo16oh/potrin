<script lang="ts">
  import { ScrollArea, type WithoutChild } from "bits-ui";
  import { css } from "styled-system/css";

  type Props = WithoutChild<ScrollArea.RootProps> & {
    orientation: "vertical" | "horizontal" | "both";
    viewportClasses?: string;
  };

  let {
    ref = $bindable(null),
    orientation = "vertical",
    viewportClasses,
    children,
    ...restProps
  }: Props = $props();
</script>

{#snippet Scrollbar({ orientation }: { orientation: "vertical" | "horizontal" })}
  <ScrollArea.Scrollbar
    {orientation}
    class={css({
      display: "flex",
      w: "2.5",
      touchAction: "none",
      userSelect: "none",
      borderRadius: "[9999px]",
      transition: "all",
      p: "[1px]",
      bg: "selected",
      borderXWidth: "thin",
      borderXColor: "transparent",
      _hover: {
        w: "3",
        bg: "selected",
      },
      duration: "500",
      "&[data-state=visible]": {
        fadeIn: "0",
      },
      "&[data-state=hidden]": {
        fadeOut: "0",
      },
    })}
  >
    <ScrollArea.Thumb
      class={css({
        flex: "1",
        borderRadius: "[9999px]",
        bg: "view.text-muted",
      })}
    />
  </ScrollArea.Scrollbar>
{/snippet}

<ScrollArea.Root bind:ref {...restProps}>
  <ScrollArea.Viewport class={viewportClasses}>
    {@render children?.()}
  </ScrollArea.Viewport>
  {#if orientation === "vertical" || orientation === "both"}
    {@render Scrollbar({ orientation: "vertical" })}
  {/if}
  {#if orientation === "horizontal" || orientation === "both"}
    {@render Scrollbar({ orientation: "horizontal" })}
  {/if}
  <ScrollArea.Corner />
</ScrollArea.Root>
