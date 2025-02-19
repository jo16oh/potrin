<script lang="ts">
  import { css } from "styled-system/css";
  import Button, { buttonStyle } from "../common/Button.svelte";
  import {
    ChevronDown,
    ClockArrowDown,
    PanelRight,
    PanelRightOpen,
    PencilLine,
    SquareArrowUpRight,
    X,
  } from "lucide-svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import Popover from "../common/Popover.svelte";
  import { commands } from "../../../generated/tauri-commands";
  import PopoverClose from "../common/PopoverClose.svelte";
  import RenamePot from "../entry/RenamePot.svelte";
  import { watch } from "runed";
  import { unwrap } from "$lib/utils";
  import CardStack from "../icon/CardStack.svelte";
  import ScrollArea from "../common/ScrollArea.svelte";

  const MAX_WIDTH_REM = 38;
  const MIN_WIDTH_REM = 10;
  const REM = 16;

  const [getWorkspaceState, updateWorkspaceState] = Workspace.state();

  const workspaceState = $derived.by(getWorkspaceState);
  const focus = $derived(workspaceState.focus);
  const tabs = $derived(workspaceState.tabs);

  // updateWorkspaceState((state) => {
  //   state.tabs = Array(19).fill({
  //     views: [
  //       {
  //         id: crypto.randomUUID(),
  //         title: "吾輩は猫である",
  //         flexGrow: 1,
  //         viewType: "outline",
  //       },
  //     ],
  //     focusedViewIdx: 0,
  //   });
  //
  //   return state;
  // });

  // svelte-ignore state_referenced_locally
  let width = $state(workspaceState.sidebar.width);
  // svelte-ignore state_referenced_locally
  let sidebarOpen = $state(!workspaceState.sidebar.isFloat);
  let dragging = $state(false);

  let potOperationsOpen = $state(false);
  let potRenameDialogOpen = $state(false);

  watch(
    () => potRenameDialogOpen,
    () => {
      if (!potRenameDialogOpen && workspaceState.sidebar.isFloat) {
        potOperationsOpen = false;
        sidebarOpen = false;
      }
    },
    { lazy: true },
  );

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

      if (MIN_WIDTH_REM <= currentWidth && currentWidth <= MAX_WIDTH_REM) {
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
    sidebarOpen = false;
    updateWorkspaceState((state) => {
      state.sidebar.isFloat = !state.sidebar.isFloat;
      return state;
    });
  }
</script>

