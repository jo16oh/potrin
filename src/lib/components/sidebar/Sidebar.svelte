<script lang="ts">
  import { css } from "styled-system/css";
  import Button, { buttonStyle } from "../common/Button.svelte";
  import { ChevronDown, PanelLeft, PanelRightOpen } from "lucide-svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import Popover from "../common/Popover.svelte";

  const MAX_WIDTH = 38;
  const MIN_WIDTH = 10;
  const REM = 16;

  const [getWorkspaceState, updateWorkspaceState] = Workspace.state();

  const workspaceState = $derived.by(getWorkspaceState);

  // svelte-ignore state_referenced_locally
  let width = $state(workspaceState.sidebar.width);
  // svelte-ignore state_referenced_locally
  let visible = $state(!workspaceState.sidebar.isFloat);
  let dragging = $state(false);

  function resize(
    e: MouseEvent & {
      currentTarget: EventTarget & HTMLDivElement;
    },
  ) {
    dragging = true;
    const prevWidth = width;
    const start = e.clientX;

    const handleMouseMove = (e: MouseEvent) => {
      const currentWidth = prevWidth + (e.clientX / REM - start / REM);

      if (MIN_WIDTH <= currentWidth && currentWidth <= MAX_WIDTH) {
        width = currentWidth;
      }
    };

    const cleanup = () => {
      updateWorkspaceState((prev) => {
        prev.sidebar.width = width;
        return prev;
      });

      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup ", cleanup);
      dragging = false;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", cleanup);
  }

  function toggleFloat() {
    visible = false;
    updateWorkspaceState((state) => {
      state.sidebar.isFloat = !state.sidebar.isFloat;
      return state;
    });
  }
</script>

{#if workspaceState.sidebar.isFloat && !visible}
  <!-- eslint-ignore a11y_no_static_element_interactions  -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    onmouseenter={() => (visible = true)}
    class={css({
      position: "fixed",
      h: "screen",
      w: "[8px]",
      top: "0",
      left: "0",
    })}
  ></div>
{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  data-visible={visible}
  data-float={workspaceState.sidebar.isFloat}
  class={containerStyle}
  style:width={`${workspaceState.sidebar.isFloat ? width + 1 : width}rem`}
  onmouseleave={(e) => {
    if (!dragging && e.clientX > 16) visible = false;
  }}
>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="separator"
    aria-orientation="vertical"
    class={resizeHandlerStyle}
    onmousedown={resize}
    data-float={workspaceState.sidebar.isFloat}
    data-dragging={dragging}
  >
    <div></div>
  </div>
  <div class={headerStyle}>
    <Popover triggerStyle={potNameButtonStyle} contentProps={{ align: "end" }}>
      {#snippet trigger()}
        <div class={potNameStyle}>
          {workspaceState.pot.name}
        </div>
        <div class={chevronDownContainerStyle}>
          <ChevronDown class={chevronDownStyle} />
        </div>
      {/snippet}
      {#snippet content()}
        text
      {/snippet}
    </Popover>
    <Button style={collapseButtonStyle} onclick={toggleFloat}>
      {#if workspaceState.sidebar.isFloat}
        <PanelLeft class={sidebarButtonIconStyle} />
      {:else}
        <PanelRightOpen class={sidebarButtonIconStyle} />
      {/if}
    </Button>
  </div>
</div>

<script module>
  const containerStyle = css({
    flexShrink: "0",
    position: "relative",
    h: "full",
    py: "2",
    display: "flex",
    flexDir: "column",
    gap: "1",
    bg: "accent.bg",
    userSelect: "none",
    "&[data-float=true]": {
      px: "2",
      bg: "accent.bg/95",
      position: "fixed",
      h: "[calc(100% - 28px)]",
      top: "[24px]",
      left: "[6px]",
      rounded: "md",
      shadow: "lg",
      animationStyle: "sidebar-slide-in",
      "&[data-visible=false]": {
        animationStyle: "sidebar-slide-out",
      },
    },
  });

  const resizeHandlerStyle = css({
    position: "absolute",
    top: "[0.25rem]",
    right: "-1.5",
    h: "[calc(100% - 0.5rem)]",
    w: "1",
    transition: "all",
    rounded: "md",
    "&:hover,&[data-dragging=true]": {
      cursor: "col-resize",
      bg: "[yellow]",
    },
    "&[data-float=true]": {
      right: "0",
    },
  });

  const headerStyle = css({
    w: "full",
    h: "9",
    display: "flex",
    flexDir: "row",
    justifyContent: "space-between",
    alignItems: "start",
    gap: "1",
  });

  const potNameButtonStyle = css.raw({
    ...buttonStyle,
    bg: "transparent",
    shadow: "[ none ]",
    display: "flex",
    flexDir: "row",
    justifyContent: "space-between",
    alignItems: "center",
    px: "2",
    py: "1",
    w: "full",
    h: "9",
    rounded: "md",
    minWidth: "0",
  });

  const potNameStyle = css({
    textOverflow: "ellipsis",
    overflow: "hidden",
    fontWeight: "semibold",
    fontSize: "md",
    textAlign: "start",
    whiteSpace: "nowrap",
    minWidth: "0",
  });

  const collapseButtonStyle = css.raw({
    bg: "transparent",
    shadow: "[ none ]",
    display: "flex",
    flexDir: "row",
    justifyContent: "center",
    alignItems: "center",
    p: "1",
    w: "10",
    h: "9",
    rounded: "md",
    fontWeight: "semibold",
    fontSize: "md",
  });

  const sidebarButtonIconStyle = css({
    w: "4",
    h: "4",
    color: "view.text",
  });

  const chevronDownContainerStyle = css({
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    w: "4",
    h: "8",
  });

  const chevronDownStyle = css({
    w: "4",
    h: "4",
    color: "transparent",
    _groupHover: {
      color: "view.text-muted",
    },
  });
</script>
