<script lang="ts">
  import Dialog from "$lib/components/common/Dialog.svelte";
  import { CardsView } from "$lib/components/view/CardsView";
  import { css } from "styled-system/css";
  import { View, Workspace } from "$lib/models/Workspace.svelte";
  import { watch } from "runed";
  import { type Snippet } from "svelte";
  import SearchView from "$lib/components/view/SearchView/SearchView.svelte";
  import Button from "$lib/components/common/Button.svelte";
  import { Columns2, Link, Maximize2, PencilLine } from "lucide-svelte";
  import { HoverViewState } from "./State.svelte";

  type Props = {
    children?: Snippet;
    createNewButton?: boolean;
  };

  let { children, createNewButton = false }: Props = $props();

  const workspace = Workspace.current;
  let lastFocusedViewId: string | null = null;

  let context = HoverViewState.current!;

  watch(
    () => context.open,
    () => {
      if (context.open) {
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

  async function handleClickMaximize(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();
    context.open = false;

    await View.save(context.view);

    setTimeout(() => {
      const newTabId = crypto.randomUUID();
      const newViewId = crypto.randomUUID();
      workspace.state.focusedTabId = newTabId;
      workspace.state.tabs.unshift({
        id: newTabId,
        views: [{ ...$state.snapshot(context.view), id: newViewId }],
        focusedViewId: newViewId,
      });

      View.open(context.view, View.new("cards"));
    });
  }

  async function handleClickSplit(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();
    context.open = false;

    await View.save(context.view);

    setTimeout(() => {
      const newViewId = crypto.randomUUID();

      const currentTab = workspace.currentTab();

      if (currentTab) {
        currentTab.views.push({
          ...$state.snapshot(context.view),
          id: newViewId,
        });
        currentTab.focusedViewId = newViewId;
      }

      View.open(context.view, View.new("cards"));
    });
  }

  async function handleClickNew(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();

    await View.save(context.view);

    setTimeout(() => {
      View.open(context.view, View.new(context.view.type));
    });
  }
</script>

<Dialog
  bind:open={context.open}
  contentStyle={hoverViewContainerStyle}
  overlayProps={{
    onmousedown: (e) => e.preventDefault(),
  }}
>
  {#snippet rootContent()}
    {@render children?.()}
  {/snippet}
  {#snippet content()}
    {#if context.view.type === "cards"}
      <CardsView
        bind:view={context.view}
        isFocused={context.open}
        onCloseButtonClick={() => (context.open = false)}
      />
    {:else if context.view.type === "search"}
      <SearchView
        bind:view={context.view}
        pinned={false}
        onCloseButtonClick={() => (context.open = false)}
      />
    {/if}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class={rightsideTopButtonsContainer}
      onmousedown={() => (context.open = false)}
    >
      <Button
        class={rightSideButtonStyle}
        onmousedown={(e: MouseEvent) => {
          e.preventDefault();
          e.stopPropagation();
        }}
        onclick={handleClickMaximize}
      >
        <Maximize2 class={iconInsideRightSideButton} />
      </Button>
      <Button
        class={rightSideButtonStyle}
        onmousedown={(e: MouseEvent) => {
          e.preventDefault();
          e.stopPropagation();
        }}
        onclick={handleClickSplit}
      >
        <Columns2 class={iconInsideRightSideButton} />
      </Button>
      <Button
        class={rightSideButtonStyle}
        onmousedown={(e: MouseEvent) => {
          e.preventDefault();
          e.stopPropagation();
        }}
      >
        <Link class={iconInsideRightSideButton} />
      </Button>
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class={rightsideBottomButtonsContainer}
      onmousedown={() => (context.open = false)}
    >
      {#if createNewButton}
        <Button
          class={rightSideButtonStyle}
          onmousedown={(e: MouseEvent) => {
            e.preventDefault();
            e.stopPropagation();
          }}
          onclick={handleClickNew}
        >
          <PencilLine class={iconInsideRightSideButton} />
        </Button>
      {/if}
    </div>
  {/snippet}
</Dialog>

<script module>
  const hoverViewContainerStyle = css.raw({
    top: "[0.5rem]",
    w: "[80vw]",
    h: "[90vh]",
    maxW: "[38.25rem]",
    p: "0",
  });

  const rightsideTopButtonsContainer = css({
    display: "flex",
    flexDir: "column",
    position: "absolute",
    top: "4",
    right: "-11",
    w: "fit",
    gap: "4",
  });

  const rightsideBottomButtonsContainer = css({
    display: "flex",
    flexDir: "column",
    position: "absolute",
    bottom: "4",
    right: "-11",
    w: "fit",
    gap: "4",
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
