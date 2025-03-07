<script lang="ts">
  import { Columns2, Link, Maximize2, PencilLine } from "lucide-svelte";
  import { css } from "styled-system/css";
  import type { ViewState } from "../../../generated/tauri-commands";
  import HoverViewDialog from "../view/HoverViewDialog.svelte";
  import Button from "../common/Button.svelte";
  import { View, Workspace } from "$lib/models/Workspace.svelte";

  type CardsViewState = Extract<ViewState, { type: "cards" }>;

  let view: CardsViewState = $state({
    id: crypto.randomUUID(),
    type: "cards",
    outlineId: null,
    title: "",
    pinned: false,
    scrollPosition: 0,
    focusPosition: { id: null, position: "start" },
    viewWidthRatio: 1,
  });

  let open = $state(false);

  let workspaceState = Workspace.current.state;

  async function handleClickMaximize(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();
    open = false;

    await View.save(view);

    setTimeout(() => {
      const newTabId = crypto.randomUUID();
      const newViewId = crypto.randomUUID();
      workspaceState.focusedTabId = newTabId;
      workspaceState.tabs.unshift({
        id: newTabId,
        views: [{ ...$state.snapshot(view), id: newViewId }],
        focusedViewId: newViewId,
      });

      View.open(view, View.new(view.type));
    });
  }

  async function handleClickNew(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();

    await View.save(view);

    setTimeout(() => {
      View.open(view, View.new(view.type));
    });
  }
</script>

<HoverViewDialog bind:view bind:open>
  {#snippet trigger()}
    <PencilLine class={floatingButtonIconStyle} />
  {/snippet}
  {#snippet rightsideTopButtons()}
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => e.stopPropagation()}
      onclick={handleClickMaximize}
    >
      <Maximize2 class={iconInsideRightSideButton} />
    </Button>
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => e.stopPropagation()}
    >
      <Columns2 class={iconInsideRightSideButton} />
    </Button>
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => e.stopPropagation()}
    >
      <Link class={iconInsideRightSideButton} />
    </Button>
  {/snippet}
  {#snippet rightsideBottomButtons()}
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => e.stopPropagation()}
      onclick={handleClickNew}
    >
      <PencilLine class={iconInsideRightSideButton} />
    </Button>
  {/snippet}
</HoverViewDialog>

<script module>
  const floatingButtonIconStyle = css({
    w: "6",
    h: "6",
    color: "[white]",
  });

  const rightSideButtonStyle = css({
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    gap: "2",
    shadow: "sm",
    transition: "colors",
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
