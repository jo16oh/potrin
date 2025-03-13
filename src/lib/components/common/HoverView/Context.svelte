<script lang="ts">
  import Dialog from "../../common/Dialog.svelte";
  import { CardsView } from "$lib/components/view/CardsView";
  import { css } from "styled-system/css";
  import { View, Workspace } from "$lib/models/Workspace.svelte";
  import { watch } from "runed";
  import type { Snippet } from "svelte";
  import SearchView from "$lib/components/view/SearchView/SearchView.svelte";

  type AllowedViewType = "cards" | "outline" | "search";

  type Props = {
    view: View<AllowedViewType>;
    open?: boolean;
    children?: Snippet;
    rightsideTopButtons?: Snippet;
    rightsideBottomButtons?: Snippet;
  };

  let {
    view = $bindable(),
    open = $bindable(false),
    children,
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
  contentStyle={hoverViewContainerStyle}
  overlayProps={{
    onmousedown: (e) => e.preventDefault(),
  }}
>
  {#snippet rootContent()}
    {@render children?.()}
  {/snippet}
  {#snippet content()}
    {#if view.type === "cards"}
      <CardsView
        bind:view
        isFocused={open}
        onCloseButtonClick={() => (open = false)}
      />
    {:else if view.type === "search"}
      <SearchView {view} pinned={false} />
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
