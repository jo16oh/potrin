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

  const REM = 16;
  const minViewWidth = 24 * REM;

  function resize(e: MouseEvent, currentView: View, viewIdx: number) {
    e.preventDefault();
    draggingViewId = currentView.id;

    const startX = e.clientX;

    // 全てのビュー要素と初期幅を取得
    const viewElements = tab.views.map((v) => document.getElementById(v.id)!);
    const initialWidths = viewElements.map(
      (el) => el.getBoundingClientRect().width,
    );

    const handleMouseMove = (e: MouseEvent) => {
      let deltaX = e.clientX - startX;

      // deltaXが0なら何もしない
      if (deltaX === 0) return;

      // 新しい幅を計算（初期値として元の幅をコピー）
      const newWidths = [...initialWidths];

      // マウスが左に移動（現在のビューを縮小）する場合
      if (deltaX < 0) {
        let remainingShrink = -deltaX;

        // まず現在のビューを可能な限り縮小
        const currentMaxShrink = initialWidths[viewIdx]! - minViewWidth;

        if (remainingShrink <= currentMaxShrink) {
          // 現在のビューだけで縮小可能
          newWidths[viewIdx] = initialWidths[viewIdx]! - remainingShrink;
          newWidths[viewIdx + 1] =
            initialWidths[viewIdx + 1]! + remainingShrink;
        } else {
          // 現在のビューを最小幅まで縮小
          newWidths[viewIdx] = minViewWidth;
          const appliedShrink = currentMaxShrink;
          remainingShrink -= appliedShrink;

          // 残りの縮小を左側のビューに適用（右から左へ順番に）
          let additionalShrink = 0;

          for (let i = viewIdx - 1; i >= 0 && remainingShrink > 0; i--) {
            const leftMaxShrink = Math.max(0, initialWidths[i]! - minViewWidth);
            const shrinkForThisView = Math.min(leftMaxShrink, remainingShrink);

            if (shrinkForThisView > 0) {
              newWidths[i] = initialWidths[i]! - shrinkForThisView;
              additionalShrink += shrinkForThisView;
              remainingShrink -= shrinkForThisView;
            }
          }

          // 次のビューを拡大（現在のビュー縮小分 + 左側ビュー縮小分）
          newWidths[viewIdx + 1] =
            initialWidths[viewIdx + 1]! + appliedShrink + additionalShrink;
        }
      }
      // マウスが右に移動（現在のビューを拡大、次のビューを縮小）する場合
      else if (deltaX > 0) {
        let remainingExpand = deltaX;

        // まず次のビューを可能な限り縮小
        const nextMaxShrink = initialWidths[viewIdx + 1]! - minViewWidth;

        if (remainingExpand <= nextMaxShrink) {
          // 次のビューだけで拡大可能
          newWidths[viewIdx] = initialWidths[viewIdx]! + remainingExpand;
          newWidths[viewIdx + 1] =
            initialWidths[viewIdx + 1]! - remainingExpand;
        } else {
          // 次のビューを最小幅まで縮小
          newWidths[viewIdx + 1] = minViewWidth;
          const appliedExpand = nextMaxShrink;
          remainingExpand -= appliedExpand;

          // 残りの拡大分を右側のビューから削減（左から右へ順番に）
          let additionalExpand = 0;

          for (
            let i = viewIdx + 2;
            i < tab.views.length && remainingExpand > 0;
            i++
          ) {
            const rightMaxShrink = Math.max(
              0,
              initialWidths[i]! - minViewWidth,
            );
            const shrinkForThisView = Math.min(rightMaxShrink, remainingExpand);

            if (shrinkForThisView > 0) {
              newWidths[i] = initialWidths[i]! - shrinkForThisView;
              additionalExpand += shrinkForThisView;
              remainingExpand -= shrinkForThisView;
            }
          }

          // 現在のビューを拡大（次のビュー縮小分 + 右側ビュー縮小分）
          newWidths[viewIdx] =
            initialWidths[viewIdx]! + appliedExpand + additionalExpand;
        }
      }

      // 比率に変換
      const totalWidth = newWidths.reduce((sum, w) => sum + w, 0);
      const newRatios = newWidths.map(
        (width) => (width / totalWidth) * tab.views.length,
      );

      // 比率を適用
      tab.views.forEach((view, i) => {
        view.viewWidthRatio = newRatios[i]!;
      });
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
  <div class={tabStyle} data-disabled={focusedTabId !== tab.id}>
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
        <div class={viewResizeHandleContainer}>
          <div
            class={viewResizeHandle}
            data-dragging={draggingViewId === view.id}
            onmousedown={(e) => resize(e, view, viewIdx)}
          ></div>
        </div>
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
    minW: "[24rem]",
    flex: "1",
    bg: "view.bg",
    h: "full",
    rounded: "md",
    shadow: "md.around",
  });

  const viewResizeHandleContainer = css({
    flexShrink: "0",
    w: "2",
    h: "full",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
  });

  const viewResizeHandle = css({
    w: "1",
    h: "[calc(100% - 0.5rem)]",
    rounded: "md",
    transition: "all",
    "&:hover,&[data-dragging=true]": {
      cursor: "col-resize",
      bg: "[yellow]",
    },
  });
</script>
