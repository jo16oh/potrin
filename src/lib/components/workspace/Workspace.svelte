<script lang="ts">
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import { css } from "styled-system/css";
  import HoverViewButton from "./HoverViewButton.svelte";
  import SearchViewButton from "./SearchViewButton.svelte";
  import Tab from "./Tab.svelte";
  import ScrollArea from "../common/ScrollArea.svelte";

  const workspace = Workspace.current;
  const pinnedTabs = $derived(workspace.state.pinnedTabs);
  const tabs = $derived(workspace.state.tabs);
  const focusedTabId = $derived(workspace.state.focusedTabId);
</script>

<div class={workspaceContainerStyle}>
  <Sidebar />

  <ScrollArea {scrollAreaStyle}>
    {#each pinnedTabs as tab, tabIdx}
      <Tab {tab} {tabIdx} />
    {/each}

    {#each tabs as tab, tabIdx}
      <Tab {tab} {tabIdx} />
    {/each}

    {#if focusedTabId === null}
      <div class={viewStyle}></div>
    {/if}
  </ScrollArea>

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
    overflow: "hidden",
    pt: "[24px]",
    pb: "[4px]",
    px: "[4px]",
  });

  const scrollAreaStyle = css.raw({
    p: "[4px]",
  });

  const viewStyle = css({
    flex: "1",
    bg: "view.bg",
    h: "full",
    rounded: "md",
    shadow: "md.around",
  });
</script>
