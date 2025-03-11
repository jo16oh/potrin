<script lang="ts">
  import { css, type Styles } from "styled-system/css";
  import type { Snippet } from "svelte";
  import type { SvelteHTMLElements } from "svelte/elements";

  type Props = {
    left?: Snippet<[{ buttonStyle: Styles; iconStyle: Styles }]>;
    center?: Snippet<[{ buttonStyle: Styles; iconStyle: Styles; textStyle: Styles; }]>;
    right?: Snippet<[{ buttonStyle: Styles; iconStyle: Styles }]>;
  } & SvelteHTMLElements["div"];

  let { left, center, right }: Props = $props();
</script>
<div class={headerStyle}>
  <div class={headerLeftContainer}>
    {@render left?.({
      buttonStyle: headerButtonStyle,
      iconStyle: headerIconStyle,
    })}
  </div>
  <div class={headerCenterContainer}>
    {@render center?.({
      buttonStyle: headerTitleButtonStyle,
      iconStyle: headerTitleIconStyle,
      textStyle: headerTitleTextStyle,
    })}
  </div>
  <div class={headerRightContainer}>
    {@render right?.({
      buttonStyle: headerButtonStyle,
      iconStyle: headerIconStyle,
    })}
  </div>
</div>

<script module>
  const headerStyle = css({
    display: "grid",
    gridTemplateColumns: "[1fr auto 1fr]",
    gap: "4",
    justifyContent: "space-between",
    alignItems: "center",
    w: "full",
    overflow: "hidden",
    h: "8",
    p: "1",
    roundedTop: "md",
  });

  const headerLeftContainer = css({
    flexBasis: "[auto]",
    flexGrow: "0",
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    gap: "2",
  });

  const headerRightContainer = css({
    flexBasis: "[auto]",
    flexGrow: "0",
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    justifyContent: "end",
    gap: "2",
  });

  const headerButtonStyle = css.raw({
    justifySelf: "end",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    transition: "colors",
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    shadow: "[none]",
    bg: "transparent",
    _disabled: {
      _hover: {
        bg: "transparent",
      },
    },
    _hover: {
      bg: "view.bg-selected",
    },
  });

  const headerIconStyle = css.raw({
    w: "4",
    h: "4",
    color: "view.text-muted",
    "&[data-disabled=true]": {
      color: "view.text-muted/50",
    },
  });

  const headerCenterContainer = css({
    justifySelf: "center",
    w: "full",
    flex: "[0 1 auto]",
    display: "flex",
    flexDir: "row",
    justifyContent: "center",
    alignItems: "center",
    h: "full",
    minW: "0",
    overflow: "hidden",
    textOverflow: "ellipsis",
  });

  const headerTitleButtonStyle = css.raw({
    flexShrink: "0",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    transition: "colors",
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
  });

  const headerTitleIconStyle = css.raw({
    w: "3",
    h: "3",
    color: "view.text-muted",
  });

  const headerTitleTextStyle = css.raw({
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
    color: "view.text-muted",
    fontSize: "xs",
    cursor: "default",
  });
</script>
