<script lang="ts">
  import type { PinnedTabState, TabState } from "generated/tauri-commands";
  import { css } from "styled-system/css";
  import { View, Workspace } from "$lib/models/Workspace.svelte";
  import { SearchView, TimelineView, CardsView } from "../view";

  type Tab = TabState & Partial<PinnedTabState>;

  type Props = {
    tab: Tab;
    tabIdx: number;
  };

  let { tab, tabIdx }: Props = $props();
  const workspace = Workspace.current;
  const focusedTabId = $derived(workspace.state.focusedTabId);

  let draggingViewId = $state<string>();
  let tabRef = $state<HTMLDivElement>()!;

  const REM = 16;
  const minViewWidth = 4 * REM;

  function resize(e: MouseEvent, currentView: View, viewIdx: number) {
    e.preventDefault();
    draggingViewId = currentView.id;

    const nextView = tab.views[viewIdx + 1]!;
    const startX = e.clientX;

    // 初期の比率を保存
    const initialCurrentRatio = currentView.viewWidthRatio;
    const initialNextRatio = nextView.viewWidthRatio;
    const totalRatio = initialCurrentRatio + initialNextRatio;

    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = e.clientX - startX;

      // 現在の実際の幅を取得
      const currentViewElement = document.getElementById(currentView.id)!;
      const nextViewElement = document.getElementById(nextView.id)!;
      const currentViewWidth = currentViewElement.clientWidth;
      const nextViewWidth = nextViewElement.clientWidth;

      // 移動後の幅を計算
      const newCurrentWidth = currentViewWidth + deltaX;
      const newNextWidth = nextViewWidth - deltaX;

      // 最小幅のチェック
      if (newCurrentWidth >= minViewWidth && newNextWidth >= minViewWidth) {
        // 移動距離に基づいて新しい比率を計算
        const ratioChange =
          (deltaX / (currentViewWidth + nextViewWidth)) * totalRatio;
        currentView.viewWidthRatio = initialCurrentRatio + ratioChange;
        nextView.viewWidthRatio = initialNextRatio - ratioChange;
      }
    };

    const cleanup = () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", cleanup);
      draggingViewId = undefined;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", cleanup);
  }
</script>

{#if workspace.isTabLoaded(tab.id)}
  <div
    bind:this={tabRef}
    class={tabStyle}
    data-disabled={focusedTabId !== tab.id}
  >
    {#each tab.views as view, viewIdx (view.id)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        id={view.id}
        class={viewStyle}
        style:flex-grow={view.viewWidthRatio}
        onmousedown={() => (tab.focusedViewId = view.id)}
      >
        {#if view.type === "timeline"}
          <TimelineView
            {view}
            pinned={tab.pinnedViewIds ? view.id in tab.pinnedViewIds : false}
          />
        {:else if view.type === "search"}
          <SearchView
            {view}
            pinned={tab.pinnedViewIds ? view.id in tab.pinnedViewIds : false}
            onCloseButtonClick={() => {
              workspace.closeView(tab, tabIdx, view, viewIdx);
            }}
          />
        {:else if view.type === "cards"}
          <CardsView
            isFocused={tab.id === focusedTabId && view.id === tab.focusedViewId}
            {view}
            onCloseButtonClick={() =>
              workspace.closeView(tab, tabIdx, view, viewIdx)}
          />
        {/if}
      </div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      {#if viewIdx !== tab.views.length - 1}
        <div
          class={viewResizeHandle}
          data-dragging={draggingViewId === view.id}
          onmousedown={(e) => resize(e, view, viewIdx)}
        ></div>
      {/if}
    {/each}
  </div>
{/if}

<script module>
  const tabStyle = css({
    w: "full",
    h: "full",
    position: "relative",
    display: "flex",
    flexDir: "row",
    "&[data-disabled=true]": {
      display: "none",
    },
  });

  const viewStyle = css({
    position: "relative",
    flex: "1",
    bg: "view.bg",
    h: "full",
    rounded: "md",
    shadow: "md.around",
  });

  const viewResizeHandle = css({
    w: "1",
    h: "[calc(100% - 0.5rem)]",
    mx: "0.5",
    my: "1",
    rounded: "md",
    transition: "all",
    "&:hover,&[data-dragging=true]": {
      cursor: "col-resize",
      bg: "[yellow]",
    },
  });
</script>
