<script lang="ts">
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import CreateNewButton from "$lib/components/workspace/CreateNewButton.svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import { css } from "styled-system/css";
  import CardsView from "../view/CardsView.svelte";

  const workspace = Workspace.current;
  const tabs = $derived(workspace.state.tabs);
  const focusedTabId = $derived(workspace.state.focusedTabId);
</script>

<div class={containerStyle}>
  <Sidebar />

  {#each tabs as tab, tabIdx}
    {#if workspace.isTabLoaded(tab.id)}
      <div class={tabStyle} data-disabled={focusedTabId !== tab.id}>
        {#each tab.views as view, viewIdx (view.id)}
          <div class={viewStyle}>
            {#if view.type === "cards"}
              <CardsView
                isFocused={tab.id === focusedTabId &&
                  view.id === tab.focusedViewId}
                viewState={view}
                onCloseButtonClick={() =>
                  workspace.closeView(tab, tabIdx, view, viewIdx)}
              />
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/each}

  {#if focusedTabId === null}
    <div class={viewStyle}></div>
  {/if}

  <CreateNewButton />
</div>

<script module>
  const containerStyle = css({
    display: "flex",
    flexDir: "row",
    gap: "2",
    bg: "workspace.bg",
    w: "screen",
    h: "screen",
    pt: "[28px]",
    pb: "[8px]",
    px: "[8px]",
  });

  const tabStyle = css({
    display: "flex",
    flexDir: "row",
    gap: "2",
    w: "full",
    h: "full",
    "&[data-disabled=true]": {
      display: "none",
    },
  });

  const viewStyle = css({
    bg: "view.bg",
    w: "full",
    h: "full",
    rounded: "md",
    shadow: "md.around",
  });
</script>
