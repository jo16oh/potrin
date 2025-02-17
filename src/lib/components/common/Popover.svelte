<script lang="ts">
  import { Popover, type WithoutChild } from "bits-ui";
  import { css, type Styles } from "styled-system/css";
  import type { Snippet } from "svelte";

  type Props = Popover.RootProps & {
    trigger: Snippet;
    content: Snippet;
    triggerProps?: WithoutChild<Popover.TriggerProps>;
    contentProps?: WithoutChild<Popover.ContentProps>;
    triggerStyle?: Styles;
    contentStyle?: Styles;
  };

  let {
    open = $bindable(false),
    children,
    contentProps,
    trigger,
    content,
    triggerStyle,
    triggerProps,
    contentStyle,
    ...restProps
  }: Props = $props();
</script>

<Popover.Root bind:open {...restProps}>
  <Popover.Trigger class={css(triggerStyle) + " group"} {...triggerProps}
    >{@render trigger()}</Popover.Trigger
  >
  <Popover.Content
    class={css(defaultContentStyle, contentStyle)}
    sideOffset={8}
    onOpenAutoFocus={(e) => {
      e.preventDefault();
    }}
    onCloseAutoFocus={(e) => {
      e.preventDefault();
    }}
    {...contentProps}
  >
    {@render content()}</Popover.Content
  >
</Popover.Root>

<script module>
  const defaultContentStyle = css.raw({
    zIndex: "global.overlay",
    w: "full",
    maxW: "80",
    rounded: "md",
    shadow: "md.around",
    p: "2",
    _open: {
      zoomIn: "0.75",
      fadeIn: "0",
    },
    _closed: {
      zoomOut: "0.75",
      fadeOut: "0",
    },
    "&[data-side=bottom]": {
      slideInY: "2",
    },
    "&[data-side=left]": {
      slideInX: "2",
    },
    "&[data-side=top]": {
      slideInY: "-2",
    },
    "&[data-side=right]": {
      slideInX: "-2",
    },
  });
</script>