{#if workspaceState.sidebar.isFloat && !sidebarOpen}
  <!-- eslint-ignore a11y_no_static_element_interactions  -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    onmouseenter={() => (sidebarOpen = true)}
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
  data-visible={sidebarOpen}
  data-float={workspaceState.sidebar.isFloat}
  class={containerStyle}
  style:width={`${workspaceState.sidebar.isFloat ? width + 1 : width}rem`}
  onmouseleave={(e) => {
    if (potRenameDialogOpen) return;
    if (workspaceState.sidebar.isFloat) potOperationsOpen = false;
    if (!dragging && e.clientX > 16) sidebarOpen = false;
  }}
>
  <div class={headerStyle}>
    <Popover
      bind:open={potOperationsOpen}
      triggerStyle={potNameButtonStyle}
      contentStyle={potOperationsContainerStyle}
      contentProps={{
        align: "end",
      }}
    >
      {#snippet trigger()}
        <div class={potNameStyle}>
          {workspaceState.pot.name}
        </div>
        <div class={chevronDownContainerStyle}>
          <ChevronDown class={chevronDownStyle} />
        </div>
      {/snippet}
      {#snippet content()}
        <RenamePot
          pot={workspaceState.pot}
          buttonStyle={potOperationsItemStyle}
          bind:open={potRenameDialogOpen}
        >
          {#snippet button()}
            <PencilLine class={iconStyle} />
            Rename the pot
          {/snippet}
        </RenamePot>
        <PopoverClose
          class={css(potOperationsItemStyle)}
          onclick={async () => await commands.openPotSelector().then(unwrap)}
        >
          <SquareArrowUpRight class={iconStyle} />
          Open another pot</PopoverClose
        >
      {/snippet}
    </Popover>
    <Button style={collapseButtonStyle} onclick={toggleFloat}>
      {#if workspaceState.sidebar.isFloat}
        <PanelRight class={sidebarButtonIconStyle} />
      {:else}
        <PanelRightOpen class={sidebarButtonIconStyle} />
      {/if}
    </Button>
  </div>
  <div class={contentContainerStyle}>
    <div class={fixedTabsContainerStyle}>
      <Button class={tabItemStyle + " group"}>
        <div class={viewItemStyle}>
          <div class={viewIconContainerStyle}>
            <ClockArrowDown class={viewIconStyle} />
          </div>
          <div class={viewTitleStyle}>Timeline</div>
        </div>
      </Button>
    </div>

    <div class={tabsContainerStyle}>
      <div class={tabsTitleStyle}>Tabs</div>
      <ScrollArea orientation="vertical" type="auto" scrollbarMode="inset">
        <div class={tabsContainerStyle}>
          {#each tabs as tab, idx}
            <Button
              class={tabItemStyle + " group"}
              data-selected={focus
                ? focus.area === "tabs" && focus.index === idx
                : false}
              onclick={() =>
                updateWorkspaceState((state) => {
                  state.focus = { area: "tabs", index: idx };
                  return state;
                })}
            >
              {#each tab.views as view}
                <div class={viewItemStyle}>
                  <div class={viewIconContainerStyle}>
                    <CardStack class={viewIconStyle} />
                  </div>
                  <div class={viewTitleStyle}>{view.title}</div>
                  <Button class={viewCloseButtonStyle}>
                    <X class={viewIconStyle} />
                  </Button>
                </div>
              {/each}
            </Button>
          {/each}
        </div>
      </ScrollArea>
    </div>
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="separator"
    aria-orientation="vertical"
    class={resizeHandlerStyle + " resize"}
    onmousedown={resize}
    data-float={workspaceState.sidebar.isFloat}
    data-dragging={dragging}
  ></div>
</div>

<script module>
  const containerStyle = css({
    maxW: "[45vw]",
    flexShrink: "0",
    position: "relative",
    h: "full",
    display: "flex",
    flexDir: "column",
    gap: "2",
    bg: "workspace.bg",
    userSelect: "none",
    "&[data-float=true]": {
      zIndex: "global.sidebar",
      maxW: "[90vw]",
      p: "2",
      bg: "workspace.bg/95",
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
    gap: "0.5",
  });

  const potNameButtonStyle = css.raw({
    ...buttonStyle,
    bg: "transparent",
    _hover: {
      bg: "workspace.bg-selected",
    },
    shadow: "[none]",
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
    fontSize: "sm",
    textAlign: "start",
    whiteSpace: "nowrap",
    minWidth: "0",
  });

  const collapseButtonStyle = css.raw({
    bg: "transparent",
    shadow: "[none]",
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
    color: "workspace.text",
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
      color: "workspace.text-muted",
    },
  });

  const potOperationsContainerStyle = css.raw({
    p: "1",
    userSelect: "none",
    bg: "view.bg",
  });

  const potOperationsItemStyle = css.raw({
    ...buttonStyle,
    justifyContent: "start",
    fontSize: "sm",
    p: "1",
    w: "full",
    h: "fit",
    rounded: "[0.25rem]",
    bg: "transparent",
    shadow: "[none]",
  });

  const iconStyle = css({
    color: "workspace.text-muted",
    w: "4",
    h: "4",
  });

  const contentContainerStyle = css({
    display: "flex",
    flexDir: "column",
    justifyContent: "start",
    alignItems: "start",
    gap: "2",
    w: "full",
    h: "full",
    flex: "auto",
    minH: "0",
    overflow: "hidden",
    rounded: "md",
  });

  const fixedTabsContainerStyle = css({
    display: "flex",
    flexDir: "column",
    justifyContent: "start",
    alignItems: "start",
    gap: "2",
    w: "full",
    flexShrink: "0",
  });

  const tabsContainerStyle = css({
    w: "full",
    flex: "auto",
    display: "flex",
    flexDir: "column",
    gap: "1",
    overflow: "hidden",
  });

  const tabsTitleStyle = css({
    px: "2",
    fontSize: "xs",
    color: "workspace.text-muted",
    h: "6",
    display: "flex",
    alignItems: "center",
  });

  const tabItemStyle = css({
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    alignItems: "center",
    justifyContent: "start",
    w: "full",
    h: "8",
    px: "1",
    gap: "0.5",
    color: "button.text",
    rounded: "md",
    bg: "transparent",
    shadow: "[none]",
    divideX: "[1px]",
    divideColor: "workspace.text-muted",
    overflow: "hidden",
    _hover: {
      bg: "selected",
    },
    "&[data-selected=true]": {
      bg: "button.bg",
      shadow: "sm",
      _hover: {
        bg: "button.bg",
      },
    },
  });

  const viewItemStyle = css({
    display: "flex",
    flexDir: "row",
    alignItems: "center",
    justifyContent: "start",
    flex: "1",
    flexShrink: "1",
    minWidth: "14",
    w: "full",
    h: "8",
    color: "button.text",
    bg: "transparent",
    transition: "all",
  });

  const viewIconContainerStyle = css({
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    w: "6",
    h: "6",
  });

  const viewIconStyle = css({
    color: "workspace.text",
    w: "4",
    h: "4",
  });

  const viewTitleStyle = css({
    pl: "2",
    fontSize: "sm",
    color: "workspace.text",
    flex: "1",
    textAlign: "start",
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
  });

  const viewCloseButtonStyle = css({
    display: "none",
    transition: "all",
    _groupHover: {
      display: "flex",
      color: "workspace.text-muted",
    },
    alignItems: "center",
    justifyContent: "center",
    w: "6",
    h: "6",
    color: "workspace.text",
    _hover: {
      bg: "selected",
    },
    rounded: "[0.25rem]",
  });
</script>
