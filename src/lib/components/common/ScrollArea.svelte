<script lang="ts">
  import { css, type Styles } from "styled-system/css";
  import type { Snippet } from "svelte";
  import type { SvelteHTMLElements } from "svelte/elements";

  type Props = SvelteHTMLElements["div"] & {
    ref?: HTMLDivElement | undefined;
    orientation?: "vertical" | "horizontal";
    type?: "auto" | "always";
    scrollAreaStyle?: Styles;
    children?: Snippet;
  };

  let {
    ref = $bindable(),
    orientation = "vertical",
    type = "auto",
    scrollAreaStyle,
    onscroll,
    children,
    ...restProps
  }: Props = $props();
</script>

<div
  bind:this={ref}
  data-orientation={orientation}
  data-scroll-type={type}
  class={css(defaultScrollAreaStyle, scrollAreaStyle)}
  onscroll={(e) => {
    onscroll?.(e);
  }}
  {...restProps}
>
  {@render children?.()}
</div>

<script module>
  const defaultScrollAreaStyle = css.raw({
    w: "full",
    h: "full",
    "&[data-orientation=horizontal]": {
      "&[data-scroll-type=auto]": {
        overflowX: "auto",
      },
      "&[data-scroll-type=always]": {
        overflowX: "scroll",
      },
    },
    "&[data-orientation=vertical]": {
      "&[data-scroll-type=auto]": {
        overflowY: "auto",
      },
      "&[data-scroll-type=always]": {
        overflowY: "scroll",
      },
    },
    _scrollbar: {
      bg: "selected",
      w: "[10px]",
      h: "[10px]",
      p: "[1px]",
      rounded: "md",
    },
    _scrollbarThumb: {
      bg: "view.text-muted",
      rounded: "[16px]",
      border: "[2px solid transparent]",
      backgroundClip: "padding-box",
    },
    _scrollbarTrack: {
      margin: "[4px]",
    },
  });
</script>
