<script lang="ts">
  import { Columns2, Maximize2, PencilLine } from "lucide-svelte";
  import Dialog from "../common/Dialog.svelte";
  import CardsView from "../view/CardsView.svelte";
  import Button, { buttonStyle } from "../common/Button.svelte";
  import { css } from "styled-system/css";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import DialogClose from "../common/DialogClose.svelte";
  import type { ViewState } from "../../../generated/tauri-commands";
  import { watch } from "runed";
  import { Outline } from "$lib/models/Outline.svelte";
  import { Paragraph } from "$lib/models/Paragraph.svelte";

  const workspace = Workspace.current;
  const workspaceState = Workspace.current.state;
  let dialogOpen = $state<boolean>(false);

  let lastFocusedViewId: string | null = null;

  watch(
    () => dialogOpen,
    () => {
      if (dialogOpen) {
        const currentTab = workspace.currentTab();
        if (currentTab) {
          lastFocusedViewId = currentTab.focusedViewId;
          currentTab.focusedViewId = null;
        }
      } else {
        const currentTab = workspace.currentTab();
        if (currentTab) {
          currentTab.focusedViewId = lastFocusedViewId;
        }
      }
    },
    { lazy: true },
  );

  type CardsViewState = Extract<ViewState, { type: "cards" }>;
  let viewState: CardsViewState = $state({
    id: crypto.randomUUID(),
    type: "cards",
    outlineId: null,
    title: "",
    pinned: false,
    scrollPosition: 0,
    focusPosition: { id: null, position: "start" },
    flexGrow: 1,
  });

  async function handleClickMaximize(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();
    dialogOpen = false;

    if (viewState.focusPosition.id) {
      await Outline.buffer.get(viewState.focusPosition.id)?.save();
      await Paragraph.buffer.get(viewState.focusPosition.id)?.save();
    }

    setTimeout(() => {
      const newTabId = crypto.randomUUID();
      workspaceState.focusedTabId = newTabId;
      workspaceState.tabs.unshift({
        id: newTabId,
        views: [$state.snapshot(viewState)],
        focusedViewId: viewState.id,
      });
      viewState = {
        id: crypto.randomUUID(),
        type: "cards",
        outlineId: null,
        title: "",
        pinned: false,
        scrollPosition: 0,
        focusPosition: { id: null, position: "start" },
        flexGrow: 1,
      };
    });
  }
</script>

<Dialog
  bind:open={dialogOpen}
  triggerStyle={floatingButtonStyle}
  contentStyle={hoverViewContainerStyle}
  triggerProps={{
    onmousedown: (e) => {
      e.preventDefault();
      dialogOpen = true;
    },
  }}
  overlayProps={{
    onmousedown: (e) => {
      e.preventDefault();
      dialogOpen = false;
    },
  }}
>
  {#snippet trigger()}
    <PencilLine class={floatingButtonIconStyle} />
  {/snippet}
  {#snippet content()}
    <CardsView
      bind:viewState
      isFocused={dialogOpen}
      onCloseButtonClick={() => (dialogOpen = false)}
    />
    <div class={rightSideButtonContainer}>
      <DialogClose
        class={css(rightSideButtonStyle)}
        onmousedown={(e) => e.preventDefault()}
        onclick={handleClickMaximize}
      >
        <Maximize2 class={iconInsideRightSideButton} />
      </DialogClose>
      <Button style={rightSideButtonStyle}>
        <Columns2 class={iconInsideRightSideButton} />
      </Button>
      <Button style={rightSideButtonStyle}>
        <PencilLine class={iconInsideRightSideButton} />
      </Button>
    </div>
  {/snippet}
</Dialog>

<script module>
  const floatingButtonStyle = css.raw({
    zIndex: "global.float",
    position: "fixed",
    right: "[16px]",
    bottom: "[16px]",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    p: "0",
    w: "14",
    h: "14",
    bg: "workspace.bg/90",
    rounded: "[100%]",
    transition: "colors",
    _hover: {
      bg: "workspace.bg-selected",
    },
  });

  const floatingButtonIconStyle = css({
    w: "6",
    h: "6",
    color: "[white]",
  });

  const hoverViewContainerStyle = css.raw({
    top: "[0.5rem]",
    w: "[80vw]",
    h: "[90vh]",
    maxW: "[38.25rem]",
    p: "0",
  });

  const rightSideButtonContainer = css({
    display: "flex",
    flexDir: "column",
    gap: "4",
    p: "0",
    position: "absolute",
    top: "4",
    left: "[calc(100% + 0.75rem)]",
    w: "fit",
    h: "fit",
  });

  const rightSideButtonStyle = css.raw({
    ...buttonStyle,
    p: "0",
    w: "8",
    h: "8",
    bg: "view.bg",
    _hover: {
      bg: "view.bg-selected",
    },
    rounded: "circle",
  });

  const iconInsideRightSideButton = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
  });
</script>
