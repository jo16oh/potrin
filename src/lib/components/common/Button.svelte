<script lang="ts">
  import { Button } from "bits-ui";
  import { css } from "styled-system/css";
  import type { Styles } from "styled-system/css";
  import type { Snippet } from "svelte";

  type ButtonProps = Omit<Button.RootProps, "style">;
  type Props = ButtonProps & {
    style?: Styles;
    disabled?: boolean;
    children: Snippet;
  };

  let {
    style,
    disabled = $bindable(false),
    children,
    ...others
  }: Props = $props();
</script>

<Button.Root class={css(buttonStyle, style) + " group"} {disabled} {...others}>
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
      bg: "darken",
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
