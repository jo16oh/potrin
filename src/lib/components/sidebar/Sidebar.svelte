<script lang="ts">
  import { css } from "styled-system/css";
  import Button from "../common/Button.svelte";
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

  const workspace = Workspace.current;

  const pot = $derived(workspace.state.pot);
  const sidebar = $derived(workspace.state.sidebar);
  const focusedTabId = $derived(workspace.state.focusedTabId);
  const tabs = $derived(workspace.state.tabs);

  let width = $state(workspace.state.sidebar.width);
  let sidebarOpen = $state(!workspace.state.sidebar.isFloat);
  let dragging = $state(false);

  let potOperationsOpen = $state(false);
  let potRenameDialogOpen = $state(false);

  watch(
    () => potRenameDialogOpen,
    () => {
      if (!potRenameDialogOpen && sidebar.isFloat) {
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
    e.preventDefault();

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
      sidebar.width = width;

      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup ", cleanup);
      dragging = false;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", cleanup);
  }

  function toggleFloat() {
    sidebarOpen = false;
    sidebar.isFloat = !sidebar.isFloat;
  }
</script>

{#if sidebar.isFloat && !sidebarOpen}
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
  data-float={sidebar.isFloat}
  class={containerStyle}
  style:width={`${sidebar.isFloat ? width + 1 : width}rem`}
  onmouseleave={(e) => {
    if (potRenameDialogOpen) return;
    if (sidebar.isFloat) potOperationsOpen = false;
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
          {pot.name}
        </div>
        <div class={chevronDownContainerStyle}>
          <ChevronDown class={chevronDownStyle} />
        </div>
      {/snippet}
      {#snippet content()}
        <RenamePot
          {pot}
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
    <Button class={collapseButtonStyle} onclick={toggleFloat}>
      {#if sidebar.isFloat}
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
          {#each tabs as tab, tabIdx (tab.id)}
            <Button
              class={tabItemStyle + " group"}
              data-selected={focusedTabId === tab.id}
              onclick={() => (workspace.state.focusedTabId = tab.id)}
              onmousedown={(e: MouseEvent) => {
                // this prevents editor from being blurred
                e.preventDefault();
              }}
            >
              {#each tab.views as view, viewIdx (view.id)}
                <div class={viewItemStyle}>
                  <div class={viewIconContainerStyle}>
                    {#if view.type === "cards"}
                      <CardStack class={viewIconStyle} />
                    {/if}
                  </div>
                  {#if view.type === "cards"}
                    <div class={viewTitleStyle}>{view.title}</div>
                  {/if}
                  <Button
                    class={viewCloseButtonStyle}
                    onclick={(e: MouseEvent) => {
                      e.stopPropagation();
                      workspace.closeView(tab, tabIdx, view, viewIdx);
                    }}
                  >
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
    data-float={sidebar.isFloat}
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
    transition: "colors",
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

  const collapseButtonStyle = css({
    _hover: {
      bg: "darken",
    },
    transition: "colors",
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
    display: "flex",
    flexDirection: "row",
    alignItems: "center",
    gap: "2",
    _hover: {
      bg: "darken",
    },
    transition: "colors",
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
    h: "full",
    flex: "auto",
    display: "flex",
    flexDir: "column",
    gap: "1",
    overflow: "hidden",
    pb: "2",
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
