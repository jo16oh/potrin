<script lang="ts">
  import { css, type Styles } from "styled-system/css";
  import type { Snippet } from "svelte";
  import type { SvelteHTMLElements } from "svelte/elements";

  type Props = SvelteHTMLElements["div"] & {
    orientation: "vertical" | "horizontal" | "both";
    scrollAreaStyle?: Styles;
    children?: Snippet;
  };

  let {
    orientation = "vertical",
    scrollAreaStyle,
    children,
    ...restProps
  }: Props = $props();

  let scrollAreaRef = $state<HTMLDivElement>()!;

  export function getScrollTop() {
    return scrollAreaRef.scrollTop;
  }

  export function getScrollLeft() {
    return scrollAreaRef.scrollLeft;
  }

  export function scrollTo(...args: Parameters<typeof window.scrollTo>) {
    scrollAreaRef.scrollTo(...args);
  }
</script>

<div
  bind:this={scrollAreaRef}
  data-orientation={orientation}
  class={css(defaultScrollAreaStyle, scrollAreaStyle)}
  {...restProps}
>
  {@render children?.()}
</div>

<script module>
  const defaultScrollAreaStyle = css.raw({
    w: "full",
    h: "full",
    "&[data-orientation=horizontal]": {
      overflowX: "auto",
    },
    "&[data-orientation=vertical]": {
      overflowY: "auto",
    },
    "&[data-orientation=both]": {
      overflowX: "auto",
      overflowY: "auto",
    },
    overscrollBehavior: "none",
    _scrollbar: {
      bg: "selected",
      w: "[10px]",
      p: "[1px]",
      rounded: "md",
    },
    _scrollbarThumb: {
      bg: "view.text-muted",
      rounded: "[4px]",
      borderRight: "[2px solid transparent]",
      borderLeft: "[2px solid transparent]",
      backgroundClip: "padding-box",
    },
    _scrollbarTrack: {
      marginTop: "[4px]",
      marginBottom: "[4px]",
    },
  });
</script>
