<script lang="ts">
  import type { Snippet } from "svelte";
  import { Dialog, type WithoutChild } from "bits-ui";
  import { css, type Styles } from "styled-system/css";
  import TitleBarHandler from "./TitleBarHandler.svelte";

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
    <Dialog.Overlay class={overlayStyle}>
      <TitleBarHandler />
    </Dialog.Overlay>
    <Dialog.Content
      class={css(defaultContentStyle, contentStyle)}
      onInteractOutside={(e) => {
        const start = performance.now();
        if (e.clientY <= 28) {
          e.preventDefault();

          const handleMouseUp = () => {
            if (performance.now() - start < 250) {
              open = false;
            }

            window.removeEventListener("mouseup", handleMouseUp);
          };

          window.addEventListener("mouseup", handleMouseUp);
        }
      }}
      onOpenAutoFocus={(e) => {
        e.preventDefault();
      }}
      onCloseAutoFocus={(e) => {
        e.preventDefault();
      }}
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
    zIndex: "global.overlay",
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
    zIndex: "global.overlay",
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
