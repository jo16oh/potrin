<script lang="ts">
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import { css } from "styled-system/css";
  import HoverViewButton from "./HoverViewButton.svelte";
  import SearchViewButton from "./SearchViewButton.svelte";
  import Tab from "./Tab.svelte";

  const workspace = Workspace.current;
  const pinnedTabs = $derived(workspace.state.pinnedTabs);
  const tabs = $derived(workspace.state.tabs);
  const focusedTabId = $derived(workspace.state.focusedTabId);
</script>

<div class={workspaceContainerStyle}>
  <Sidebar />

  <div class={tabsContainerStyle}>
    {#each pinnedTabs as tab, tabIdx}
      <Tab {tab} {tabIdx} />
    {/each}

    {#each tabs as tab, tabIdx}
      <Tab {tab} {tabIdx} />
    {/each}

    {#if focusedTabId === null}
      <div class={viewStyle}></div>
    {/if}
  </div>

  <SearchViewButton />
  <HoverViewButton />
</div>

<script module>
  const workspaceContainerStyle = css({
    display: "flex",
    position: "relative",
    bg: "workspace.bg",
    w: "screen",
    h: "screen",
    pt: "[28px]",
    pb: "[8px]",
    px: "[8px]",
    overflow: "hidden",
  });

  const tabsContainerStyle = css({
    w: "full",
    h: "full",
  });

  const viewStyle = css({
    flex: "1",
    bg: "view.bg",
    h: "full",
    rounded: "md",
    shadow: "md.around",
  });
</script>
