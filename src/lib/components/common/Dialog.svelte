<script lang="ts">
  import type { Snippet } from "svelte";
  import { Dialog, type WithoutChild } from "bits-ui";
  import { css, type Styles } from "styled-system/css";

  type Props = Dialog.RootProps & {
    trigger: Snippet;
    content: Snippet;
    triggerProps?: WithoutChild<Dialog.TriggerProps>;
    triggerStyle?: Styles;
    contentProps?: WithoutChild<Dialog.ContentProps>;
    contentStyle?: Styles;
  };

  let {
    open = $bindable(false),
    trigger,
    content,
    triggerStyle,
    triggerProps,
    contentStyle,
    contentProps,
    ...restProps
  }: Props = $props();
</script>

<Dialog.Root bind:open {...restProps}>
  <Dialog.Trigger class={css(triggerStyle)} {...triggerProps}>
    {@render trigger()}
  </Dialog.Trigger>
  <Dialog.Portal>
    <Dialog.Overlay class={overlayStyle} />
    <Dialog.Content
      class={css(defaultContentStyle, contentStyle)}
      {...contentProps}
    >
      {@render content()}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<script module>
  const overlayStyle = css({
    position: "fixed",
    inset: "0",
    top: "0",
    left: "0",
    zIndex: "50",
    bg: "overlay",
    duration: "150",
    _open: {
      fadeIn: "0",
    },
    _closed: {
      fadeOut: "0",
    },
  });

  const defaultContentStyle = css.raw({
    display: "flex",
    flexDir: "column",
    gap: "2",
    position: "fixed",
    inset: "0",
    margin: "auto",
    w: "80",
    h: "fit",
    p: "1",
    zIndex: "[50]",
    rounded: "md",
    bg: "view.bg",
    shadow: "lg",
    outline: "none",
    duration: "150",
    _open: {
      zoomIn: "0.75",
      fadeIn: "0",
    },
    _closed: {
      zoomOut: "0.75",
      fadeOut: "0",
    },
  });
</script>
