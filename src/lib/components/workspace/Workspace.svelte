<script lang="ts">
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import { css } from "styled-system/css";
  import { CardsView, TimelineView, SearchView } from "$lib/components/view";
  import HoverViewButton from "./HoverViewButton.svelte";
  import SearchViewButton from "./SearchViewButton.svelte";

  const workspace = Workspace.current;
  const pinnedTabs = $derived(workspace.state.pinnedTabs);
  const tabs = $derived(workspace.state.tabs);
  const focusedTabId = $derived(workspace.state.focusedTabId);
</script>

<div class={workspaceContainerStyle}>
  <Sidebar />

  <div class={tabsContainerStyle}>
    {#each pinnedTabs as tab}
      {#if workspace.isTabLoaded(tab.id)}
        <div class={tabStyle} data-disabled={focusedTabId !== tab.id}>
          {#each tab.views as view (view.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class={viewStyle}
              onmousedown={() => (tab.focusedViewId = view.id)}
            >
              {#if view.type === "timeline"}
                <TimelineView {view} pinned={view.id in tab.pinnedViewIds} />
              {:else if view.type === "search"}
                <SearchView {view} pinned={view.id in tab.pinnedViewIds} />
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/each}

    {#each tabs as tab, tabIdx}
      {#if workspace.isTabLoaded(tab.id)}
        <div class={tabStyle} data-disabled={focusedTabId !== tab.id}>
          {#each tab.views as view, viewIdx (view.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class={viewStyle}
              onmousedown={() => (tab.focusedViewId = view.id)}
            >
              {#if view.type === "cards"}
                <CardsView
                  isFocused={tab.id === focusedTabId &&
                    view.id === tab.focusedViewId}
                  {view}
                  onCloseButtonClick={() =>
                    workspace.closeView(tab, tabIdx, view, viewIdx)}
                />
              {:else if view.type === "search"}
                <SearchView {view} pinned={false} />
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/each}
    {#if focusedTabId === null}
      <div class={viewStyle}></div>
    {/if}

    <SearchViewButton />
    <HoverViewButton />
  </div>
</div>

<script module>
  const workspaceContainerStyle = css({
    display: "flex",
    gap: "2",
    bg: "workspace.bg",
    w: "full",
    h: "[100vh]",
    pt: "[28px]",
    pb: "[8px]",
    px: "[8px]",
    overflow: "hidden",
  });

  const tabsContainerStyle = css({
    w: "full",
    h: "full",
  });

  const tabStyle = css({
    flex: "auto",
    h: "full",
    display: "flex",
    flexDir: "row",
    gap: "2",
    "&[data-disabled=true]": {
      display: "none",
    },
  });

  const viewStyle = css({
    flex: "1",
    bg: "view.bg",
    h: "full",
    rounded: "md",
    shadow: "md.around",
    overflow: "hidden",
  });
</script>
