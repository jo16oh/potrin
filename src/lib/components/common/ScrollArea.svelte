<script lang="ts">
  import { ScrollArea, type WithoutChild } from "bits-ui";
  import { css, type Styles } from "styled-system/css";
  import { onMount } from "svelte";

  type RootProps = Omit<ScrollArea.RootProps, "style">;
  type Props = RootProps & {
    orientation: "vertical" | "horizontal" | "both";
    scrollbarMode: "overlay" | "inset";
    viewportProps?: WithoutChild<ScrollArea.ViewportProps>;
    style?: Styles;
    viewportStyle?: Styles;
  };

  let {
    orientation = "vertical",
    scrollbarMode,
    viewportProps,
    style,
    viewportStyle,
    children,
    ...restProps
  }: Props = $props();

  let viewportRef = $state<HTMLDivElement | null>(null);
  let viewportHeight = $state<number | undefined>();
  let viewportWidth = $state<number | undefined>();
  let contentWidth = $state<number | undefined>();
  let contentHeight = $state<number | undefined>();

  onMount(() => {
    const resizeObserver = new ResizeObserver(() => {
      window.requestAnimationFrame(() => {
        if (viewportRef) {
          viewportHeight = viewportRef.clientHeight;
          viewportWidth = viewportRef.clientWidth;
        }
      });
    });

    if (viewportRef) resizeObserver.observe(viewportRef);

    return () => resizeObserver.disconnect();
  });

  const overflowVertical = $derived(
    viewportHeight !== undefined && contentHeight !== undefined
      ? viewportHeight < contentHeight
      : false,
  );

  const overflowHorizontal = $derived(
    viewportWidth !== undefined && contentWidth !== undefined
      ? viewportWidth < contentWidth
      : false,
  );

  export function scrollTo(...args: Parameters<typeof window.scrollTo>) {
    viewportRef?.scrollTo(...args);
  }

  export function getScrollTop() {
    return viewportRef!.scrollTop;
  }

  export function getScrollLeft() {
    return viewportRef!.scrollLeft;
  }

  export function isOverflowing(direction: "horizontal" | "vertical") {
    if (direction === "horizontal") {
      return overflowHorizontal;
    } else {
      return overflowVertical;
    }
  }
</script>

{#snippet Scrollbar({
  orientation,
}: {
  orientation: "vertical" | "horizontal";
})}
  <ScrollArea.Scrollbar {orientation} class={scrollbarStyle}>
    <ScrollArea.Thumb class={scrollThumbStyle} />
  </ScrollArea.Scrollbar>
{/snippet}

<ScrollArea.Root
  class={css(defaultRootStyle, style)}
  data-orientation={orientation}
  data-scrollbar-mode={scrollbarMode}
  data-overflow-vertical={overflowVertical}
  data-overflow-horizontal={overflowHorizontal}
  {...restProps}
>
  <ScrollArea.Viewport
    bind:ref={viewportRef}
    class={css(defaultViewportStyle, viewportStyle)}
    {...viewportProps}
  >
    <div
      bind:clientWidth={contentWidth}
      bind:clientHeight={contentHeight}
      class={contentContainerSytle}
    >
      {@render children?.()}
    </div>
  </ScrollArea.Viewport>
  {#if orientation === "vertical" || orientation === "both"}
    {@render Scrollbar({ orientation: "vertical" })}
  {/if}
  {#if orientation === "horizontal" || orientation === "both"}
    {@render Scrollbar({ orientation: "horizontal" })}
  {/if}
  <ScrollArea.Corner />
</ScrollArea.Root>

<script module>
  const defaultRootStyle = css.raw({
    w: "full",
    h: "full",
    flex: "auto",
    overflow: "auto",
    "&[data-scrollbar-mode=inset][data-orientation=vertical][data-overflow-vertical=true]":
      {
        pr: "3",
      },
    "&[data-scrollbar-mode=inset][data-orientation=horizontal][data-overflow-horizontal=true]":
      {
        pb: "3",
      },
    "&[data-scrollbar-mode=inset][data-orientation=both]": {
      "&[data-overflow-vertical]": {
        pl: "3",
      },
      "&[data-overflow-horizontal]": {
        pb: "3",
      },
    },
  });

  const defaultViewportStyle = css.raw({
    w: "full",
    h: "full",
  });

  const contentContainerSytle = css({
    "&[data-orientation=vertical]": {
      w: "full",
      h: "fit",
    },
    "&[data-orientation=horizontal]": {
      w: "fit",
      h: "full",
    },
    "&[data-orientation=both]": {
      w: "fit",
      h: "fit",
    },
  });

  const scrollbarStyle = css({
    zIndex: "local.scrollbar",
    display: "flex",
    w: "2.5",
    touchAction: "none",
    userSelect: "none",
    rounded: "lg",
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
    "&[data-state=hidden]": {
      fadeOut: "0",
    },
  });

  const scrollThumbStyle = css({
    flex: "1",
    rounded: "lg",
    bg: "view.text-muted",
  });
</script>
