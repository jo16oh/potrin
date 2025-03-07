<script lang="ts">
  import Dialog from "../common/Dialog.svelte";
  import CardsView from "../view/CardsView.svelte";
  import { css } from "styled-system/css";
  import { View, Workspace } from "$lib/models/Workspace.svelte";
  import { watch } from "runed";
  import type { Snippet } from "svelte";

  type ViewStateTypes = "cards" | "outline";
  type ViewState = Extract<View, { type: ViewStateTypes }>;

  type Props = {
    view: ViewState;
    open: boolean;
    trigger: Snippet;
    rightsideTopButtons?: Snippet;
    rightsideBottomButtons?: Snippet;
  };

  let {
    view = $bindable(),
    open = $bindable(false),
    trigger,
    rightsideTopButtons,
    rightsideBottomButtons,
  }: Props = $props();

  const workspace = Workspace.current;
  let lastFocusedViewId: string | null = null;

  watch(
    () => open,
    () => {
      if (open) {
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
</script>

<Dialog
  bind:open
  triggerStyle={floatingButtonStyle}
  contentStyle={hoverViewContainerStyle}
  triggerProps={{
    onmousedown: (e) => {
      e.preventDefault();
      open = true;
    },
  }}
  {trigger}
>
  {#snippet content()}
    {#if view.type === "cards"}
      <CardsView
        bind:viewState={view}
        isFocused={open}
        onCloseButtonClick={() => (open = false)}
      />
    {/if}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class={rightsideTopButtonsContainer}
      onmousedown={() => (open = false)}
    >
      {@render rightsideTopButtons?.()}
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class={rightsideBottomButtonsContainer}
      onmousedown={() => (open = false)}
    >
      {@render rightsideBottomButtons?.()}
    </div>
  {/snippet}
</Dialog>

<script module>
  const floatingButtonStyle = css.raw({
    zIndex: "global.float",
    position: "fixed",
    right: "[24px]",
    bottom: "[16px]",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    p: "0",
    w: "14",
    h: "14",
    bg: "workspace.bg/90",
    rounded: "circle",
    transition: "colors",
    _hover: {
      bg: "workspace.bg-selected",
    },
  });

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
</script>
