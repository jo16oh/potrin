<script lang="ts">
  import { Button } from "bits-ui";
  import { css } from "styled-system/css";
  import type { Styles } from "styled-system/css";
  import type { Snippet } from "svelte";

  let {
    style,
    disabled = $bindable(),
    children,
    ...others
  }: {
    style?: Styles;
    disabled?: boolean;
    children: Snippet;
    onclick?: () => void;
  } = $props();
</script>

<Button.Root class={css(buttonStyle, style)} {disabled} {...others}>
  {@render children?.()}</Button.Root
>

<script module>
  export const buttonStyle = css.raw({
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    gap: "2",
    paddingX: "4",
    paddingY: "2",
    bg: "button.bg",
    shadow: "sm",
    rounded: "lg",
    width: "fit",
    height: "fit",
    _hover: {
      bg: "selected",
    },
    _disabled: {
      color: "button.text-muted",
      bg: "button.bg/20",
      _hover: {
        bg: "button.bg/20",
      },
    },
    transition: "colors",
  });
</script>
